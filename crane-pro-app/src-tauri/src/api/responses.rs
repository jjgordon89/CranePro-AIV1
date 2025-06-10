//! Response DTOs for API commands
//! 
//! This module contains all response structures used by Tauri command handlers
//! to send data to the frontend.

use crate::models::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;

// =============================================================================
// Common Response Types
// =============================================================================

/// Paginated response wrapper
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginationMeta {
    pub total_count: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

impl<T> From<PaginatedResult<T>> for PaginatedResponse<T> {
    fn from(result: PaginatedResult<T>) -> Self {
        let has_next_page = result.page < result.total_pages;
        let has_previous_page = result.page > 1;

        Self {
            data: result.data,
            pagination: PaginationMeta {
                total_count: result.total_count,
                page: result.page,
                limit: result.limit,
                total_pages: result.total_pages,
                has_next_page,
                has_previous_page,
            },
        }
    }
}

/// Simple success response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuccessResponse {
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl SuccessResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            timestamp: Utc::now(),
        }
    }
}

// =============================================================================
// Asset Management Responses
// =============================================================================

/// Asset response with computed fields
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetResponse {
    #[serde(flatten)]
    pub asset: Asset,
    pub component_count: Option<i64>,
    pub last_inspection_date: Option<DateTime<Utc>>,
    pub next_inspection_date: Option<DateTime<Utc>>,
    pub compliance_score: Option<f64>,
}

impl From<Asset> for AssetResponse {
    fn from(asset: Asset) -> Self {
        Self {
            asset,
            component_count: None,
            last_inspection_date: None,
            next_inspection_date: None,
            compliance_score: None,
        }
    }
}

/// Component response with parent/child relationships
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentResponse {
    #[serde(flatten)]
    pub component: Component,
    pub asset_name: Option<String>,
    pub parent_component_name: Option<String>,
    pub child_components: Option<Vec<Component>>,
}

impl From<Component> for ComponentResponse {
    fn from(component: Component) -> Self {
        Self {
            component,
            asset_name: None,
            parent_component_name: None,
            child_components: None,
        }
    }
}

// =============================================================================
// Inspection Management Responses
// =============================================================================

/// Inspection response with related data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InspectionResponse {
    #[serde(flatten)]
    pub inspection: Inspection,
    pub asset_name: Option<String>,
    pub inspector_name: Option<String>,
    pub item_count: Option<i64>,
    pub compliance_score: Option<f64>,
    pub critical_findings_count: Option<i64>,
}

impl From<Inspection> for InspectionResponse {
    fn from(inspection: Inspection) -> Self {
        Self {
            inspection,
            asset_name: None,
            inspector_name: None,
            item_count: None,
            compliance_score: None,
            critical_findings_count: None,
        }
    }
}

/// Inspection item response with component details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InspectionItemResponse {
    #[serde(flatten)]
    pub item: InspectionItem,
    pub component_name: Option<String>,
    pub component_type: Option<String>,
}

impl From<InspectionItem> for InspectionItemResponse {
    fn from(item: InspectionItem) -> Self {
        Self {
            item,
            component_name: None,
            component_type: None,
        }
    }
}

// =============================================================================
// Compliance Management Responses
// =============================================================================

/// Compliance record response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplianceRecordResponse {
    pub id: i64,
    pub asset_id: i64,
    pub asset_name: String,
    pub standard_id: i64,
    pub standard_name: String,
    pub compliance_status: String,
    pub last_inspection_date: Option<DateTime<Utc>>,
    pub next_inspection_date: Option<DateTime<Utc>>,
    pub compliance_score: f64,
    pub findings: Option<JsonValue>,
    pub corrective_actions: Option<JsonValue>,
    pub verified_by: i64,
    pub verified_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compliance status summary
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplianceStatusResponse {
    pub asset_id: i64,
    pub asset_name: String,
    pub overall_status: String,
    pub compliance_score: f64,
    pub last_inspection_date: Option<DateTime<Utc>>,
    pub next_required_inspection: Option<DateTime<Utc>>,
    pub critical_findings: i64,
    pub pending_actions: i64,
    pub standards: Vec<ComplianceStandardStatus>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplianceStandardStatus {
    pub standard_code: String,
    pub standard_name: String,
    pub status: String,
    pub compliance_score: f64,
    pub last_check: Option<DateTime<Utc>>,
}

/// Upcoming compliance requirements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplianceRequirementResponse {
    pub asset_id: i64,
    pub asset_name: String,
    pub asset_type: String,
    pub requirement_type: String,
    pub due_date: DateTime<Utc>,
    pub days_until_due: i64,
    pub priority: String,
    pub description: String,
    pub assigned_inspector: Option<String>,
}

