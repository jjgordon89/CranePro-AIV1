//! Core Database module for CranePro Bridge Inspection Application
//!
//! This module handles database initialization, connections, and core database
//! operations using SQLite with connection pooling. It integrates with the
//! enhanced migration system for robust database management.

use crate::errors::{AppError, AppResult};
use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use log::{info, debug};

/// Database connection pool size
const POOL_SIZE: usize = 10;

/// Current database schema version
const CURRENT_SCHEMA_VERSION: i32 = 1;

/// Database connection pool
pub struct DatabasePool {
    connections: Arc<Mutex<Vec<Connection>>>,
    db_path: PathBuf,
}

impl DatabasePool {
    /// Create a new database pool
    pub async fn new(db_path: PathBuf) -> AppResult<Self> {
        let mut connections = Vec::with_capacity(POOL_SIZE);
        
        // Create initial connections
        for _ in 0..POOL_SIZE {
            let conn = Self::create_connection(&db_path)?;
            connections.push(conn);
        }

        Ok(DatabasePool {
            connections: Arc::new(Mutex::new(connections)),
            db_path,
        })
    }

    /// Create a new in-memory database pool for testing
    pub async fn new_in_memory() -> AppResult<Self> {
        let mut connections = Vec::with_capacity(POOL_SIZE);
        
        // Create initial in-memory connections
        for _ in 0..POOL_SIZE {
            let conn = Self::create_in_memory_connection()?;
            connections.push(conn);
        }

        Ok(DatabasePool {
            connections: Arc::new(Mutex::new(connections)),
            db_path: PathBuf::from(":memory:"),
        })
    }

    /// Create a new database connection
    fn create_connection(db_path: &Path) -> AppResult<Connection> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        // Configure connection
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        conn.execute("PRAGMA journal_mode = WAL", [])?;
        conn.execute("PRAGMA synchronous = NORMAL", [])?;
        conn.execute("PRAGMA cache_size = -64000", [])?; // 64MB cache
        conn.execute("PRAGMA temp_store = memory", [])?;

        Ok(conn)
    }

    /// Create a new in-memory database connection
    fn create_in_memory_connection() -> AppResult<Connection> {
        let conn = Connection::open_in_memory()?;

        // Configure connection
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        conn.execute("PRAGMA synchronous = NORMAL", [])?;
        conn.execute("PRAGMA cache_size = -64000", [])?; // 64MB cache
        conn.execute("PRAGMA temp_store = memory", [])?;

        Ok(conn)
    }

    /// Get a connection from the pool
    pub fn get_connection(&self) -> AppResult<Connection> {
        let mut pool = self.connections.lock()
            .map_err(|_| AppError::database("Failed to acquire connection pool lock"))?;
        
        if let Some(conn) = pool.pop() {
            Ok(conn)
        } else {
            // Pool exhausted, create a new connection
            if self.db_path.to_str() == Some(":memory:") {
                Self::create_in_memory_connection()
            } else {
                Self::create_connection(&self.db_path)
            }
        }
    }

    /// Return a connection to the pool
    pub fn return_connection(&self, conn: Connection) {
        if let Ok(mut pool) = self.connections.lock() {
            if pool.len() < POOL_SIZE {
                pool.push(conn);
            }
        }
        // If we can't return to pool, just drop the connection
    }
}

/// Main database service
pub struct Database {
    pool: DatabasePool,
    migrations: LegacyMigrationManager,
}

impl Database {
    /// Initialize the database with the given path
    pub async fn init(db_path: Option<PathBuf>) -> AppResult<Self> {
        let db_path = db_path.unwrap_or_else(|| {
            let mut path = std::env::current_dir().unwrap_or_default();
            path.push("crane_pro.db");
            path
        });

        info!("Initializing database at: {:?}", db_path);

        // Ensure the directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let pool = DatabasePool::new(db_path).await?;
        let migrations = LegacyMigrationManager::new();

        let db = Self { pool, migrations };

        // Run migrations
        db.migrate().await?;

        Ok(db)
    }

    /// Initialize an in-memory database for testing
    pub async fn new_in_memory() -> AppResult<Self> {
        info!("Initializing in-memory database");

        let pool = DatabasePool::new_in_memory().await?;
        let migrations = LegacyMigrationManager::new();

        let db = Self { pool, migrations };

        // Run migrations
        db.migrate().await?;

        Ok(db)
    }

