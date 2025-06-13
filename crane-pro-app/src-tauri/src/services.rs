//! Services module containing business logic for all major entities
//! 
//! This module implements the repository pattern with comprehensive
//! business logic, CRUD operations, and transaction management.

use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::models::*;
use rusqlite::{params, Row};
use chrono::{DateTime, Utc, NaiveDate};
use serde_json::Value as JsonValue;
use log::{info, debug};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Data Transfer Objects (DTOs)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetUpdateData {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentUpdateData {
    pub component_name: Option<String>,
    pub component_type: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub parent_component_id: Option<i64>,
    pub specifications: Option<JsonValue>,
    pub status: Option<ComponentStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionUpdateData {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionItemUpdateData {
    pub component_id: Option<i64>,
    pub item_name: Option<String>,
    pub item_category: Option<String>,
    pub condition: Option<Condition>,
    pub finding: Option<String>,
    pub severity: Option<Severity>,
    pub is_compliant: Option<bool>,
    pub corrective_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdateData {
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchCriteria {
    pub username: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<UserRole>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordStrengthResult {
    pub is_valid: bool,
    pub score: u8, // 0-100
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLockoutInfo {
    pub user_id: i64,
    pub failed_attempts: i32,
    pub locked_at: Option<DateTime<Utc>>,
    pub locked_until: Option<DateTime<Utc>>,
    pub is_locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFileUpdateData {
    pub file_name: Option<String>,
    pub description: Option<String>,
    pub ai_analysis_metadata: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub compliance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSummaryReport {
    pub asset_id: i64,
    pub asset_name: String,
    pub total_inspections: i64,
    pub completed_inspections: i64,
    pub pending_inspections: i64,
    pub last_inspection_date: Option<DateTime<Utc>>,
    pub next_inspection_date: Option<DateTime<Utc>>,
    pub overall_condition: Option<Condition>,
    pub maintenance_records: i64,
    pub compliance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionCompletionReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_scheduled: i64,
    pub total_completed: i64,
    pub completion_rate: f64,
    pub by_inspector: HashMap<String, i64>,
    pub by_asset_type: HashMap<String, i64>,
    pub average_completion_time_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatusReport {
    pub location_id: Option<i64>,
    pub total_assets: i64,
    pub compliant_assets: i64,
    pub non_compliant_assets: i64,
    pub overdue_inspections: i64,
    pub compliance_percentage: f64,
    pub critical_findings: i64,
    pub by_standard: HashMap<String, ComplianceStandardStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStandardStatus {
    pub standard_code: String,
    pub total_assets: i64,
    pub compliant: i64,
    pub compliance_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceHistoryReport {
    pub asset_id: i64,
    pub asset_name: String,
    pub total_maintenance_records: i64,
    pub preventive_maintenance: i64,
    pub corrective_maintenance: i64,
    pub emergency_maintenance: i64,
    pub total_cost: f64,
    pub average_cost_per_maintenance: f64,
    pub last_maintenance_date: Option<DateTime<Utc>>,
    pub next_scheduled_maintenance: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSummary {
    pub asset_id: i64,
    pub asset_name: String,
    pub asset_number: String,
    pub asset_type: String,
    pub location_name: String,
    pub status: AssetStatus,
    pub total_inspections: i64,
    pub completed_inspections: i64,
    pub pending_inspections: i64,
    pub last_inspection_date: Option<DateTime<Utc>>,
    pub next_inspection_date: Option<DateTime<Utc>>,
    pub overall_condition: Option<Condition>,
    pub maintenance_records_count: i64,
    pub last_maintenance_date: Option<DateTime<Utc>>,
    pub next_maintenance_date: Option<DateTime<Utc>>,
    pub compliance_score: f64,
    pub critical_findings_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportResult {
    pub total_processed: i64,
    pub successful_imports: i64,
    pub failed_imports: i64,
    pub results: Vec<AssetImportResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetImportResult {
    pub asset_number: String,
    pub success: bool,
    pub asset_id: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStatusFilter {
    pub status: AssetStatus,
    pub include_inactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetComplianceSummary {
    pub asset_id: i64,
    pub asset_name: String,
    pub overall_compliance_score: f64,
    pub last_inspection_date: Option<DateTime<Utc>>,
    pub next_required_inspection: Option<DateTime<Utc>>,
    pub critical_findings: i64,
    pub overdue_inspections: i64,
    pub compliance_status: String, // "Compliant", "Non-Compliant", "Overdue", "No Data"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTransferRequest {
    pub asset_id: i64,
    pub from_location_id: i64,
    pub to_location_id: i64,
    pub transfer_reason: String,
    pub transferred_by: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceHistoryEntry {
    pub id: i64,
    pub maintenance_type: String,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub completed_date: Option<DateTime<Utc>>,
    pub performed_by: String,
    pub description: String,
    pub cost: Option<f64>,
    pub status: String,
}

// =============================================================================
// Asset Service
// =============================================================================

pub struct AssetService {
    database: Arc<Database>,
}

impl AssetService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub fn create_asset(&self, asset: Asset) -> AppResult<Asset> {
        info!("Creating new asset: {}", asset.asset_number);
        asset.validate()?;

        self.database.with_transaction(|conn| {
            let id = conn.query_row(
                "INSERT INTO assets (asset_number, asset_name, asset_type, manufacturer, model, 
                 serial_number, manufacture_date, installation_date, capacity, capacity_unit, 
                 location_id, status, description, specifications, created_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                 RETURNING id",
                params![
                    asset.asset_number, asset.asset_name, asset.asset_type,
                    asset.manufacturer, asset.model, asset.serial_number,
                    asset.manufacture_date, asset.installation_date,
                    asset.capacity, asset.capacity_unit, asset.location_id,
                    asset.status.to_string(), asset.description,
                    asset.specifications.as_ref().map(|s| s.to_string()),
                    asset.created_by
                ],
                |row| row.get::<_, i64>(0),
            )?;

            debug!("Asset created with ID: {}", id);
            self.get_asset_by_id(id)
        })
    }

    pub fn get_asset_by_id(&self, id: i64) -> AppResult<Asset> {
        debug!("Fetching asset by ID: {}", id);
        let conn = self.database.get_connection()?;
        
        let asset = conn.query_row(
            "SELECT id, asset_number, asset_name, asset_type, manufacturer, model,
             serial_number, manufacture_date, installation_date, capacity, capacity_unit,
             location_id, status, description, specifications, created_by, created_at, updated_at
             FROM assets WHERE id = ?1",
            params![id],
            |row| self.row_to_asset(row),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "Asset".to_string(),
            field: "id".to_string(),
            value: id.to_string(),
        })?;

        self.database.return_connection(conn);
        Ok(asset)
    }

    pub fn get_assets_by_location(&self, location_id: i64, filter: QueryFilter) -> AppResult<PaginatedResult<Asset>> {
        info!("Fetching assets for location: {} with filter: {:?}", location_id, filter);
        let conn = self.database.get_connection()?;

        let offset = ((filter.page.unwrap_or(1) - 1) * filter.limit.unwrap_or(50)).max(0);
        let limit = filter.limit.unwrap_or(50);
        let sort_order = filter.sort_order.unwrap_or(SortOrder::Desc);

        // Simple implementation without dynamic filters for now
        let order_by = format!(" ORDER BY {} {}",
            filter.sort_by.unwrap_or("created_at".to_string()), sort_order);

        let query = format!(
            "SELECT id, asset_number, asset_name, asset_type, manufacturer, model,
             serial_number, manufacture_date, installation_date, capacity, capacity_unit,
             location_id, status, description, specifications, created_by, created_at, updated_at
             FROM assets WHERE location_id = ?1 {} LIMIT {} OFFSET {}",
            order_by, limit, offset
        );

        let mut stmt = conn.prepare(&query)?;
        let asset_iter = stmt.query_map([location_id], |row| self.row_to_asset(row))?;

        let mut assets = Vec::new();
        for asset in asset_iter {
            assets.push(asset?);
        }

        // Get total count
        let total_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM assets WHERE location_id = ?1",
            [location_id],
            |row| row.get(0),
        )?;

        drop(stmt);
        self.database.return_connection(conn);
        Ok(PaginatedResult::new(assets, total_count, filter.page.unwrap_or(1), limit))
    }

    pub fn update_asset(&self, id: i64, updates: AssetUpdateData) -> AppResult<Asset> {
        info!("Updating asset: {}", id);
        
        self.database.with_transaction(|conn| {
            // Simple implementation - update individual fields
            if let Some(asset_name) = &updates.asset_name {
                conn.execute("UPDATE assets SET asset_name = ?1 WHERE id = ?2", params![asset_name, id])?;
            }
            if let Some(asset_type) = &updates.asset_type {
                conn.execute("UPDATE assets SET asset_type = ?1 WHERE id = ?2", params![asset_type, id])?;
            }
            if let Some(manufacturer) = &updates.manufacturer {
                conn.execute("UPDATE assets SET manufacturer = ?1 WHERE id = ?2", params![manufacturer, id])?;
            }
            if let Some(model) = &updates.model {
                conn.execute("UPDATE assets SET model = ?1 WHERE id = ?2", params![model, id])?;
            }
            if let Some(status) = &updates.status {
                conn.execute("UPDATE assets SET status = ?1 WHERE id = ?2", params![status.to_string(), id])?;
            }
            if let Some(description) = &updates.description {
                conn.execute("UPDATE assets SET description = ?1 WHERE id = ?2", params![description, id])?;
            }

            debug!("Asset {} updated successfully", id);
            self.get_asset_by_id(id)
        })
    }

    pub fn delete_asset(&self, id: i64) -> AppResult<()> {
        info!("Deleting asset: {}", id);
        
        self.database.with_transaction(|conn| {
            let rows_affected = conn.execute("DELETE FROM assets WHERE id = ?1", params![id])?;
            
            if rows_affected == 0 {
                return Err(AppError::RecordNotFound {
                    entity: "Asset".to_string(),
                    field: "id".to_string(),
                    value: id.to_string(),
                });
            }
            
            debug!("Asset {} deleted successfully", id);
            Ok(())
        })
    }

    pub fn search_assets(&self, query: String, filter: QueryFilter) -> AppResult<PaginatedResult<Asset>> {
        info!("Searching assets with query: {}", query);
        let conn = self.database.get_connection()?;

        let search_term = format!("%{}%", query);
        let offset = ((filter.page.unwrap_or(1) - 1) * filter.limit.unwrap_or(50)).max(0);
        let limit = filter.limit.unwrap_or(50);

        let search_query = format!(
            "SELECT id, asset_number, asset_name, asset_type, manufacturer, model,
             serial_number, manufacture_date, installation_date, capacity, capacity_unit,
             location_id, status, description, specifications, created_by, created_at, updated_at
             FROM assets
             WHERE asset_name LIKE ?1 OR asset_number LIKE ?1 OR asset_type LIKE ?1 OR manufacturer LIKE ?1
             ORDER BY created_at DESC LIMIT {} OFFSET {}",
            limit, offset
        );

        let mut stmt = conn.prepare(&search_query)?;
        let asset_iter = stmt.query_map([&search_term], |row| self.row_to_asset(row))?;

        let mut assets = Vec::new();
        for asset in asset_iter {
            assets.push(asset?);
        }

        let total_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM assets
             WHERE asset_name LIKE ?1 OR asset_number LIKE ?1 OR asset_type LIKE ?1 OR manufacturer LIKE ?1",
            [&search_term],
            |row| row.get(0),
        )?;

        drop(stmt);
        self.database.return_connection(conn);
        Ok(PaginatedResult::new(assets, total_count, filter.page.unwrap_or(1), limit))
    }

    pub fn get_asset_components(&self, asset_id: i64) -> AppResult<Vec<Component>> {
        debug!("Fetching components for asset: {}", asset_id);
        let conn = self.database.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, asset_id, component_name, component_type, manufacturer, model,
             serial_number, parent_component_id, specifications, status, created_at, updated_at
             FROM components WHERE asset_id = ?1 ORDER BY component_name"
        )?;

        let component_iter = stmt.query_map([asset_id], |row| self.row_to_component(row))?;

        let mut components = Vec::new();
        for component in component_iter {
            components.push(component?);
        }

        drop(stmt);
        self.database.return_connection(conn);
        Ok(components)
    }

    pub fn create_component(&self, component: Component) -> AppResult<Component> {
        info!("Creating new component: {}", component.component_name);
        component.validate()?;

        self.database.with_transaction(|conn| {
            let id = conn.query_row(
                "INSERT INTO components (asset_id, component_name, component_type, manufacturer,
                 model, serial_number, parent_component_id, specifications, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 RETURNING id",
                params![
                    component.asset_id, component.component_name, component.component_type,
                    component.manufacturer, component.model, component.serial_number,
                    component.parent_component_id,
                    component.specifications.as_ref().map(|s| s.to_string()),
                    component.status.to_string()
                ],
                |row| row.get::<_, i64>(0),
            )?;

            debug!("Component created with ID: {}", id);
            self.get_component_by_id(id)
        })
    }

    pub fn update_component(&self, id: i64, updates: ComponentUpdateData) -> AppResult<Component> {
        info!("Updating component: {}", id);
        
        self.database.with_transaction(|conn| {
            if let Some(component_name) = &updates.component_name {
                conn.execute("UPDATE components SET component_name = ?1 WHERE id = ?2", params![component_name, id])?;
            }
            if let Some(component_type) = &updates.component_type {
                conn.execute("UPDATE components SET component_type = ?1 WHERE id = ?2", params![component_type, id])?;
            }
            if let Some(status) = &updates.status {
                conn.execute("UPDATE components SET status = ?1 WHERE id = ?2", params![status.to_string(), id])?;
            }

            debug!("Component {} updated successfully", id);
            self.get_component_by_id(id)
        })
    }

    fn get_component_by_id(&self, id: i64) -> AppResult<Component> {
        let conn = self.database.get_connection()?;
        let component = conn.query_row(
            "SELECT id, asset_id, component_name, component_type, manufacturer, model,
             serial_number, parent_component_id, specifications, status, created_at, updated_at
             FROM components WHERE id = ?1",
            params![id],
            |row| self.row_to_component(row),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "Component".to_string(),
            field: "id".to_string(),
            value: id.to_string(),
        })?;

        self.database.return_connection(conn);
        Ok(component)
    }

    fn row_to_asset(&self, row: &Row) -> rusqlite::Result<Asset> {
        Ok(Asset {
            id: row.get(0)?,
            asset_number: row.get(1)?,
            asset_name: row.get(2)?,
            asset_type: row.get(3)?,
            manufacturer: row.get(4)?,
            model: row.get(5)?,
            serial_number: row.get(6)?,
            manufacture_date: row.get(7)?,
            installation_date: row.get(8)?,
            capacity: row.get(9)?,
            capacity_unit: row.get(10)?,
            location_id: row.get(11)?,
            status: row.get::<_, String>(12)?.parse().unwrap_or(AssetStatus::Active),
            description: row.get(13)?,
            specifications: row.get::<_, Option<String>>(14)?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_by: row.get(15)?,
            created_at: row.get(16)?,
            updated_at: row.get(17)?,
        })
    }

    /// Get comprehensive asset summary including inspections, maintenance, and compliance data
    ///
    /// # Arguments
    /// * `asset_id` - The asset ID to get summary for
    ///
    /// # Returns
    /// * `AssetSummary` with comprehensive asset data
    pub fn get_asset_summary(&self, asset_id: i64) -> AppResult<AssetSummary> {
        info!("Getting asset summary for asset: {}", asset_id);
        let conn = self.database.get_connection()?;

        // Get basic asset information with location name
        let (asset_name, asset_number, asset_type, location_name, status): (String, String, String, String, String) = conn.query_row(
            "SELECT a.asset_name, a.asset_number, a.asset_type, l.name, a.status
             FROM assets a
             JOIN locations l ON a.location_id = l.id
             WHERE a.id = ?1",
            params![asset_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "Asset".to_string(),
            field: "id".to_string(),
            value: asset_id.to_string(),
        })?;

        // Get inspection counts and dates
        let (total_inspections, completed_inspections, pending_inspections): (i64, i64, i64) = conn.query_row(
            "SELECT
                COUNT(*) as total,
                COUNT(CASE WHEN status = 'Completed' THEN 1 END) as completed,
                COUNT(CASE WHEN status IN ('Scheduled', 'In Progress') THEN 1 END) as pending
             FROM inspections WHERE asset_id = ?1",
            params![asset_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        // Get last and next inspection dates, and overall condition
        let (last_inspection_date, overall_condition): (Option<DateTime<Utc>>, Option<String>) = conn.query_row(
            "SELECT actual_date, overall_condition FROM inspections
             WHERE asset_id = ?1 AND status = 'Completed'
             ORDER BY actual_date DESC LIMIT 1",
            params![asset_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap_or((None, None));

        // Calculate next inspection date (simple heuristic: 1 year from last inspection or 30 days from now)
        let next_inspection_date = last_inspection_date
            .map(|date| date + chrono::Duration::days(365))
            .or_else(|| Some(Utc::now() + chrono::Duration::days(30)));

        // Get maintenance records count and dates
        let maintenance_records_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM maintenance_records WHERE asset_id = ?1",
            params![asset_id],
            |row| row.get(0),
        )?;

        let (last_maintenance_date, next_maintenance_date): (Option<DateTime<Utc>>, Option<DateTime<Utc>>) = (
            conn.query_row(
                "SELECT MAX(completed_date) FROM maintenance_records
                 WHERE asset_id = ?1 AND status = 'Completed'",
                params![asset_id],
                |row| row.get(0),
            ).unwrap_or(None),
            conn.query_row(
                "SELECT MIN(scheduled_date) FROM maintenance_records
                 WHERE asset_id = ?1 AND status = 'Scheduled' AND scheduled_date > datetime('now')",
                params![asset_id],
                |row| row.get(0),
            ).unwrap_or(None)
        );

        // Calculate compliance score (average of all completed inspections)
        let compliance_score: f64 = if completed_inspections > 0 {
            let mut total_score = 0.0;
            let mut stmt = conn.prepare(
                "SELECT id FROM inspections WHERE asset_id = ?1 AND status = 'Completed'"
            )?;
            let inspection_iter = stmt.query_map(params![asset_id], |row| row.get::<_, i64>(0))?;
            
            for inspection_result in inspection_iter {
                let inspection_id = inspection_result?;
                let (total_items, compliant_items): (i64, i64) = conn.query_row(
                    "SELECT
                        COUNT(*) as total,
                        COUNT(CASE WHEN is_compliant = 1 THEN 1 END) as compliant
                     FROM inspection_items WHERE inspection_id = ?1",
                    params![inspection_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;
                
                if total_items > 0 {
                    total_score += (compliant_items as f64 / total_items as f64) * 100.0;
                }
            }
            total_score / completed_inspections as f64
        } else {
            0.0
        };

        // Get critical findings count
        let critical_findings_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM inspection_items ii
             JOIN inspections i ON ii.inspection_id = i.id
             WHERE i.asset_id = ?1 AND ii.severity = 'Critical' AND i.status = 'Completed'",
            params![asset_id],
            |row| row.get(0),
        )?;

        self.database.return_connection(conn);

        debug!("Asset summary generated for asset: {}", asset_id);
        Ok(AssetSummary {
            asset_id,
            asset_name,
            asset_number,
            asset_type,
            location_name,
            status: status.parse().unwrap_or(AssetStatus::Active),
            total_inspections,
            completed_inspections,
            pending_inspections,
            last_inspection_date,
            next_inspection_date,
            overall_condition: overall_condition.and_then(|s| s.parse().ok()),
            maintenance_records_count,
            last_maintenance_date,
            next_maintenance_date,
            compliance_score,
            critical_findings_count,
        })
    }

    /// Bulk import assets with validation and transaction handling
    ///
    /// # Arguments
    /// * `assets` - Vector of Asset objects to import
    ///
    /// # Returns
    /// * `BulkImportResult` with detailed results for each asset
    pub fn bulk_import_assets(&self, assets: Vec<Asset>) -> AppResult<BulkImportResult> {
        info!("Starting bulk import of {} assets", assets.len());
        let mut results = Vec::new();
        let mut successful_imports = 0i64;
        let mut failed_imports = 0i64;

        for asset in assets.iter() {
            debug!("Processing asset import: {}", asset.asset_number);
            
            // Validate asset
            match asset.validate() {
                Ok(_) => {
                    // Check if location exists
                    let location_exists = {
                        let conn = self.database.get_connection()?;
                        let count: i64 = conn.query_row(
                            "SELECT COUNT(*) FROM locations WHERE id = ?1",
                            params![asset.location_id],
                            |row| row.get(0),
                        )?;
                        self.database.return_connection(conn);
                        count > 0
                    };

                    if !location_exists {
                        failed_imports += 1;
                        results.push(AssetImportResult {
                            asset_number: asset.asset_number.clone(),
                            success: false,
                            asset_id: None,
                            error_message: Some(format!("Location with ID {} does not exist", asset.location_id)),
                        });
                        continue;
                    }

                    // Try to create the asset
                    match self.create_asset(asset.clone()) {
                        Ok(created_asset) => {
                            successful_imports += 1;
                            results.push(AssetImportResult {
                                asset_number: asset.asset_number.clone(),
                                success: true,
                                asset_id: Some(created_asset.id),
                                error_message: None,
                            });
                            debug!("Successfully imported asset: {}", asset.asset_number);
                        }
                        Err(e) => {
                            failed_imports += 1;
                            results.push(AssetImportResult {
                                asset_number: asset.asset_number.clone(),
                                success: false,
                                asset_id: None,
                                error_message: Some(e.to_string()),
                            });
                            debug!("Failed to import asset {}: {}", asset.asset_number, e);
                        }
                    }
                }
                Err(validation_error) => {
                    failed_imports += 1;
                    results.push(AssetImportResult {
                        asset_number: asset.asset_number.clone(),
                        success: false,
                        asset_id: None,
                        error_message: Some(validation_error.to_string()),
                    });
                    debug!("Asset validation failed for {}: {}", asset.asset_number, validation_error);
                }
            }
        }

        let total_processed = assets.len() as i64;
        info!("Bulk import completed: {}/{} successful", successful_imports, total_processed);

        Ok(BulkImportResult {
            total_processed,
            successful_imports,
            failed_imports,
            results,
        })
    }

    /// Get maintenance history for a specific asset
    ///
    /// # Arguments
    /// * `asset_id` - The asset ID to get maintenance history for
    ///
    /// # Returns
    /// * `Vec<MaintenanceHistoryEntry>` with structured maintenance history data
    pub fn get_asset_maintenance_history(&self, asset_id: i64) -> AppResult<Vec<MaintenanceHistoryEntry>> {
        info!("Getting maintenance history for asset: {}", asset_id);
        let conn = self.database.get_connection()?;

        // Verify asset exists
        let _asset = self.get_asset_by_id(asset_id)?;

        let mut stmt = conn.prepare(
            "SELECT id, maintenance_type, scheduled_date, completed_date, performed_by, description, cost, status
             FROM maintenance_records WHERE asset_id = ?1 ORDER BY created_at DESC"
        )?;

        let maintenance_iter = stmt.query_map(params![asset_id], |row| {
            Ok(MaintenanceHistoryEntry {
                id: row.get(0)?,
                maintenance_type: row.get(1)?,
                scheduled_date: row.get(2)?,
                completed_date: row.get(3)?,
                performed_by: row.get(4)?,
                description: row.get(5)?,
                cost: row.get(6)?,
                status: row.get(7)?,
            })
        })?;

        let mut maintenance_history = Vec::new();
        for maintenance in maintenance_iter {
            maintenance_history.push(maintenance?);
        }

        drop(stmt);
        self.database.return_connection(conn);
        debug!("Retrieved {} maintenance records for asset: {}", maintenance_history.len(), asset_id);
        Ok(maintenance_history)
    }

    /// Validate asset-location assignment
    ///
    /// # Arguments
    /// * `asset_id` - The asset ID to validate
    /// * `location_id` - The location ID to validate
    ///
    /// # Returns
    /// * `AppResult<()>` indicating validation success or failure
    pub fn validate_asset_location_assignment(&self, asset_id: i64, location_id: i64) -> AppResult<()> {
        debug!("Validating asset-location assignment: asset={}, location={}", asset_id, location_id);
        
        // Check if asset exists
        let _asset = self.get_asset_by_id(asset_id)?;
        
        // Check if location exists
        {
            let conn = self.database.get_connection()?;
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM locations WHERE id = ?1",
                params![location_id],
                |row| row.get(0),
            )?;
            self.database.return_connection(conn);
            
            if count == 0 {
                return Err(AppError::RecordNotFound {
                    entity: "Location".to_string(),
                    field: "id".to_string(),
                    value: location_id.to_string(),
                });
            }
        }
        
        debug!("Asset-location assignment validation successful");
        Ok(())
    }

    /// Get assets filtered by status with pagination
    ///
    /// # Arguments
    /// * `status_filter` - The status filter criteria
    /// * `filter` - Query filter with pagination parameters
    ///
    /// # Returns
    /// * `PaginatedResult<Asset>` with assets matching the status filter
    pub fn get_assets_by_status(&self, status_filter: AssetStatusFilter, filter: QueryFilter) -> AppResult<PaginatedResult<Asset>> {
        info!("Fetching assets by status: {:?}", status_filter);
        let conn = self.database.get_connection()?;

        let offset = ((filter.page.unwrap_or(1) - 1) * filter.limit.unwrap_or(50)).max(0);
        let limit = filter.limit.unwrap_or(50);
        let sort_order = filter.sort_order.unwrap_or(SortOrder::Desc);
        let sort_by = filter.sort_by.unwrap_or("created_at".to_string());

        // Build WHERE conditions
        let where_clause = if status_filter.include_inactive {
            "WHERE status = ?"
        } else {
            "WHERE status = ? AND status != 'Inactive'"
        };

        let order_by = format!(" ORDER BY {} {}", sort_by, sort_order);

        let query = format!(
            "SELECT id, asset_number, asset_name, asset_type, manufacturer, model,
             serial_number, manufacture_date, installation_date, capacity, capacity_unit,
             location_id, status, description, specifications, created_by, created_at, updated_at
             FROM assets {} {} LIMIT {} OFFSET {}",
            where_clause, order_by, limit, offset
        );

        let mut stmt = conn.prepare(&query)?;
        let asset_iter = stmt.query_map([status_filter.status.to_string()], |row| self.row_to_asset(row))?;

        let mut assets = Vec::new();
        for asset in asset_iter {
            assets.push(asset?);
        }

        // Get total count
        let total_count: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM assets {}", where_clause),
            [status_filter.status.to_string()],
            |row| row.get(0),
        )?;

        drop(stmt);
        self.database.return_connection(conn);
        Ok(PaginatedResult::new(assets, total_count, filter.page.unwrap_or(1), limit))
    }

    /// Get compliance summary for a specific asset
    ///
    /// # Arguments
    /// * `asset_id` - The asset ID to get compliance summary for
    ///
    /// # Returns
    /// * `AssetComplianceSummary` with compliance status and critical findings
    pub fn get_asset_compliance_summary(&self, asset_id: i64) -> AppResult<AssetComplianceSummary> {
        info!("Getting compliance summary for asset: {}", asset_id);
        let conn = self.database.get_connection()?;

        // Get asset name
        let asset_name: String = conn.query_row(
            "SELECT asset_name FROM assets WHERE id = ?1",
            params![asset_id],
            |row| row.get(0),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "Asset".to_string(),
            field: "id".to_string(),
            value: asset_id.to_string(),
        })?;

        // Get last inspection date
        let last_inspection_date: Option<DateTime<Utc>> = conn.query_row(
            "SELECT MAX(actual_date) FROM inspections
             WHERE asset_id = ?1 AND status = 'Completed'",
            params![asset_id],
            |row| row.get(0),
        ).unwrap_or(None);

        // Calculate next required inspection (1 year from last or 30 days from now)
        let next_required_inspection = last_inspection_date
            .map(|date| date + chrono::Duration::days(365))
            .or_else(|| Some(Utc::now() + chrono::Duration::days(30)));

        // Calculate overall compliance score
        let overall_compliance_score: f64 = {
            let completed_inspections: i64 = conn.query_row(
                "SELECT COUNT(*) FROM inspections WHERE asset_id = ?1 AND status = 'Completed'",
                params![asset_id],
                |row| row.get(0),
            )?;

            if completed_inspections > 0 {
                let mut total_score = 0.0;
                let mut stmt = conn.prepare(
                    "SELECT id FROM inspections WHERE asset_id = ?1 AND status = 'Completed'"
                )?;
                let inspection_iter = stmt.query_map(params![asset_id], |row| row.get::<_, i64>(0))?;
                
                for inspection_result in inspection_iter {
                    let inspection_id = inspection_result?;
                    let (total_items, compliant_items): (i64, i64) = conn.query_row(
                        "SELECT
                            COUNT(*) as total,
                            COUNT(CASE WHEN is_compliant = 1 THEN 1 END) as compliant
                         FROM inspection_items WHERE inspection_id = ?1",
                        params![inspection_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )?;
                    
                    if total_items > 0 {
                        total_score += (compliant_items as f64 / total_items as f64) * 100.0;
                    }
                }
                total_score / completed_inspections as f64
            } else {
                0.0
            }
        };

        // Get critical findings count
        let critical_findings: i64 = conn.query_row(
            "SELECT COUNT(*) FROM inspection_items ii
             JOIN inspections i ON ii.inspection_id = i.id
             WHERE i.asset_id = ?1 AND ii.severity = 'Critical' AND i.status = 'Completed'",
            params![asset_id],
            |row| row.get(0),
        )?;

        // Get overdue inspections count
        let overdue_inspections: i64 = conn.query_row(
            "SELECT COUNT(*) FROM inspections
             WHERE asset_id = ?1 AND scheduled_date < datetime('now') AND status NOT IN ('Completed', 'Cancelled')",
            params![asset_id],
            |row| row.get(0),
        )?;

        // Determine compliance status
        let compliance_status = if overall_compliance_score == 0.0 {
            "No Data".to_string()
        } else if overdue_inspections > 0 {
            "Overdue".to_string()
        } else if overall_compliance_score >= 80.0 {
            "Compliant".to_string()
        } else {
            "Non-Compliant".to_string()
        };

        self.database.return_connection(conn);

        debug!("Compliance summary generated for asset: {}", asset_id);
        Ok(AssetComplianceSummary {
            asset_id,
            asset_name,
            overall_compliance_score,
            last_inspection_date,
            next_required_inspection,
            critical_findings,
            overdue_inspections,
            compliance_status,
        })
    }

    /// Transfer asset from one location to another with validation and audit logging
    ///
    /// # Arguments
    /// * `transfer_request` - The transfer request details
    ///
    /// # Returns
    /// * `AppResult<Asset>` the updated asset
    pub fn transfer_asset_location(&self, transfer_request: AssetTransferRequest) -> AppResult<Asset> {
        info!("Transferring asset {} from location {} to location {}",
              transfer_request.asset_id, transfer_request.from_location_id, transfer_request.to_location_id);

        self.database.with_transaction(|conn| {
            // Validate asset exists and is at the source location
            let current_location_id: i64 = conn.query_row(
                "SELECT location_id FROM assets WHERE id = ?1",
                params![transfer_request.asset_id],
                |row| row.get(0),
            ).map_err(|_| AppError::RecordNotFound {
                entity: "Asset".to_string(),
                field: "id".to_string(),
                value: transfer_request.asset_id.to_string(),
            })?;

            if current_location_id != transfer_request.from_location_id {
                return Err(AppError::validation(
                    "from_location_id",
                    format!("Asset is not currently at location {}", transfer_request.from_location_id)
                ));
            }

            // Validate target location exists
            let target_location_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM locations WHERE id = ?1",
                params![transfer_request.to_location_id],
                |row| row.get(0),
            )?;

            if target_location_count == 0 {
                return Err(AppError::RecordNotFound {
                    entity: "Location".to_string(),
                    field: "id".to_string(),
                    value: transfer_request.to_location_id.to_string(),
                });
            }

            // Validate user exists
            let user_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM users WHERE id = ?1",
                params![transfer_request.transferred_by],
                |row| row.get(0),
            )?;

            if user_count == 0 {
                return Err(AppError::RecordNotFound {
                    entity: "User".to_string(),
                    field: "id".to_string(),
                    value: transfer_request.transferred_by.to_string(),
                });
            }

            // Update asset location
            let rows_affected = conn.execute(
                "UPDATE assets SET location_id = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![transfer_request.to_location_id, transfer_request.asset_id]
            )?;

            if rows_affected == 0 {
                return Err(AppError::RecordNotFound {
                    entity: "Asset".to_string(),
                    field: "id".to_string(),
                    value: transfer_request.asset_id.to_string(),
                });
            }

            // Log the transfer for audit (this would typically go to an audit log table)
            info!("Asset transfer completed: asset_id={}, from_location={}, to_location={}, transferred_by={}, reason='{}'",
                  transfer_request.asset_id, transfer_request.from_location_id,
                  transfer_request.to_location_id, transfer_request.transferred_by, transfer_request.transfer_reason);

            debug!("Asset {} transferred successfully", transfer_request.asset_id);
            self.get_asset_by_id(transfer_request.asset_id)
        })
    }

    fn row_to_component(&self, row: &Row) -> rusqlite::Result<Component> {
        Ok(Component {
            id: row.get(0)?,
            asset_id: row.get(1)?,
            component_name: row.get(2)?,
            component_type: row.get(3)?,
            manufacturer: row.get(4)?,
            model: row.get(5)?,
            serial_number: row.get(6)?,
            parent_component_id: row.get(7)?,
            specifications: row.get::<_, Option<String>>(8)?
                .and_then(|s| serde_json::from_str(&s).ok()),
            status: row.get::<_, String>(9)?.parse().unwrap_or(ComponentStatus::Active),
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }
}

// =============================================================================
// Inspection Service
// =============================================================================

pub struct InspectionService {
    database: Arc<Database>,
}

impl InspectionService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub fn create_inspection(&self, inspection: Inspection) -> AppResult<Inspection> {
        info!("Creating new inspection for asset: {}", inspection.asset_id);
        inspection.validate()?;

        self.database.with_transaction(|conn| {
            let id = conn.query_row(
                "INSERT INTO inspections (asset_id, inspector_id, inspection_type, compliance_standard,
                 scheduled_date, actual_date, status, overall_condition, checklist_data, notes, ai_analysis_results)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 RETURNING id",
                params![
                    inspection.asset_id, inspection.inspector_id, inspection.inspection_type.to_string(),
                    inspection.compliance_standard, inspection.scheduled_date, inspection.actual_date,
                    inspection.status.to_string(),
                    inspection.overall_condition.as_ref().map(|c| c.to_string()),
                    inspection.checklist_data.as_ref().map(|d| d.to_string()),
                    inspection.notes,
                    inspection.ai_analysis_results.as_ref().map(|r| r.to_string())
                ],
                |row| row.get::<_, i64>(0),
            )?;

            debug!("Inspection created with ID: {}", id);
            self.get_inspection_by_id(id)
        })
    }

    pub fn get_inspection_by_id(&self, id: i64) -> AppResult<Inspection> {
        debug!("Fetching inspection by ID: {}", id);
        let conn = self.database.get_connection()?;
        
        let inspection = conn.query_row(
            "SELECT id, asset_id, inspector_id, inspection_type, compliance_standard,
             scheduled_date, actual_date, status, overall_condition, checklist_data, notes,
             ai_analysis_results, created_at, updated_at
             FROM inspections WHERE id = ?1",
            params![id],
            |row| self.row_to_inspection(row),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "Inspection".to_string(),
            field: "id".to_string(),
            value: id.to_string(),
        })?;

        self.database.return_connection(conn);
        Ok(inspection)
    }

    pub fn update_inspection(&self, id: i64, updates: InspectionUpdateData) -> AppResult<Inspection> {
        info!("Updating inspection: {}", id);
        
        self.database.with_transaction(|conn| {
            if let Some(status) = &updates.status {
                conn.execute("UPDATE inspections SET status = ?1 WHERE id = ?2", params![status.to_string(), id])?;
            }
            if let Some(actual_date) = &updates.actual_date {
                conn.execute("UPDATE inspections SET actual_date = ?1 WHERE id = ?2", params![actual_date, id])?;
            }
            if let Some(overall_condition) = &updates.overall_condition {
                conn.execute("UPDATE inspections SET overall_condition = ?1 WHERE id = ?2", params![overall_condition.to_string(), id])?;
            }
            if let Some(notes) = &updates.notes {
                conn.execute("UPDATE inspections SET notes = ?1 WHERE id = ?2", params![notes, id])?;
            }

            debug!("Inspection {} updated successfully", id);
            self.get_inspection_by_id(id)
        })
    }

    pub fn submit_inspection(&self, id: i64) -> AppResult<Inspection> {
        info!("Submitting inspection: {}", id);
        
        self.database.with_transaction(|conn| {
            conn.execute(
                "UPDATE inspections SET status = 'Completed', actual_date = CURRENT_TIMESTAMP WHERE id = ?1",
                params![id]
            )?;
            
            debug!("Inspection {} submitted successfully", id);
            self.get_inspection_by_id(id)
        })
    }

    pub fn get_inspections_by_asset(&self, asset_id: i64, filter: QueryFilter) -> AppResult<PaginatedResult<Inspection>> {
        info!("Fetching inspections for asset: {}", asset_id);
        let conn = self.database.get_connection()?;

        let offset = ((filter.page.unwrap_or(1) - 1) * filter.limit.unwrap_or(50)).max(0);
        let limit = filter.limit.unwrap_or(50);

        let mut stmt = conn.prepare(
            "SELECT id, asset_id, inspector_id, inspection_type, compliance_standard,
             scheduled_date, actual_date, status, overall_condition, checklist_data, notes,
             ai_analysis_results, created_at, updated_at
             FROM inspections WHERE asset_id = ?1 
             ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
        )?;

        let inspection_iter = stmt.query_map(params![asset_id, limit, offset], |row| self.row_to_inspection(row))?;

        let mut inspections = Vec::new();
        for inspection in inspection_iter {
            inspections.push(inspection?);
        }

        let total_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM inspections WHERE asset_id = ?1",
            params![asset_id],
            |row| row.get(0),
        )?;

        drop(stmt);
        self.database.return_connection(conn);
        Ok(PaginatedResult::new(inspections, total_count, filter.page.unwrap_or(1), limit))
    }

    pub fn get_pending_inspections(&self, inspector_id: Option<i64>) -> AppResult<Vec<Inspection>> {
        info!("Fetching pending inspections");
        let conn = self.database.get_connection()?;

        let query = if let Some(_inspector_id) = inspector_id {
            "SELECT id, asset_id, inspector_id, inspection_type, compliance_standard,
             scheduled_date, actual_date, status, overall_condition, checklist_data, notes,
             ai_analysis_results, created_at, updated_at
             FROM inspections WHERE status IN ('Scheduled', 'In Progress') AND inspector_id = ?1
             ORDER BY scheduled_date ASC"
        } else {
            "SELECT id, asset_id, inspector_id, inspection_type, compliance_standard,
             scheduled_date, actual_date, status, overall_condition, checklist_data, notes,
             ai_analysis_results, created_at, updated_at
             FROM inspections WHERE status IN ('Scheduled', 'In Progress')
             ORDER BY scheduled_date ASC"
        };

        let mut stmt = conn.prepare(query)?;
        let row_mapper = |row: &Row| self.row_to_inspection(row);
        let inspection_iter = if let Some(inspector_id) = inspector_id {
            stmt.query_map(params![inspector_id], row_mapper)?
        } else {
            stmt.query_map([], row_mapper)?
        };

        let mut inspections = Vec::new();
        for inspection in inspection_iter {
            inspections.push(inspection?);
        }

        drop(stmt);
        self.database.return_connection(conn);
        Ok(inspections)
    }

    pub fn create_inspection_item(&self, item: InspectionItem) -> AppResult<InspectionItem> {
        info!("Creating inspection item: {}", item.item_name);
        item.validate()?;

        self.database.with_transaction(|conn| {
            let id = conn.query_row(
                "INSERT INTO inspection_items (inspection_id, component_id, item_name, item_category,
                 condition, finding, severity, is_compliant, corrective_action)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 RETURNING id",
                params![
                    item.inspection_id, item.component_id, item.item_name, item.item_category,
                    item.condition.as_ref().map(|c| c.to_string()), item.finding,
                    item.severity.as_ref().map(|s| s.to_string()), item.is_compliant,
                    item.corrective_action
                ],
                |row| row.get::<_, i64>(0),
            )?;

            debug!("Inspection item created with ID: {}", id);
            self.get_inspection_item_by_id(id)
        })
    }

    pub fn update_inspection_item(&self, id: i64, updates: InspectionItemUpdateData) -> AppResult<InspectionItem> {
        info!("Updating inspection item: {}", id);
        
        self.database.with_transaction(|conn| {
            // Simple implementation - update individual fields
            if let Some(condition) = &updates.condition {
                conn.execute("UPDATE inspection_items SET condition = ?1 WHERE id = ?2", params![condition.to_string(), id])?;
            }
            if let Some(finding) = &updates.finding {
                conn.execute("UPDATE inspection_items SET finding = ?1 WHERE id = ?2", params![finding, id])?;
            }
            if let Some(is_compliant) = &updates.is_compliant {
                conn.execute("UPDATE inspection_items SET is_compliant = ?1 WHERE id = ?2", params![is_compliant, id])?;
            }
            if let Some(component_id) = &updates.component_id {
                conn.execute("UPDATE inspection_items SET component_id = ?1 WHERE id = ?2", params![component_id, id])?;
            }
            if let Some(item_name) = &updates.item_name {
                conn.execute("UPDATE inspection_items SET item_name = ?1 WHERE id = ?2", params![item_name, id])?;
            }
            if let Some(item_category) = &updates.item_category {
                conn.execute("UPDATE inspection_items SET item_category = ?1 WHERE id = ?2", params![item_category, id])?;
            }
            if let Some(severity) = &updates.severity {
                conn.execute("UPDATE inspection_items SET severity = ?1 WHERE id = ?2", params![severity.to_string(), id])?;
            }
            if let Some(corrective_action) = &updates.corrective_action {
                conn.execute("UPDATE inspection_items SET corrective_action = ?1 WHERE id = ?2", params![corrective_action, id])?;
            }

            debug!("Inspection item {} updated successfully", id);
            self.get_inspection_item_by_id(id)
        })
    }

    pub fn get_inspection_items(&self, inspection_id: i64) -> AppResult<Vec<InspectionItem>> {
        debug!("Fetching inspection items for inspection: {}", inspection_id);
        let conn = self.database.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, inspection_id, component_id, item_name, item_category, condition,
             finding, severity, is_compliant, corrective_action, created_at
             FROM inspection_items WHERE inspection_id = ?1 ORDER BY item_name"
        )?;

        let item_iter = stmt.query_map(params![inspection_id], |row| self.row_to_inspection_item(row))?;

        let mut items = Vec::new();
        for item in item_iter {
            items.push(item?);
        }

        drop(stmt);
        self.database.return_connection(conn);
        Ok(items)
    }

    fn get_inspection_item_by_id(&self, id: i64) -> AppResult<InspectionItem> {
        let conn = self.database.get_connection()?;
        let item = conn.query_row(
            "SELECT id, inspection_id, component_id, item_name, item_category, condition,
             finding, severity, is_compliant, corrective_action, created_at
             FROM inspection_items WHERE id = ?1",
            params![id],
            |row| self.row_to_inspection_item(row),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "InspectionItem".to_string(),
            field: "id".to_string(),
            value: id.to_string(),
        })?;

        self.database.return_connection(conn);
        Ok(item)
    }

    fn row_to_inspection(&self, row: &Row) -> rusqlite::Result<Inspection> {
        Ok(Inspection {
            id: row.get(0)?,
            asset_id: row.get(1)?,
            inspector_id: row.get(2)?,
            inspection_type: row.get::<_, String>(3)?.parse().unwrap_or(InspectionType::Frequent),
            compliance_standard: row.get(4)?,
            scheduled_date: row.get(5)?,
            actual_date: row.get(6)?,
            status: row.get::<_, String>(7)?.parse().unwrap_or(InspectionStatus::Scheduled),
            overall_condition: row.get::<_, Option<String>>(8)?
                .and_then(|s| s.parse().ok()),
            checklist_data: row.get::<_, Option<String>>(9)?
                .and_then(|s| serde_json::from_str(&s).ok()),
            notes: row.get(10)?,
            ai_analysis_results: row.get::<_, Option<String>>(11)?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    }

    fn row_to_inspection_item(&self, row: &Row) -> rusqlite::Result<InspectionItem> {
        Ok(InspectionItem {
            id: row.get(0)?,
            inspection_id: row.get(1)?,
            component_id: row.get(2)?,
            item_name: row.get(3)?,
            item_category: row.get(4)?,
            condition: row.get::<_, Option<String>>(5)?
                .and_then(|s| s.parse().ok()),
            finding: row.get(6)?,
            severity: row.get::<_, Option<String>>(7)?
                .and_then(|s| s.parse().ok()),
            is_compliant: row.get(8)?,
            corrective_action: row.get(9)?,
            created_at: row.get(10)?,
        })
    }
}

// =============================================================================
// Compliance Service
// =============================================================================

pub struct ComplianceService {
    database: Arc<Database>,
}

impl ComplianceService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub fn get_compliance_standards(&self) -> AppResult<Vec<ComplianceStandard>> {
        debug!("Fetching all compliance standards");
        let conn = self.database.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, standard_code, standard_name, version, requirements, is_active, created_at, updated_at
             FROM compliance_standards WHERE is_active = 1 ORDER BY standard_code"
        )?;

        let standard_iter = stmt.query_map([], |row| self.row_to_compliance_standard(row))?;

        let mut standards = Vec::new();
        for standard in standard_iter {
            standards.push(standard?);
        }

        drop(stmt);
        self.database.return_connection(conn);
        Ok(standards)
    }

    pub fn get_compliance_standard_by_code(&self, code: String) -> AppResult<ComplianceStandard> {
        debug!("Fetching compliance standard by code: {}", code);
        let conn = self.database.get_connection()?;

        let standard = conn.query_row(
            "SELECT id, standard_code, standard_name, version, requirements, is_active, created_at, updated_at
             FROM compliance_standards WHERE standard_code = ?1 AND is_active = 1",
            params![code],
            |row| self.row_to_compliance_standard(row),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "ComplianceStandard".to_string(),
            field: "standard_code".to_string(),
            value: code,
        })?;

        self.database.return_connection(conn);
        Ok(standard)
    }

    pub fn get_checklist_templates_by_standard(&self, standard_id: i64) -> AppResult<Vec<ComplianceChecklistTemplate>> {
        debug!("Fetching checklist templates for standard: {}", standard_id);
        let conn = self.database.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, standard_id, template_name, inspection_type, checklist_structure, created_at, updated_at
             FROM compliance_checklist_templates WHERE standard_id = ?1 ORDER BY template_name"
        )?;

        let template_iter = stmt.query_map(params![standard_id], |row| self.row_to_checklist_template(row))?;

        let mut templates = Vec::new();
        for template in template_iter {
            templates.push(template?);
        }

        drop(stmt);
        self.database.return_connection(conn);
        Ok(templates)
    }

    pub fn generate_inspection_checklist(&self, standard_id: i64, inspection_type: InspectionType) -> AppResult<JsonValue> {
        info!("Generating inspection checklist for standard: {} and type: {}", standard_id, inspection_type);
        let conn = self.database.get_connection()?;

        let template = conn.query_row(
            "SELECT checklist_structure FROM compliance_checklist_templates 
             WHERE standard_id = ?1 AND inspection_type = ?2",
            params![standard_id, inspection_type.to_string()],
            |row| row.get::<_, String>(0),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "ChecklistTemplate".to_string(),
            field: "standard_id_inspection_type".to_string(),
            value: format!("{}_{}", standard_id, inspection_type),
        })?;

        self.database.return_connection(conn);
        
        serde_json::from_str(&template).map_err(|e| AppError::InvalidFormat {
            field: "checklist_structure".to_string(),
            expected: "valid JSON".to_string(),
            actual: e.to_string(),
        })
    }

    pub fn validate_inspection_completion(&self, inspection_id: i64) -> AppResult<ValidationResult> {
        info!("Validating inspection completion: {}", inspection_id);
        let conn = self.database.get_connection()?;

        // Get inspection details
        let inspection = conn.query_row(
            "SELECT status, checklist_data FROM inspections WHERE id = ?1",
            params![inspection_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                ))
            },
        ).map_err(|_| AppError::RecordNotFound {
            entity: "Inspection".to_string(),
            field: "id".to_string(),
            value: inspection_id.to_string(),
        })?;

        let (status, checklist_data) = inspection;
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Basic validation
        if status != "Completed" {
            errors.push("Inspection must be completed".to_string());
        }

        if checklist_data.is_none() {
            errors.push("Checklist data is required".to_string());
        }

        // Get inspection items and check completion
        let item_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM inspection_items WHERE inspection_id = ?1",
            params![inspection_id],
            |row| row.get(0),
        )?;

        if item_count == 0 {
            warnings.push("No inspection items found".to_string());
        }

        // Check for items without compliance status
        let incomplete_items: i64 = conn.query_row(
            "SELECT COUNT(*) FROM inspection_items WHERE inspection_id = ?1 AND is_compliant IS NULL",
            params![inspection_id],
            |row| row.get(0),
        )?;

        if incomplete_items > 0 {
            warnings.push(format!("{} items missing compliance status", incomplete_items));
        }

        // Calculate compliance score
        let compliance_score = self.calculate_compliance_score(inspection_id)?;

        self.database.return_connection(conn);

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            compliance_score,
        })
    }

    pub fn calculate_compliance_score(&self, inspection_id: i64) -> AppResult<f64> {
        debug!("Calculating compliance score for inspection: {}", inspection_id);
        let conn = self.database.get_connection()?;

        let (total_items, compliant_items): (i64, i64) = conn.query_row(
            "SELECT 
                COUNT(*) as total,
                COUNT(CASE WHEN is_compliant = 1 THEN 1 END) as compliant
             FROM inspection_items WHERE inspection_id = ?1",
            params![inspection_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        self.database.return_connection(conn);

        if total_items == 0 {
            return Ok(0.0);
        }

        Ok((compliant_items as f64 / total_items as f64) * 100.0)
    }

    pub fn calculate_next_inspection_date(&self, asset_id: i64, inspection_type: InspectionType) -> AppResult<DateTime<Utc>> {
        info!("Calculating next inspection date for asset: {} type: {}", asset_id, inspection_type);
        let conn = self.database.get_connection()?;

        // Get the last inspection date
        let last_inspection: Option<DateTime<Utc>> = conn.query_row(
            "SELECT MAX(actual_date) FROM inspections 
             WHERE asset_id = ?1 AND inspection_type = ?2 AND status = 'Completed'",
            params![asset_id, inspection_type.to_string()],
            |row| row.get(0),
        ).unwrap_or(None);

        self.database.return_connection(conn);

        let base_date = last_inspection.unwrap_or_else(Utc::now);
        
        // Calculate next inspection based on type
        let next_date = match inspection_type {
            InspectionType::Frequent => base_date + chrono::Duration::days(30),  // Monthly
            InspectionType::Periodic => base_date + chrono::Duration::days(365), // Yearly
            InspectionType::Initial => base_date + chrono::Duration::days(1),    // Immediate
            InspectionType::Special => base_date + chrono::Duration::days(90),   // Quarterly
        };

        Ok(next_date)
    }

    fn row_to_compliance_standard(&self, row: &Row) -> rusqlite::Result<ComplianceStandard> {
        Ok(ComplianceStandard {
            id: row.get(0)?,
            standard_code: row.get(1)?,
            standard_name: row.get(2)?,
            version: row.get(3)?,
            requirements: row.get::<_, Option<String>>(4)?
                .and_then(|s| serde_json::from_str(&s).ok()),
            is_active: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }

    fn row_to_checklist_template(&self, row: &Row) -> rusqlite::Result<ComplianceChecklistTemplate> {
        Ok(ComplianceChecklistTemplate {
            id: row.get(0)?,
            standard_id: row.get(1)?,
            template_name: row.get(2)?,
            inspection_type: row.get(3)?,
            checklist_structure: serde_json::from_str(&row.get::<_, String>(4)?)
                .unwrap_or(JsonValue::Null),
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }
}

// =============================================================================
// User Service
// =============================================================================

pub struct UserService {
    database: Arc<Database>,
}

impl UserService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Create a new user with plain text password that will be hashed
    ///
    /// # Arguments
    /// * `user` - User object with plain text password in password_hash field
    /// * `plain_password` - The plain text password to hash
    pub fn create_user(&self, mut user: User, plain_password: String) -> AppResult<User> {
        info!("Creating new user: {}", user.username);
        user.validate()?;

        // Validate password strength before proceeding
        let password_validation = self.validate_password_strength(&plain_password)?;
        if !password_validation.is_valid {
            return Err(AppError::validation(
                "password",
                format!("Password does not meet strength requirements: {}",
                    password_validation.issues.join(", "))
            ));
        }

        // Check for email uniqueness
        if self.email_exists(&user.email)? {
            return Err(AppError::DuplicateRecord {
                entity: "User".to_string(),
                field: "email".to_string(),
                value: user.email.clone(),
            });
        }

        // Check for username uniqueness
        if self.username_exists(&user.username)? {
            return Err(AppError::DuplicateRecord {
                entity: "User".to_string(),
                field: "username".to_string(),
                value: user.username.clone(),
            });
        }

        // Hash the password
        let password_hash = bcrypt::hash(&plain_password, bcrypt::DEFAULT_COST)?;
        user.password_hash = password_hash;

        self.database.with_transaction(|conn| {
            let id = conn.query_row(
                "INSERT INTO users (username, email, password_hash, role, first_name, last_name, phone, is_active)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 RETURNING id",
                params![
                    user.username, user.email, user.password_hash, user.role.to_string(),
                    user.first_name, user.last_name, user.phone, user.is_active
                ],
                |row| row.get::<_, i64>(0),
            )?;

            debug!("User created with ID: {}", id);
            info!("User {} created successfully with proper password hashing", user.username);
            self.get_user_by_id(id)
        })
    }

    pub fn get_user_by_id(&self, id: i64) -> AppResult<User> {
        debug!("Fetching user by ID: {}", id);
        let conn = self.database.get_connection()?;
        
        let user = conn.query_row(
            "SELECT id, username, email, password_hash, role, first_name, last_name, phone,
             created_at, updated_at, is_active
             FROM users WHERE id = ?1",
            params![id],
            |row| self.row_to_user(row),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "User".to_string(),
            field: "id".to_string(),
            value: id.to_string(),
        })?;

        self.database.return_connection(conn);
        Ok(user)
    }

    pub fn get_user_by_username(&self, username: String) -> AppResult<User> {
        debug!("Fetching user by username: {}", username);
        let conn = self.database.get_connection()?;
        
        let user = conn.query_row(
            "SELECT id, username, email, password_hash, role, first_name, last_name, phone,
             created_at, updated_at, is_active
             FROM users WHERE username = ?1",
            params![username],
            |row| self.row_to_user(row),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "User".to_string(),
            field: "username".to_string(),
            value: username,
        })?;

        self.database.return_connection(conn);
        Ok(user)
    }

    pub fn get_user_by_email(&self, email: String) -> AppResult<User> {
        debug!("Fetching user by email: {}", email);
        let conn = self.database.get_connection()?;
        
        let user = conn.query_row(
            "SELECT id, username, email, password_hash, role, first_name, last_name, phone,
             created_at, updated_at, is_active
             FROM users WHERE email = ?1",
            params![email],
            |row| self.row_to_user(row),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "User".to_string(),
            field: "email".to_string(),
            value: email,
        })?;

        self.database.return_connection(conn);
        Ok(user)
    }

    pub fn update_user(&self, id: i64, updates: UserUpdateData) -> AppResult<User> {
        info!("Updating user: {}", id);
        
        self.database.with_transaction(|conn| {
            // Simple implementation - update individual fields
            if let Some(username) = &updates.username {
                conn.execute("UPDATE users SET username = ?1 WHERE id = ?2", params![username, id])?;
            }
            if let Some(email) = &updates.email {
                conn.execute("UPDATE users SET email = ?1 WHERE id = ?2", params![email, id])?;
            }
            if let Some(role) = &updates.role {
                conn.execute("UPDATE users SET role = ?1 WHERE id = ?2", params![role.to_string(), id])?;
            }
            if let Some(first_name) = &updates.first_name {
                conn.execute("UPDATE users SET first_name = ?1 WHERE id = ?2", params![first_name, id])?;
            }
            if let Some(last_name) = &updates.last_name {
                conn.execute("UPDATE users SET last_name = ?1 WHERE id = ?2", params![last_name, id])?;
            }
            if let Some(phone) = &updates.phone {
                conn.execute("UPDATE users SET phone = ?1 WHERE id = ?2", params![phone, id])?;
            }
            if let Some(is_active) = &updates.is_active {
                conn.execute("UPDATE users SET is_active = ?1 WHERE id = ?2", params![is_active, id])?;
            }

            debug!("User {} updated successfully", id);
            self.get_user_by_id(id)
        })
    }

    pub fn delete_user(&self, id: i64) -> AppResult<()> {
        info!("Deleting user: {}", id);
        
        self.database.with_transaction(|conn| {
            let rows_affected = conn.execute("DELETE FROM users WHERE id = ?1", params![id])?;
            
            if rows_affected == 0 {
                return Err(AppError::RecordNotFound {
                    entity: "User".to_string(),
                    field: "id".to_string(),
                    value: id.to_string(),
                });
            }
            
            debug!("User {} deleted successfully", id);
            Ok(())
        })
    }

    /// Verify a user's password using bcrypt
    ///
    /// # Arguments
    /// * `user_id` - The user's ID
    /// * `password` - The plain text password to verify
    ///
    /// # Returns
    /// * `Ok(true)` if password matches
    /// * `Ok(false)` if password doesn't match
    /// * `Err` if user not found or other error
    pub fn verify_password(&self, user_id: i64, password: String) -> AppResult<bool> {
        debug!("Verifying password for user: {}", user_id);
        let conn = self.database.get_connection()?;
        
        let (password_hash, is_active): (String, bool) = conn.query_row(
            "SELECT password_hash, is_active FROM users WHERE id = ?1",
            params![user_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "User".to_string(),
            field: "id".to_string(),
            value: user_id.to_string(),
        })?;

        self.database.return_connection(conn);
        
        // Check if user account is active
        if !is_active {
            return Err(AppError::Authentication {
                reason: "User account is inactive".to_string(),
            });
        }

        // Verify password using bcrypt
        match bcrypt::verify(&password, &password_hash) {
            Ok(is_valid) => {
                if is_valid {
                    debug!("Password verification successful for user: {}", user_id);
                } else {
                    debug!("Password verification failed for user: {}", user_id);
                }
                Ok(is_valid)
            }
            Err(e) => {
                debug!("Password verification error for user {}: {}", user_id, e);
                Err(AppError::Authentication {
                    reason: "Password verification failed".to_string(),
                })
            }
        }
    }

    /// Update a user's password with validation and proper hashing
    ///
    /// # Arguments
    /// * `user_id` - The user's ID
    /// * `new_password` - The new plain text password
    pub fn update_password(&self, user_id: i64, new_password: String) -> AppResult<()> {
        info!("Updating password for user: {}", user_id);
        
        // Validate password strength
        let password_validation = self.validate_password_strength(&new_password)?;
        if !password_validation.is_valid {
            return Err(AppError::validation(
                "password",
                format!("New password does not meet strength requirements: {}",
                    password_validation.issues.join(", "))
            ));
        }

        // Check if user exists and is active
        let conn = self.database.get_connection()?;
        let is_active: bool = conn.query_row(
            "SELECT is_active FROM users WHERE id = ?1",
            params![user_id],
            |row| row.get(0),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "User".to_string(),
            field: "id".to_string(),
            value: user_id.to_string(),
        })?;
        self.database.return_connection(conn);

        if !is_active {
            return Err(AppError::Authentication {
                reason: "Cannot update password for inactive user".to_string(),
            });
        }
        
        // Hash the new password
        let password_hash = bcrypt::hash(&new_password, bcrypt::DEFAULT_COST)?;
        
        self.database.with_transaction(|conn| {
            let rows_affected = conn.execute(
                "UPDATE users SET password_hash = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![password_hash, user_id]
            )?;
            
            if rows_affected == 0 {
                return Err(AppError::RecordNotFound {
                    entity: "User".to_string(),
                    field: "id".to_string(),
                    value: user_id.to_string(),
                });
            }
            
            debug!("Password updated successfully for user: {}", user_id);
            info!("Password updated for user: {} with proper validation and hashing", user_id);
            Ok(())
        })
    }

    pub fn get_users_by_role(&self, role: UserRole, filter: QueryFilter) -> AppResult<PaginatedResult<User>> {
        info!("Fetching users by role: {}", role);
        let conn = self.database.get_connection()?;

        let offset = ((filter.page.unwrap_or(1) - 1) * filter.limit.unwrap_or(50)).max(0);
        let limit = filter.limit.unwrap_or(50);

        let mut stmt = conn.prepare(
            "SELECT id, username, email, password_hash, role, first_name, last_name, phone,
             created_at, updated_at, is_active
             FROM users WHERE role = ?1 AND is_active = 1
             ORDER BY last_name, first_name LIMIT ?2 OFFSET ?3"
        )?;

        let user_iter = stmt.query_map(params![role.to_string(), limit, offset], |row| self.row_to_user(row))?;

        let mut users = Vec::new();
        for user in user_iter {
            users.push(user?);
        }

        let total_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE role = ?1 AND is_active = 1",
            params![role.to_string()],
            |row| row.get(0),
        )?;

        drop(stmt);
        self.database.return_connection(conn);
        Ok(PaginatedResult::new(users, total_count, filter.page.unwrap_or(1), limit))
    }

    /// Validate password strength according to security requirements
    ///
    /// # Arguments
    /// * `password` - The plain text password to validate
    ///
    /// # Returns
    /// * `PasswordStrengthResult` with validation details
    pub fn validate_password_strength(&self, password: &str) -> AppResult<PasswordStrengthResult> {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut score = 0u8;

        // Check minimum length
        if password.len() >= 8 {
            score += 20;
        } else {
            issues.push("Password must be at least 8 characters long".to_string());
            suggestions.push("Use at least 8 characters".to_string());
        }

        // Check for uppercase letter
        if password.chars().any(|c| c.is_uppercase()) {
            score += 20;
        } else {
            issues.push("Password must contain at least one uppercase letter".to_string());
            suggestions.push("Add an uppercase letter (A-Z)".to_string());
        }

        // Check for lowercase letter
        if password.chars().any(|c| c.is_lowercase()) {
            score += 20;
        } else {
            issues.push("Password must contain at least one lowercase letter".to_string());
            suggestions.push("Add a lowercase letter (a-z)".to_string());
        }

        // Check for number
        if password.chars().any(|c| c.is_numeric()) {
            score += 20;
        } else {
            issues.push("Password must contain at least one number".to_string());
            suggestions.push("Add a number (0-9)".to_string());
        }

        // Check for special character
        if password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            score += 20;
        } else {
            issues.push("Password must contain at least one special character".to_string());
            suggestions.push("Add a special character (!@#$%^&*()_+-=[]{}|;:,.<>?)".to_string());
        }

        // Additional strength checks
        if password.len() >= 12 {
            score = score.saturating_add(10);
        }
        if password.chars().filter(|c| c.is_uppercase()).count() >= 2 {
            score = score.saturating_add(5);
        }
        if password.chars().filter(|c| c.is_numeric()).count() >= 2 {
            score = score.saturating_add(5);
        }

        let is_valid = issues.is_empty();
        
        debug!("Password strength validation: score={}, valid={}, issues={:?}", score, is_valid, issues);
        
        Ok(PasswordStrengthResult {
            is_valid,
            score,
            issues,
            suggestions,
        })
    }

    /// Check if an email already exists in the database
    ///
    /// # Arguments
    /// * `email` - The email to check
    ///
    /// # Returns
    /// * `true` if email exists, `false` otherwise
    pub fn email_exists(&self, email: &str) -> AppResult<bool> {
        let conn = self.database.get_connection()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE LOWER(email) = LOWER(?1)",
            params![email],
            |row| row.get(0),
        )?;
        self.database.return_connection(conn);
        Ok(count > 0)
    }

    /// Check if a username already exists in the database
    ///
    /// # Arguments
    /// * `username` - The username to check
    ///
    /// # Returns
    /// * `true` if username exists, `false` otherwise
    pub fn username_exists(&self, username: &str) -> AppResult<bool> {
        let conn = self.database.get_connection()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE LOWER(username) = LOWER(?1)",
            params![username],
            |row| row.get(0),
        )?;
        self.database.return_connection(conn);
        Ok(count > 0)
    }

    /// Get all users with pagination support
    ///
    /// # Arguments
    /// * `filter` - Query filter with pagination parameters
    ///
    /// # Returns
    /// * `PaginatedResult<User>` with users and pagination metadata
    pub fn get_all_users(&self, filter: QueryFilter) -> AppResult<PaginatedResult<User>> {
        info!("Fetching all users with filter: {:?}", filter);
        let conn = self.database.get_connection()?;

        let offset = ((filter.page.unwrap_or(1) - 1) * filter.limit.unwrap_or(50)).max(0);
        let limit = filter.limit.unwrap_or(50);
        let sort_order = filter.sort_order.unwrap_or(SortOrder::Desc);
        let sort_by = filter.sort_by.unwrap_or("created_at".to_string());

        // Build the ORDER BY clause
        let order_by = format!(" ORDER BY {} {}", sort_by, sort_order);

        let query = format!(
            "SELECT id, username, email, password_hash, role, first_name, last_name, phone,
             created_at, updated_at, is_active
             FROM users {} LIMIT {} OFFSET {}",
            order_by, limit, offset
        );

        let mut stmt = conn.prepare(&query)?;
        let user_iter = stmt.query_map([], |row| self.row_to_user(row))?;

        let mut users = Vec::new();
        for user in user_iter {
            users.push(user?);
        }

        // Get total count
        let total_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users",
            [],
            |row| row.get(0),
        )?;

        drop(stmt);
        self.database.return_connection(conn);
        Ok(PaginatedResult::new(users, total_count, filter.page.unwrap_or(1), limit))
    }

    /// Search users by various criteria
    ///
    /// # Arguments
    /// * `criteria` - Search criteria
    /// * `filter` - Query filter with pagination parameters
    ///
    /// # Returns
    /// * `PaginatedResult<User>` with matching users and pagination metadata
    pub fn search_users(&self, criteria: UserSearchCriteria, filter: QueryFilter) -> AppResult<PaginatedResult<User>> {
        info!("Searching users with criteria: {:?}", criteria);
        let conn = self.database.get_connection()?;

        let offset = ((filter.page.unwrap_or(1) - 1) * filter.limit.unwrap_or(50)).max(0);
        let limit = filter.limit.unwrap_or(50);
        let sort_order = filter.sort_order.unwrap_or(SortOrder::Desc);
        let sort_by = filter.sort_by.unwrap_or("created_at".to_string());

        // Build WHERE conditions
        let mut where_conditions = Vec::new();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
        
        // Create owned values for parameters to avoid lifetime issues
        let mut owned_params: Vec<String> = Vec::new();

        if let Some(ref username) = criteria.username {
            where_conditions.push("username LIKE ?");
            owned_params.push(format!("%{}%", username));
        }
        if let Some(ref email) = criteria.email {
            where_conditions.push("email LIKE ?");
            owned_params.push(format!("%{}%", email));
        }
        if let Some(ref first_name) = criteria.first_name {
            where_conditions.push("first_name LIKE ?");
            owned_params.push(format!("%{}%", first_name));
        }
        if let Some(ref last_name) = criteria.last_name {
            where_conditions.push("last_name LIKE ?");
            owned_params.push(format!("%{}%", last_name));
        }
        if let Some(ref role) = criteria.role {
            where_conditions.push("role = ?");
            owned_params.push(role.to_string());
        }
        if let Some(is_active) = criteria.is_active {
            where_conditions.push("is_active = ?");
            owned_params.push(is_active.to_string());
        }
        
        // Now create references to the owned values
        for param in &owned_params {
            params.push(param);
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_conditions.join(" AND "))
        };

        let order_by = format!(" ORDER BY {} {}", sort_by, sort_order);

        let query = format!(
            "SELECT id, username, email, password_hash, role, first_name, last_name, phone,
             created_at, updated_at, is_active
             FROM users{} {} LIMIT {} OFFSET {}",
            where_clause, order_by, limit, offset
        );

        let mut stmt = conn.prepare(&query)?;
        let user_iter = stmt.query_map(params.as_slice(), |row| self.row_to_user(row))?;

        let mut users = Vec::new();
        for user in user_iter {
            users.push(user?);
        }

        // Get total count with same WHERE conditions
        let count_query = format!("SELECT COUNT(*) FROM users{}", where_clause);
        let mut count_stmt = conn.prepare(&count_query)?;
        let total_count: i64 = count_stmt.query_row(params.as_slice(), |row| row.get(0))?;

        drop(stmt);
        drop(count_stmt);
        self.database.return_connection(conn);
        Ok(PaginatedResult::new(users, total_count, filter.page.unwrap_or(1), limit))
    }

    /// Enhanced get_users_by_role with better filtering
    ///
    /// # Arguments
    /// * `role` - The user role to filter by
    /// * `filter` - Query filter with pagination parameters
    /// * `include_inactive` - Whether to include inactive users
    ///
    /// # Returns
    /// * `PaginatedResult<User>` with users of the specified role
    pub fn get_users_by_role_enhanced(&self, role: UserRole, filter: QueryFilter, include_inactive: bool) -> AppResult<PaginatedResult<User>> {
        info!("Fetching users by role: {} (include_inactive: {})", role, include_inactive);
        let conn = self.database.get_connection()?;

        let offset = ((filter.page.unwrap_or(1) - 1) * filter.limit.unwrap_or(50)).max(0);
        let limit = filter.limit.unwrap_or(50);
        let sort_order = filter.sort_order.unwrap_or(SortOrder::Desc);
        let sort_by = filter.sort_by.unwrap_or("last_name".to_string());

        let where_clause = if include_inactive {
            "WHERE role = ?"
        } else {
            "WHERE role = ? AND is_active = 1"
        };

        let order_by = format!(" ORDER BY {} {}", sort_by, sort_order);

        let query = format!(
            "SELECT id, username, email, password_hash, role, first_name, last_name, phone,
             created_at, updated_at, is_active
             FROM users {} {} LIMIT {} OFFSET {}",
            where_clause, order_by, limit, offset
        );

        let mut stmt = conn.prepare(&query)?;
        let user_iter = stmt.query_map([role.to_string()], |row| self.row_to_user(row))?;

        let mut users = Vec::new();
        for user in user_iter {
            users.push(user?);
        }

        let total_count: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM users {}", where_clause),
            [role.to_string()],
            |row| row.get(0),
        )?;

        drop(stmt);
        self.database.return_connection(conn);
        Ok(PaginatedResult::new(users, total_count, filter.page.unwrap_or(1), limit))
    }

    // =============================================================================
    // Security Enhancement Methods (Placeholder implementations for future use)
    // =============================================================================

    /// Initialize rate limiting tracking for a user (placeholder)
    ///
    /// Note: This is a placeholder implementation for future rate limiting functionality
    pub fn init_rate_limiting(&self, user_id: i64) -> AppResult<()> {
        debug!("Initializing rate limiting for user: {} (placeholder)", user_id);
        // TODO: Implement actual rate limiting logic with Redis or in-memory store
        Ok(())
    }

    /// Check if user is rate limited (placeholder)
    ///
    /// Note: This is a placeholder implementation for future rate limiting functionality
    pub fn is_rate_limited(&self, user_id: i64, _action: &str) -> AppResult<bool> {
        debug!("Checking rate limit for user: {} (placeholder)", user_id);
        // TODO: Implement actual rate limiting check
        Ok(false)
    }

    /// Log user activity for audit purposes (placeholder)
    ///
    /// Note: This is a placeholder implementation for future audit functionality
    pub fn log_user_activity(&self, user_id: i64, activity: &str, metadata: Option<&str>) -> AppResult<()> {
        info!("User activity - ID: {}, Action: {}, Metadata: {:?} (placeholder)",
              user_id, activity, metadata);
        // TODO: Implement actual activity logging to database or external service
        Ok(())
    }

    /// Get user activity history (placeholder)
    ///
    /// Note: This is a placeholder implementation for future audit functionality
    pub fn get_user_activity_history(&self, user_id: i64, _filter: QueryFilter) -> AppResult<Vec<String>> {
        debug!("Getting activity history for user: {} (placeholder)", user_id);
        // TODO: Implement actual activity history retrieval
        Ok(vec!["Activity logging not yet implemented".to_string()])
    }

    /// Get account lockout information for a user
    ///
    /// Note: Basic implementation - extend as needed for production use
    pub fn get_account_lockout_info(&self, user_id: i64) -> AppResult<AccountLockoutInfo> {
        debug!("Getting account lockout info for user: {}", user_id);
        
        // For now, return a basic implementation
        // TODO: Implement actual lockout tracking with database table
        Ok(AccountLockoutInfo {
            user_id,
            failed_attempts: 0,
            locked_at: None,
            locked_until: None,
            is_locked: false,
        })
    }

    /// Lock user account (placeholder)
    ///
    /// Note: This is a placeholder implementation for future account lockout functionality
    pub fn lock_user_account(&self, user_id: i64, _reason: &str) -> AppResult<()> {
        info!("Locking user account: {} (placeholder)", user_id);
        // TODO: Implement actual account locking logic
        self.database.with_transaction(|conn| {
            conn.execute(
                "UPDATE users SET is_active = 0, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
                params![user_id]
            )?;
            Ok(())
        })
    }

    /// Unlock user account (placeholder)
    ///
    /// Note: This is a placeholder implementation for future account lockout functionality
    pub fn unlock_user_account(&self, user_id: i64) -> AppResult<()> {
        info!("Unlocking user account: {} (placeholder)", user_id);
        // TODO: Implement actual account unlocking logic
        self.database.with_transaction(|conn| {
            conn.execute(
                "UPDATE users SET is_active = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
                params![user_id]
            )?;
            Ok(())
        })
    }

    fn row_to_user(&self, row: &Row) -> rusqlite::Result<User> {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            email: row.get(2)?,
            password_hash: row.get(3)?,
            role: row.get::<_, String>(4)?.parse().unwrap_or(UserRole::Inspector),
            first_name: row.get(5)?,
            last_name: row.get(6)?,
            phone: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            is_active: row.get(10)?,
        })
    }
}

