//! Data models for CranePro Bridge Inspection Application
//!
//! This module contains data structures and models representing
//! the core entities in the bridge inspection system.

use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Base model trait for common functionality
pub trait BaseModel {
    /// Get the unique identifier for this model
    fn id(&self) -> i64;
    
    /// Get the creation timestamp
    fn created_at(&self) -> DateTime<Utc>;
    
    /// Get the last update timestamp
    fn updated_at(&self) -> DateTime<Utc>;
}

/// Validation trait for models
pub trait Validate {
    /// Validate the model and return any validation errors
    fn validate(&self) -> AppResult<()>;
}

// =============================================================================
// User Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Inspector,
    Supervisor,
    Administrator,
    SuperAdmin,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Inspector => write!(f, "Inspector"),
            UserRole::Supervisor => write!(f, "Supervisor"),
            UserRole::Administrator => write!(f, "Administrator"),
            UserRole::SuperAdmin => write!(f, "SuperAdmin"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Inspector" => Ok(UserRole::Inspector),
            "Supervisor" => Ok(UserRole::Supervisor),
            "Administrator" => Ok(UserRole::Administrator),
            "SuperAdmin" => Ok(UserRole::SuperAdmin),
            _ => Err(AppError::validation("role", format!("Invalid user role: {}", s))),
        }
    }
}

impl BaseModel for User {
    fn id(&self) -> i64 {
        self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Validate for User {
    fn validate(&self) -> AppResult<()> {
        if self.username.trim().is_empty() {
            return Err(AppError::validation("username", "Username cannot be empty"));
        }
        if self.email.trim().is_empty() {
            return Err(AppError::validation("email", "Email cannot be empty"));
        }
        if !self.email.contains('@') {
            return Err(AppError::validation("email", "Invalid email format"));
        }
        if self.first_name.trim().is_empty() {
            return Err(AppError::validation("first_name", "First name cannot be empty"));
        }
        if self.last_name.trim().is_empty() {
            return Err(AppError::validation("last_name", "Last name cannot be empty"));
        }
        Ok(())
    }
}

// =============================================================================
// Location Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: i64,
    pub name: String,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
    pub created_by: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BaseModel for Location {
    fn id(&self) -> i64 {
        self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Validate for Location {
    fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::validation("name", "Location name cannot be empty"));
        }
        if let (Some(lat), Some(lng)) = (self.latitude, self.longitude) {
            if lat < -90.0 || lat > 90.0 {
                return Err(AppError::validation("latitude", "Latitude must be between -90 and 90"));
            }
            if lng < -180.0 || lng > 180.0 {
                return Err(AppError::validation("longitude", "Longitude must be between -180 and 180"));
            }
        }
        Ok(())
    }
}

// =============================================================================
// Asset Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: i64,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssetStatus {
    Active,
    Inactive,
    Maintenance,
    Decommissioned,
}

impl std::fmt::Display for AssetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetStatus::Active => write!(f, "Active"),
            AssetStatus::Inactive => write!(f, "Inactive"),
            AssetStatus::Maintenance => write!(f, "Maintenance"),
            AssetStatus::Decommissioned => write!(f, "Decommissioned"),
        }
    }
}

impl std::str::FromStr for AssetStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Active" => Ok(AssetStatus::Active),
            "Inactive" => Ok(AssetStatus::Inactive),
            "Maintenance" => Ok(AssetStatus::Maintenance),
            "Decommissioned" => Ok(AssetStatus::Decommissioned),
            _ => Err(AppError::validation("status", format!("Invalid asset status: {}", s))),
        }
    }
}

impl BaseModel for Asset {
    fn id(&self) -> i64 {
        self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Validate for Asset {
    fn validate(&self) -> AppResult<()> {
        if self.asset_number.trim().is_empty() {
            return Err(AppError::validation("asset_number", "Asset number cannot be empty"));
        }
        if self.asset_name.trim().is_empty() {
            return Err(AppError::validation("asset_name", "Asset name cannot be empty"));
        }
        if self.asset_type.trim().is_empty() {
            return Err(AppError::validation("asset_type", "Asset type cannot be empty"));
        }
        if let Some(capacity) = self.capacity {
            if capacity <= 0.0 {
                return Err(AppError::validation("capacity", "Capacity must be greater than 0"));
            }
        }
        Ok(())
    }
}

// =============================================================================
// Component Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: i64,
    pub asset_id: i64,
    pub component_name: String,
    pub component_type: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub parent_component_id: Option<i64>,
    pub specifications: Option<JsonValue>,
    pub status: ComponentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentStatus {
    Active,
    Inactive,
    Maintenance,
    Replaced,
}

impl std::fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentStatus::Active => write!(f, "Active"),
            ComponentStatus::Inactive => write!(f, "Inactive"),
            ComponentStatus::Maintenance => write!(f, "Maintenance"),
            ComponentStatus::Replaced => write!(f, "Replaced"),
        }
    }
}