// =============================================================================
// User Management Responses
// =============================================================================

/// User response without sensitive data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub inspection_count: Option<i64>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            first_name: user.first_name,
            last_name: user.last_name,
            phone: user.phone,
            created_at: user.created_at,
            updated_at: user.updated_at,
            is_active: user.is_active,
            last_login: None,
            inspection_count: None,
        }
    }
}

/// Login response with session information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub permissions: Vec<String>,
    pub session_id: String,
}

// =============================================================================
// Media Management Responses
// =============================================================================

/// Media file response with additional metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaFileResponse {
    #[serde(flatten)]
    pub media_file: MediaFile,
    pub inspection_name: Option<String>,
    pub component_name: Option<String>,
    pub file_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub ai_analysis_status: Option<String>,
    pub ai_confidence_score: Option<f64>,
}

impl From<MediaFile> for MediaFileResponse {
    fn from(media_file: MediaFile) -> Self {
        Self {
            media_file,
            inspection_name: None,
            component_name: None,
            file_url: None,
            thumbnail_url: None,
            ai_analysis_status: None,
            ai_confidence_score: None,
        }
    }
}

/// File upload response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadResponse {
    pub file_id: i64,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub file_url: String,
    pub uploaded_at: DateTime<Utc>,
}

// =============================================================================
// Report Management Responses
// =============================================================================

/// Report generation response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportResponse {
    pub report_id: String,
    pub report_type: String,
    pub format: String,
    pub status: String,
    pub file_path: Option<String>,
    pub file_url: Option<String>,
    pub generated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub parameters: JsonValue,
}

/// Available report templates
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportTemplateResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub supported_formats: Vec<String>,
    pub parameters: Vec<ReportParameterResponse>,
    pub estimated_generation_time: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportParameterResponse {
    pub name: String,
    pub parameter_type: String,
    pub required: bool,
    pub description: String,
    pub default_value: Option<String>,
    pub validation_rules: Option<JsonValue>,
}

// =============================================================================
// Dashboard and Analytics Responses
// =============================================================================

/// Dashboard summary statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardStatsResponse {
    pub total_assets: i64,
    pub active_inspections: i64,
    pub overdue_inspections: i64,
    pub compliance_score: f64,
    pub critical_findings: i64,
    pub pending_maintenance: i64,
    pub recent_activities: Vec<ActivityResponse>,
    pub upcoming_inspections: Vec<UpcomingInspectionResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActivityResponse {
    pub id: String,
    pub activity_type: String,
    pub description: String,
    pub user_name: String,
    pub timestamp: DateTime<Utc>,
    pub related_entity_type: String,
    pub related_entity_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpcomingInspectionResponse {
    pub inspection_id: i64,
    pub asset_name: String,
    pub asset_type: String,
    pub inspection_type: String,
    pub scheduled_date: DateTime<Utc>,
    pub inspector_name: Option<String>,
    pub priority: String,
    pub days_until_due: i64,
}

// =============================================================================
// Search and Filter Responses
// =============================================================================

/// Search results with highlighting
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResponse<T> {
    pub results: Vec<T>,
    pub total_count: i64,
    pub search_time_ms: i64,
    pub suggestions: Vec<String>,
    pub facets: Option<JsonValue>,
}

/// Filter options for dynamic filtering
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterOptionsResponse {
    pub asset_types: Vec<String>,
    pub asset_statuses: Vec<String>,
    pub inspection_types: Vec<String>,
    pub inspection_statuses: Vec<String>,
    pub user_roles: Vec<String>,
    pub compliance_standards: Vec<String>,
    pub severity_levels: Vec<String>,
    pub locations: Vec<LocationFilterOption>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationFilterOption {
    pub id: i64,
    pub name: String,
    pub asset_count: i64,
}