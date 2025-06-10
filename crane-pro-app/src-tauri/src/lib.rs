use log::info;
use std::sync::Arc;
use tauri::Manager;

// Module declarations
pub mod database;
pub mod services;
pub mod security;
pub mod models;
pub mod errors;
pub mod api;
pub mod middleware;
pub mod commands;

use crate::errors::AppResult;
use crate::database::Database;
use crate::services::Services;
use crate::middleware::auth::AuthManager;
use crate::commands::AppState;

// Import all command handlers
use crate::commands::{
    // Asset commands
    create_asset_command, get_asset_command, get_assets_by_location_command,
    update_asset_command, delete_asset_command, search_assets_command,
    get_asset_components_command, create_component_command, update_component_command,
    
    // Inspection commands
    create_inspection_command, get_inspection_command, update_inspection_command,
    submit_inspection_command, get_inspections_by_asset_command, get_pending_inspections_command,
    create_inspection_item_command, update_inspection_item_command, get_inspection_items_command,
    
    // Compliance commands
    create_compliance_record_command, get_compliance_record_command, get_compliance_records_by_asset_command,
    update_compliance_record_command, get_compliance_status_command, get_upcoming_requirements_command,
    mark_compliance_complete_command,
    
    // User commands
    create_user_command, get_user_command, get_current_user_command, update_user_command,
    delete_user_command, login_command, logout_command, get_users_command, change_password_command,
    
    // Media commands
    upload_file_command, get_file_command, get_files_by_inspection_command, delete_file_command,
    get_file_url_command, upload_inspection_photo_command, get_inspection_photos_command,
    
    // Report commands
    generate_inspection_report_command, generate_compliance_report_command, get_report_command,
    list_available_reports_command,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Health check command for system status
#[tauri::command]
async fn health_check() -> AppResult<String> {
    info!("Health check requested");
    Ok("CranePro Backend is running".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    env_logger::init();
    info!("Starting CranePro Bridge Inspection Application");

    tauri::Builder::default()
        // Core plugins
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        // .plugin(tauri_plugin_stronghold::Builder::new(|_| Ok(())).build()) // Temporarily disabled
        
        // Setup handler for app initialization
        .setup(|app| {
            info!("Initializing CranePro application...");
            
            // Initialize database
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            let database = rt.block_on(async {
                Database::new_in_memory().await
                    .expect("Failed to initialize database")
            });
            let database = Arc::new(database);
            
            // Initialize services
            let services = rt.block_on(async {
                Services::init(database.clone()).await
                    .expect("Failed to initialize services")
            });
            let services = Arc::new(services);
            
            // Initialize authentication manager
            let jwt_secret = std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default-secret-key-change-in-production".to_string());
            let auth_manager = Arc::new(AuthManager::new(services.clone(), &jwt_secret));
            
            // Create app state
            let app_state = AppState::new(services, auth_manager);
            
            // Manage state
            app.manage(app_state);
            
            info!("Application initialization completed");
            Ok(())
        })
        
        // Register all command handlers
        .invoke_handler(tauri::generate_handler![
            // Core commands
            greet,
            health_check,
            
            // Asset management commands (9 commands)
            create_asset_command,
            get_asset_command,
            get_assets_by_location_command,
            update_asset_command,
            delete_asset_command,
            search_assets_command,
            get_asset_components_command,
            create_component_command,
            update_component_command,
            
            // Inspection management commands (9 commands)
            create_inspection_command,
            get_inspection_command,
            update_inspection_command,
            submit_inspection_command,
            get_inspections_by_asset_command,
            get_pending_inspections_command,
            create_inspection_item_command,
            update_inspection_item_command,
            get_inspection_items_command,
            
            // Compliance management commands (7 commands)
            create_compliance_record_command,
            get_compliance_record_command,
            get_compliance_records_by_asset_command,
            update_compliance_record_command,
            get_compliance_status_command,
            get_upcoming_requirements_command,
            mark_compliance_complete_command,
            
            // User management commands (9 commands)
            create_user_command,
            get_user_command,
            get_current_user_command,
            update_user_command,
            delete_user_command,
            login_command,
            logout_command,
            get_users_command,
            change_password_command,
            
            // Media management commands (7 commands)
            upload_file_command,
            get_file_command,
            get_files_by_inspection_command,
            delete_file_command,
            get_file_url_command,
            upload_inspection_photo_command,
            get_inspection_photos_command,
            
            // Report generation commands (4 commands)
            generate_inspection_report_command,
            generate_compliance_report_command,
            get_report_command,
            list_available_reports_command,
        ])
        
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
