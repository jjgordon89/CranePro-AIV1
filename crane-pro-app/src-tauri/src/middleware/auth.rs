//! Authentication middleware for session management and token validation
//! 
//! This module provides JWT-based authentication, session management,
//! and permission checking for the CranePro application.

use crate::errors::{AppError, AppResult};
use crate::middleware::{UserSession, Permissions, RequestContext};
use crate::models::User;
use crate::services::Services;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{Utc, Duration};
use log::{debug, warn, error};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: String,        // Subject (user ID)
    pub username: String,   // Username
    pub role: String,       // User role
    pub session_id: String, // Session identifier
    pub iat: i64,          // Issued at
    pub exp: i64,          // Expiration time
    pub permissions: Vec<String>, // User permissions
}

/// Authentication manager for handling sessions and tokens
pub struct AuthManager {
    services: Arc<Services>,
    active_sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    token_expiry_hours: i64,
}

impl AuthManager {
    pub fn new(services: Arc<Services>, jwt_secret: &str) -> Self {
        Self {
            services,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            encoding_key: EncodingKey::from_secret(jwt_secret.as_ref()),
            decoding_key: DecodingKey::from_secret(jwt_secret.as_ref()),
            token_expiry_hours: 8, // 8-hour token expiry
        }
    }

    /// Authenticate user with username and password
    pub async fn authenticate(&self, username: &str, password: &str) -> AppResult<(UserSession, String)> {
        debug!("Authenticating user: {}", username);

        // Get user by username
        let user = self.services.users.get_user_by_username(username.to_string())?;

        // Verify user is active
        if !user.is_active {
            warn!("Authentication failed: user {} is inactive", username);
            return Err(AppError::authentication("User account is inactive"));
        }

        // Verify password
        let password_valid = self.services.users.verify_password(user.id, password.to_string())?;
        if !password_valid {
            warn!("Authentication failed: invalid password for user {}", username);
            return Err(AppError::authentication("Invalid credentials"));
        }

        // Generate session and token
        let session_id = uuid::Uuid::new_v4().to_string();
        let permissions = Permissions::for_role(&user.role);
        let session = UserSession::new(&user, session_id.clone(), permissions.clone());
        let token = self.generate_token(&user, &session_id, &permissions)?;

        // Store session
        {
            let mut sessions = self.active_sessions.write().unwrap();
            sessions.insert(session_id.clone(), session.clone());
        }

        debug!("User {} authenticated successfully with session {}", username, session_id);
        Ok((session, token))
    }

    /// Validate token and return session
    pub fn validate_token(&self, token: &str) -> AppResult<UserSession> {
        debug!("Validating token");

        // Decode and validate token
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<TokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::authentication(format!("Invalid token: {}", e)))?;

        let claims = token_data.claims;

        // Check if session exists and is valid
        let mut sessions = self.active_sessions.write().unwrap();
        if let Some(mut session) = sessions.get(&claims.session_id).cloned() {
            if session.is_expired() {
                warn!("Session {} has expired", claims.session_id);
                sessions.remove(&claims.session_id);
                return Err(AppError::authentication("Session expired"));
            }

            // Update last activity
            session.update_activity();
            sessions.insert(claims.session_id.clone(), session.clone());
            
            debug!("Token validated successfully for user {}", claims.username);
            Ok(session)
        } else {
            warn!("Session {} not found", claims.session_id);
            Err(AppError::authentication("Invalid session"))
        }
    }

    /// Logout user and invalidate session
    pub fn logout(&self, session_id: &str) -> AppResult<()> {
        debug!("Logging out session: {}", session_id);

        let mut sessions = self.active_sessions.write().unwrap();
        if sessions.remove(session_id).is_some() {
            debug!("Session {} logged out successfully", session_id);
            Ok(())
        } else {
            warn!("Attempted to logout non-existent session: {}", session_id);
            Err(AppError::authentication("Session not found"))
        }
    }

    /// Refresh token for existing session
    pub fn refresh_token(&self, old_token: &str) -> AppResult<String> {
        debug!("Refreshing token");

        let session = self.validate_token(old_token)?;
        
        // Get fresh user data
        let user = self.services.users.get_user_by_id(session.user_id)?;
        let permissions = Permissions::for_role(&user.role);
        
        // Generate new token
        let new_token = self.generate_token(&user, &session.session_id, &permissions)?;
        
        debug!("Token refreshed successfully for user {}", user.username);
        Ok(new_token)
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) {
        debug!("Cleaning up expired sessions");

        let mut sessions = self.active_sessions.write().unwrap();
        let expired_sessions: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| session.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        for session_id in expired_sessions {
            sessions.remove(&session_id);
            debug!("Removed expired session: {}", session_id);
        }
    }

    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        let sessions = self.active_sessions.read().unwrap();
        sessions.len()
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<UserSession> {
        let sessions = self.active_sessions.read().unwrap();
        sessions.get(session_id).cloned()
    }

    /// Force logout user (admin function)
    pub fn force_logout_user(&self, user_id: i64) -> AppResult<usize> {
        debug!("Force logging out all sessions for user: {}", user_id);

        let mut sessions = self.active_sessions.write().unwrap();
        let user_sessions: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| session.user_id == user_id)
            .map(|(id, _)| id.clone())
            .collect();

        let count = user_sessions.len();
        for session_id in user_sessions {
            sessions.remove(&session_id);
        }

        debug!("Force logged out {} sessions for user {}", count, user_id);
        Ok(count)
    }

    /// Generate JWT token
    fn generate_token(&self, user: &User, session_id: &str, permissions: &[String]) -> AppResult<String> {
        let now = Utc::now();
        let expiration = now + Duration::hours(self.token_expiry_hours);

        let claims = TokenClaims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            role: user.role.to_string(),
            session_id: session_id.to_string(),
            iat: now.timestamp(),
            exp: expiration.timestamp(),
            permissions: permissions.to_vec(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Token {
                operation: "generation".to_string(),
                reason: e.to_string(),
            })
    }

    /// Validate permission for current session
    pub fn check_permission(&self, session: &UserSession, permission: &str) -> AppResult<()> {
        if session.has_permission(permission) {
            Ok(())
        } else {
            Err(AppError::Authorization {
                user: session.username.clone(),
                action: permission.to_string(),
                resource: "system".to_string(),
            })
        }
    }

    /// Validate resource access for current session
    pub fn check_resource_access(&self, session: &UserSession, resource: &str, action: &str) -> AppResult<()> {
        if session.can_access_resource(resource, action) {
            Ok(())
        } else {
            Err(AppError::Authorization {
                user: session.username.clone(),
                action: action.to_string(),
                resource: resource.to_string(),
            })
        }
    }
}

