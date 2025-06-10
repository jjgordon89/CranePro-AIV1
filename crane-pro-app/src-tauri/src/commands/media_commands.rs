//! Media management command handlers
//! 
//! This module contains all Tauri command handlers for media file management
//! operations including file upload, retrieval, and deletion.

use crate::api::{ApiResponse, UploadFileRequest, MediaFileUpdateRequest};
use crate::commands::AppState;
use crate::middleware::{Permissions, auth::AuthHelper};
use crate::models::{MediaFile, MediaType};
use crate::services::MediaFileUpdateData;
use crate::{require_auth, require_resource_access, time_command, command_handler};
use tauri::State;
use log::{info, debug, error, warn};
use chrono::Utc;
use std::path::Path;
use std::fs;

/// Upload a file
#[tauri::command]
pub async fn upload_file_command(
    state: State<'_, AppState>,
    token: Option<String>,
    file_data: UploadFileRequest,
) -> Result<ApiResponse<MediaFile>, String> {
    let result = time_command!("upload_file", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "media", "upload");

        // Validate file size (limit to 50MB)
        const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50MB
        if file_data.file_data.len() > MAX_FILE_SIZE {
            return Err("File size exceeds 50MB limit".to_string());
        }

        // Validate file type
        let allowed_types = match file_data.file_type {
            MediaType::Image => vec!["image/jpeg", "image/png", "image/tiff", "image/bmp"],
            MediaType::Video => vec!["video/mp4", "video/avi", "video/mov"],
            MediaType::Document => vec!["application/pdf", "text/plain", "application/msword"],
            MediaType::Audio => vec!["audio/mp3", "audio/wav", "audio/m4a"],
        };

        if !allowed_types.contains(&file_data.mime_type.as_str()) {
            return Err(format!("Unsupported file type: {}", file_data.mime_type));
        }

        // Generate unique filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let file_extension = Path::new(&file_data.file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");
        let unique_filename = format!("{}_{}.{}", timestamp, uuid::Uuid::new_v4(), file_extension);

        // Create upload directory structure
        let upload_dir = format!("uploads/{}/{}", 
                                file_data.file_type.to_string(), 
                                Utc::now().format("%Y/%m"));
        let full_upload_path = format!("./data/{}", upload_dir);
        
        fs::create_dir_all(&full_upload_path)
            .map_err(|e| format!("Failed to create upload directory: {}", e))?;

        // Write file to disk
        let file_path = format!("{}/{}", upload_dir, unique_filename);
        let full_file_path = format!("./data/{}", file_path);
        
        fs::write(&full_file_path, &file_data.file_data)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        // Create media file record
        let media_file = file_data.to_media_file(file_path, file_data.file_data.len() as i64);
        let created_media = state.services.media.create_media_file(media_file)
            .map_err(|e| {
                // Clean up file if database operation fails
                let _ = fs::remove_file(&full_file_path);
                format!("Failed to create media file record: {}", e)
            })?;

        // Queue for AI analysis if it's an image
        if matches!(file_data.file_type, MediaType::Image) {
            let _ = state.services.media.queue_for_ai_analysis(created_media.id);
        }

        info!("File uploaded: {} (ID: {}) by user {}", 
              created_media.file_name, 
              created_media.id,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(created_media)
    });

    Ok(command_handler!("upload_file", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get file by ID
#[tauri::command]
pub async fn get_file_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<MediaFile>, String> {
    let result = time_command!("get_file", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "media", "read");

        // Get media file
        let media_file = state.services.media.get_media_file_by_id(id)
            .map_err(|e| format!("Failed to get media file: {}", e))?;

        debug!("Media file retrieved: {} (ID: {})", media_file.file_name, id);
        Ok(media_file)
    });

    Ok(command_handler!("get_file", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get files by inspection ID
#[tauri::command]
pub async fn get_files_by_inspection_command(
    state: State<'_, AppState>,
    token: Option<String>,
    inspection_id: i64,
) -> Result<ApiResponse<Vec<MediaFile>>, String> {
    let result = time_command!("get_files_by_inspection", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "media", "read");

        // Get media files for inspection
        let media_files = state.services.media.get_media_files_by_inspection(inspection_id)
            .map_err(|e| format!("Failed to get media files by inspection: {}", e))?;

        debug!("Retrieved {} media files for inspection {}", 
               media_files.len(), inspection_id);

        Ok(media_files)
    });

    Ok(command_handler!("get_files_by_inspection", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Delete file
#[tauri::command]
pub async fn delete_file_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<()>, String> {
    let result = time_command!("delete_file", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "media", "delete");

        // Get file info before deletion for cleanup
        let media_file = state.services.media.get_media_file_by_id(id)
            .map_err(|e| format!("Failed to get media file for deletion: {}", e))?;

        // Delete from database
        state.services.media.delete_media_file(id)
            .map_err(|e| format!("Failed to delete media file from database: {}", e))?;

        // Delete physical file
        let full_file_path = format!("./data/{}", media_file.file_path);
        if let Err(e) = fs::remove_file(&full_file_path) {
            warn!("Failed to delete physical file {}: {}", full_file_path, e);
            // Don't fail the operation if file deletion fails
        }

        info!("Media file deleted: {} (ID: {}) by user {}", 
              media_file.file_name, id,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(())
    });

    Ok(command_handler!("delete_file", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get file URL for download/viewing
#[tauri::command]
pub async fn get_file_url_command(
    state: State<'_, AppState>,
    token: Option<String>,
    id: i64,
) -> Result<ApiResponse<String>, String> {
    let result = time_command!("get_file_url", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "media", "read");

        // Get media file
        let media_file = state.services.media.get_media_file_by_id(id)
            .map_err(|e| format!("Failed to get media file: {}", e))?;

        // Generate secure file URL (in production, this would be a signed URL with expiration)
        let file_url = format!("/api/files/{}/download", id);

        debug!("File URL generated for media file: {} (ID: {})", 
               media_file.file_name, id);

        Ok(file_url)
    });

    Ok(command_handler!("get_file_url", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Upload inspection photo (specialized upload for inspections)
#[tauri::command]
pub async fn upload_inspection_photo_command(
    state: State<'_, AppState>,
    token: Option<String>,
    inspection_id: i64,
    file_data: UploadFileRequest,
) -> Result<ApiResponse<MediaFile>, String> {
    let result = time_command!("upload_inspection_photo", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "media", "upload");

        // Validate that this is an image file
        if !matches!(file_data.file_type, MediaType::Image) {
            return Err("Only image files are allowed for inspection photos".to_string());
        }

        // Validate file size (limit to 20MB for photos)
        const MAX_PHOTO_SIZE: usize = 20 * 1024 * 1024; // 20MB
        if file_data.file_data.len() > MAX_PHOTO_SIZE {
            return Err("Photo size exceeds 20MB limit".to_string());
        }

        // Create a new upload request with the inspection ID set
        let mut photo_data = file_data;
        photo_data.inspection_id = Some(inspection_id);

        // Generate unique filename for inspection photo
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let file_extension = Path::new(&photo_data.file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("jpg");
        let unique_filename = format!("inspection_{}_{}.{}", 
                                    inspection_id, timestamp, file_extension);

        // Create upload directory for inspection photos
        let upload_dir = format!("uploads/inspections/{}", inspection_id);
        let full_upload_path = format!("./data/{}", upload_dir);
        
        fs::create_dir_all(&full_upload_path)
            .map_err(|e| format!("Failed to create upload directory: {}", e))?;

        // Write file to disk
        let file_path = format!("{}/{}", upload_dir, unique_filename);
        let full_file_path = format!("./data/{}", file_path);
        
        fs::write(&full_file_path, &photo_data.file_data)
            .map_err(|e| format!("Failed to write photo: {}", e))?;

        // Create media file record
        let media_file = photo_data.to_media_file(file_path, photo_data.file_data.len() as i64);
        let created_media = state.services.media.create_media_file(media_file)
            .map_err(|e| {
                // Clean up file if database operation fails
                let _ = fs::remove_file(&full_file_path);
                format!("Failed to create media file record: {}", e)
            })?;

        // Queue for AI analysis
        let _ = state.services.media.queue_for_ai_analysis(created_media.id);

        info!("Inspection photo uploaded: {} for inspection {} by user {}", 
              created_media.file_name, inspection_id,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(created_media)
    });

    Ok(command_handler!("upload_inspection_photo", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get inspection photos
#[tauri::command]
pub async fn get_inspection_photos_command(
    state: State<'_, AppState>,
    token: Option<String>,
    inspection_id: i64,
) -> Result<ApiResponse<Vec<MediaFile>>, String> {
    let result = time_command!("get_inspection_photos", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "media", "read");

        // Get media files for inspection (filter for images only)
        let all_media_files = state.services.media.get_media_files_by_inspection(inspection_id)
            .map_err(|e| format!("Failed to get media files by inspection: {}", e))?;

        // Filter for image files only
        let photo_files: Vec<MediaFile> = all_media_files
            .into_iter()
            .filter(|file| matches!(file.file_type, MediaType::Image))
            .collect();

        debug!("Retrieved {} photos for inspection {}", 
               photo_files.len(), inspection_id);

        Ok(photo_files)
    });

    Ok(command_handler!("get_inspection_photos", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}