// =============================================================================
// Media Service
// =============================================================================

pub struct MediaService {
    database: Arc<Database>,
}

impl MediaService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub fn create_media_file(&self, media: MediaFile) -> AppResult<MediaFile> {
        info!("Creating new media file: {}", media.file_name);
        media.validate()?;

        self.database.with_transaction(|conn| {
            let id = conn.query_row(
                "INSERT INTO media_files (inspection_id, component_id, file_name, file_path,
                 file_type, mime_type, file_size, description, ai_analysis_metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 RETURNING id",
                params![
                    media.inspection_id, media.component_id, media.file_name, media.file_path,
                    media.file_type.to_string(), media.mime_type, media.file_size,
                    media.description,
                    media.ai_analysis_metadata.as_ref().map(|m| m.to_string())
                ],
                |row| row.get::<_, i64>(0),
            )?;

            debug!("Media file created with ID: {}", id);
            self.get_media_file_by_id(id)
        })
    }

    pub fn get_media_file_by_id(&self, id: i64) -> AppResult<MediaFile> {
        debug!("Fetching media file by ID: {}", id);
        let conn = self.database.get_connection()?;
        
        let media_file = conn.query_row(
            "SELECT id, inspection_id, component_id, file_name, file_path, file_type,
             mime_type, file_size, description, ai_analysis_metadata, created_at
             FROM media_files WHERE id = ?1",
            params![id],
            |row| self.row_to_media_file(row),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "MediaFile".to_string(),
            field: "id".to_string(),
            value: id.to_string(),
        })?;

        self.database.return_connection(conn);
        Ok(media_file)
    }

    pub fn get_media_files_by_inspection(&self, inspection_id: i64) -> AppResult<Vec<MediaFile>> {
        debug!("Fetching media files for inspection: {}", inspection_id);
        let conn = self.database.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, inspection_id, component_id, file_name, file_path, file_type,
             mime_type, file_size, description, ai_analysis_metadata, created_at
             FROM media_files WHERE inspection_id = ?1 ORDER BY created_at DESC"
        )?;

        let media_iter = stmt.query_map(params![inspection_id], |row| self.row_to_media_file(row))?;

        let mut media_files = Vec::new();
        for media in media_iter {
            media_files.push(media?);
        }

        drop(stmt);
        self.database.return_connection(conn);
        Ok(media_files)
    }

    pub fn get_media_files_by_component(&self, component_id: i64) -> AppResult<Vec<MediaFile>> {
        debug!("Fetching media files for component: {}", component_id);
        let conn = self.database.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, inspection_id, component_id, file_name, file_path, file_type,
             mime_type, file_size, description, ai_analysis_metadata, created_at
             FROM media_files WHERE component_id = ?1 ORDER BY created_at DESC"
        )?;

        let media_iter = stmt.query_map(params![component_id], |row| self.row_to_media_file(row))?;

        let mut media_files = Vec::new();
        for media in media_iter {
            media_files.push(media?);
        }

        drop(stmt);
        self.database.return_connection(conn);
        Ok(media_files)
    }

    pub fn update_media_file(&self, id: i64, updates: MediaFileUpdateData) -> AppResult<MediaFile> {
        info!("Updating media file: {}", id);
        
        self.database.with_transaction(|conn| {
            // Simple implementation - update individual fields
            if let Some(file_name) = &updates.file_name {
                conn.execute("UPDATE media_files SET file_name = ?1 WHERE id = ?2", params![file_name, id])?;
            }
            if let Some(description) = &updates.description {
                conn.execute("UPDATE media_files SET description = ?1 WHERE id = ?2", params![description, id])?;
            }
            if let Some(ai_analysis_metadata) = &updates.ai_analysis_metadata {
                conn.execute("UPDATE media_files SET ai_analysis_metadata = ?1 WHERE id = ?2", params![ai_analysis_metadata.to_string(), id])?;
            }

            debug!("Media file {} updated successfully", id);
            self.get_media_file_by_id(id)
        })
    }

    pub fn delete_media_file(&self, id: i64) -> AppResult<()> {
        info!("Deleting media file: {}", id);
        
        self.database.with_transaction(|conn| {
            let rows_affected = conn.execute("DELETE FROM media_files WHERE id = ?1", params![id])?;
            
            if rows_affected == 0 {
                return Err(AppError::RecordNotFound {
                    entity: "MediaFile".to_string(),
                    field: "id".to_string(),
                    value: id.to_string(),
                });
            }
            
            debug!("Media file {} deleted successfully", id);
            Ok(())
        })
    }

    pub fn queue_for_ai_analysis(&self, media_file_id: i64) -> AppResult<()> {
        info!("Queueing media file {} for AI analysis", media_file_id);
        
        // This is a stub implementation - in a real system, this would:
        // 1. Add the media file to an AI processing queue
        // 2. Create a pending AI model result record
        // 3. Trigger the AI processing pipeline
        
        self.database.with_transaction(|conn| {
            // Create a pending AI analysis record
            conn.execute(
                "INSERT INTO ai_model_results (media_file_id, model_name, model_version,
                 predictions, confidence_score, status)
                 VALUES (?1, 'vision_model_v1', '1.0', '{}', 0.0, 'Pending')",
                params![media_file_id]
            )?;
            
            debug!("Media file {} queued for AI analysis", media_file_id);
            Ok(())
        })
    }

    fn row_to_media_file(&self, row: &Row) -> rusqlite::Result<MediaFile> {
        Ok(MediaFile {
            id: row.get(0)?,
            inspection_id: row.get(1)?,
            component_id: row.get(2)?,
            file_name: row.get(3)?,
            file_path: row.get(4)?,
            file_type: row.get::<_, String>(5)?.parse().unwrap_or(MediaType::Image),
            mime_type: row.get(6)?,
            file_size: row.get(7)?,
            description: row.get(8)?,
            ai_analysis_metadata: row.get::<_, Option<String>>(9)?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.get(10)?,
        })
    }
}