impl std::str::FromStr for ComponentStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Active" => Ok(ComponentStatus::Active),
            "Inactive" => Ok(ComponentStatus::Inactive),
            "Maintenance" => Ok(ComponentStatus::Maintenance),
            "Replaced" => Ok(ComponentStatus::Replaced),
            _ => Err(AppError::validation("status", format!("Invalid component status: {}", s))),
        }
    }
}

impl BaseModel for Component {
    fn id(&self) -> i64 {
        self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Validate for Component {
    fn validate(&self) -> AppResult<()> {
        if self.component_name.trim().is_empty() {
            return Err(AppError::validation("component_name", "Component name cannot be empty"));
        }
        if self.component_type.trim().is_empty() {
            return Err(AppError::validation("component_type", "Component type cannot be empty"));
        }
        Ok(())
    }
}

// =============================================================================
// Compliance Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStandard {
    pub id: i64,
    pub standard_code: String,
    pub standard_name: String,
    pub version: String,
    pub requirements: Option<JsonValue>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BaseModel for ComplianceStandard {
    fn id(&self) -> i64 {
        self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Validate for ComplianceStandard {
    fn validate(&self) -> AppResult<()> {
        if self.standard_code.trim().is_empty() {
            return Err(AppError::validation("standard_code", "Standard code cannot be empty"));
        }
        if self.standard_name.trim().is_empty() {
            return Err(AppError::validation("standard_name", "Standard name cannot be empty"));
        }
        if self.version.trim().is_empty() {
            return Err(AppError::validation("version", "Version cannot be empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceChecklistTemplate {
    pub id: i64,
    pub standard_id: i64,
    pub template_name: String,
    pub inspection_type: String,
    pub checklist_structure: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BaseModel for ComplianceChecklistTemplate {
    fn id(&self) -> i64 {
        self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Validate for ComplianceChecklistTemplate {
    fn validate(&self) -> AppResult<()> {
        if self.template_name.trim().is_empty() {
            return Err(AppError::validation("template_name", "Template name cannot be empty"));
        }
        if self.inspection_type.trim().is_empty() {
            return Err(AppError::validation("inspection_type", "Inspection type cannot be empty"));
        }
        Ok(())
    }
}

// =============================================================================
// Inspection Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inspection {
    pub id: i64,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InspectionType {
    Frequent,
    Periodic,
    Initial,
    Special,
}

impl std::fmt::Display for InspectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InspectionType::Frequent => write!(f, "Frequent"),
            InspectionType::Periodic => write!(f, "Periodic"),
            InspectionType::Initial => write!(f, "Initial"),
            InspectionType::Special => write!(f, "Special"),
        }
    }
}

impl std::str::FromStr for InspectionType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Frequent" => Ok(InspectionType::Frequent),
            "Periodic" => Ok(InspectionType::Periodic),
            "Initial" => Ok(InspectionType::Initial),
            "Special" => Ok(InspectionType::Special),
            _ => Err(AppError::validation("inspection_type", format!("Invalid inspection type: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InspectionStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl std::fmt::Display for InspectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InspectionStatus::Scheduled => write!(f, "Scheduled"),
            InspectionStatus::InProgress => write!(f, "In Progress"),
            InspectionStatus::Completed => write!(f, "Completed"),
            InspectionStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::str::FromStr for InspectionStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Scheduled" => Ok(InspectionStatus::Scheduled),
            "In Progress" => Ok(InspectionStatus::InProgress),
            "Completed" => Ok(InspectionStatus::Completed),
            "Cancelled" => Ok(InspectionStatus::Cancelled),
            _ => Err(AppError::validation("status", format!("Invalid inspection status: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Condition {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::Excellent => write!(f, "Excellent"),
            Condition::Good => write!(f, "Good"),
            Condition::Fair => write!(f, "Fair"),
            Condition::Poor => write!(f, "Poor"),
            Condition::Critical => write!(f, "Critical"),
        }
    }
}

impl std::str::FromStr for Condition {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Excellent" => Ok(Condition::Excellent),
            "Good" => Ok(Condition::Good),
            "Fair" => Ok(Condition::Fair),
            "Poor" => Ok(Condition::Poor),
            "Critical" => Ok(Condition::Critical),
            _ => Err(AppError::validation("condition", format!("Invalid condition: {}", s))),
        }
    }
}

impl BaseModel for Inspection {
    fn id(&self) -> i64 {
        self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Validate for Inspection {
    fn validate(&self) -> AppResult<()> {
        if self.compliance_standard.trim().is_empty() {
            return Err(AppError::validation("compliance_standard", "Compliance standard cannot be empty"));
        }
        if let (Some(scheduled), Some(actual)) = (self.scheduled_date, self.actual_date) {
            if actual < scheduled {
                return Err(AppError::validation("actual_date", "Actual date cannot be before scheduled date"));
            }
        }
        Ok(())
    }
}

// =============================================================================
// Inspection Item Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionItem {
    pub id: i64,
    pub inspection_id: i64,
    pub component_id: Option<i64>,
    pub item_name: String,
    pub item_category: String,
    pub condition: Option<Condition>,
    pub finding: Option<String>,
    pub severity: Option<Severity>,
    pub is_compliant: Option<bool>,
    pub corrective_action: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "Low"),
            Severity::Medium => write!(f, "Medium"),
            Severity::High => write!(f, "High"),
            Severity::Critical => write!(f, "Critical"),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Low" => Ok(Severity::Low),
            "Medium" => Ok(Severity::Medium),
            "High" => Ok(Severity::High),
            "Critical" => Ok(Severity::Critical),
            _ => Err(AppError::validation("severity", format!("Invalid severity: {}", s))),
        }
    }
}

impl Validate for InspectionItem {
    fn validate(&self) -> AppResult<()> {
        if self.item_name.trim().is_empty() {
            return Err(AppError::validation("item_name", "Item name cannot be empty"));
        }
        if self.item_category.trim().is_empty() {
            return Err(AppError::validation("item_category", "Item category cannot be empty"));
        }
        Ok(())
    }
}

// =============================================================================
// Media File Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: i64,
    pub inspection_id: Option<i64>,
    pub component_id: Option<i64>,
    pub file_name: String,
    pub file_path: String,
    pub file_type: MediaType,
    pub mime_type: String,
    pub file_size: i64,
    pub description: Option<String>,
    pub ai_analysis_metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MediaType {
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "video")]
    Video,
    #[serde(rename = "document")]
    Document,
    #[serde(rename = "audio")]
    Audio,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Image => write!(f, "image"),
            MediaType::Video => write!(f, "video"),
            MediaType::Document => write!(f, "document"),
            MediaType::Audio => write!(f, "audio"),
        }
    }
}

impl std::str::FromStr for MediaType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "image" => Ok(MediaType::Image),
            "video" => Ok(MediaType::Video),
            "document" => Ok(MediaType::Document),
            "audio" => Ok(MediaType::Audio),
            _ => Err(AppError::validation("file_type", format!("Invalid media type: {}", s))),
        }
    }
}

