//! Middleware module for CranePro Bridge Inspection Application
//! 
//! This module contains middleware components for authentication,
//! authorization, logging, and request processing.

pub mod auth;

// Re-export commonly used types
pub use auth::*;

use crate::errors::{AppError, AppResult};
use crate::models::{User, UserRole};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session information for authenticated users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: i64,
    pub username: String,
    pub role: UserRole,
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub permissions: Vec<String>,
}

impl UserSession {
    pub fn new(user: &User, session_id: String, permissions: Vec<String>) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(8); // 8-hour session

        Self {
            user_id: user.id,
            username: user.username.clone(),
            role: user.role.clone(),
            session_id,
            created_at: now,
            expires_at,
            last_activity: now,
            permissions,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string()) ||
        self.permissions.contains(&"*".to_string()) // Admin wildcard
    }

    pub fn can_access_resource(&self, resource: &str, action: &str) -> bool {
        let permission = format!("{}:{}", resource, action);
        self.has_permission(&permission) || self.has_permission(&format!("{}:*", resource))
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }
}

/// Permission definitions for the application
pub struct Permissions;

impl Permissions {
    // Asset permissions
    pub const ASSET_CREATE: &'static str = "asset:create";
    pub const ASSET_READ: &'static str = "asset:read";
    pub const ASSET_UPDATE: &'static str = "asset:update";
    pub const ASSET_DELETE: &'static str = "asset:delete";
    pub const ASSET_ALL: &'static str = "asset:*";

    // Inspection permissions
    pub const INSPECTION_CREATE: &'static str = "inspection:create";
    pub const INSPECTION_READ: &'static str = "inspection:read";
    pub const INSPECTION_UPDATE: &'static str = "inspection:update";
    pub const INSPECTION_DELETE: &'static str = "inspection:delete";
    pub const INSPECTION_SUBMIT: &'static str = "inspection:submit";
    pub const INSPECTION_ALL: &'static str = "inspection:*";

    // Compliance permissions
    pub const COMPLIANCE_READ: &'static str = "compliance:read";
    pub const COMPLIANCE_UPDATE: &'static str = "compliance:update";
    pub const COMPLIANCE_VERIFY: &'static str = "compliance:verify";
    pub const COMPLIANCE_ALL: &'static str = "compliance:*";

    // User management permissions
    pub const USER_CREATE: &'static str = "user:create";
    pub const USER_READ: &'static str = "user:read";
    pub const USER_UPDATE: &'static str = "user:update";
    pub const USER_DELETE: &'static str = "user:delete";
    pub const USER_ALL: &'static str = "user:*";

    // Media permissions
    pub const MEDIA_UPLOAD: &'static str = "media:upload";
    pub const MEDIA_READ: &'static str = "media:read";
    pub const MEDIA_DELETE: &'static str = "media:delete";
    pub const MEDIA_ALL: &'static str = "media:*";

    // Report permissions
    pub const REPORT_GENERATE: &'static str = "report:generate";
    pub const REPORT_READ: &'static str = "report:read";
    pub const REPORT_ALL: &'static str = "report:*";

    // Location permissions
    pub const LOCATION_CREATE: &'static str = "location:create";
    pub const LOCATION_READ: &'static str = "location:read";
    pub const LOCATION_UPDATE: &'static str = "location:update";
    pub const LOCATION_DELETE: &'static str = "location:delete";
    pub const LOCATION_ALL: &'static str = "location:*";

    // System permissions
    pub const SYSTEM_ADMIN: &'static str = "system:admin";
    pub const SYSTEM_ALL: &'static str = "*";

    /// Get default permissions for a user role
    pub fn for_role(role: &UserRole) -> Vec<String> {
        match role {
            UserRole::Inspector => vec![
                Self::ASSET_READ.to_string(),
                Self::INSPECTION_CREATE.to_string(),
                Self::INSPECTION_READ.to_string(),
                Self::INSPECTION_UPDATE.to_string(),
                Self::INSPECTION_SUBMIT.to_string(),
                Self::COMPLIANCE_READ.to_string(),
                Self::MEDIA_UPLOAD.to_string(),
                Self::MEDIA_READ.to_string(),
                Self::REPORT_READ.to_string(),
                Self::LOCATION_READ.to_string(),
            ],
            UserRole::Supervisor => vec![
                Self::ASSET_READ.to_string(),
                Self::ASSET_UPDATE.to_string(),
                Self::INSPECTION_ALL.to_string(),
                Self::COMPLIANCE_READ.to_string(),
                Self::COMPLIANCE_UPDATE.to_string(),
                Self::USER_READ.to_string(),
                Self::MEDIA_ALL.to_string(),
                Self::REPORT_ALL.to_string(),
                Self::LOCATION_READ.to_string(),
                Self::LOCATION_UPDATE.to_string(),
            ],
            UserRole::Administrator => vec![
                Self::ASSET_ALL.to_string(),
                Self::INSPECTION_ALL.to_string(),
                Self::COMPLIANCE_ALL.to_string(),
                Self::USER_CREATE.to_string(),
                Self::USER_READ.to_string(),
                Self::USER_UPDATE.to_string(),
                Self::MEDIA_ALL.to_string(),
                Self::REPORT_ALL.to_string(),
                Self::LOCATION_ALL.to_string(),
            ],
            UserRole::SuperAdmin => vec![
                Self::SYSTEM_ALL.to_string(),
            ],
        }
    }
}

/// Context information passed to command handlers
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub session: Option<UserSession>,
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            session: None,
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user_agent: None,
            ip_address: None,
        }
    }

    pub fn with_session(mut self, session: UserSession) -> Self {
        self.session = Some(session);
        self
    }

    pub fn current_user(&self) -> AppResult<&UserSession> {
        self.session.as_ref().ok_or_else(|| {
            AppError::authentication("No active session")
        })
    }

    pub fn require_permission(&self, permission: &str) -> AppResult<()> {
        let session = self.current_user()?;
        
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

    pub fn require_resource_access(&self, resource: &str, action: &str) -> AppResult<()> {
        let session = self.current_user()?;
        
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

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit log entry for tracking user actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl AuditLogEntry {
    pub fn new(
        context: &RequestContext,
        action: impl Into<String>,
        resource_type: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: context.session.as_ref().map(|s| s.user_id),
            username: context.session.as_ref().map(|s| s.username.clone()),
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: None,
            details: HashMap::new(),
            ip_address: context.ip_address.clone(),
            user_agent: context.user_agent.clone(),
            timestamp: context.timestamp,
            success: true,
            error_message: None,
        }
    }

    pub fn with_resource_id(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }

    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = details;
        self
    }

    pub fn with_error(mut self, error: &AppError) -> Self {
        self.success = false;
        self.error_message = Some(error.to_string());
        self
    }
}