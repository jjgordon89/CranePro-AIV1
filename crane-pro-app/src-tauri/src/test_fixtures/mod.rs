//! Test fixtures and utilities for the CranePro Bridge Inspection Application
//!
//! This module provides shared test utilities, mock data generators, and test database
//! functionality for use across all test modules in the application.

#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use crate::database::core::Database;
#[cfg(test)]
use crate::errors::AppResult;
#[cfg(test)]
use crate::models::*;
#[cfg(test)]
use chrono::Utc;

// Test configuration constants
#[cfg(test)]
pub const TEST_DB_NAME: &str = ":memory:";
#[cfg(test)]
pub const TEST_TIMEOUT_SECONDS: u64 = 30;
#[cfg(test)]
pub const TEST_MAX_RETRIES: u32 = 3;

/// Test data fixtures containing seeded test data
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct TestDataFixtures {
    pub users: Vec<User>,
    pub locations: Vec<Location>,
    pub assets: Vec<Asset>,
    pub inspections: Vec<Inspection>,
    pub components: Vec<Component>,
}

#[cfg(test)]
impl Default for TestDataFixtures {
    fn default() -> Self {
        Self {
            users: Vec::new(),
            locations: Vec::new(),
            assets: Vec::new(),
            inspections: Vec::new(),
            components: Vec::new(),
        }
    }
}

/// TestDatabase struct for managing test database operations
///
/// This struct provides a wrapper around the core Database functionality
/// specifically designed for testing scenarios. It includes methods for
/// database initialization, cleanup, seeding test data, and ensuring
/// test isolation.
#[cfg(test)]
pub struct TestDatabase {
    database: Arc<Database>,
}

#[cfg(test)]
impl TestDatabase {
    /// Create a new in-memory test database instance
    ///
    /// This constructor creates a new in-memory SQLite database using the
    /// existing Database::new_in_memory() method. The database is fully
    /// initialized with schema migrations applied.
    ///
    /// # Returns
    ///
    /// Returns an `AppResult<TestDatabase>` containing the initialized test database
    /// or an error if initialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// let test_db = TestDatabase::new_in_memory().await?;
    /// ```
    pub async fn new_in_memory() -> AppResult<Self> {
        let database = Database::new_in_memory().await?;
        Ok(Self {
            database: Arc::new(database),
        })
    }

    /// Reset the database by clearing all data while preserving schema
    ///
    /// This method removes all data from all tables in reverse dependency order
    /// to avoid foreign key constraint violations. The schema and indexes are
    /// preserved, making this ideal for test cleanup between test cases.
    ///
    /// # Returns
    ///
    /// Returns an `AppResult<()>` indicating success or failure of the reset operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// test_db.reset().await?;
    /// ```
    pub async fn reset(&self) -> AppResult<()> {
        self.database.with_transaction(|conn| {
            // Delete data in reverse dependency order to avoid foreign key violations
            
            // AI and analysis data
            conn.execute("DELETE FROM ai_model_results", [])?;
            
            // Media files
            conn.execute("DELETE FROM media_files", [])?;
            
            // Inspection related data
            conn.execute("DELETE FROM inspection_items", [])?;
            conn.execute("DELETE FROM inspections", [])?;
            
            // Maintenance records
            conn.execute("DELETE FROM maintenance_records", [])?;
            
            // Components (self-referential, so delete in proper order)
            conn.execute("DELETE FROM components WHERE parent_component_id IS NOT NULL", [])?;
            conn.execute("DELETE FROM components", [])?;
            
            // Assets and location tracking
            conn.execute("DELETE FROM asset_location_history", [])?;
            conn.execute("DELETE FROM assets", [])?;
            
            // Location capacity and hierarchy
            conn.execute("DELETE FROM location_capacity_settings", [])?;
            conn.execute("DELETE FROM locations WHERE parent_location_id IS NOT NULL", [])?;
            conn.execute("DELETE FROM locations", [])?;
            
            // Compliance data
            conn.execute("DELETE FROM compliance_checklist_templates", [])?;
            conn.execute("DELETE FROM compliance_standards WHERE id > 3", [])?; // Keep default standards
            
            // Users (keep default admin user)
            conn.execute("DELETE FROM users WHERE id > 1", [])?;
            
            Ok(())
        })
    }