/// Authentication helper functions for command handlers
pub struct AuthHelper;

impl AuthHelper {
    /// Extract and validate token from request
    pub fn validate_request(auth_manager: &AuthManager, token: Option<String>) -> AppResult<RequestContext> {
        let mut context = RequestContext::new();

        if let Some(token) = token {
            match auth_manager.validate_token(&token) {
                Ok(session) => {
                    context = context.with_session(session);
                }
                Err(e) => {
                    error!("Token validation failed: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(context)
    }

    /// Require authentication for request
    pub fn require_auth(context: &RequestContext) -> AppResult<&UserSession> {
        context.current_user()
    }

    /// Require specific permission
    pub fn require_permission(context: &RequestContext, permission: &str) -> AppResult<()> {
        context.require_permission(permission)
    }

    /// Require resource access
    pub fn require_resource_access(context: &RequestContext, resource: &str, action: &str) -> AppResult<()> {
        context.require_resource_access(resource, action)
    }

    /// Check if user owns resource (for self-management)
    pub fn check_resource_ownership(context: &RequestContext, resource_user_id: i64) -> AppResult<()> {
        let session = context.current_user()?;
        
        // Allow if user is accessing their own resource or has admin permissions
        if session.user_id == resource_user_id || session.has_permission(Permissions::SYSTEM_ALL) {
            Ok(())
        } else {
            Err(AppError::Authorization {
                user: session.username.clone(),
                action: "access".to_string(),
                resource: "owned_resource".to_string(),
            })
        }
    }

    /// Create audit log entry
    pub fn audit_action(
        context: &RequestContext,
        action: &str,
        resource_type: &str,
        resource_id: Option<&str>,
        success: bool,
        error: Option<&AppError>,
    ) {
        let mut entry = crate::middleware::AuditLogEntry::new(context, action, resource_type);
        
        if let Some(id) = resource_id {
            entry = entry.with_resource_id(id);
        }
        
        if let Some(err) = error {
            entry = entry.with_error(err);
        }
        
        if !success {
            entry.success = false;
        }

        // In a production system, this would be logged to a database or audit service
        debug!("Audit: {:?}", entry);
    }
}

/// Authentication decorator for command handlers
#[macro_export]
macro_rules! require_auth {
    ($auth_manager:expr, $token:expr) => {{
        let context = crate::middleware::auth::AuthHelper::validate_request(&$auth_manager, $token)?;
        crate::middleware::auth::AuthHelper::require_auth(&context)?;
        context
    }};
}

#[macro_export]
macro_rules! require_permission {
    ($context:expr, $permission:expr) => {{
        crate::middleware::auth::AuthHelper::require_permission(&$context, $permission)?;
    }};
}

#[macro_export]
macro_rules! require_resource_access {
    ($context:expr, $resource:expr, $action:expr) => {{
        crate::middleware::auth::AuthHelper::require_resource_access(&$context, $resource, $action)?;
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_generation_and_validation() {
        // Simple test for token generation without database dependency
        let jwt_secret = "test_secret_key_for_testing_only";
        
        // Create mock user for testing
        use crate::models::{User, UserRole};
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            role: UserRole::Inspector,
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            phone: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            is_active: true,
        };

        // Test basic token generation components
        let session_id = uuid::Uuid::new_v4().to_string();
        let permissions = vec!["read:inspections".to_string()];
        
        // Create claims
        let now = Utc::now();
        let expiration = now + Duration::hours(8);
        
        let claims = TokenClaims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            role: user.role.to_string(),
            session_id: session_id.clone(),
            iat: now.timestamp(),
            exp: expiration.timestamp(),
            permissions: permissions.clone(),
        };

        // Test encoding/decoding
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
        
        let token = encode(&Header::default(), &claims, &encoding_key).unwrap();
        let validation = Validation::new(Algorithm::HS256);
        let decoded = decode::<TokenClaims>(&token, &decoding_key, &validation).unwrap();
        
        assert_eq!(decoded.claims.username, user.username);
        assert_eq!(decoded.claims.session_id, session_id);
        assert_eq!(decoded.claims.permissions, permissions);
    }
}