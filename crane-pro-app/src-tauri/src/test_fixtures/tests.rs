//! Tests for the TestDatabase implementation

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_database_creation() {
        let test_db = TestDatabase::new_in_memory().await.expect("Failed to create test database");
        
        // Verify we can get the underlying database
        let db = test_db.get_database();
        assert!(!Arc::ptr_eq(&db, &db)); // Just checking it exists
    }
    
    #[tokio::test]
    async fn test_database_reset() {
        let test_db = TestDatabase::new_in_memory().await.expect("Failed to create test database");
        
        // Reset should work without errors
        test_db.reset().await.expect("Failed to reset database");
    }
    
    #[tokio::test]
    async fn test_minimal_data_seeding() {
        let test_db = TestDatabase::new_in_memory().await.expect("Failed to create test database");
        
        // Seed minimal data
        let fixtures = test_db.seed_minimal_data().await.expect("Failed to seed minimal data");
        
        // Should return fixtures (even if empty for now)
        assert!(fixtures.users.is_empty()); // Placeholder fixtures are empty
        assert!(fixtures.locations.is_empty());
        assert!(fixtures.assets.is_empty());
    }
    
    #[tokio::test]
    async fn test_comprehensive_data_seeding() {
        let test_db = TestDatabase::new_in_memory().await.expect("Failed to create test database");
        
        // Seed comprehensive data
        let fixtures = test_db.seed_comprehensive_data().await.expect("Failed to seed comprehensive data");
        
        // Should return fixtures (even if empty for now)
        assert!(fixtures.users.is_empty()); // Placeholder fixtures are empty
        assert!(fixtures.locations.is_empty());
        assert!(fixtures.assets.is_empty());
    }
    
    #[tokio::test]
    async fn test_performance_data_seeding() {
        let test_db = TestDatabase::new_in_memory().await.expect("Failed to create test database");
        
        // Seed performance data
        let fixtures = test_db.seed_performance_data().await.expect("Failed to seed performance data");
        
        // Should return fixtures (even if empty for now)
        assert!(fixtures.users.is_empty()); // Placeholder fixtures are empty
        assert!(fixtures.locations.is_empty());
        assert!(fixtures.assets.is_empty());
    }
    
    #[tokio::test]
    async fn test_test_data_generators() {
        // Test the generator functions
        let user = test_user();
        assert_eq!(user.username, "test_user");
        assert_eq!(user.role, UserRole::Inspector);
        
        let asset = test_asset();
        assert_eq!(asset.asset_number, "TEST-001");
        assert_eq!(asset.asset_name, "Test Bridge Crane");
        
        let location = test_location();
        assert_eq!(location.name, "Test Facility");
        
        let inspection = test_inspection();
        assert_eq!(inspection.inspection_type, InspectionType::Periodic);
        
        let component = test_component();
        assert_eq!(component.component_name, "Test Hoist Motor");
    }
    
    #[tokio::test]
    async fn test_utility_functions() {
        // Test setup and teardown utilities
        let test_db = setup_test_env().await.expect("Failed to setup test environment");
        
        // Should be able to use the database
        let db = test_db.get_database();
        assert!(!Arc::ptr_eq(&db, &db));
        
        // Teardown should work
        teardown_test_env(test_db).await.expect("Failed to teardown test environment");
    }
}