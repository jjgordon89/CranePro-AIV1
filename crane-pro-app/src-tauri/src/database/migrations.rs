//! Enhanced Migration Infrastructure for CranePro Bridge Inspection Application
//!
//! This module provides a comprehensive migration system with dependency resolution,
//! rollback capabilities, checksum validation, and atomic operations. It supports
//! complex migration scenarios and ensures database integrity throughout the process.

use crate::errors::{AppError, AppResult};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Enhanced Migration struct with comprehensive metadata and dependency tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    /// Migration version number (must be unique and sequential)
    pub version: i32,
    /// Human-readable name for the migration
    pub name: String,
    /// Detailed description of what this migration does
    pub description: String,
    /// SQL statements to apply the migration (forward direction)
    pub up_sql: String,
    /// SQL statements to rollback the migration (reverse direction)
    pub down_sql: String,
    /// Migration versions this migration depends on (must be applied first)
    pub dependencies: Vec<i32>,
    /// SHA-256 checksum of up_sql for integrity verification
    pub checksum: String,
    /// Timestamp when this migration was applied (None if not applied)
    pub applied_at: Option<DateTime<Utc>>,
}

impl Migration {
    /// Create a new migration with automatic checksum generation
    pub fn new(
        version: i32,
        name: String,
        description: String,
        up_sql: String,
        down_sql: String,
        dependencies: Vec<i32>,
    ) -> Self {
        let checksum = Self::calculate_checksum(&up_sql);
        
        Self {
            version,
            name,
            description,
            up_sql,
            down_sql,
            dependencies,
            checksum,
            applied_at: None,
        }
    }

    /// Calculate SHA-256 checksum of SQL content
    fn calculate_checksum(sql: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(sql.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Validate migration integrity by comparing checksums
    pub fn validate_checksum(&self) -> AppResult<()> {
        let calculated = Self::calculate_checksum(&self.up_sql);
        if calculated != self.checksum {
            return Err(AppError::DatabaseMigration {
                version: format!(
                    "{} - Checksum mismatch: expected {}, got {}",
                    self.version, self.checksum, calculated
                ),
            });
        }
        Ok(())
    }

    /// Check if this migration has been applied
    pub fn is_applied(&self) -> bool {
        self.applied_at.is_some()
    }

    /// Mark this migration as applied
    pub fn mark_applied(&mut self) {
        self.applied_at = Some(Utc::now());
    }

    /// Mark this migration as not applied (for rollbacks)
    pub fn mark_unapplied(&mut self) {
        self.applied_at = None;
    }
}

/// Migration execution result with detailed information
#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub version: i32,
    pub name: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub applied_at: DateTime<Utc>,
}

/// Progress tracking for migration operations
#[derive(Debug, Clone)]
pub struct MigrationProgress {
    pub total_migrations: usize,
    pub completed_migrations: usize,
    pub current_migration: Option<String>,
    pub failed_migration: Option<String>,
    pub start_time: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Enhanced Migration Runner with comprehensive features
pub struct MigrationRunner {
    /// Thread-safe storage for migration results and progress
    results: Arc<Mutex<Vec<MigrationResult>>>,
    progress: Arc<Mutex<MigrationProgress>>,
    /// Available migrations indexed by version
    migrations: HashMap<i32, Migration>,
    /// Dependency graph for migration ordering
    dependency_graph: HashMap<i32, Vec<i32>>,
}

impl MigrationRunner {
    /// Create a new migration runner
    pub fn new() -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            progress: Arc::new(Mutex::new(MigrationProgress {
                total_migrations: 0,
                completed_migrations: 0,
                current_migration: None,
                failed_migration: None,
                start_time: Utc::now(),
                estimated_completion: None,
            })),
            migrations: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }

    /// Add a migration to the runner
    pub fn add_migration(&mut self, migration: Migration) -> AppResult<()> {
        // Validate migration integrity
        migration.validate_checksum()?;

        // Check for version conflicts
        if self.migrations.contains_key(&migration.version) {
            return Err(AppError::DatabaseMigration {
                version: format!(
                    "Migration version {} already exists",
                    migration.version
                ),
            });
        }

        // Build dependency graph
        self.dependency_graph.insert(migration.version, migration.dependencies.clone());
        
        // Store migration
        self.migrations.insert(migration.version, migration);
        
        info!("Added migration {} to runner", self.migrations.len());
        Ok(())
    }

    /// Add multiple migrations in batch
    pub fn add_migrations(&mut self, migrations: Vec<Migration>) -> AppResult<()> {
        for migration in migrations {
            self.add_migration(migration)?;
        }
        Ok(())
    }

    /// Get current migration progress
    pub fn get_progress(&self) -> AppResult<MigrationProgress> {
        let progress = self.progress.lock()
            .map_err(|_| AppError::internal("Failed to acquire progress lock"))?;
        Ok(progress.clone())
    }

    /// Get migration execution results
    pub fn get_results(&self) -> AppResult<Vec<MigrationResult>> {
        let results = self.results.lock()
            .map_err(|_| AppError::internal("Failed to acquire results lock"))?;
        Ok(results.clone())
    }

    /// Resolve migration dependencies and return execution order
    fn resolve_dependencies(&self, target_versions: &[i32]) -> AppResult<Vec<i32>> {
        let mut visited = HashSet::new();
        let mut resolved = Vec::new();
        let mut temp_visited = HashSet::new();

        for &version in target_versions {
            self.visit_migration(version, &mut visited, &mut resolved, &mut temp_visited)?;
        }

        Ok(resolved)
    }

    /// Recursive dependency resolution with cycle detection
    fn visit_migration(
        &self,
        version: i32,
        visited: &mut HashSet<i32>,
        resolved: &mut Vec<i32>,
        temp_visited: &mut HashSet<i32>,
    ) -> AppResult<()> {
        if temp_visited.contains(&version) {
            return Err(AppError::DatabaseMigration {
                version: format!("Circular dependency detected involving migration {}", version),
            });
        }

        if visited.contains(&version) {
            return Ok(());
        }

        temp_visited.insert(version);

        // Visit dependencies first
        if let Some(dependencies) = self.dependency_graph.get(&version) {
            for &dep_version in dependencies {
                if !self.migrations.contains_key(&dep_version) {
                    return Err(AppError::DatabaseMigration {
                        version: format!(
                            "Migration {} depends on non-existent migration {}",
                            version, dep_version
                        ),
                    });
                }
                self.visit_migration(dep_version, visited, resolved, temp_visited)?;
            }
        }

        temp_visited.remove(&version);
        visited.insert(version);
        resolved.push(version);

        Ok(())
    }