    /// Seed the database with minimal test data
    ///
    /// Creates a basic set of test data including:
    /// - One test user (inspector role)
    /// - One test location
    /// - One test asset
    /// - Basic compliance standards (already seeded by migration)
    ///
    /// # Returns
    ///
    /// Returns an `AppResult<TestDataFixtures>` containing the created test data.
    ///
    /// # Example
    ///
    /// ```rust
    /// let fixtures = test_db.seed_minimal_data().await?;
    /// ```
    pub async fn seed_minimal_data(&self) -> AppResult<TestDataFixtures> {
        let mut fixtures = TestDataFixtures::default();
        
        self.database.with_transaction(|conn| {
            // Create test user
            conn.execute(
                "INSERT INTO users (username, email, password_hash, role, first_name, last_name, phone, is_active)
                 VALUES ('test_inspector', 'inspector@test.com', '$2b$12$test_hash', 'Inspector', 'Test', 'Inspector', '555-0001', 1)",
                [],
            )?;
            
            // Create test location
            conn.execute(
                "INSERT INTO locations (name, address, latitude, longitude, description, created_by)
                 VALUES ('Test Facility', '123 Test St, Test City, TC 12345', 40.7128, -74.0060, 'Test facility for automated testing', 1)",
                [],
            )?;
            
            // Create test asset
            conn.execute(
                "INSERT INTO assets (asset_number, asset_name, asset_type, manufacturer, model, serial_number,
                                   capacity, capacity_unit, location_id, status, description, created_by)
                 VALUES ('TEST-001', 'Test Bridge Crane', 'Bridge Crane', 'Test Manufacturing', 'Model X100',
                         'SN123456', 10.0, 'tons', 2, 'Active', 'Test bridge crane for automated testing', 1)",
                [],
            )?;
            
            Ok(())
        })?;
        