    /// Execute a transaction
    pub fn with_transaction<F, R>(&self, f: F) -> AppResult<R>
    where
        F: FnOnce(&Connection) -> AppResult<R>,
    {
        let conn = self.pool.get_connection()?;
        
        let transaction = conn.unchecked_transaction()?;
        
        match f(&conn) {
            Ok(result) => {
                transaction.commit()?;
                self.pool.return_connection(conn);
                Ok(result)
            }
            Err(err) => {
                let _ = transaction.rollback();
                self.pool.return_connection(conn);
                Err(err)
            }
        }
    }

    /// Get a database connection for read operations
    pub fn get_connection(&self) -> AppResult<Connection> {
        self.pool.get_connection()
    }

    /// Return a connection to the pool
    pub fn return_connection(&self, conn: Connection) {
        self.pool.return_connection(conn);
    }

    /// Run database migrations
    async fn migrate(&self) -> AppResult<()> {
        info!("Running database migrations");
        
        let conn = self.pool.get_connection()?;
        
        // Get current version
        let current_version = self.get_schema_version(&conn)?;
        info!("Current schema version: {}", current_version);
        
        if current_version < CURRENT_SCHEMA_VERSION {
            info!("Migrating from version {} to {}", current_version, CURRENT_SCHEMA_VERSION);
            self.migrations.run_migrations(&conn, current_version, CURRENT_SCHEMA_VERSION)?;
            self.set_schema_version(&conn, CURRENT_SCHEMA_VERSION)?;
            info!("Migration completed successfully");
        } else {
            info!("Database schema is up to date");
        }
        
        self.pool.return_connection(conn);
        Ok(())
    }

    /// Get the current schema version
    fn get_schema_version(&self, conn: &Connection) -> AppResult<i32> {
        // Create schema_version table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            )",
            [],
        )?;

        let version = conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
                Ok(row.get::<_, i32>(0)?)
            })
            .unwrap_or(0);

        Ok(version)
    }

    /// Set the schema version
    fn set_schema_version(&self, conn: &Connection, version: i32) -> AppResult<()> {
        conn.execute("DELETE FROM schema_version", [])?;
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [version],
        )?;
        Ok(())
    }
}

/// Legacy Migration manager for database schema changes (for backward compatibility)
pub struct LegacyMigrationManager {
    migrations: Vec<LegacyMigration>,
}

impl LegacyMigrationManager {
    pub fn new() -> Self {
        let mut migrations = Vec::new();
        
        // Add initial schema migration
        migrations.push(LegacyMigration {
            version: 1,
            description: "Initial schema creation".to_string(),
            up_sql: INITIAL_SCHEMA.to_string(),
            down_sql: "".to_string(), // Initial migration cannot be rolled back
        });

        LegacyMigrationManager { migrations }
    }

    /// Run migrations from current version to target version
    pub fn run_migrations(
        &self,
        conn: &Connection,
        from_version: i32,
        to_version: i32,
    ) -> AppResult<()> {
        for migration in &self.migrations {
            if migration.version > from_version && migration.version <= to_version {
                info!("Running migration {}: {}", migration.version, migration.description);
                
                // Split SQL by semicolon and execute each statement
                for statement in migration.up_sql.split(';') {
                    let statement = statement.trim();
                    if !statement.is_empty() {
                        conn.execute(statement, [])?;
                    }
                }
                
                debug!("Migration {} completed", migration.version);
            }
        }
        Ok(())
    }
}

/// Represents a legacy database migration (for backward compatibility)
#[derive(Debug, Clone)]
pub struct LegacyMigration {
    pub version: i32,
    pub description: String,
    pub up_sql: String,
    pub down_sql: String,
}

/// Initial database schema SQL
const INITIAL_SCHEMA: &str = r#"
-- Users table
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('Inspector', 'Supervisor', 'Administrator', 'SuperAdmin')),
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    phone TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT 1
);

-- Locations table
CREATE TABLE locations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    address TEXT,
    latitude REAL,
    longitude REAL,
    description TEXT,
    created_by INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Assets table
