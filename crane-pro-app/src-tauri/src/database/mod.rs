//! Database module for CranePro Bridge Inspection Application
//!
//! This module provides enhanced database functionality including:
//! - Core database operations with connection pooling
//! - Advanced migration infrastructure with dependency resolution
//! - Rollback capabilities and integrity checking
//! - Progress tracking and detailed logging
//! - Thread-safe migration operations

pub mod core;
pub mod migrations;

// Export core database functionality (for backward compatibility)
pub use core::{Database, DatabasePool, LegacyMigration, LegacyMigrationManager};

// Export enhanced migration infrastructure
pub use migrations::{Migration, MigrationRunner, MigrationResult, MigrationProgress};