    /// Execute migrations from current version to target version with atomic transactions
    pub fn run_migrations(
        &mut self,
        conn: &Connection,
        from_version: i32,
        to_version: i32,
    ) -> AppResult<Vec<MigrationResult>> {
        info!("Running migrations from version {} to {}", from_version, to_version);

        // Ensure migration history table exists
        self.ensure_migration_history_table(conn)?;

        // Get currently applied migrations
        let applied_migrations = self.get_applied_migrations(conn)?;
        
        // Determine which migrations need to be run
        let target_migrations: Vec<i32> = self.migrations.keys()
            .filter(|&&v| v > from_version && v <= to_version)
            .cloned()
            .collect();

        if target_migrations.is_empty() {
            info!("No migrations to run");
            return Ok(Vec::new());
        }

        // Resolve dependencies
        let execution_order = self.resolve_dependencies(&target_migrations)?;
        
        // Filter out already applied migrations
        let pending_migrations: Vec<i32> = execution_order.into_iter()
            .filter(|v| !applied_migrations.contains(v))
            .collect();

        if pending_migrations.is_empty() {
            info!("All required migrations are already applied");
            return Ok(Vec::new());
        }

        // Initialize progress tracking
        self.init_progress(pending_migrations.len())?;

        let mut results = Vec::new();

        // Execute migrations in order with individual transactions
        for migration_version in pending_migrations {
            let migration = self.migrations.get(&migration_version).unwrap().clone();
            
            info!("Executing migration {}: {}", migration.version, migration.name);
            self.update_current_migration(&migration.name)?;

            let start_time = std::time::Instant::now();
            let result = self.execute_single_migration(conn, &migration);
            let execution_time = start_time.elapsed().as_millis() as u64;

            let migration_result = match &result {
                Ok(_) => {
                    info!("Migration {} completed successfully", migration.version);
                    self.record_migration_applied(conn, &migration)?;
                    self.increment_progress()?;
                    
                    MigrationResult {
                        version: migration.version,
                        name: migration.name.clone(),
                        success: true,
                        error_message: None,
                        execution_time_ms: execution_time,
                        applied_at: Utc::now(),
                    }
                }
                Err(e) => {
                    error!("Migration {} failed: {}", migration.version, e);
                    self.mark_migration_failed(&migration.name)?;
                    
                    MigrationResult {
                        version: migration.version,
                        name: migration.name.clone(),
                        success: false,
                        error_message: Some(e.to_string()),
                        execution_time_ms: execution_time,
                        applied_at: Utc::now(),
                    }
                }
            };

            results.push(migration_result.clone());
            self.add_result(migration_result)?;

            // Stop on first failure
            if result.is_err() {
                break;
            }
        }

        Ok(results)
    }

    /// Execute a single migration within a transaction
    fn execute_single_migration(&self, conn: &Connection, migration: &Migration) -> AppResult<()> {
        // Start transaction
        let tx = conn.unchecked_transaction()
            .map_err(|e| AppError::Database {
                message: format!("Failed to start transaction for migration {}: {}", migration.version, e),
            })?;

        // Execute migration SQL statements
        let statements: Vec<&str> = migration.up_sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for (i, statement) in statements.iter().enumerate() {
            debug!("Executing statement {}/{} for migration {}", i + 1, statements.len(), migration.version);
            
            if let Err(e) = tx.execute(statement, []) {
                let _ = tx.rollback();
                return Err(AppError::Database {
                    message: format!(
                        "Failed to execute statement in migration {}: {}. Statement: {}",
                        migration.version, e, statement
                    ),
                });
            }
        }

        // Commit transaction
        tx.commit().map_err(|e| AppError::Database {
            message: format!("Failed to commit migration {}: {}", migration.version, e),
        })?;

        Ok(())
    }

