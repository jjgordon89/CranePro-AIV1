//! Asset management command handlers
//! 
//! This module contains all Tauri command handlers for asset management
//! operations including CRUD operations for assets and components.

use crate::api::{ApiResponse, QueryFilterRequest, CreateAssetRequest, AssetUpdateRequest,
                CreateComponentRequest, ComponentUpdateRequest, PaginatedResponse};
use crate::commands::AppState;
use crate::middleware::auth::AuthHelper;
use crate::models::{Asset, Component};
use crate::services::AssetUpdateData;
use crate::{require_resource_access, time_command, command_handler};
use tauri::State;
use log::{info, debug};

/// Create a new asset
#[tauri::command]
pub async fn create_asset_command(
    state: State<'_, AppState>,
    token: Option<String>,
    asset_data: CreateAssetRequest,
) -> Result<ApiResponse<Asset>, String> {
    let result = time_command!("create_asset", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "asset", "create");

        // Validate and create asset
        let asset = asset_data.to_asset();
        let created_asset = state.services.assets.create_asset(asset)
            .map_err(|e| format!("Failed to create asset: {}", e))?;

        info!("Asset created: {} by user {}", 
              created_asset.asset_number, 
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(created_asset)
    });

    Ok(command_handler!("create_asset", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get asset by ID
#[tauri::command]
pub async fn get_asset_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<Asset>, String> {
    let result = time_command!("get_asset", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "asset", "read");

        // Get asset
        let asset = state.services.assets.get_asset_by_id(id)
            .map_err(|e| format!("Failed to get asset: {}", e))?;

        debug!("Asset retrieved: {} (ID: {})", asset.asset_name, id);
        Ok(asset)
    });

    Ok(command_handler!("get_asset", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get assets by location with filtering
#[tauri::command]
pub async fn get_assets_by_location_command(
    state: State<'_, AppState>,
    token: Option<String>,
    location_id: i64,
    filter: QueryFilterRequest,
) -> Result<ApiResponse<PaginatedResponse<Asset>>, String> {
    let result = time_command!("get_assets_by_location", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "asset", "read");

        // Get assets with filters
        let query_filter = filter.into();
        let paginated_assets = state.services.assets.get_assets_by_location(location_id, query_filter)
            .map_err(|e| format!("Failed to get assets by location: {}", e))?;

        debug!("Retrieved {} assets for location {}", 
               paginated_assets.data.len(), location_id);

        let response = PaginatedResponse::from(paginated_assets);
        Ok(response)
    });

    Ok(command_handler!("get_assets_by_location", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Update asset
#[tauri::command]
pub async fn update_asset_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
    updates: AssetUpdateRequest,
) -> Result<ApiResponse<Asset>, String> {
    let result = time_command!("update_asset", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "asset", "update");

        // Convert request to service update data
        let update_data = AssetUpdateData {
            asset_name: updates.asset_name,
            asset_type: updates.asset_type,
            manufacturer: updates.manufacturer,
            model: updates.model,
            serial_number: updates.serial_number,
            manufacture_date: updates.manufacture_date,
            installation_date: updates.installation_date,
            capacity: updates.capacity,
            capacity_unit: updates.capacity_unit,
            location_id: updates.location_id,
            status: updates.status,
            description: updates.description,
            specifications: updates.specifications,
        };

        // Update asset
        let updated_asset = state.services.assets.update_asset(id, update_data)
            .map_err(|e| format!("Failed to update asset: {}", e))?;

        info!("Asset updated: {} (ID: {}) by user {}", 
              updated_asset.asset_name, id,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(updated_asset)
    });

    Ok(command_handler!("update_asset", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Delete asset
#[tauri::command]
pub async fn delete_asset_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<()>, String> {
    let result = time_command!("delete_asset", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "asset", "delete");

        // Delete asset
        state.services.assets.delete_asset(id)
            .map_err(|e| format!("Failed to delete asset: {}", e))?;

        info!("Asset deleted: ID {} by user {}", 
              id, context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(())
    });

    Ok(command_handler!("delete_asset", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Search assets with query and filters
#[tauri::command]
pub async fn search_assets_command(
    state: State<'_, AppState>,
    token: Option<String>,
    query: String,
    filter: QueryFilterRequest,
) -> Result<ApiResponse<PaginatedResponse<Asset>>, String> {
    let result = time_command!("search_assets", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "asset", "read");

        // Search assets
        let query_filter = filter.into();
        let search_results = state.services.assets.search_assets(query.clone(), query_filter)
            .map_err(|e| format!("Failed to search assets: {}", e))?;

        debug!("Asset search returned {} results for query: '{}'",
               search_results.data.len(), query);

        let response = PaginatedResponse::from(search_results);
        Ok(response)
    });

    Ok(command_handler!("search_assets", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get components for an asset
#[tauri::command]
pub async fn get_asset_components_command(
    state: State<'_, AppState>,
    token: Option<String>,
    asset_id: i64,
) -> Result<ApiResponse<Vec<Component>>, String> {
    let result = time_command!("get_asset_components", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "asset", "read");

        // Get components
        let components = state.services.assets.get_asset_components(asset_id)
            .map_err(|e| format!("Failed to get asset components: {}", e))?;

        debug!("Retrieved {} components for asset {}", 
               components.len(), asset_id);

        Ok(components)
    });

    Ok(command_handler!("get_asset_components", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Create a new component
#[tauri::command]
pub async fn create_component_command(
    state: State<'_, AppState>,
    token: Option<String>,
    component_data: CreateComponentRequest,
) -> Result<ApiResponse<Component>, String> {
    let result = time_command!("create_component", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "asset", "update");

        // Create component
        let component = component_data.to_component();
        let created_component = state.services.assets.create_component(component)
            .map_err(|e| format!("Failed to create component: {}", e))?;

        info!("Component created: {} for asset {} by user {}", 
              created_component.component_name, 
              created_component.asset_id,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(created_component)
    });

    Ok(command_handler!("create_component", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Update component
#[tauri::command]
pub async fn update_component_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
    updates: ComponentUpdateRequest,
) -> Result<ApiResponse<Component>, String> {
    let result = time_command!("update_component", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "asset", "update");

        // Convert request to service update data
        let update_data = crate::services::ComponentUpdateData {
            component_name: updates.component_name,
            component_type: updates.component_type,
            manufacturer: updates.manufacturer,
            model: updates.model,
            serial_number: updates.serial_number,
            parent_component_id: updates.parent_component_id,
            specifications: updates.specifications,
            status: updates.status,
        };

        // Update component
        let updated_component = state.services.assets.update_component(id, update_data)
            .map_err(|e| format!("Failed to update component: {}", e))?;

        info!("Component updated: {} (ID: {}) by user {}", 
              updated_component.component_name, id,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(updated_component)
    });

    Ok(command_handler!("update_component", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}