//! Comprehensive error handling for CranePro Bridge Inspection Application
//!
//! This module provides a centralized error handling system that covers all
//! aspects of the application including database operations, file I/O,
//! security, validation, and external service interactions.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Application-wide result type
pub type AppResult<T> = Result<T, AppError>;

/// Main application error enum covering all error categories
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum AppError {
    // Database Errors
    #[error("Database operation failed: {message}")]
    Database { message: String },

    #[error("Database connection failed: {reason}")]
    DatabaseConnection { reason: String },

    #[error("Database migration failed: {version}")]
    DatabaseMigration { version: String },

    #[error("Record not found: {entity} with {field} = {value}")]
    RecordNotFound {
        entity: String,
        field: String,
        value: String,
    },

    #[error("Duplicate record: {entity} with {field} = {value} already exists")]
    DuplicateRecord {
        entity: String,
        field: String,
        value: String,
    },

    // Validation Errors
    #[error("Validation failed: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("Required field missing: {field}")]
    RequiredField { field: String },

    #[error("Invalid format: {field} - expected {expected}, got {actual}")]
    InvalidFormat {
        field: String,
        expected: String,
        actual: String,
    },

    #[error("Value out of range: {field} - {value} not in range {min}-{max}")]
    OutOfRange {
        field: String,
        value: String,
        min: String,
        max: String,
    },

    // File System Errors
    #[error("File operation failed: {operation} on {path} - {reason}")]
    FileSystem {
        operation: String,
        path: String,
        reason: String,
    },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied: {path} - {operation}")]
    PermissionDenied { path: String, operation: String },

    #[error("Invalid file format: {path} - expected {expected}, got {actual}")]
    InvalidFileFormat {
        path: String,
        expected: String,
        actual: String,
    },

    // Image Processing Errors
    #[error("Image processing failed: {operation} - {reason}")]
    ImageProcessing { operation: String, reason: String },

    #[error("Unsupported image format: {format} for file {path}")]
    UnsupportedImageFormat { format: String, path: String },

    #[error("Image too large: {path} - size {size}MB exceeds limit {limit}MB")]
    ImageTooLarge {
        path: String,
        size: f64,
        limit: f64,
    },

    #[error("EXIF data extraction failed: {path} - {reason}")]
    ExifExtraction { path: String, reason: String },

    // Security Errors
    #[error("Authentication failed: {reason}")]
    Authentication { reason: String },

    #[error("Authorization failed: user {user} cannot {action} {resource}")]
    Authorization {
        user: String,
        action: String,
        resource: String,
    },

    #[error("Token error: {operation} - {reason}")]
    Token { operation: String, reason: String },

    #[error("Encryption failed: {reason}")]
    Encryption { reason: String },

    #[error("Decryption failed: {reason}")]
    Decryption { reason: String },

    // Network Errors
    #[error("Network request failed: {method} {url} - {status}: {message}")]
    NetworkRequest {
        method: String,
        url: String,
        status: u16,
        message: String,
    },

    #[error("Connection timeout: {url} after {timeout}s")]
    ConnectionTimeout { url: String, timeout: u64 },

    #[error("API error: {service} - {code}: {message}")]
    ApiError {
        service: String,
        code: String,
        message: String,
    },

    // Configuration Errors
    #[error("Configuration error: {key} - {reason}")]
    Configuration { key: String, reason: String },

    #[error("Missing configuration: {key}")]
    MissingConfiguration { key: String },

    #[error("Invalid configuration: {key} - {value} is not valid")]
    InvalidConfiguration { key: String, value: String },

    // Business Logic Errors
    #[error("Inspection error: {inspection_id} - {reason}")]
    Inspection {
        inspection_id: String,
        reason: String,
    },

    #[error("Crane operation error: {crane_id} - {operation}: {reason}")]
    CraneOperation {
        crane_id: String,
        operation: String,
        reason: String,
    },

    #[error("Report generation failed: {report_type} - {reason}")]
    ReportGeneration {
        report_type: String,
        reason: String,
    },

    #[error("Schedule conflict: {inspection_id} - {reason}")]
    ScheduleConflict {
        inspection_id: String,
        reason: String,
    },

    // AI/ML Integration Errors
    #[error("AI analysis failed: {model} - {reason}")]
    AiAnalysis { model: String, reason: String },

    #[error("AI service unavailable: {service}")]
    AiServiceUnavailable { service: String },

    #[error("AI quota exceeded: {service} - {limit}")]
    AiQuotaExceeded { service: String, limit: String },

    // Generic Errors
    #[error("Internal server error: {message}")]
    Internal { message: String },

    #[error("Operation timeout: {operation} exceeded {timeout}s")]
    Timeout { operation: String, timeout: u64 },

    #[error("Resource unavailable: {resource}")]
    ResourceUnavailable { resource: String },

    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },
}

