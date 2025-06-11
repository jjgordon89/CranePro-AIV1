//! Inspection management command handlers
//! 
//! This module contains all Tauri command handlers for inspection management
//! operations including CRUD operations for inspections and inspection items.

use crate::api::{ApiResponse, QueryFilterRequest, CreateInspectionRequest, InspectionUpdateRequest,
                CreateInspectionItemRequest, InspectionItemUpdateRequest, PaginatedResponse};
use crate::commands::AppState;
use crate::middleware::auth::AuthHelper;
use crate::models::{Inspection, InspectionItem};
use crate::services::{InspectionUpdateData, InspectionItemUpdateData};
use crate::{require_resource_access, time_command, command_handler};
use tauri::State;
use log::{info, debug};

/// Create a new inspection
#[tauri::command]
pub async fn create_inspection_command(
    state: State<'_, AppState>,
    token: Option<String>,
    inspection_data: CreateInspectionRequest,
) -> Result<ApiResponse<Inspection>, String> {
    let result = time_command!("create_inspection", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "inspection", "create");

        // Create inspection
        let inspection = inspection_data.to_inspection();
        let created_inspection = state.services.inspections.create_inspection(inspection)
            .map_err(|e| format!("Failed to create inspection: {}", e))?;

        info!("Inspection created: ID {} for asset {} by user {}", 
              created_inspection.id,
              created_inspection.asset_id,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(created_inspection)
    });

    Ok(command_handler!("create_inspection", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get inspection by ID
#[tauri::command]
pub async fn get_inspection_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<Inspection>, String> {
    let result = time_command!("get_inspection", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "inspection", "read");

        // Get inspection
        let inspection = state.services.inspections.get_inspection_by_id(id)
            .map_err(|e| format!("Failed to get inspection: {}", e))?;

        debug!("Inspection retrieved: ID {} for asset {}", id, inspection.asset_id);
        Ok(inspection)
    });

    Ok(command_handler!("get_inspection", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Update inspection
#[tauri::command]
pub async fn update_inspection_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
    updates: InspectionUpdateRequest,
) -> Result<ApiResponse<Inspection>, String> {
    let result = time_command!("update_inspection", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "inspection", "update");

        // Convert request to service update data
        let update_data = InspectionUpdateData {
            inspector_id: updates.inspector_id,
            inspection_type: updates.inspection_type,
            compliance_standard: updates.compliance_standard,
            scheduled_date: updates.scheduled_date,
            actual_date: updates.actual_date,
            status: updates.status,
            overall_condition: updates.overall_condition,
            checklist_data: updates.checklist_data,
            notes: updates.notes,
            ai_analysis_results: updates.ai_analysis_results,
        };

        // Update inspection
        let updated_inspection = state.services.inspections.update_inspection(id, update_data)
            .map_err(|e| format!("Failed to update inspection: {}", e))?;

        info!("Inspection updated: ID {} by user {}", 
              id, context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(updated_inspection)
    });

    Ok(command_handler!("update_inspection", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Submit inspection (mark as completed)
#[tauri::command]
pub async fn submit_inspection_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<Inspection>, String> {
    let result = time_command!("submit_inspection", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "inspection", "submit");

        // Submit inspection
        let submitted_inspection = state.services.inspections.submit_inspection(id)
            .map_err(|e| format!("Failed to submit inspection: {}", e))?;

        info!("Inspection submitted: ID {} by user {}", 
              id, context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(submitted_inspection)
    });

    Ok(command_handler!("submit_inspection", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get inspections by asset with filtering
#[tauri::command]
pub async fn get_inspections_by_asset_command(
    state: State<'_, AppState>,
    token: Option<String>,
    asset_id: i64,
    filter: QueryFilterRequest,
) -> Result<ApiResponse<PaginatedResponse<Inspection>>, String> {
    let result = time_command!("get_inspections_by_asset", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "inspection", "read");

        // Get inspections with filters
        let query_filter = filter.into();
        let paginated_inspections = state.services.inspections
            .get_inspections_by_asset(asset_id, query_filter)
            .map_err(|e| format!("Failed to get inspections by asset: {}", e))?;

        debug!("Retrieved {} inspections for asset {}", 
               paginated_inspections.data.len(), asset_id);

        let response = PaginatedResponse::from(paginated_inspections);
        Ok(response)
    });

    Ok(command_handler!("get_inspections_by_asset", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get pending inspections for inspector
#[tauri::command]
pub async fn get_pending_inspections_command(
    state: State<'_, AppState>,
    token: Option<String>,
    inspector_id: Option<i64>,
) -> Result<ApiResponse<Vec<Inspection>>, String> {
    let result = time_command!("get_pending_inspections", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "inspection", "read");

        // If no inspector_id provided, use current user's ID if they're an inspector
        let final_inspector_id = match inspector_id {
            Some(id) => Some(id),
            None => {
                let session = context.current_user()?;
                // Only return current user's inspections if they're an inspector
                // Supervisors and above can see all pending inspections
                match session.role {
                    crate::models::UserRole::Inspector => Some(session.user_id),
                    _ => None, // Supervisors see all pending inspections
                }
            }
        };

        // Get pending inspections
        let pending_inspections = state.services.inspections
            .get_pending_inspections(final_inspector_id)
            .map_err(|e| format!("Failed to get pending inspections: {}", e))?;

        debug!("Retrieved {} pending inspections for inspector {:?}", 
               pending_inspections.len(), final_inspector_id);

        Ok(pending_inspections)
    });

    Ok(command_handler!("get_pending_inspections", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Create inspection item
#[tauri::command]
pub async fn create_inspection_item_command(
    state: State<'_, AppState>,
    token: Option<String>,
    item_data: CreateInspectionItemRequest,
) -> Result<ApiResponse<InspectionItem>, String> {
    let result = time_command!("create_inspection_item", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "inspection", "update");

        // Create inspection item
        let inspection_item = item_data.to_inspection_item();
        let created_item = state.services.inspections.create_inspection_item(inspection_item)
            .map_err(|e| format!("Failed to create inspection item: {}", e))?;

        info!("Inspection item created: {} for inspection {} by user {}", 
              created_item.item_name,
              created_item.inspection_id,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(created_item)
    });

    Ok(command_handler!("create_inspection_item", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Update inspection item
#[tauri::command]
pub async fn update_inspection_item_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
    updates: InspectionItemUpdateRequest,
) -> Result<ApiResponse<InspectionItem>, String> {
    let result = time_command!("update_inspection_item", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "inspection", "update");

        // Convert request to service update data
        let update_data = InspectionItemUpdateData {
            component_id: updates.component_id,
            item_name: updates.item_name,
            item_category: updates.item_category,
            condition: updates.condition,
            finding: updates.finding,
            severity: updates.severity,
            is_compliant: updates.is_compliant,
            corrective_action: updates.corrective_action,
        };

        // Update inspection item
        let updated_item = state.services.inspections.update_inspection_item(id, update_data)
            .map_err(|e| format!("Failed to update inspection item: {}", e))?;

        info!("Inspection item updated: ID {} by user {}", 
              id, context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(updated_item)
    });

    Ok(command_handler!("update_inspection_item", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get inspection items for an inspection
#[tauri::command]
pub async fn get_inspection_items_command(
    state: State<'_, AppState>,
    token: Option<String>,
    inspection_id: i64,
) -> Result<ApiResponse<Vec<InspectionItem>>, String> {
    let result = time_command!("get_inspection_items", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "inspection", "read");

        // Get inspection items
        let inspection_items = state.services.inspections.get_inspection_items(inspection_id)
            .map_err(|e| format!("Failed to get inspection items: {}", e))?;

        debug!("Retrieved {} inspection items for inspection {}", 
               inspection_items.len(), inspection_id);

        Ok(inspection_items)
    });

    Ok(command_handler!("get_inspection_items", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}