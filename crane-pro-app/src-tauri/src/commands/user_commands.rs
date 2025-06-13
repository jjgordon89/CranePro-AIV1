//! User management command handlers
//! 
//! This module contains all Tauri command handlers for user management
//! operations including authentication, user CRUD, and session management.

use crate::api::{ApiResponse, QueryFilterRequest, CreateUserRequest, UserUpdateRequest,
                LoginRequest, ChangePasswordRequest, PaginatedResponse, LoginResponse};
use crate::commands::AppState;
use crate::middleware::auth::AuthHelper;
use crate::models::User;
use crate::services::UserUpdateData;
use crate::{require_resource_access, time_command, command_handler};
use tauri::State;
use log::{info, debug, warn};

/// Create a new user
#[tauri::command]
pub async fn create_user_command(
    state: State<'_, AppState>,
    token: Option<String>,
    user_data: CreateUserRequest,
) -> Result<ApiResponse<User>, String> {
    let result = time_command!("create_user", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "user", "create");

        // Create user - the service will handle password validation and hashing
        let plain_password = user_data.password.clone(); // Extract password before move
        let user = user_data.to_user(String::new()); // Temporary password_hash, service will replace it
        let created_user = state.services.users.create_user(user, plain_password)
            .map_err(|e| format!("Failed to create user: {}", e))?;

        info!("User created: {} by admin {}", 
              created_user.username,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(created_user)
    });

    Ok(command_handler!("create_user", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get user by ID
#[tauri::command]
pub async fn get_user_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<User>, String> {
    let result = time_command!("get_user", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        // Check if user is accessing their own profile or has admin permissions
        let session = context.current_user()?;
        if session.user_id != id {
            require_resource_access!(context, "user", "read");
        }

        // Get user
        let user = state.services.users.get_user_by_id(id)
            .map_err(|e| format!("Failed to get user: {}", e))?;

        debug!("User retrieved: {} (ID: {})", user.username, id);
        Ok(user)
    });

    Ok(command_handler!("get_user", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get current authenticated user
#[tauri::command]
pub async fn get_current_user_command(
    state: State<'_, AppState>,
    token: Option<String>,
) -> Result<ApiResponse<User>, String> {
    let result = time_command!("get_current_user", {
        // Authenticate (required for this endpoint)
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        let session = context.current_user()?;

        // Get current user
        let user = state.services.users.get_user_by_id(session.user_id)
            .map_err(|e| format!("Failed to get current user: {}", e))?;

        debug!("Current user retrieved: {}", user.username);
        Ok(user)
    });

    Ok(command_handler!("get_current_user", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Update user
#[tauri::command]
pub async fn update_user_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
    updates: UserUpdateRequest,
) -> Result<ApiResponse<User>, String> {
    let result = time_command!("update_user", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        // Check if user is updating their own profile or has admin permissions
        let session = context.current_user()?;
        if session.user_id != id {
            require_resource_access!(context, "user", "update");
        }

        // Convert request to service update data
        let update_data = UserUpdateData {
            username: updates.username,
            email: updates.email,
            role: updates.role,
            first_name: updates.first_name,
            last_name: updates.last_name,
            phone: updates.phone,
            is_active: updates.is_active,
        };

        // Update user
        let updated_user = state.services.users.update_user(id, update_data)
            .map_err(|e| format!("Failed to update user: {}", e))?;

        info!("User updated: {} (ID: {}) by user {}", 
              updated_user.username, id, session.user_id);

        Ok(updated_user)
    });

    Ok(command_handler!("update_user", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Delete user
#[tauri::command]
pub async fn delete_user_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<()>, String> {
    let result = time_command!("delete_user", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "user", "delete");

        // Prevent user from deleting themselves
        let session = context.current_user()?;
        if session.user_id == id {
            return Err("Cannot delete your own account".to_string());
        }

        // Delete user
        state.services.users.delete_user(id)
            .map_err(|e| format!("Failed to delete user: {}", e))?;

        // Force logout all sessions for the deleted user
        let _ = state.auth_manager.force_logout_user(id);

        info!("User deleted: ID {} by admin {}", id, session.user_id);
        Ok(())
    });

    Ok(command_handler!("delete_user", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// User login
#[tauri::command]
pub async fn login_command(
    state: State<'_, AppState>,
    credentials: LoginRequest,
) -> Result<ApiResponse<LoginResponse>, String> {
    let result = time_command!("login", {
        // Authenticate user
        let (session, token) = state.auth_manager
            .authenticate(&credentials.username, &credentials.password)
            .await
            .map_err(|e| {
                warn!("Login failed for user {}: {}", credentials.username, e);
                format!("Authentication failed: {}", e)
            })?;

        // Get user details (without password hash)
        let user = state.services.users.get_user_by_id(session.user_id)
            .map_err(|e| format!("Failed to get user details: {}", e))?;        let login_response = LoginResponse {
            user: user.into(),
            token,
            expires_at: session.expires_at,
            permissions: session.permissions.clone(),
            session_id: session.session_id.clone(),
        };

        info!("User logged in: {} (session: {})", 
              credentials.username, session.session_id);

        Ok(login_response)
    });

    Ok(command_handler!("login", None, { result }))
}

/// User logout
#[tauri::command]
pub async fn logout_command(
    state: State<'_, AppState>,
    token: Option<String>,
) -> Result<ApiResponse<()>, String> {
    let result = time_command!("logout", {
        // Validate token to get session
        if let Some(token) = token {
            match state.auth_manager.validate_token(&token) {
                Ok(session) => {
                    // Logout session
                    state.auth_manager.logout(&session.session_id)
                        .map_err(|e| format!("Failed to logout: {}", e))?;
                    
                    info!("User logged out: {} (session: {})", 
                          session.username, session.session_id);
                }
                Err(e) => {
                    warn!("Logout with invalid token: {}", e);
                    // Don't fail logout for invalid tokens
                }
            }
        }

        Ok(())
    });

    Ok(command_handler!("logout", None, { result }))
}

/// Get users with filtering
#[tauri::command]
pub async fn get_users_command(
    state: State<'_, AppState>,
    token: Option<String>,
    filter: QueryFilterRequest,
) -> Result<ApiResponse<PaginatedResponse<User>>, String> {
    let result = time_command!("get_users", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "user", "read");

        // Get users with filters
        // Note: For now, we'll get all users by role and apply basic pagination
        let query_filter = filter.into();
        
        // Default to getting inspectors if no specific role filter
        let user_role = crate::models::UserRole::Inspector; // This would be extracted from filters in a real implementation
        
        let paginated_users = state.services.users.get_users_by_role(user_role, query_filter)
            .map_err(|e| format!("Failed to get users: {}", e))?;

        debug!("Retrieved {} users", paginated_users.data.len());

        let response = PaginatedResponse::from(paginated_users);
        Ok(response)
    });

    Ok(command_handler!("get_users", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Change user password
#[tauri::command]
pub async fn change_password_command(
    state: State<'_, AppState>,
    token: Option<String>,
    password_data: ChangePasswordRequest,
) -> Result<ApiResponse<()>, String> {
    let result = time_command!("change_password", {
        // Authenticate (required for this endpoint)
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        let session = context.current_user()?;

        // Verify current password
        let password_valid = state.services.users
            .verify_password(session.user_id, password_data.current_password)
            .map_err(|e| format!("Failed to verify current password: {}", e))?;

        if !password_valid {
            warn!("Password change failed: invalid current password for user {}", session.user_id);
            return Err("Invalid current password".to_string());
        }

        // Update password
        state.services.users
            .update_password(session.user_id, password_data.new_password)
            .map_err(|e| format!("Failed to update password: {}", e))?;

        // Force logout all other sessions for this user (security measure)
        let logged_out_sessions = state.auth_manager.force_logout_user(session.user_id)
            .map_err(|e| format!("Failed to logout other sessions: {}", e))?;

        info!("Password changed for user {} (logged out {} other sessions)", 
              session.user_id, logged_out_sessions);

        Ok(())
    });

    Ok(command_handler!("change_password", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}