    /// Rollback migrations from current version to target version
    pub fn rollback_migrations(
        &mut self,
        conn: &Connection,
        from_version: i32,
        to_version: i32,
    ) -> AppResult<Vec<MigrationResult>> {
        info!("Rolling back migrations from version {} to {}", from_version, to_version);

        if to_version >= from_version {
            return Err(AppError::DatabaseMigration {
                version: "Target version must be less than current version for rollback".to_string(),
            });
        }

        // Get applied migrations in reverse order
        let applied_migrations = self.get_applied_migrations(conn)?;
        let rollback_migrations: Vec<i32> = applied_migrations.into_iter()
            .filter(|&v| v > to_version && v <= from_version)
            .collect::<Vec<_>>()
            .into_iter()
            .rev() // Reverse order for rollback
            .collect();

        if rollback_migrations.is_empty() {
            info!("No migrations to rollback");
            return Ok(Vec::new());
        }

        self.init_progress(rollback_migrations.len())?;
        let mut results = Vec::new();

        for migration_version in rollback_migrations {
            let migration = self.migrations.get(&migration_version).unwrap().clone();
            
            if migration.down_sql.trim().is_empty() {
                warn!("Migration {} has no rollback SQL, skipping", migration.version);
                continue;
            }

            info!("Rolling back migration {}: {}", migration.version, migration.name);
            self.update_current_migration(&format!("Rollback: {}", migration.name))?;

            let start_time = std::time::Instant::now();
            let result = self.execute_rollback_migration(conn, &migration);
            let execution_time = start_time.elapsed().as_millis() as u64;

            let migration_result = match &result {
                Ok(_) => {
                    info!("Migration {} rolled back successfully", migration.version);
                    self.remove_migration_record(conn, migration.version)?;
                    self.increment_progress()?;
                    
                    MigrationResult {
                        version: migration.version,
                        name: format!("Rollback: {}", migration.name),
                        success: true,
                        error_message: None,
                        execution_time_ms: execution_time,
                        applied_at: Utc::now(),
                    }
                }
                Err(e) => {
                    error!("Migration {} rollback failed: {}", migration.version, e);
                    self.mark_migration_failed(&format!("Rollback: {}", migration.name))?;
                    
                    MigrationResult {
                        version: migration.version,
                        name: format!("Rollback: {}", migration.name),
                        success: false,
                        error_message: Some(e.to_string()),
                        execution_time_ms: execution_time,
                        applied_at: Utc::now(),
                    }
                }
            };

            results.push(migration_result.clone());
            self.add_result(migration_result)?;

            if result.is_err() {
                break;
            }
        }

        Ok(results)
    }

    /// Execute rollback for a single migration
    fn execute_rollback_migration(&self, conn: &Connection, migration: &Migration) -> AppResult<()> {
        let tx = conn.unchecked_transaction()
            .map_err(|e| AppError::Database {
                message: format!("Failed to start rollback transaction for migration {}: {}", migration.version, e),
            })?;

        let statements: Vec<&str> = migration.down_sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for statement in statements {
            if let Err(e) = tx.execute(statement, []) {
                let _ = tx.rollback();
                return Err(AppError::Database {
                    message: format!(
                        "Failed to execute rollback statement for migration {}: {}",
                        migration.version, e
                    ),
                });
            }
        }

        tx.commit().map_err(|e| AppError::Database {
            message: format!("Failed to commit rollback for migration {}: {}", migration.version, e),
        })?;

        Ok(())
    }

    /// Ensure the migration history table exists
    fn ensure_migration_history_table(&self, conn: &Connection) -> AppResult<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS migration_history (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                checksum TEXT NOT NULL,
                applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                execution_time_ms INTEGER NOT NULL DEFAULT 0
            );
            
            CREATE INDEX IF NOT EXISTS idx_migration_history_applied_at 
            ON migration_history(applied_at);
        "#;

        conn.execute_batch(sql)
            .map_err(|e| AppError::Database {
                message: format!("Failed to create migration history table: {}", e),
            })?;

        Ok(())
    }

    /// Get list of applied migration versions
    fn get_applied_migrations(&self, conn: &Connection) -> AppResult<Vec<i32>> {
        let mut stmt = conn.prepare("SELECT version FROM migration_history ORDER BY version")
            .map_err(|e| AppError::Database {
                message: format!("Failed to prepare applied migrations query: {}", e),
            })?;

        let versions: Vec<i32> = stmt.query_map([], |row| Ok(row.get::<_, i32>(0)?))
            .map_err(|e| AppError::Database {
                message: format!("Failed to query applied migrations: {}", e),
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Database {
                message: format!("Failed to collect applied migrations: {}", e),
            })?;

        Ok(versions)
    }

    /// Record a migration as applied
    fn record_migration_applied(&self, conn: &Connection, migration: &Migration) -> AppResult<()> {
        conn.execute(
            "INSERT INTO migration_history (version, name, description, checksum, applied_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &migration.version.to_string(),
                &migration.name,
                &migration.description,
                &migration.checksum,
                &Utc::now().to_rfc3339(),
            ],
        ).map_err(|e| AppError::Database {
            message: format!("Failed to record migration {} as applied: {}", migration.version, e),
        })?;

        Ok(())
    }