impl AppError {
    /// Create a database error
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a file system error
    pub fn file_system(
        operation: impl Into<String>,
        path: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::FileSystem {
            operation: operation.into(),
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create an authentication error
    pub fn authentication(reason: impl Into<String>) -> Self {
        Self::Authentication {
            reason: reason.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Get the error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            Self::Database { .. }
            | Self::DatabaseConnection { .. }
            | Self::DatabaseMigration { .. }
            | Self::RecordNotFound { .. }
            | Self::DuplicateRecord { .. } => "database",

            Self::Validation { .. }
            | Self::RequiredField { .. }
            | Self::InvalidFormat { .. }
            | Self::OutOfRange { .. } => "validation",

            Self::FileSystem { .. }
            | Self::FileNotFound { .. }
            | Self::PermissionDenied { .. }
            | Self::InvalidFileFormat { .. } => "filesystem",

            Self::ImageProcessing { .. }
            | Self::UnsupportedImageFormat { .. }
            | Self::ImageTooLarge { .. }
            | Self::ExifExtraction { .. } => "image",

            Self::Authentication { .. }
            | Self::Authorization { .. }
            | Self::Token { .. }
            | Self::Encryption { .. }
            | Self::Decryption { .. } => "security",

            Self::NetworkRequest { .. }
            | Self::ConnectionTimeout { .. }
            | Self::ApiError { .. } => "network",

            Self::Configuration { .. }
            | Self::MissingConfiguration { .. }
            | Self::InvalidConfiguration { .. } => "configuration",

            Self::Inspection { .. }
            | Self::CraneOperation { .. }
            | Self::ReportGeneration { .. }
            | Self::ScheduleConflict { .. } => "business",

            Self::AiAnalysis { .. }
            | Self::AiServiceUnavailable { .. }
            | Self::AiQuotaExceeded { .. } => "ai",

            Self::Internal { .. }
            | Self::Timeout { .. }
            | Self::ResourceUnavailable { .. }
            | Self::ExternalService { .. } => "system",
        }
    }

    /// Check if this error should be retried
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionTimeout { .. }
                | Self::NetworkRequest { status: 503..=599, .. }
                | Self::ResourceUnavailable { .. }
                | Self::AiServiceUnavailable { .. }
                | Self::ExternalService { .. }
        )
    }

    /// Get the HTTP status code that should be returned for this error
    pub fn http_status(&self) -> u16 {
        match self {
            Self::RecordNotFound { .. } | Self::FileNotFound { .. } => 404,

            Self::Validation { .. }
            | Self::RequiredField { .. }
            | Self::InvalidFormat { .. }
            | Self::OutOfRange { .. }
            | Self::InvalidFileFormat { .. }
            | Self::UnsupportedImageFormat { .. }
            | Self::ImageTooLarge { .. } => 400,

            Self::Authentication { .. } => 401,

            Self::Authorization { .. } | Self::PermissionDenied { .. } => 403,

            Self::DuplicateRecord { .. } => 409,

            Self::ConnectionTimeout { .. } | Self::Timeout { .. } => 408,

            Self::ResourceUnavailable { .. }
            | Self::AiServiceUnavailable { .. }
            | Self::ExternalService { .. } => 503,

            Self::AiQuotaExceeded { .. } => 429,

            _ => 500,
        }
    }
}

// Implement conversions from common error types
impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::SqliteFailure(code, Some(msg)) => {
                if code.code == rusqlite::ErrorCode::ConstraintViolation {
                    Self::DuplicateRecord {
                        entity: "unknown".to_string(),
                        field: "unknown".to_string(),
                        value: msg,
                    }
                } else {
                    Self::Database {
                        message: format!("SQLite error: {}", msg),
                    }
                }
            }
            _ => Self::Database {
                message: err.to_string(),
            },
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind;
        match err.kind() {
            ErrorKind::NotFound => Self::FileNotFound {
                path: "unknown".to_string(),
            },
            ErrorKind::PermissionDenied => Self::PermissionDenied {
                path: "unknown".to_string(),
                operation: "unknown".to_string(),
            },
            _ => Self::FileSystem {
                operation: "unknown".to_string(),
                path: "unknown".to_string(),
                reason: err.to_string(),
            },
        }
    }
}

impl From<image::ImageError> for AppError {
    fn from(err: image::ImageError) -> Self {
        match err {
            image::ImageError::Unsupported(_) => Self::UnsupportedImageFormat {
                format: "unknown".to_string(),
                path: "unknown".to_string(),
            },
            _ => Self::ImageProcessing {
                operation: "unknown".to_string(),
                reason: err.to_string(),
            },
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::ConnectionTimeout {
                url: err.url().map(|u| u.to_string()).unwrap_or_default(),
                timeout: 30, // Default timeout assumption
            }
        } else if let Some(status) = err.status() {
            Self::NetworkRequest {
                method: "unknown".to_string(),
                url: err.url().map(|u| u.to_string()).unwrap_or_default(),
                status: status.as_u16(),
                message: err.to_string(),
            }
        } else {
            Self::ExternalService {
                service: "http_client".to_string(),
                message: err.to_string(),
            }
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::InvalidFormat {
            field: "json".to_string(),
            expected: "valid JSON".to_string(),
            actual: err.to_string(),
        }
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        Self::InvalidFormat {
            field: "uuid".to_string(),
            expected: "valid UUID".to_string(),
            actual: err.to_string(),
        }
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        Self::InvalidFormat {
            field: "datetime".to_string(),
            expected: "valid datetime".to_string(),
            actual: err.to_string(),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::Token {
            operation: "jwt_processing".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        Self::Encryption {
            reason: err.to_string(),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal {
            message: err.to_string(),
        }
    }
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

// Note: Display implementation is automatically generated by thiserror derive macro

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        assert_eq!(AppError::database("test").category(), "database");
        assert_eq!(AppError::validation("field", "message").category(), "validation");
        assert_eq!(AppError::authentication("reason").category(), "security");
    }

    #[test]
    fn test_http_status_codes() {
        assert_eq!(AppError::authentication("test").http_status(), 401);
        assert_eq!(AppError::validation("field", "message").http_status(), 400);
        assert_eq!(AppError::internal("message").http_status(), 500);
    }

    #[test]
    fn test_retryable_errors() {
        assert!(AppError::ConnectionTimeout {
            url: "test".to_string(),
            timeout: 30
        }
        .is_retryable());

        assert!(!AppError::validation("field", "message").is_retryable());
    }
}