//! Location management command handlers
//! 
//! This module contains all Tauri command handlers for location management
//! operations including CRUD operations for locations and location hierarchies.

use crate::api::{ApiResponse, QueryFilterRequest, CreateLocationRequest, LocationUpdateRequest,
                PaginatedResponse};
use crate::commands::AppState;
use crate::middleware::auth::AuthHelper;
use crate::models::{Location, LocationUpdateData, LocationWithAssets, LocationAssetSummary,
                   LocationWithAssetCount, LocationDeletionResult};
use crate::{require_resource_access, time_command, command_handler};
use tauri::State;
use log::{info, debug};

/// Validate coordinates if provided
fn validate_coordinates(lat: Option<f64>, lng: Option<f64>) -> Result<(), String> {
    if let (Some(lat), Some(lng)) = (lat, lng) {
        if lat.is_nan() || lng.is_nan() {
            return Err("Coordinates cannot be NaN".to_string());
        }
        if lat.is_infinite() || lng.is_infinite() {
            return Err("Coordinates cannot be infinite".to_string());
        }
        if lat < -90.0 || lat > 90.0 {
            return Err("Latitude must be between -90 and 90 degrees".to_string());
        }
        if lng < -180.0 || lng > 180.0 {
            return Err("Longitude must be between -180 and 180 degrees".to_string());
        }
    }
    Ok(())
}

/// Create a new location
#[tauri::command]
pub async fn create_location_command(
    state: State<'_, AppState>,
    token: Option<String>,
    location_data: CreateLocationRequest,
) -> Result<ApiResponse<Location>, String> {
    let result = time_command!("create_location", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "location", "create");

        // Validate request data
        if location_data.name.trim().is_empty() {
            return Err("Location name cannot be empty".to_string());
        }

        validate_coordinates(location_data.latitude, location_data.longitude)?;

        if let Some(parent_id) = location_data.parent_location_id {
            if parent_id == 0 {
                return Err("Invalid parent location ID".to_string());
            }
        }

        // Create location
        let location = location_data.to_location();
        let created_location = state.services.locations.create_location(location)
            .map_err(|e| format!("Failed to create location: {}", e))?;

        info!("Location created: {} by user {}", 
              created_location.name, 
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(created_location)
    });

    Ok(command_handler!("create_location", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get location by ID
#[tauri::command]
pub async fn get_location_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<Location>, String> {
    let result = time_command!("get_location", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "location", "read");

        // Get location
        let location = state.services.locations.get_location_by_id(id)
            .map_err(|e| format!("Failed to get location: {}", e))?;

        debug!("Location retrieved: {} (ID: {})", location.name, id);
        Ok(location)
    });

    Ok(command_handler!("get_location", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Update location
#[tauri::command]
pub async fn update_location_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
    updates: LocationUpdateRequest,
) -> Result<ApiResponse<Location>, String> {
    let result = time_command!("update_location", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "location", "update");

        // Validate update data
        if let Some(ref name) = updates.name {
            if name.trim().is_empty() {
                return Err("Location name cannot be empty".to_string());
            }
        }

        validate_coordinates(updates.latitude, updates.longitude)?;

        if let Some(Some(parent_id)) = updates.parent_location_id {
            if parent_id == id {
                return Err("Location cannot be its own parent".to_string());
            }
        }

        // Convert request to service update data
        let update_data: LocationUpdateData = updates.into();

        // Update location
        let updated_location = state.services.locations.update_location(id, update_data)
            .map_err(|e| format!("Failed to update location: {}", e))?;

        info!("Location updated: {} (ID: {}) by user {}", 
              updated_location.name, id,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(updated_location)
    });

    Ok(command_handler!("update_location", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Delete location (safe deletion with dependency checks)
#[tauri::command]
pub async fn delete_location_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<LocationDeletionResult>, String> {
    let result = time_command!("delete_location", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "location", "delete");

        // Safe delete location
        let deletion_result = state.services.locations.delete_location_safe(id)
            .map_err(|e| format!("Failed to delete location: {}", e))?;

        if deletion_result.success {
            info!("Location deleted: ID {} by user {}", 
                  id, context.current_user().map(|u| u.user_id).unwrap_or(0));
        } else {
            debug!("Location deletion prevented: {}", deletion_result.message);
        }

        Ok(deletion_result)
    });

    Ok(command_handler!("delete_location", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get location with all its assets
#[tauri::command]
pub async fn get_location_with_assets_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<LocationWithAssets>, String> {
    let result = time_command!("get_location_with_assets", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "location", "read");

        // Get location with assets
        let location_with_assets = state.services.locations.get_location_with_assets(id)
            .map_err(|e| format!("Failed to get location with assets: {}", e))?;

        debug!("Location with assets retrieved: {} ({} assets)", 
               location_with_assets.name, location_with_assets.assets.len());

        Ok(location_with_assets)
    });

    Ok(command_handler!("get_location_with_assets", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get location with asset summary
#[tauri::command]
pub async fn get_location_asset_summary_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<LocationAssetSummary>, String> {
    let result = time_command!("get_location_asset_summary", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "location", "read");

        // Get location with asset summary
        let location_summary = state.services.locations.get_location_with_asset_summary(id)
            .map_err(|e| format!("Failed to get location asset summary: {}", e))?;

        debug!("Location asset summary retrieved: {} ({} total assets, {} critical)", 
               location_summary.name, location_summary.asset_count, location_summary.critical_assets);

        Ok(location_summary)
    });

    Ok(command_handler!("get_location_asset_summary", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Validate asset-location assignment
#[tauri::command]
pub async fn validate_asset_location_assignment_command(
    state: State<'_, AppState>,
    token: Option<String>,
    asset_id: i64,
    location_id: i64,
) -> Result<ApiResponse<()>, String> {
    let result = time_command!("validate_asset_location_assignment", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        // Require both location and asset read permissions
        require_resource_access!(context, "location", "read");
        require_resource_access!(context, "asset", "read");

        // Validate assignment
        state.services.locations.validate_asset_location_assignment(asset_id, location_id)
            .map_err(|e| format!("Failed to validate asset-location assignment: {}", e))?;

        debug!("Asset-location assignment validated: asset {} to location {}", 
               asset_id, location_id);

        Ok(())
    });

    Ok(command_handler!("validate_asset_location_assignment", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Search locations with asset counts
#[tauri::command]
pub async fn search_locations_with_asset_counts_command(
    state: State<'_, AppState>,
    token: Option<String>,
    query: String,
    filter: QueryFilterRequest,
) -> Result<ApiResponse<PaginatedResponse<LocationWithAssetCount>>, String> {
    let result = time_command!("search_locations_with_asset_counts", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "location", "read");

        // Validate search parameters
        if query.len() < 3 && filter.limit.unwrap_or(50) > 20 {
            return Err("Query too short for large result sets. Please provide at least 3 characters.".to_string());
        }

        const MAX_PAGE_SIZE: i64 = 100;
        if filter.limit.unwrap_or(50) > MAX_PAGE_SIZE {
            return Err(format!("Page size cannot exceed {}", MAX_PAGE_SIZE));
        }

        // Search locations
        let query_filter = filter.into();
        let search_results = state.services.locations.search_locations_with_asset_counts(query.clone(), query_filter)
            .map_err(|e| format!("Failed to search locations: {}", e))?;

        debug!("Location search returned {} results for query: '{}'",
               search_results.data.len(), query);

        let response = PaginatedResponse::from(search_results);
        Ok(response)
    });

    Ok(command_handler!("search_locations_with_asset_counts", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}