    /// Remove migration record (for rollbacks)
    fn remove_migration_record(&self, conn: &Connection, version: i32) -> AppResult<()> {
        conn.execute(
            "DELETE FROM migration_history WHERE version = ?1",
            [version],
        ).map_err(|e| AppError::Database {
            message: format!("Failed to remove migration {} record: {}", version, e),
        })?;

        Ok(())
    }

    /// Initialize progress tracking
    fn init_progress(&self, total: usize) -> AppResult<()> {
        let mut progress = self.progress.lock()
            .map_err(|_| AppError::internal("Failed to acquire progress lock"))?;
        
        progress.total_migrations = total;
        progress.completed_migrations = 0;
        progress.current_migration = None;
        progress.failed_migration = None;
        progress.start_time = Utc::now();
        progress.estimated_completion = None;

        Ok(())
    }

    /// Update current migration being processed
    fn update_current_migration(&self, name: &str) -> AppResult<()> {
        let mut progress = self.progress.lock()
            .map_err(|_| AppError::internal("Failed to acquire progress lock"))?;
        
        progress.current_migration = Some(name.to_string());
        Ok(())
    }

    /// Mark a migration as failed
    fn mark_migration_failed(&self, name: &str) -> AppResult<()> {
        let mut progress = self.progress.lock()
            .map_err(|_| AppError::internal("Failed to acquire progress lock"))?;
        
        progress.failed_migration = Some(name.to_string());
        Ok(())
    }

    /// Increment progress counter
    fn increment_progress(&self) -> AppResult<()> {
        let mut progress = self.progress.lock()
            .map_err(|_| AppError::internal("Failed to acquire progress lock"))?;
        
        progress.completed_migrations += 1;
        
        // Estimate completion time
        if progress.total_migrations > 0 {
            let elapsed = Utc::now().signed_duration_since(progress.start_time);
            let remaining = progress.total_migrations - progress.completed_migrations;
            
            if remaining > 0 && progress.completed_migrations > 0 {
                let avg_time_per_migration = elapsed.num_milliseconds() / progress.completed_migrations as i64;
                let estimated_remaining_ms = avg_time_per_migration * remaining as i64;
                
                progress.estimated_completion = Some(
                    Utc::now() + chrono::Duration::milliseconds(estimated_remaining_ms)
                );
            }
        }

        Ok(())
    }

    /// Add migration result to results list
    fn add_result(&self, result: MigrationResult) -> AppResult<()> {
        let mut results = self.results.lock()
            .map_err(|_| AppError::internal("Failed to acquire results lock"))?;
        
        results.push(result);
        Ok(())
    }

    /// Validate all migrations for consistency and integrity
    pub fn validate_migrations(&self) -> AppResult<()> {
        info!("Validating {} migrations", self.migrations.len());

        // Check for gaps in version numbers
        let mut versions: Vec<i32> = self.migrations.keys().cloned().collect();
        versions.sort();

        for (i, &version) in versions.iter().enumerate() {
            if i > 0 && version != versions[i - 1] + 1 {
                warn!("Gap detected in migration versions: {} -> {}", versions[i - 1], version);
            }
        }

        // Validate each migration
        for migration in self.migrations.values() {
            migration.validate_checksum()?;
            
            // Check dependencies exist
            for &dep_version in &migration.dependencies {
                if !self.migrations.contains_key(&dep_version) {
                    return Err(AppError::DatabaseMigration {
                        version: format!(
                            "Migration {} depends on non-existent migration {}",
                            migration.version, dep_version
                        ),
                    });
                }
            }
        }

        // Check for circular dependencies
        for &version in versions.iter() {
            let mut visited = HashSet::new();
            let mut resolved = Vec::new();
            let mut temp_visited = HashSet::new();
            
            self.visit_migration(version, &mut visited, &mut resolved, &mut temp_visited)?;
        }

        info!("All migrations validated successfully");
        Ok(())
    }