impl Validate for MediaFile {
    fn validate(&self) -> AppResult<()> {
        if self.file_name.trim().is_empty() {
            return Err(AppError::validation("file_name", "File name cannot be empty"));
        }
        if self.file_path.trim().is_empty() {
            return Err(AppError::validation("file_path", "File path cannot be empty"));
        }
        if self.mime_type.trim().is_empty() {
            return Err(AppError::validation("mime_type", "MIME type cannot be empty"));
        }
        if self.file_size <= 0 {
            return Err(AppError::validation("file_size", "File size must be greater than 0"));
        }
        Ok(())
    }
}

// =============================================================================
// AI Model Result Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelResult {
    pub id: i64,
    pub inspection_id: Option<i64>,
    pub media_file_id: Option<i64>,
    pub model_name: String,
    pub model_version: String,
    pub predictions: JsonValue,
    pub confidence_score: f64,
    pub status: AiAnalysisStatus,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiAnalysisStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl std::fmt::Display for AiAnalysisStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiAnalysisStatus::Pending => write!(f, "Pending"),
            AiAnalysisStatus::Processing => write!(f, "Processing"),
            AiAnalysisStatus::Completed => write!(f, "Completed"),
            AiAnalysisStatus::Failed => write!(f, "Failed"),
        }
    }
}

impl std::str::FromStr for AiAnalysisStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(AiAnalysisStatus::Pending),
            "Processing" => Ok(AiAnalysisStatus::Processing),
            "Completed" => Ok(AiAnalysisStatus::Completed),
            "Failed" => Ok(AiAnalysisStatus::Failed),
            _ => Err(AppError::validation("status", format!("Invalid AI analysis status: {}", s))),
        }
    }
}

impl Validate for AiModelResult {
    fn validate(&self) -> AppResult<()> {
        if self.model_name.trim().is_empty() {
            return Err(AppError::validation("model_name", "Model name cannot be empty"));
        }
        if self.model_version.trim().is_empty() {
            return Err(AppError::validation("model_version", "Model version cannot be empty"));
        }
        if self.confidence_score < 0.0 || self.confidence_score > 1.0 {
            return Err(AppError::validation("confidence_score", "Confidence score must be between 0 and 1"));
        }
        Ok(())
    }
}

