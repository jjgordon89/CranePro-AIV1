//! Command handlers module for CranePro Bridge Inspection Application
//! 
//! This module contains all Tauri command handlers that expose backend
//! functionality to the frontend via IPC (Inter-Process Communication).

pub mod asset_commands;
pub mod inspection_commands;
pub mod compliance_commands;
pub mod user_commands;
pub mod media_commands;
pub mod report_commands;
pub mod location_commands;

// Re-export all command handlers for easy registration
pub use asset_commands::*;
pub use inspection_commands::*;
pub use compliance_commands::*;
pub use user_commands::*;
pub use media_commands::*;
pub use report_commands::*;
pub use location_commands::*;

use crate::api::ApiResponse;
use crate::errors::AppError;
use crate::services::Services;
use crate::middleware::auth::AuthManager;
use std::sync::Arc;
use log::{info, error, debug};

/// Shared state for command handlers
#[derive(Clone)]
pub struct AppState {
    pub services: Arc<Services>,
    pub auth_manager: Arc<AuthManager>,
}

impl AppState {
    pub fn new(services: Arc<Services>, auth_manager: Arc<AuthManager>) -> Self {
        Self {
            services,
            auth_manager,
        }
    }
}

/// Helper function to convert AppError to ApiResponse
pub fn handle_error<T>(result: Result<T, AppError>) -> ApiResponse<T> {
    match result {
        Ok(data) => ApiResponse::success(data),
        Err(error) => {
            error!("Command execution failed: {}", error);
            ApiResponse::error(error)
        }
    }
}

/// Helper function for logging command execution
pub fn log_command_start(command_name: &str, user_id: Option<i64>) {
    if let Some(user_id) = user_id {
        info!("Executing command '{}' for user {}", command_name, user_id);
    } else {
        info!("Executing command '{}' (unauthenticated)", command_name);
    }
}

/// Helper function for logging command completion
pub fn log_command_end(command_name: &str, success: bool, duration_ms: u64) {
    if success {
        debug!("Command '{}' completed successfully in {}ms", command_name, duration_ms);
    } else {
        error!("Command '{}' failed after {}ms", command_name, duration_ms);
    }
}

/// Macro for timing command execution
#[macro_export]
macro_rules! time_command {
    ($command_name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        let success = result.is_ok();
        crate::commands::log_command_end($command_name, success, duration.as_millis() as u64);
        result
    }};
}

/// Macro for wrapping command handlers with error handling and logging
#[macro_export]
macro_rules! command_handler {
    ($name:expr, $user_id:expr, $body:block) => {{
        crate::commands::log_command_start($name, $user_id);
        let result = crate::time_command!($name, $body);
        crate::commands::handle_error(result)
    }};
}

/// Response wrapper that includes metadata
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CommandResponse<T> {
    pub data: T,
    pub metadata: ResponseMetadata,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ResponseMetadata {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: String,
    pub version: String,
}

impl<T> CommandResponse<T> {
    pub fn new(data: T, request_id: String) -> Self {
        Self {
            data,
            metadata: ResponseMetadata {
                timestamp: chrono::Utc::now(),
                request_id,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}

/// Helper trait for adding metadata to responses
pub trait WithMetadata<T> {
    fn with_metadata(self, request_id: String) -> CommandResponse<T>;
}

impl<T> WithMetadata<T> for T {
    fn with_metadata(self, request_id: String) -> CommandResponse<T> {
        CommandResponse::new(self, request_id)
    }
}