CREATE TABLE assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_number TEXT NOT NULL UNIQUE,
    asset_name TEXT NOT NULL,
    asset_type TEXT NOT NULL,
    manufacturer TEXT,
    model TEXT,
    serial_number TEXT,
    manufacture_date DATE,
    installation_date DATE,
    capacity REAL,
    capacity_unit TEXT,
    location_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'Active' CHECK(status IN ('Active', 'Inactive', 'Maintenance', 'Decommissioned')),
    description TEXT,
    specifications JSON,
    created_by INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (location_id) REFERENCES locations(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Components table
CREATE TABLE components (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    component_name TEXT NOT NULL,
    component_type TEXT NOT NULL,
    manufacturer TEXT,
    model TEXT,
    serial_number TEXT,
    parent_component_id INTEGER,
    specifications JSON,
    status TEXT NOT NULL DEFAULT 'Active' CHECK(status IN ('Active', 'Inactive', 'Maintenance', 'Replaced')),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (asset_id) REFERENCES assets(id),
    FOREIGN KEY (parent_component_id) REFERENCES components(id)
);

-- Compliance Standards table
CREATE TABLE compliance_standards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    standard_code TEXT NOT NULL UNIQUE,
    standard_name TEXT NOT NULL,
    version TEXT NOT NULL,
    requirements JSON,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Compliance Checklist Templates table
CREATE TABLE compliance_checklist_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    standard_id INTEGER NOT NULL,
    template_name TEXT NOT NULL,
    inspection_type TEXT NOT NULL,
    checklist_structure JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (standard_id) REFERENCES compliance_standards(id)
);

-- Inspections table
CREATE TABLE inspections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    inspector_id INTEGER NOT NULL,
    inspection_type TEXT NOT NULL CHECK(inspection_type IN ('Frequent', 'Periodic', 'Initial', 'Special')),
    compliance_standard TEXT NOT NULL,
    scheduled_date DATETIME,
    actual_date DATETIME,
    status TEXT NOT NULL DEFAULT 'Scheduled' CHECK(status IN ('Scheduled', 'In Progress', 'Completed', 'Cancelled')),
    overall_condition TEXT CHECK(overall_condition IN ('Excellent', 'Good', 'Fair', 'Poor', 'Critical')),
    checklist_data JSON,
    notes TEXT,
    ai_analysis_results JSON,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (asset_id) REFERENCES assets(id),
    FOREIGN KEY (inspector_id) REFERENCES users(id)
);

-- Inspection Items table
CREATE TABLE inspection_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inspection_id INTEGER NOT NULL,
    component_id INTEGER,
    item_name TEXT NOT NULL,
    item_category TEXT NOT NULL,
    condition TEXT CHECK(condition IN ('Excellent', 'Good', 'Fair', 'Poor', 'Critical')),
    finding TEXT,
    severity TEXT CHECK(severity IN ('Low', 'Medium', 'High', 'Critical')),
    is_compliant BOOLEAN,
    corrective_action TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (inspection_id) REFERENCES inspections(id),
    FOREIGN KEY (component_id) REFERENCES components(id)
);

-- Media Files table
CREATE TABLE media_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inspection_id INTEGER,
    component_id INTEGER,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_type TEXT NOT NULL CHECK(file_type IN ('image', 'video', 'document', 'audio')),
    mime_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    description TEXT,
    ai_analysis_metadata JSON,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (inspection_id) REFERENCES inspections(id),
    FOREIGN KEY (component_id) REFERENCES components(id)
);

-- AI Model Results table
CREATE TABLE ai_model_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inspection_id INTEGER,
    media_file_id INTEGER,
    model_name TEXT NOT NULL,
    model_version TEXT NOT NULL,
    predictions JSON NOT NULL,
    confidence_score REAL NOT NULL CHECK(confidence_score >= 0 AND confidence_score <= 1),
    status TEXT NOT NULL DEFAULT 'Pending' CHECK(status IN ('Pending', 'Processing', 'Completed', 'Failed')),
    processed_at DATETIME,
    FOREIGN KEY (inspection_id) REFERENCES inspections(id),
    FOREIGN KEY (media_file_id) REFERENCES media_files(id)
);