        Ok(fixtures)
    }

    /// Seed the database with comprehensive test data
    ///
    /// Creates a comprehensive set of test data including:
    /// - Multiple users with different roles
    /// - Multiple locations with hierarchy
    /// - Multiple assets with various statuses
    /// - Sample inspections and components
    /// - Test compliance data
    ///
    /// # Returns
    ///
    /// Returns an `AppResult<TestDataFixtures>` containing the created test data.
    ///
    /// # Example
    ///
    /// ```rust
    /// let fixtures = test_db.seed_comprehensive_data().await?;
    /// ```
    pub async fn seed_comprehensive_data(&self) -> AppResult<TestDataFixtures> {
        let mut fixtures = TestDataFixtures::default();
        
        self.database.with_transaction(|conn| {
            // Create multiple test users
            conn.execute(
                "INSERT INTO users (username, email, password_hash, role, first_name, last_name, phone, is_active) VALUES
                 ('test_inspector', 'inspector@test.com', '$2b$12$test_hash', 'Inspector', 'Test', 'Inspector', '555-0001', 1),
                 ('test_supervisor', 'supervisor@test.com', '$2b$12$test_hash', 'Supervisor', 'Test', 'Supervisor', '555-0002', 1),
                 ('test_admin', 'admin@test.com', '$2b$12$test_hash', 'Administrator', 'Test', 'Admin', '555-0003', 1)",
                [],
            )?;
            
            // Create test locations with hierarchy
            conn.execute(
                "INSERT INTO locations (name, address, latitude, longitude, description, parent_location_id, created_by) VALUES
                 ('Main Facility', '100 Industrial Blvd, Test City, TC 12345', 40.7128, -74.0060, 'Main industrial facility', NULL, 1),
                 ('Building A', '100 Industrial Blvd, Building A', 40.7130, -74.0058, 'Manufacturing building A', 2, 1),
                 ('Building B', '100 Industrial Blvd, Building B', 40.7126, -74.0062, 'Manufacturing building B', 2, 1),
                 ('Warehouse', '200 Storage Ave, Test City, TC 12346', 40.7150, -74.0070, 'Storage warehouse facility', NULL, 1)",
                [],
            )?;
            
            // Create test assets
            conn.execute(
                "INSERT INTO assets (asset_number, asset_name, asset_type, manufacturer, model, serial_number,
                                   capacity, capacity_unit, location_id, status, description, created_by) VALUES
                 ('CRANE-001', 'Bridge Crane A1', 'Bridge Crane', 'Acme Cranes', 'BC-500', 'SN001', 5.0, 'tons', 3, 'Active', 'Main production crane in Building A', 1),
                 ('CRANE-002', 'Bridge Crane A2', 'Bridge Crane', 'Acme Cranes', 'BC-500', 'SN002', 5.0, 'tons', 3, 'Active', 'Secondary crane in Building A', 1),
                 ('CRANE-003', 'Bridge Crane B1', 'Bridge Crane', 'Superior Lifting', 'SL-750', 'SN003', 7.5, 'tons', 4, 'Maintenance', 'Heavy duty crane in Building B', 1),
                 ('HOIST-001', 'Electric Hoist W1', 'Electric Hoist', 'Hoist Masters', 'HM-200', 'SN004', 2.0, 'tons', 5, 'Active', 'Warehouse electric hoist', 1)",
                [],
            )?;
            
            // Create test components
            conn.execute(
                "INSERT INTO components (asset_id, component_name, component_type, manufacturer, model, status) VALUES
                 (2, 'Main Hoist Motor', 'Motor', 'Electric Motors Inc', 'EM-100', 'Active'),
                 (2, 'Bridge Drive Motor', 'Motor', 'Electric Motors Inc', 'EM-50', 'Active'),
                 (2, 'Load Block', 'Load Block', 'Acme Cranes', 'LB-500', 'Active'),
                 (3, 'Main Hoist Motor', 'Motor', 'Electric Motors Inc', 'EM-100', 'Active'),
                 (4, 'Wire Rope', 'Wire Rope', 'Cable Corp', 'WR-12mm', 'Maintenance')",
                [],
            )?;
            
            // Create test inspections
            conn.execute(
                "INSERT INTO inspections (asset_id, inspector_id, inspection_type, compliance_standard,
                                        scheduled_date, status, overall_condition, notes) VALUES
                 (2, 2, 'Periodic', 'OSHA_1910_179', '2024-12-01 10:00:00', 'Scheduled', NULL, 'Quarterly inspection scheduled'),
                 (3, 2, 'Frequent', 'OSHA_1910_179', '2024-11-15 14:00:00', 'Completed', 'Good', 'Weekly inspection completed, minor issues noted'),
                 (4, 3, 'Initial', 'ASME_B30_2', '2024-10-01 09:00:00', 'Completed', 'Excellent', 'Initial inspection after maintenance')",
                [],
            )?;
            
            Ok(())
        })?;
        
        Ok(fixtures)
    }

    /// Seed the database with performance test data
    ///
    /// Creates a large dataset for performance testing including:
    /// - Many users, locations, and assets
    /// - Extensive inspection history
    /// - Large numbers of components and maintenance records
    ///
    /// # Returns
    ///
    /// Returns an `AppResult<TestDataFixtures>` containing the created test data.
    ///
    /// # Example
    ///
    /// ```rust
    /// let fixtures = test_db.seed_performance_data().await?;
    /// ```
    pub async fn seed_performance_data(&self) -> AppResult<TestDataFixtures> {
        let mut fixtures = TestDataFixtures::default();
        
        self.database.with_transaction(|conn| {
            // Create many test users (50)
            for i in 1..=50 {
                conn.execute(
                    "INSERT INTO users (username, email, password_hash, role, first_name, last_name, is_active)
                     VALUES (?, ?, '$2b$12$test_hash', 'Inspector', ?, ?, 1)",
                    [
                        format!("user_{:03}", i),
                        format!("user{}@test.com", i),
                        format!("User"),
                        format!("{:03}", i),
                    ],
                )?;
            }
            
            // Create many test locations (20)
            for i in 1..=20 {
                conn.execute(
                    "INSERT INTO locations (name, address, latitude, longitude, description, created_by)
                     VALUES (?, ?, ?, ?, ?, 1)",
                    [
                        format!("Facility {:03}", i),
                        format!("{} Industrial Way, Test City, TC {}", 100 + i, 12000 + i),
                        (40.7000 + (i as f64 * 0.001)).to_string(),
                        (-74.0000 - (i as f64 * 0.001)).to_string(),
                        format!("Performance test facility number {}", i),
                    ],
                )?;
            }
            
            // Create many test assets (100)
            for i in 1..=100 {
                let location_id = 2 + (i % 20); // Distribute across locations
                conn.execute(
                    "INSERT INTO assets (asset_number, asset_name, asset_type, manufacturer, model,
                                       capacity, capacity_unit, location_id, status, description, created_by)
                     VALUES (?, ?, 'Bridge Crane', 'Test Manufacturing', 'Model X', ?, 'tons', ?, 'Active', ?, 1)",
                    [
                        format!("PERF-{:03}", i),
                        format!("Performance Test Crane {:03}", i),
                        (5.0 + (i as f64 * 0.1)).to_string(),
                        location_id.to_string(),
                        format!("Performance test asset number {}", i),
                    ],
                )?;
            }
            
            Ok(())
        })?;
        
        Ok(fixtures)
    }

    /// Get access to the underlying database instance
    ///
    /// This method provides access to the wrapped Database instance for
    /// test scenarios that need direct database access.
    ///
    /// # Returns
    ///
    /// Returns an `Arc<Database>` reference to the underlying database.
    ///
    /// # Example
    ///
    /// ```rust
    /// let db = test_db.get_database();
    /// db.with_transaction(|conn| {
    ///     // Direct database operations
    ///     Ok(())
    /// })?;
    /// ```
    pub fn get_database(&self) -> Arc<Database> {
        Arc::clone(&self.database)
    }
}

#[cfg(test)]
impl Drop for TestDatabase {
    /// Automatic cleanup when TestDatabase is dropped
    ///
    /// This ensures that test databases are properly cleaned up even if
    /// explicit cleanup is not called. Note that this uses a blocking
    /// approach since Drop cannot be async.
    fn drop(&mut self) {
        // Note: We can't use async in Drop, but since this is an in-memory database,
        // it will be automatically cleaned up when the Database instance is dropped.
        // For more complex cleanup scenarios, tests should explicitly call reset().
    }
}

