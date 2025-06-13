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

// =============================================================================
// Enhanced Asset Management Responses
// =============================================================================

/// Enhanced asset summary response with computed fields for frontend display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetSummaryResponse {
    #[serde(flatten)]
    pub summary: crate::services::AssetSummary,
    // Enhanced fields for frontend
    /// Color-coded status indicator (green, yellow, red)
    pub status_indicator: String,
    /// Human-readable compliance level (Excellent, Good, Fair, Poor)
    pub compliance_level: String,
    /// Next action required with description
    pub next_action_required: Option<String>,
    /// Formatted date strings for display
    pub formatted_dates: FormattedDates,
    /// Human-readable maintenance status
    pub maintenance_status: String,
}

/// Formatted date strings for frontend display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormattedDates {
    pub last_inspection: Option<String>,
    pub next_inspection: Option<String>,
    pub last_maintenance: Option<String>,
    pub next_maintenance: Option<String>,
}

impl From<crate::services::AssetSummary> for AssetSummaryResponse {
    fn from(summary: crate::services::AssetSummary) -> Self {
        // Determine status indicator color
        let status_indicator = match summary.status {
            AssetStatus::Active => "green".to_string(),
            AssetStatus::Maintenance => "yellow".to_string(),
            AssetStatus::Decommissioned => "gray".to_string(),
            AssetStatus::Inactive => "red".to_string(),
        };

        // Determine compliance level
        let compliance_level = if summary.compliance_score >= 90.0 {
            "Excellent".to_string()
        } else if summary.compliance_score >= 80.0 {
            "Good".to_string()
        } else if summary.compliance_score >= 60.0 {
            "Fair".to_string()
        } else {
            "Poor".to_string()
        };

        // Determine next action required
        let next_action_required = if summary.pending_inspections > 0 {
            Some("Schedule pending inspection".to_string())
        } else if summary.critical_findings_count > 0 {
            Some("Address critical findings".to_string())
        } else if summary.next_inspection_date.map_or(false, |date| date <= Utc::now() + chrono::Duration::days(7)) {
            Some("Upcoming inspection due".to_string())
        } else {
            None
        };

        // Format dates for display
        let formatted_dates = FormattedDates {
            last_inspection: summary.last_inspection_date.map(|d| d.format("%Y-%m-%d").to_string()),
            next_inspection: summary.next_inspection_date.map(|d| d.format("%Y-%m-%d").to_string()),
            last_maintenance: summary.last_maintenance_date.map(|d| d.format("%Y-%m-%d").to_string()),
            next_maintenance: summary.next_maintenance_date.map(|d| d.format("%Y-%m-%d").to_string()),
        };

        // Determine maintenance status
        let maintenance_status = if summary.maintenance_records_count == 0 {
            "No maintenance history".to_string()
        } else if summary.last_maintenance_date.map_or(true, |date| date < Utc::now() - chrono::Duration::days(365)) {
            "Maintenance overdue".to_string()
        } else {
            "Up to date".to_string()
        };

        Self {
            summary,
            status_indicator,
            compliance_level,
            next_action_required,
            formatted_dates,
            maintenance_status,
        }
    }
}

/// Enhanced bulk import result response with user-friendly information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BulkImportResultResponse {
    #[serde(flatten)]
    pub result: crate::services::BulkImportResult,
    // Enhanced fields for frontend
    /// Success rate as a percentage (0.0 to 100.0)
    pub success_rate: f64,
    /// Processing time in milliseconds
    pub processing_time_ms: i64,
    /// User-friendly error messages with suggestions
    pub user_friendly_errors: Vec<UserFriendlyError>,
    /// Overall summary message for the import operation
    pub summary_message: String,
    /// Recommendations for addressing failures
    pub recommendations: Vec<String>,
}

/// User-friendly error information for failed imports
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserFriendlyError {
    /// Reference to the asset that failed (asset number or identifier)
    pub asset_reference: String,
    /// Category of error (validation, duplicate, system)
    pub error_type: String,
    /// Human-readable error message
    pub message: String,
    /// Suggested action to resolve the error
    pub suggestion: Option<String>,
    /// Field that caused the error (if applicable)
    pub field: Option<String>,
}

