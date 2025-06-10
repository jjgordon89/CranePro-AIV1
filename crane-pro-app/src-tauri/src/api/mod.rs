//! API module for CranePro Bridge Inspection Application
//! 
//! This module contains request/response DTOs and common API types
//! for communication between the frontend and backend via Tauri IPC.

use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod requests;
pub mod responses;

// Re-export common types
pub use requests::*;
pub use responses::*;

/// Standard API response wrapper for all command handlers
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum ApiResponse<T> {
    #[serde(rename = "success")]
    Success(T),
    #[serde(rename = "error")]
    Error(ApiError),
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self::Success(data)
    }

    pub fn error(error: AppError) -> Self {
        Self::Error(ApiError::from(error))
    }
}

/// API-specific error type for frontend consumption
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, String>>,
}

impl From<AppError> for ApiError {
    fn from(app_error: AppError) -> Self {
        Self {
            code: app_error.category().to_string(),
            message: app_error.to_string(),
            details: None,
        }
    }
}

/// Date range for filtering and reporting
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateRange {
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
}

/// Report format options
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ReportFormat {
    #[serde(rename = "pdf")]
    Pdf,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "csv")]
    Csv,
}

/// Report generation result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportResult {
    pub report_id: String,
    pub format: ReportFormat,
    pub file_path: Option<String>,
    pub file_url: Option<String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Report template metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub supported_formats: Vec<ReportFormat>,
    pub parameters: Vec<ReportParameter>,
}

/// Report parameter definition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportParameter {
    pub name: String,
    pub parameter_type: String,
    pub required: bool,
    pub description: String,
    pub default_value: Option<String>,
}

/// Compliance status overview
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplianceStatus {
    pub asset_id: i64,
    pub overall_status: String,
    pub compliance_score: f64,
    pub last_inspection_date: Option<chrono::DateTime<chrono::Utc>>,
    pub next_required_inspection: Option<chrono::DateTime<chrono::Utc>>,
    pub critical_findings: i64,
    pub pending_actions: i64,
}

/// Compliance requirement for upcoming inspections
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplianceRequirement {
    pub asset_id: i64,
    pub asset_name: String,
    pub requirement_type: String,
    pub due_date: chrono::DateTime<chrono::Utc>,
    pub priority: String,
    pub description: String,
}

/// Login response with user session data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub user: crate::models::User,
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub permissions: Vec<String>,
    pub session_id: String,
}

/// Common query filter for paginated endpoints
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryFilterRequest {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub search: Option<String>,
    pub filters: Option<HashMap<String, String>>,
}

impl From<QueryFilterRequest> for crate::models::QueryFilter {
    fn from(req: QueryFilterRequest) -> Self {
        let sort_order = req.sort_order.and_then(|s| match s.to_lowercase().as_str() {
            "asc" => Some(crate::models::SortOrder::Asc),
            "desc" => Some(crate::models::SortOrder::Desc),
            _ => None,
        });

        Self {
            page: req.page,
            limit: req.limit,
            sort_by: req.sort_by,
            sort_order,
            filters: req.filters.unwrap_or_default(),
        }
    }
}