-- Maintenance Records table
CREATE TABLE maintenance_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    component_id INTEGER,
    maintenance_type TEXT NOT NULL CHECK(maintenance_type IN ('Preventive', 'Corrective', 'Emergency', 'Overhaul')),
    scheduled_date DATETIME,
    completed_date DATETIME,
    performed_by TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Scheduled' CHECK(status IN ('Scheduled', 'In Progress', 'Completed', 'Cancelled')),
    parts_used JSON,
    cost REAL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (asset_id) REFERENCES assets(id),
    FOREIGN KEY (component_id) REFERENCES components(id)
);

-- Create indexes for performance
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_assets_asset_number ON assets(asset_number);
CREATE INDEX idx_assets_location_id ON assets(location_id);
CREATE INDEX idx_assets_status ON assets(status);
CREATE INDEX idx_components_asset_id ON components(asset_id);
CREATE INDEX idx_components_parent_id ON components(parent_component_id);
CREATE INDEX idx_inspections_asset_id ON inspections(asset_id);
CREATE INDEX idx_inspections_inspector_id ON inspections(inspector_id);
CREATE INDEX idx_inspections_status ON inspections(status);
CREATE INDEX idx_inspections_scheduled_date ON inspections(scheduled_date);
CREATE INDEX idx_inspection_items_inspection_id ON inspection_items(inspection_id);
CREATE INDEX idx_inspection_items_component_id ON inspection_items(component_id);
CREATE INDEX idx_media_files_inspection_id ON media_files(inspection_id);
CREATE INDEX idx_media_files_component_id ON media_files(component_id);
CREATE INDEX idx_ai_results_inspection_id ON ai_model_results(inspection_id);
CREATE INDEX idx_ai_results_media_file_id ON ai_model_results(media_file_id);
CREATE INDEX idx_maintenance_asset_id ON maintenance_records(asset_id);
CREATE INDEX idx_maintenance_component_id ON maintenance_records(component_id);
CREATE INDEX idx_maintenance_status ON maintenance_records(status);

-- Create triggers for updating updated_at timestamps
CREATE TRIGGER update_users_timestamp 
    AFTER UPDATE ON users 
    FOR EACH ROW 
    BEGIN 
        UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; 
    END;

CREATE TRIGGER update_locations_timestamp 
    AFTER UPDATE ON locations 
    FOR EACH ROW 
    BEGIN 
        UPDATE locations SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; 
    END;

CREATE TRIGGER update_assets_timestamp 
    AFTER UPDATE ON assets 
    FOR EACH ROW 
    BEGIN 
        UPDATE assets SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; 
    END;

CREATE TRIGGER update_components_timestamp 
    AFTER UPDATE ON components 
    FOR EACH ROW 
    BEGIN 
        UPDATE components SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; 
    END;

CREATE TRIGGER update_compliance_standards_timestamp 
    AFTER UPDATE ON compliance_standards 
    FOR EACH ROW 
    BEGIN 
        UPDATE compliance_standards SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; 
    END;

CREATE TRIGGER update_compliance_checklist_templates_timestamp 
    AFTER UPDATE ON compliance_checklist_templates 
    FOR EACH ROW 
    BEGIN 
        UPDATE compliance_checklist_templates SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; 
    END;

CREATE TRIGGER update_inspections_timestamp 
    AFTER UPDATE ON inspections 
    FOR EACH ROW 
    BEGIN 
        UPDATE inspections SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; 
    END;

-- Insert default compliance standards
INSERT INTO compliance_standards (standard_code, standard_name, version, requirements, is_active) VALUES
('OSHA_1910_179', 'Overhead and Gantry Cranes', '2023', '{}', 1),
('ASME_B30_2', 'Overhead and Gantry Cranes (Top Running Bridge)', '2023', '{}', 1),
('CMAA_75', 'Crane Manufacturers Association of America Specification No. 75', '2022', '{}', 1);

-- Insert default admin user (password: admin123 - should be changed in production)
INSERT INTO users (username, email, password_hash, role, first_name, last_name, is_active) VALUES
('admin', 'admin@cranepro.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewf5FuiOJ/rZNpyC', 'SuperAdmin', 'System', 'Administrator', 1);
"#;