// =============================================================================
// Maintenance Record Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRecord {
    pub id: i64,
    pub asset_id: i64,
    pub component_id: Option<i64>,
    pub maintenance_type: MaintenanceType,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub completed_date: Option<DateTime<Utc>>,
    pub performed_by: String,
    pub description: String,
    pub status: MaintenanceStatus,
    pub parts_used: Option<JsonValue>,
    pub cost: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaintenanceType {
    Preventive,
    Corrective,
    Emergency,
    Overhaul,
}

impl std::fmt::Display for MaintenanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaintenanceType::Preventive => write!(f, "Preventive"),
            MaintenanceType::Corrective => write!(f, "Corrective"),
            MaintenanceType::Emergency => write!(f, "Emergency"),
            MaintenanceType::Overhaul => write!(f, "Overhaul"),
        }
    }
}

impl std::str::FromStr for MaintenanceType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Preventive" => Ok(MaintenanceType::Preventive),
            "Corrective" => Ok(MaintenanceType::Corrective),
            "Emergency" => Ok(MaintenanceType::Emergency),
            "Overhaul" => Ok(MaintenanceType::Overhaul),
            _ => Err(AppError::validation("maintenance_type", format!("Invalid maintenance type: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaintenanceStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl std::fmt::Display for MaintenanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaintenanceStatus::Scheduled => write!(f, "Scheduled"),
            MaintenanceStatus::InProgress => write!(f, "In Progress"),
            MaintenanceStatus::Completed => write!(f, "Completed"),
            MaintenanceStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::str::FromStr for MaintenanceStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Scheduled" => Ok(MaintenanceStatus::Scheduled),
            "In Progress" => Ok(MaintenanceStatus::InProgress),
            "Completed" => Ok(MaintenanceStatus::Completed),
            "Cancelled" => Ok(MaintenanceStatus::Cancelled),
            _ => Err(AppError::validation("status", format!("Invalid maintenance status: {}", s))),
        }
    }
}

impl Validate for MaintenanceRecord {
    fn validate(&self) -> AppResult<()> {
        if self.performed_by.trim().is_empty() {
            return Err(AppError::validation("performed_by", "Performed by cannot be empty"));
        }
        if self.description.trim().is_empty() {
            return Err(AppError::validation("description", "Description cannot be empty"));
        }
        if let (Some(scheduled), Some(completed)) = (self.scheduled_date, self.completed_date) {
            if completed < scheduled {
                return Err(AppError::validation("completed_date", "Completed date cannot be before scheduled date"));
            }
        }
        if let Some(cost) = self.cost {
            if cost < 0.0 {
                return Err(AppError::validation("cost", "Cost cannot be negative"));
            }
        }
        Ok(())
    }
}

// =============================================================================
// Helper Types and Utilities
// =============================================================================

/// Common query filters for database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
    pub filters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "ASC"),
            SortOrder::Desc => write!(f, "DESC"),
        }
    }
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(50),
            sort_by: None,
            sort_order: Some(SortOrder::Desc),
            filters: HashMap::new(),
        }
    }
}

/// Database operation result with pagination metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub total_count: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

impl<T> PaginatedResult<T> {
    pub fn new(data: Vec<T>, total_count: i64, page: i64, limit: i64) -> Self {
        let total_pages = (total_count + limit - 1) / limit; // Ceiling division
        Self {
            data,
            total_count,
            page,
            limit,
            total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_parsing() {
        assert_eq!("Inspector".parse::<UserRole>().unwrap(), UserRole::Inspector);
        assert_eq!("Supervisor".parse::<UserRole>().unwrap(), UserRole::Supervisor);
        assert!("InvalidRole".parse::<UserRole>().is_err());
    }

    #[test]
    fn test_user_validation() {
        let mut user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            role: UserRole::Inspector,
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            phone: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
        };

        assert!(user.validate().is_ok());

        user.email = "invalid-email".to_string();
        assert!(user.validate().is_err());
    }

    #[test]
    fn test_asset_status_parsing() {
        assert_eq!("Active".parse::<AssetStatus>().unwrap(), AssetStatus::Active);
        assert_eq!("Maintenance".parse::<AssetStatus>().unwrap(), AssetStatus::Maintenance);
        assert!("InvalidStatus".parse::<AssetStatus>().is_err());
    }

    #[test]
    fn test_paginated_result() {
        let data = vec![1, 2, 3, 4, 5];
        let result = PaginatedResult::new(data, 25, 1, 5);
        
        assert_eq!(result.total_count, 25);
        assert_eq!(result.page, 1);
        assert_eq!(result.limit, 5);
        assert_eq!(result.total_pages, 5);
    }
}