impl From<crate::services::BulkImportResult> for BulkImportResultResponse {
    fn from(result: crate::services::BulkImportResult) -> Self {
        let success_rate = if result.total_processed > 0 {
            (result.successful_imports as f64 / result.total_processed as f64) * 100.0
        } else {
            0.0
        };

        // Generate user-friendly errors
        let mut user_friendly_errors = Vec::new();
        let mut recommendations = Vec::new();
        
        for asset_result in &result.results {
            if !asset_result.success {
                if let Some(error_msg) = &asset_result.error_message {
                    let (error_type, suggestion) = if error_msg.contains("duplicate") {
                        ("duplicate".to_string(), Some("Check for existing asset with same number".to_string()))
                    } else if error_msg.contains("location") {
                        ("validation".to_string(), Some("Verify location exists or create it first".to_string()))
                    } else if error_msg.contains("validation") {
                        ("validation".to_string(), Some("Check required fields are populated".to_string()))
                    } else {
                        ("system".to_string(), Some("Contact system administrator".to_string()))
                    };

                    user_friendly_errors.push(UserFriendlyError {
                        asset_reference: asset_result.asset_number.clone(),
                        error_type: error_type.clone(),
                        message: error_msg.clone(),
                        suggestion,
                        field: None, // Could be enhanced to parse field from error message
                    });
                }
            }
        }

        // Generate recommendations
        if result.failed_imports > 0 {
            if user_friendly_errors.iter().any(|e| e.error_type == "duplicate") {
                recommendations.push("Review duplicate asset numbers and update them to be unique".to_string());
            }
            if user_friendly_errors.iter().any(|e| e.error_type == "validation") {
                recommendations.push("Validate all required fields are populated before import".to_string());
            }
            if user_friendly_errors.iter().any(|e| e.message.contains("location")) {
                recommendations.push("Create missing locations or verify location IDs".to_string());
            }
        }

        let summary_message = if result.failed_imports == 0 {
            format!("All {} assets imported successfully", result.successful_imports)
        } else if result.successful_imports == 0 {
            format!("Import failed: {} assets could not be imported", result.failed_imports)
        } else {
            format!("Partial success: {} of {} assets imported successfully",
                   result.successful_imports, result.total_processed)
        };

        Self {
            result,
            success_rate,
            processing_time_ms: 0, // Would be populated by the actual import process
            user_friendly_errors,
            summary_message,
            recommendations,
        }
    }
}

/// Enhanced asset compliance summary with risk assessment and actionable items
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetComplianceSummaryResponse {
    #[serde(flatten)]
    pub compliance: crate::services::AssetComplianceSummary,
    // Enhanced fields for frontend
    /// Visual compliance indicator with color and icon
    pub compliance_indicator: ComplianceIndicator,
    /// Risk level assessment
    pub risk_level: RiskLevel,
    /// List of actionable items to improve compliance
    pub actionable_items: Vec<ActionableItem>,
    /// Compliance trend indicator (improving, stable, declining)
    pub compliance_trend: Option<String>,
}

/// Visual indicator for compliance status
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplianceIndicator {
    /// Color code for display (green, yellow, red)
    pub color: String,
    /// Icon name for display
    pub icon: String,
    /// Label text
    pub label: String,
}

/// Risk level enumeration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Actionable item for improving compliance
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionableItem {
    /// Priority level (high, medium, low)
    pub priority: String,
    /// Description of the action needed
    pub action: String,
    /// Due date for the action
    pub due_date: Option<DateTime<Utc>>,
    /// Person or role responsible for the action
    pub responsible_party: Option<String>,
}

impl From<crate::services::AssetComplianceSummary> for AssetComplianceSummaryResponse {
    fn from(compliance: crate::services::AssetComplianceSummary) -> Self {
        // Determine compliance indicator
        let compliance_indicator = if compliance.overall_compliance_score >= 80.0 {
            ComplianceIndicator {
                color: "green".to_string(),
                icon: "check-circle".to_string(),
                label: "Compliant".to_string(),
            }
        } else if compliance.overall_compliance_score >= 60.0 {
            ComplianceIndicator {
                color: "yellow".to_string(),
                icon: "warning".to_string(),
                label: "At Risk".to_string(),
            }
        } else {
            ComplianceIndicator {
                color: "red".to_string(),
                icon: "x-circle".to_string(),
                label: "Non-Compliant".to_string(),
            }
        };

        // Determine risk level
        let risk_level = if compliance.critical_findings > 0 {
            RiskLevel::Critical
        } else if compliance.overdue_inspections > 0 {
            RiskLevel::High
        } else if compliance.overall_compliance_score < 70.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // Generate actionable items
        let mut actionable_items = Vec::new();
        
        if compliance.overdue_inspections > 0 {
            actionable_items.push(ActionableItem {
                priority: "high".to_string(),
                action: format!("Schedule {} overdue inspection(s)", compliance.overdue_inspections),
                due_date: Some(Utc::now() + chrono::Duration::days(7)),
                responsible_party: Some("Inspection Manager".to_string()),
            });
        }

        if compliance.critical_findings > 0 {
            actionable_items.push(ActionableItem {
                priority: "critical".to_string(),
                action: format!("Address {} critical finding(s)", compliance.critical_findings),
                due_date: Some(Utc::now() + chrono::Duration::days(3)),
                responsible_party: Some("Maintenance Team".to_string()),
            });
        }

        if compliance.overall_compliance_score < 80.0 {
            actionable_items.push(ActionableItem {
                priority: "medium".to_string(),
                action: "Review and improve compliance procedures".to_string(),
                due_date: Some(Utc::now() + chrono::Duration::days(30)),
                responsible_party: Some("Compliance Officer".to_string()),
            });
        }

        Self {
            compliance,
            compliance_indicator,
            risk_level,
            actionable_items,
            compliance_trend: None, // Would require historical data to determine trend
        }
    }
}