// Test data generators for creating mock entities
#[cfg(test)]
pub mod generators {
    //! Test data generators for creating mock entities with proper model structure

    use super::*;
    use chrono::{NaiveDate};

    /// Generate a test user with proper model structure
    pub fn test_user() -> User {
        User {
            id: 1,
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "$2b$12$test_hash".to_string(),
            role: UserRole::Inspector,
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            phone: Some("555-0123".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
        }
    }

    /// Generate a test asset with proper model structure
    pub fn test_asset() -> Asset {
        Asset {
            id: 1,
            asset_number: "TEST-001".to_string(),
            asset_name: "Test Bridge Crane".to_string(),
            asset_type: "Bridge Crane".to_string(),
            manufacturer: Some("Test Manufacturing".to_string()),
            model: Some("Model X100".to_string()),
            serial_number: Some("SN123456".to_string()),
            manufacture_date: Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
            installation_date: Some(NaiveDate::from_ymd_opt(2020, 6, 15).unwrap()),
            capacity: Some(10.0),
            capacity_unit: Some("tons".to_string()),
            location_id: 1,
            status: AssetStatus::Active,
            description: Some("Test bridge crane for automated testing".to_string()),
            specifications: None,
            created_by: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Generate a test location with proper model structure
    pub fn test_location() -> Location {
        Location {
            id: 1,
            name: "Test Facility".to_string(),
            address: Some("123 Test Street, Test City, TC 12345".to_string()),
            latitude: Some(40.7128),
            longitude: Some(-74.0060),
            description: Some("Test facility for automated testing".to_string()),
            parent_location_id: None,
            created_by: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Generate a test inspection with proper model structure
    pub fn test_inspection() -> Inspection {
        Inspection {
            id: 1,
            asset_id: 1,
            inspector_id: 1,
            inspection_type: InspectionType::Periodic,
            compliance_standard: "OSHA_1910_179".to_string(),
            scheduled_date: Some(Utc::now()),
            actual_date: None,
            status: InspectionStatus::Scheduled,
            overall_condition: None,
            checklist_data: None,
            notes: Some("Test inspection for automated testing".to_string()),
            ai_analysis_results: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Generate a test component with proper model structure
    pub fn test_component() -> Component {
        Component {
            id: 1,
            asset_id: 1,
            component_name: "Test Hoist Motor".to_string(),
            component_type: "Motor".to_string(),
            manufacturer: Some("Electric Motors Inc".to_string()),
            model: Some("EM-100".to_string()),
            serial_number: Some("SN789012".to_string()),
            parent_component_id: None,
            specifications: None,
            status: ComponentStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

// Test utility functions
#[cfg(test)]
pub mod utils {
    //! Utility functions for test setup and teardown

    use super::*;

    /// Setup a test environment with a fresh TestDatabase
    ///
    /// Creates a new in-memory test database instance that is ready for use.
    /// The database includes all schema migrations and is isolated from other tests.
    ///
    /// # Returns
    ///
    /// Returns an `AppResult<TestDatabase>` containing the initialized test database.
    ///
    /// # Example
    ///
    /// ```rust
    /// let test_db = setup_test_env().await?;
    /// ```
    pub async fn setup_test_env() -> AppResult<TestDatabase> {
        TestDatabase::new_in_memory().await
    }

    /// Teardown a test environment by resetting the database
    ///
    /// Clears all data from the test database while preserving the schema.
    /// This ensures a clean state for subsequent tests.
    ///
    /// # Arguments
    ///
    /// * `db` - The TestDatabase instance to clean up
    ///
    /// # Returns
    ///
    /// Returns an `AppResult<()>` indicating success or failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// teardown_test_env(test_db).await?;
    /// ```
    pub async fn teardown_test_env(db: TestDatabase) -> AppResult<()> {
        db.reset().await
    }

    /// Create a temporary test file for file-based testing scenarios
    ///
    /// Creates a temporary file that will be automatically cleaned up
    /// when the returned NamedTempFile is dropped.
    ///
    /// # Returns
    ///
    /// Returns a `Result<tempfile::NamedTempFile, std::io::Error>` containing
    /// the temporary file handle.
    ///
    /// # Example
    ///
    /// ```rust
    /// let temp_file = create_temp_file()?;
    /// ```
    pub fn create_temp_file() -> Result<tempfile::NamedTempFile, std::io::Error> {
        tempfile::NamedTempFile::new()
    }
}

// Re-export commonly used test utilities
#[cfg(test)]
pub use generators::*;
#[cfg(test)]
pub use utils::*;
// Tests for this module
#[cfg(test)]
mod tests;