    /// Get migration by version
    pub fn get_migration(&self, version: i32) -> Option<&Migration> {
        self.migrations.get(&version)
    }

    /// Get all migrations sorted by version
    pub fn get_all_migrations(&self) -> Vec<&Migration> {
        let mut migrations: Vec<&Migration> = self.migrations.values().collect();
        migrations.sort_by_key(|m| m.version);
        migrations
    }

    /// Check if migration exists
    pub fn has_migration(&self, version: i32) -> bool {
        self.migrations.contains_key(&version)
    }

    /// Get migration count
    pub fn migration_count(&self) -> usize {
        self.migrations.len()
    }
}

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migration_creation() {
        let migration = Migration::new(
            1,
            "test_migration".to_string(),
            "Test migration".to_string(),
            "CREATE TABLE test (id INTEGER);".to_string(),
            "DROP TABLE test;".to_string(),
            vec![],
        );

        assert_eq!(migration.version, 1);
        assert_eq!(migration.name, "test_migration");
        assert!(!migration.checksum.is_empty());
        assert!(!migration.is_applied());
    }

    #[test]
    fn test_checksum_validation() {
        let migration = Migration::new(
            1,
            "test".to_string(),
            "Test".to_string(),
            "CREATE TABLE test (id INTEGER);".to_string(),
            "DROP TABLE test;".to_string(),
            vec![],
        );

        assert!(migration.validate_checksum().is_ok());
    }

    #[test]
    fn test_dependency_resolution() {
        let mut runner = MigrationRunner::new();
        
        let migration1 = Migration::new(
            1,
            "migration1".to_string(),
            "First migration".to_string(),
            "CREATE TABLE users (id INTEGER);".to_string(),
            "DROP TABLE users;".to_string(),
            vec![],
        );

        let migration2 = Migration::new(
            2,
            "migration2".to_string(),
            "Second migration".to_string(),
            "CREATE TABLE posts (id INTEGER, user_id INTEGER);".to_string(),
            "DROP TABLE posts;".to_string(),
            vec![1],
        );

        runner.add_migration(migration1).unwrap();
        runner.add_migration(migration2).unwrap();

        let order = runner.resolve_dependencies(&[2]).unwrap();
        assert_eq!(order, vec![1, 2]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut runner = MigrationRunner::new();
        
        let migration1 = Migration::new(
            1,
            "migration1".to_string(),
            "First migration".to_string(),
            "CREATE TABLE test1 (id INTEGER);".to_string(),
            "DROP TABLE test1;".to_string(),
            vec![2],
        );

        let migration2 = Migration::new(
            2,
            "migration2".to_string(),
            "Second migration".to_string(),
            "CREATE TABLE test2 (id INTEGER);".to_string(),
            "DROP TABLE test2;".to_string(),
            vec![1],
        );

        runner.add_migration(migration1).unwrap();
        runner.add_migration(migration2).unwrap();

        // This should detect a circular dependency
        let result = runner.resolve_dependencies(&[1, 2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_migration_history_table_creation() {
        let conn = Connection::open_in_memory().unwrap();
        let runner = MigrationRunner::new();
        
        assert!(runner.ensure_migration_history_table(&conn).is_ok());
        
        // Verify table exists
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='migration_history'").unwrap();
        let table_exists = stmt.exists([]).unwrap();
        assert!(table_exists);
    }

    #[test]
    fn test_migration_runner_validation() {
        let mut runner = MigrationRunner::new();
        
        let migration = Migration::new(
            1,
            "test_migration".to_string(),
            "Test migration".to_string(),
            "CREATE TABLE test (id INTEGER);".to_string(),
            "DROP TABLE test;".to_string(),
            vec![],
        );

        runner.add_migration(migration).unwrap();
        assert!(runner.validate_migrations().is_ok());
    }
}
