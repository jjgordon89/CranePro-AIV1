//! Request DTOs for API commands
//! 
//! This module contains all request structures used by Tauri command handlers
//! to receive data from the frontend.

use crate::models::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};
use serde_json::Value as JsonValue;

// =============================================================================
// Asset Management Requests
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAssetRequest {
    pub asset_number: String,
    pub asset_name: String,
    pub asset_type: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub manufacture_date: Option<NaiveDate>,
    pub installation_date: Option<NaiveDate>,
    pub capacity: Option<f64>,
    pub capacity_unit: Option<String>,
    pub location_id: i64,
    pub status: AssetStatus,
    pub description: Option<String>,
    pub specifications: Option<JsonValue>,
    pub created_by: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetUpdateRequest {
    pub asset_name: Option<String>,
    pub asset_type: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub manufacture_date: Option<NaiveDate>,
    pub installation_date: Option<NaiveDate>,
    pub capacity: Option<f64>,
    pub capacity_unit: Option<String>,
    pub location_id: Option<i64>,
    pub status: Option<AssetStatus>,
    pub description: Option<String>,
    pub specifications: Option<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateComponentRequest {
    pub asset_id: i64,
    pub component_name: String,
    pub component_type: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub parent_component_id: Option<i64>,
    pub specifications: Option<JsonValue>,
    pub status: ComponentStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentUpdateRequest {
    pub component_name: Option<String>,
    pub component_type: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub parent_component_id: Option<i64>,
    pub specifications: Option<JsonValue>,
    pub status: Option<ComponentStatus>,
}

// =============================================================================
// Inspection Management Requests
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateInspectionRequest {
    pub asset_id: i64,
    pub inspector_id: i64,
    pub inspection_type: InspectionType,
    pub compliance_standard: String,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub actual_date: Option<DateTime<Utc>>,
    pub status: InspectionStatus,
    pub overall_condition: Option<Condition>,
    pub checklist_data: Option<JsonValue>,
    pub notes: Option<String>,
    pub ai_analysis_results: Option<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InspectionUpdateRequest {
    pub inspector_id: Option<i64>,
    pub inspection_type: Option<InspectionType>,
    pub compliance_standard: Option<String>,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub actual_date: Option<DateTime<Utc>>,
    pub status: Option<InspectionStatus>,
    pub overall_condition: Option<Condition>,
    pub checklist_data: Option<JsonValue>,
    pub notes: Option<String>,
    pub ai_analysis_results: Option<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateInspectionItemRequest {
    pub inspection_id: i64,
    pub component_id: Option<i64>,
    pub item_name: String,
    pub item_category: String,
    pub condition: Option<Condition>,
    pub finding: Option<String>,
    pub severity: Option<Severity>,
    pub is_compliant: Option<bool>,
    pub corrective_action: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InspectionItemUpdateRequest {
    pub component_id: Option<i64>,
    pub item_name: Option<String>,
    pub item_category: Option<String>,
    pub condition: Option<Condition>,
    pub finding: Option<String>,
    pub severity: Option<Severity>,
    pub is_compliant: Option<bool>,
    pub corrective_action: Option<String>,
}

// =============================================================================
// Compliance Management Requests
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateComplianceRecordRequest {
    pub asset_id: i64,
    pub standard_id: i64,
    pub compliance_status: String,
    pub last_inspection_date: Option<DateTime<Utc>>,
    pub next_inspection_date: Option<DateTime<Utc>>,
    pub compliance_score: f64,
    pub findings: Option<JsonValue>,
    pub corrective_actions: Option<JsonValue>,
    pub verified_by: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComplianceRecordUpdateRequest {
    pub compliance_status: Option<String>,
    pub last_inspection_date: Option<DateTime<Utc>>,
    pub next_inspection_date: Option<DateTime<Utc>>,
    pub compliance_score: Option<f64>,
    pub findings: Option<JsonValue>,
    pub corrective_actions: Option<JsonValue>,
    pub verified_by: Option<i64>,
}

// =============================================================================
// User Management Requests
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserUpdateRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

// =============================================================================
// Media Management Requests
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadFileRequest {
    pub inspection_id: Option<i64>,
    pub component_id: Option<i64>,
    pub file_name: String,
    pub file_data: Vec<u8>,
    pub file_type: MediaType,
    pub mime_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaFileUpdateRequest {
    pub file_name: Option<String>,
    pub description: Option<String>,
    pub ai_analysis_metadata: Option<JsonValue>,
}

// =============================================================================
// Helper Functions
// =============================================================================

impl CreateAssetRequest {
    pub fn to_asset(self) -> Asset {
        Asset {
            id: 0, // Will be set by database
            asset_number: self.asset_number,
            asset_name: self.asset_name,
            asset_type: self.asset_type,
            manufacturer: self.manufacturer,
            model: self.model,
            serial_number: self.serial_number,
            manufacture_date: self.manufacture_date,
            installation_date: self.installation_date,
            capacity: self.capacity,
            capacity_unit: self.capacity_unit,
            location_id: self.location_id,
            status: self.status,
            description: self.description,
            specifications: self.specifications,
            created_by: self.created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl CreateComponentRequest {
    pub fn to_component(self) -> Component {
        Component {
            id: 0, // Will be set by database
            asset_id: self.asset_id,
            component_name: self.component_name,
            component_type: self.component_type,
            manufacturer: self.manufacturer,
            model: self.model,
            serial_number: self.serial_number,
            parent_component_id: self.parent_component_id,
            specifications: self.specifications,
            status: self.status,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl CreateInspectionRequest {
    pub fn to_inspection(self) -> Inspection {
        Inspection {
            id: 0, // Will be set by database
            asset_id: self.asset_id,
            inspector_id: self.inspector_id,
            inspection_type: self.inspection_type,
            compliance_standard: self.compliance_standard,
            scheduled_date: self.scheduled_date,
            actual_date: self.actual_date,
            status: self.status,
            overall_condition: self.overall_condition,
            checklist_data: self.checklist_data,
            notes: self.notes,
            ai_analysis_results: self.ai_analysis_results,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl CreateInspectionItemRequest {
    pub fn to_inspection_item(self) -> InspectionItem {
        InspectionItem {
            id: 0, // Will be set by database
            inspection_id: self.inspection_id,
            component_id: self.component_id,
            item_name: self.item_name,
            item_category: self.item_category,
            condition: self.condition,
            finding: self.finding,
            severity: self.severity,
            is_compliant: self.is_compliant,
            corrective_action: self.corrective_action,
            created_at: Utc::now(),
        }
    }
}

impl CreateUserRequest {
    pub fn to_user(self, password_hash: String) -> User {
        User {
            id: 0, // Will be set by database
            username: self.username,
            email: self.email,
            password_hash,
            role: self.role,
            first_name: self.first_name,
            last_name: self.last_name,
            phone: self.phone,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: self.is_active,
        }
    }
}

impl UploadFileRequest {
    pub fn to_media_file(self, file_path: String, file_size: i64) -> MediaFile {
        MediaFile {
            id: 0, // Will be set by database
            inspection_id: self.inspection_id,
            component_id: self.component_id,
            file_name: self.file_name,
            file_path,
            file_type: self.file_type,
            mime_type: self.mime_type,
            file_size,
            description: self.description,
            ai_analysis_metadata: None,
            created_at: Utc::now(),
        }
    }
}

// =============================================================================
// Location Management Requests
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLocationRequest {
    pub name: String,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
    pub parent_location_id: Option<i64>,
    pub created_by: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationUpdateRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
    pub parent_location_id: Option<Option<i64>>, // Note: nested Option for nullability
}

// =============================================================================
// Location Helper Functions
// =============================================================================

impl CreateLocationRequest {
    pub fn to_location(self) -> Location {
        Location {
            id: 0, // Will be set by database
            name: self.name,
            address: self.address,
            latitude: self.latitude,
            longitude: self.longitude,
            description: self.description,
            parent_location_id: self.parent_location_id,
            created_by: self.created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl From<LocationUpdateRequest> for LocationUpdateData {
    fn from(req: LocationUpdateRequest) -> Self {
        LocationUpdateData {
            name: req.name,
            address: req.address,
            latitude: req.latitude,
            longitude: req.longitude,
            description: req.description,
            parent_location_id: req.parent_location_id,
        }
    }
}