// =============================================================================
// Report Service
// =============================================================================

pub struct ReportService {
    database: Arc<Database>,
}

impl ReportService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub fn generate_asset_summary_report(&self, asset_id: i64) -> AppResult<AssetSummaryReport> {
        info!("Generating asset summary report for asset: {}", asset_id);
        let conn = self.database.get_connection()?;

        // Get asset basic info
        let (asset_name, _asset_status): (String, String) = conn.query_row(
            "SELECT asset_name, status FROM assets WHERE id = ?1",
            params![asset_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "Asset".to_string(),
            field: "id".to_string(),
            value: asset_id.to_string(),
        })?;

        // Get inspection counts
        let (total_inspections, completed_inspections, pending_inspections): (i64, i64, i64) = conn.query_row(
            "SELECT
                COUNT(*) as total,
                COUNT(CASE WHEN status = 'Completed' THEN 1 END) as completed,
                COUNT(CASE WHEN status IN ('Scheduled', 'In Progress') THEN 1 END) as pending
             FROM inspections WHERE asset_id = ?1",
            params![asset_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        // Get last inspection date and condition
        let (last_inspection_date, overall_condition): (Option<DateTime<Utc>>, Option<String>) = conn.query_row(
            "SELECT actual_date, overall_condition FROM inspections
             WHERE asset_id = ?1 AND status = 'Completed'
             ORDER BY actual_date DESC LIMIT 1",
            params![asset_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap_or((None, None));

        // Get maintenance record count
        let maintenance_records: i64 = conn.query_row(
            "SELECT COUNT(*) FROM maintenance_records WHERE asset_id = ?1",
            params![asset_id],
            |row| row.get(0),
        )?;

        // Calculate compliance score (average of all completed inspections)
        let compliance_score: f64 = if completed_inspections > 0 {
            let mut total_score = 0.0;
            let mut stmt = conn.prepare(
                "SELECT id FROM inspections WHERE asset_id = ?1 AND status = 'Completed'"
            )?;
            let inspection_iter = stmt.query_map(params![asset_id], |row| row.get::<_, i64>(0))?;
            
            for inspection_result in inspection_iter {
                let inspection_id = inspection_result?;
                // Calculate compliance score for this inspection
                let (total_items, compliant_items): (i64, i64) = conn.query_row(
                    "SELECT
                        COUNT(*) as total,
                        COUNT(CASE WHEN is_compliant = 1 THEN 1 END) as compliant
                     FROM inspection_items WHERE inspection_id = ?1",
                    params![inspection_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;
                
                if total_items > 0 {
                    total_score += (compliant_items as f64 / total_items as f64) * 100.0;
                }
            }
            total_score / completed_inspections as f64
        } else {
            0.0
        };

        // Calculate next inspection date (placeholder logic)
        let next_inspection_date = last_inspection_date
            .map(|date| date + chrono::Duration::days(365)) // Assume yearly inspections
            .or_else(|| Some(Utc::now() + chrono::Duration::days(30)));

        self.database.return_connection(conn);

        Ok(AssetSummaryReport {
            asset_id,
            asset_name,
            total_inspections,
            completed_inspections,
            pending_inspections,
            last_inspection_date,
            next_inspection_date,
            overall_condition: overall_condition.and_then(|s| s.parse().ok()),
            maintenance_records,
            compliance_score,
        })
    }

    pub fn generate_inspection_completion_report(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> AppResult<InspectionCompletionReport> {
        info!("Generating inspection completion report from {} to {}", start_date, end_date);
        let conn = self.database.get_connection()?;

        // Get total scheduled and completed inspections
        let (total_scheduled, total_completed): (i64, i64) = conn.query_row(
            "SELECT
                COUNT(*) as scheduled,
                COUNT(CASE WHEN status = 'Completed' THEN 1 END) as completed
             FROM inspections
             WHERE scheduled_date BETWEEN ?1 AND ?2",
            params![start_date, end_date],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let completion_rate = if total_scheduled > 0 {
            (total_completed as f64 / total_scheduled as f64) * 100.0
        } else {
            0.0
        };

        // Get inspections by inspector
        let mut by_inspector = HashMap::new();
        let mut stmt = conn.prepare(
            "SELECT u.first_name || ' ' || u.last_name as inspector_name, COUNT(*) as count
             FROM inspections i
             JOIN users u ON i.inspector_id = u.id
             WHERE i.scheduled_date BETWEEN ?1 AND ?2 AND i.status = 'Completed'
             GROUP BY i.inspector_id, inspector_name"
        )?;

        let inspector_iter = stmt.query_map(params![start_date, end_date], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for result in inspector_iter {
            let (inspector_name, count) = result?;
            by_inspector.insert(inspector_name, count);
        }

        drop(stmt);

        // Get inspections by asset type
        let mut by_asset_type = HashMap::new();
        let mut stmt = conn.prepare(
            "SELECT a.asset_type, COUNT(*) as count
             FROM inspections i
             JOIN assets a ON i.asset_id = a.id
             WHERE i.scheduled_date BETWEEN ?1 AND ?2 AND i.status = 'Completed'
             GROUP BY a.asset_type"
        )?;

        let asset_type_iter = stmt.query_map(params![start_date, end_date], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for result in asset_type_iter {
            let (asset_type, count) = result?;
            by_asset_type.insert(asset_type, count);
        }

        drop(stmt);

        // Calculate average completion time
        let average_completion_time_hours: f64 = conn.query_row(
            "SELECT AVG(
                CASE
                    WHEN scheduled_date IS NOT NULL AND actual_date IS NOT NULL
                    THEN (julianday(actual_date) - julianday(scheduled_date)) * 24
                    ELSE 0
                END
             ) FROM inspections
             WHERE scheduled_date BETWEEN ?1 AND ?2 AND status = 'Completed'",
            params![start_date, end_date],
            |row| row.get(0),
        ).unwrap_or(0.0);

        self.database.return_connection(conn);

        Ok(InspectionCompletionReport {
            period_start: start_date,
            period_end: end_date,
            total_scheduled,
            total_completed,
            completion_rate,
            by_inspector,
            by_asset_type,
            average_completion_time_hours,
        })
    }

    pub fn generate_compliance_status_report(&self, location_id: Option<i64>) -> AppResult<ComplianceStatusReport> {
        info!("Generating compliance status report for location: {:?}", location_id);
        let conn = self.database.get_connection()?;

        // Get total assets
        let total_assets: i64 = if let Some(loc_id) = location_id {
            conn.query_row(
                "SELECT COUNT(*) FROM assets a WHERE a.location_id = ?1",
                params![loc_id],
                |row| row.get(0),
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM assets a",
                [],
                |row| row.get(0),
            )?
        };

        // Get assets with recent inspections and their compliance status
        let (assets_with_inspections, compliant_assets): (i64, i64) = if let Some(loc_id) = location_id {
            conn.query_row(
                "SELECT
                    COUNT(DISTINCT a.id) as total_with_inspections,
                    COUNT(DISTINCT CASE
                        WHEN recent_inspections.compliance_score >= 80 THEN a.id
                    END) as compliant_assets
                 FROM assets a
                 LEFT JOIN (
                     SELECT
                         i.asset_id,
                         AVG(compliance_scores.score) as compliance_score
                     FROM inspections i
                     LEFT JOIN (
                         SELECT
                             inspection_id,
                             (COUNT(CASE WHEN is_compliant = 1 THEN 1 END) * 100.0 / COUNT(*)) as score
                         FROM inspection_items
                         GROUP BY inspection_id
                     ) compliance_scores ON i.id = compliance_scores.inspection_id
                     WHERE i.status = 'Completed'
                     GROUP BY i.asset_id
                 ) recent_inspections ON a.id = recent_inspections.asset_id
                 WHERE a.location_id = ?1",
                params![loc_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?
        } else {
            conn.query_row(
                "SELECT
                    COUNT(DISTINCT a.id) as total_with_inspections,
                    COUNT(DISTINCT CASE
                        WHEN recent_inspections.compliance_score >= 80 THEN a.id
                    END) as compliant_assets
                 FROM assets a
                 LEFT JOIN (
                     SELECT
                         i.asset_id,
                         AVG(compliance_scores.score) as compliance_score
                     FROM inspections i
                     LEFT JOIN (
                         SELECT
                             inspection_id,
                             (COUNT(CASE WHEN is_compliant = 1 THEN 1 END) * 100.0 / COUNT(*)) as score
                         FROM inspection_items
                         GROUP BY inspection_id
                     ) compliance_scores ON i.id = compliance_scores.inspection_id
                     WHERE i.status = 'Completed'
                     GROUP BY i.asset_id
                 ) recent_inspections ON a.id = recent_inspections.asset_id",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?
        };

        let non_compliant_assets = assets_with_inspections - compliant_assets;
        let compliance_percentage = if assets_with_inspections > 0 {
            (compliant_assets as f64 / assets_with_inspections as f64) * 100.0
        } else {
            0.0
        };

        // Get overdue inspections
        let overdue_inspections: i64 = if let Some(loc_id) = location_id {
            conn.query_row(
                "SELECT COUNT(DISTINCT a.id)
                 FROM assets a
                 LEFT JOIN inspections i ON a.id = i.asset_id
                 WHERE (i.scheduled_date < datetime('now') AND i.status NOT IN ('Completed', 'Cancelled'))
                    OR (i.id IS NULL)  -- Assets with no inspections
                 AND a.location_id = ?1",
                params![loc_id],
                |row| row.get(0),
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(DISTINCT a.id)
                 FROM assets a
                 LEFT JOIN inspections i ON a.id = i.asset_id
                 WHERE (i.scheduled_date < datetime('now') AND i.status NOT IN ('Completed', 'Cancelled'))
                    OR (i.id IS NULL)",
                [],
                |row| row.get(0),
            )?
        };

        // Get critical findings
        let critical_findings: i64 = if let Some(loc_id) = location_id {
            conn.query_row(
                "SELECT COUNT(*)
                 FROM inspection_items ii
                 JOIN inspections i ON ii.inspection_id = i.id
                 JOIN assets a ON i.asset_id = a.id
                 WHERE ii.severity = 'Critical' AND i.status = 'Completed'
                 AND a.location_id = ?1",
                params![loc_id],
                |row| row.get(0),
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(*)
                 FROM inspection_items ii
                 JOIN inspections i ON ii.inspection_id = i.id
                 JOIN assets a ON i.asset_id = a.id
                 WHERE ii.severity = 'Critical' AND i.status = 'Completed'",
                [],
                |row| row.get(0),
            )?
        };

        // Get compliance by standard
        let mut by_standard = HashMap::new();
        let stmt = if let Some(loc_id) = location_id {
            let mut stmt = conn.prepare(
                "SELECT
                    i.compliance_standard,
                    COUNT(DISTINCT a.id) as total_assets,
                    COUNT(DISTINCT CASE
                        WHEN compliance_scores.score >= 80 THEN a.id
                    END) as compliant
                 FROM assets a
                 JOIN inspections i ON a.id = i.asset_id
                 LEFT JOIN (
                     SELECT
                         inspection_id,
                         (COUNT(CASE WHEN is_compliant = 1 THEN 1 END) * 100.0 / COUNT(*)) as score
                     FROM inspection_items
                     GROUP BY inspection_id
                 ) compliance_scores ON i.id = compliance_scores.inspection_id
                 WHERE i.status = 'Completed' AND a.location_id = ?1
                 GROUP BY i.compliance_standard"
            )?;
            let standard_iter = stmt.query_map(params![loc_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?;
            for result in standard_iter {
                let (standard_code, total, compliant) = result?;
                let compliance_rate = if total > 0 {
                    (compliant as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                
                by_standard.insert(standard_code.clone(), ComplianceStandardStatus {
                    standard_code,
                    total_assets: total,
                    compliant,
                    compliance_rate,
                });
            }
            stmt
        } else {
            let mut stmt = conn.prepare(
                "SELECT
                    i.compliance_standard,
                    COUNT(DISTINCT a.id) as total_assets,
                    COUNT(DISTINCT CASE
                        WHEN compliance_scores.score >= 80 THEN a.id
                    END) as compliant
                 FROM assets a
                 JOIN inspections i ON a.id = i.asset_id
                 LEFT JOIN (
                     SELECT
                         inspection_id,
                         (COUNT(CASE WHEN is_compliant = 1 THEN 1 END) * 100.0 / COUNT(*)) as score
                     FROM inspection_items
                     GROUP BY inspection_id
                 ) compliance_scores ON i.id = compliance_scores.inspection_id
                 WHERE i.status = 'Completed'
                 GROUP BY i.compliance_standard"
            )?;
            let standard_iter = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?;
            for result in standard_iter {
                let (standard_code, total, compliant) = result?;
                let compliance_rate = if total > 0 {
                    (compliant as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                
                by_standard.insert(standard_code.clone(), ComplianceStandardStatus {
                    standard_code,
                    total_assets: total,
                    compliant,
                    compliance_rate,
                });
            }
            stmt
        };

        drop(stmt);
        self.database.return_connection(conn);

        Ok(ComplianceStatusReport {
            location_id,
            total_assets,
            compliant_assets,
            non_compliant_assets,
            overdue_inspections,
            compliance_percentage,
            critical_findings,
            by_standard,
        })
    }

    pub fn generate_maintenance_history_report(&self, asset_id: i64) -> AppResult<MaintenanceHistoryReport> {
        info!("Generating maintenance history report for asset: {}", asset_id);
        let conn = self.database.get_connection()?;

        // Get asset name
        let asset_name: String = conn.query_row(
            "SELECT asset_name FROM assets WHERE id = ?1",
            params![asset_id],
            |row| row.get(0),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "Asset".to_string(),
            field: "id".to_string(),
            value: asset_id.to_string(),
        })?;

        // Get maintenance statistics
        let (total_maintenance_records, preventive_maintenance, corrective_maintenance, emergency_maintenance): (i64, i64, i64, i64) = conn.query_row(
            "SELECT
                COUNT(*) as total,
                COUNT(CASE WHEN maintenance_type = 'Preventive' THEN 1 END) as preventive,
                COUNT(CASE WHEN maintenance_type = 'Corrective' THEN 1 END) as corrective,
                COUNT(CASE WHEN maintenance_type = 'Emergency' THEN 1 END) as emergency
             FROM maintenance_records WHERE asset_id = ?1 AND status = 'Completed'",
            params![asset_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;

        // Get cost statistics
        let (total_cost, average_cost_per_maintenance): (f64, f64) = conn.query_row(
            "SELECT
                COALESCE(SUM(cost), 0.0) as total_cost,
                COALESCE(AVG(cost), 0.0) as average_cost
             FROM maintenance_records
             WHERE asset_id = ?1 AND status = 'Completed' AND cost IS NOT NULL",
            params![asset_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        // Get last maintenance date
        let last_maintenance_date: Option<DateTime<Utc>> = conn.query_row(
            "SELECT MAX(completed_date) FROM maintenance_records
             WHERE asset_id = ?1 AND status = 'Completed'",
            params![asset_id],
            |row| row.get(0),
        ).unwrap_or(None);

        // Get next scheduled maintenance
        let next_scheduled_maintenance: Option<DateTime<Utc>> = conn.query_row(
            "SELECT MIN(scheduled_date) FROM maintenance_records
             WHERE asset_id = ?1 AND status = 'Scheduled' AND scheduled_date > datetime('now')",
            params![asset_id],
            |row| row.get(0),
        ).unwrap_or(None);

        self.database.return_connection(conn);

        Ok(MaintenanceHistoryReport {
            asset_id,
            asset_name,
            total_maintenance_records,
            preventive_maintenance,
            corrective_maintenance,
            emergency_maintenance,
            total_cost,
            average_cost_per_maintenance,
            last_maintenance_date,
            next_scheduled_maintenance,
        })
    }
}

// =============================================================================
// Location Service
// =============================================================================

pub struct LocationService {
    database: Arc<Database>,
    asset_service: Arc<AssetService>,
}

impl LocationService {
    pub fn new(database: Arc<Database>, asset_service: Arc<AssetService>) -> Self {
        Self { database, asset_service }
    }

    pub fn create_location(&self, location: Location) -> AppResult<Location> {
        info!("Creating new location: {}", location.name);
        location.validate()?;

        self.database.with_transaction(|conn| {
            let id = conn.query_row(
                "INSERT INTO locations (name, address, latitude, longitude, description, parent_location_id, created_by, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'), datetime('now'))
                 RETURNING id",
                params![
                    location.name, location.address, location.latitude, location.longitude,
                    location.description, location.parent_location_id, location.created_by
                ],
                |row| row.get::<_, i64>(0),
            )?;

            debug!("Location created with ID: {}", id);
            self.get_location_by_id(id)
        })
    }

    pub fn get_location_by_id(&self, id: i64) -> AppResult<Location> {
        debug!("Fetching location by ID: {}", id);
        let conn = self.database.get_connection()?;
        
        let location = conn.query_row(
            "SELECT id, name, address, latitude, longitude, description, parent_location_id, created_by, created_at, updated_at
             FROM locations WHERE id = ?1",
            params![id],
            |row| self.row_to_location(row),
        ).map_err(|_| AppError::RecordNotFound {
            entity: "Location".to_string(),
            field: "id".to_string(),
            value: id.to_string(),
        })?;

        self.database.return_connection(conn);
        Ok(location)
    }

    pub fn update_location(&self, id: i64, updates: LocationUpdateData) -> AppResult<Location> {
        info!("Updating location: {}", id);
        
        self.database.with_transaction(|conn| {
            if let Some(name) = &updates.name {
                conn.execute("UPDATE locations SET name = ?1, updated_at = datetime('now') WHERE id = ?2", params![name, id])?;
            }
            if let Some(address) = &updates.address {
                conn.execute("UPDATE locations SET address = ?1, updated_at = datetime('now') WHERE id = ?2", params![address, id])?;
            }
            if let Some(latitude) = &updates.latitude {
                conn.execute("UPDATE locations SET latitude = ?1, updated_at = datetime('now') WHERE id = ?2", params![latitude, id])?;
            }
            if let Some(longitude) = &updates.longitude {
                conn.execute("UPDATE locations SET longitude = ?1, updated_at = datetime('now') WHERE id = ?2", params![longitude, id])?;
            }
            if let Some(description) = &updates.description {
                conn.execute("UPDATE locations SET description = ?1, updated_at = datetime('now') WHERE id = ?2", params![description, id])?;
            }
            if let Some(parent_location_id) = &updates.parent_location_id {
                conn.execute("UPDATE locations SET parent_location_id = ?1, updated_at = datetime('now') WHERE id = ?2", params![parent_location_id, id])?;
            }

            debug!("Location {} updated successfully", id);
            self.get_location_by_id(id)
        })
    }

    pub fn delete_location_safe(&self, id: i64) -> AppResult<LocationDeletionResult> {
        info!("Safely deleting location: {}", id);
        
        self.database.with_transaction(|conn| {
            // Check for dependent assets first
            let asset_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM assets WHERE location_id = ?1",
                params![id],
                |row| row.get(0),
            )?;

            if asset_count > 0 {
                return Ok(LocationDeletionResult {
                    success: false,
                    location_id: id,
                    affected_assets: asset_count,
                    message: format!("Cannot delete location: {} assets are still assigned to this location", asset_count),
                });
            }

            // Check for child locations
            let child_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM locations WHERE parent_location_id = ?1",
                params![id],
                |row| row.get(0),
            )?;

            if child_count > 0 {
                return Ok(LocationDeletionResult {
                    success: false,
                    location_id: id,
                    affected_assets: 0,
                    message: format!("Cannot delete location: {} child locations exist", child_count),
                });
            }

            // Safe to delete
            let rows_affected = conn.execute("DELETE FROM locations WHERE id = ?1", params![id])?;
            
            if rows_affected == 0 {
                return Err(AppError::RecordNotFound {
                    entity: "Location".to_string(),
                    field: "id".to_string(),
                    value: id.to_string(),
                });
            }
            
            debug!("Location {} deleted successfully", id);
            Ok(LocationDeletionResult {
                success: true,
                location_id: id,
                affected_assets: 0,
                message: "Location deleted successfully".to_string(),
            })
        })
    }

    pub fn get_location_with_assets(&self, id: i64) -> AppResult<LocationWithAssets> {
        debug!("Fetching location with assets: {}", id);
        let location = self.get_location_by_id(id)?;
        
        // Get assets using the asset service
        let filter = QueryFilter {
            page: Some(1),
            limit: Some(1000), // Get all assets for this location
            sort_by: Some("asset_name".to_string()),
            sort_order: Some(SortOrder::Asc),
            filters: HashMap::new(),
        };
        
        let asset_result = self.asset_service.get_assets_by_location(id, filter)?;
        
        Ok(LocationWithAssets {
            id: location.id,
            name: location.name,
            address: location.address,
            latitude: location.latitude,
            longitude: location.longitude,
            description: location.description,
            parent_location_id: location.parent_location_id,
            created_by: location.created_by,
            created_at: location.created_at,
            updated_at: location.updated_at,
            assets: asset_result.data,
        })
    }

    pub fn get_location_with_asset_summary(&self, id: i64) -> AppResult<LocationAssetSummary> {
        debug!("Fetching location with asset summary: {}", id);
        let location = self.get_location_by_id(id)?;
        let conn = self.database.get_connection()?;

        // Get asset count
        let asset_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM assets WHERE location_id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        // Get critical assets count (assuming assets with status 'Maintenance' or condition 'Critical')
        let critical_assets: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT a.id) FROM assets a
             LEFT JOIN inspections i ON a.id = i.asset_id AND i.status = 'Completed'
             WHERE a.location_id = ?1 AND (a.status = 'Maintenance' OR i.overall_condition = 'Critical')",
            params![id],
            |row| row.get(0),
        )?;

        self.database.return_connection(conn);

        Ok(LocationAssetSummary {
            id: location.id,
            name: location.name,
            address: location.address,
            latitude: location.latitude,
            longitude: location.longitude,
            description: location.description,
            parent_location_id: location.parent_location_id,
            created_by: location.created_by,
            created_at: location.created_at,
            updated_at: location.updated_at,
            asset_count,
            critical_assets,
        })
    }

    pub fn validate_asset_location_assignment(&self, asset_id: i64, location_id: i64) -> AppResult<()> {
        debug!("Validating asset-location assignment: asset={}, location={}", asset_id, location_id);
        
        // Check if location exists
        let _location = self.get_location_by_id(location_id)?;
        
        // Check if asset exists using asset service
        let _asset = self.asset_service.get_asset_by_id(asset_id)?;
        
        debug!("Asset-location assignment validation successful");
        Ok(())
    }

    pub fn search_locations_with_asset_counts(&self, query: String, filter: QueryFilter) -> AppResult<PaginatedResult<LocationWithAssetCount>> {
        info!("Searching locations with asset counts: {}", query);
        let conn = self.database.get_connection()?;

        let search_term = format!("%{}%", query);
        let offset = ((filter.page.unwrap_or(1) - 1) * filter.limit.unwrap_or(50)).max(0);
        let limit = filter.limit.unwrap_or(50);
        let sort_order = filter.sort_order.unwrap_or(SortOrder::Desc);
        let sort_by = filter.sort_by.unwrap_or("name".to_string());

        let order_by = format!(" ORDER BY {} {}", sort_by, sort_order);

        let search_query = format!(
            "SELECT l.id, l.name, l.address, l.latitude, l.longitude, l.description,
                    l.parent_location_id, l.created_by, l.created_at, l.updated_at,
                    COUNT(a.id) as asset_count
             FROM locations l
             LEFT JOIN assets a ON l.id = a.location_id
             WHERE l.name LIKE ?1 OR l.address LIKE ?1 OR l.description LIKE ?1
             GROUP BY l.id, l.name, l.address, l.latitude, l.longitude, l.description,
                      l.parent_location_id, l.created_by, l.created_at, l.updated_at
             {} LIMIT {} OFFSET {}",
            order_by, limit, offset
        );

        let mut stmt = conn.prepare(&search_query)?;
        let location_iter = stmt.query_map([&search_term], |row| {
            Ok(LocationWithAssetCount {
                id: row.get(0)?,
                name: row.get(1)?,
                address: row.get(2)?,
                latitude: row.get(3)?,
                longitude: row.get(4)?,
                description: row.get(5)?,
                parent_location_id: row.get(6)?,
                created_by: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                asset_count: row.get(10)?,
            })
        })?;

        let mut locations = Vec::new();
        for location in location_iter {
            locations.push(location?);
        }

        let total_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM locations
             WHERE name LIKE ?1 OR address LIKE ?1 OR description LIKE ?1",
            [&search_term],
            |row| row.get(0),
        )?;

        drop(stmt);
        self.database.return_connection(conn);
        Ok(PaginatedResult::new(locations, total_count, filter.page.unwrap_or(1), limit))
    }

    fn row_to_location(&self, row: &Row) -> rusqlite::Result<Location> {
        Ok(Location {
            id: row.get(0)?,
            name: row.get(1)?,
            address: row.get(2)?,
            latitude: row.get(3)?,
            longitude: row.get(4)?,
            description: row.get(5)?,
            parent_location_id: row.get(6)?,
            created_by: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }
}

// =============================================================================
// Main Services Struct
// =============================================================================

pub struct Services {
    pub assets: Arc<AssetService>,
    pub inspections: Arc<InspectionService>,
    pub compliance: Arc<ComplianceService>,
    pub users: Arc<UserService>,
    pub media: Arc<MediaService>,
    pub reports: Arc<ReportService>,
    pub locations: Arc<LocationService>,
}

impl Services {
    pub async fn init(database: Arc<Database>) -> AppResult<Self> {
        info!("Initializing services layer");
        
        let assets = Arc::new(AssetService::new(database.clone()));
        let inspections = Arc::new(InspectionService::new(database.clone()));
        let compliance = Arc::new(ComplianceService::new(database.clone()));
        let users = Arc::new(UserService::new(database.clone()));
        let media = Arc::new(MediaService::new(database.clone()));
        let reports = Arc::new(ReportService::new(database.clone()));
        let locations = Arc::new(LocationService::new(database.clone(), assets.clone()));
        
        info!("Services layer initialized successfully");
        Ok(Services {
            assets,
            inspections,
            compliance,
            users,
            media,
            reports,
            locations,
        })
    }
}

// Test modules
#[cfg(test)]
pub mod tests;
