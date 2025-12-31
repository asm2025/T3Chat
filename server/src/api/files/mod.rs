use crate::{
    AppState,
    db::{
        models::{File, NewFile},
        repositories::file_repository::TFileRepository,
    },
    middleware::auth::AuthenticatedUser,
};
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::{Json, Response},
};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

pub mod stt;

/// List files
pub async fn list_files(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<File>>, StatusCode> {
    let files = state.file_repository.list(&user.0.id).await.map_err(|e| {
        tracing::error!("Failed to list files: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(files))
}

/// Upload a file
pub async fn upload_file(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<File>, StatusCode> {
    // Basic multipart handling - expects one file
    if let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let data = field
            .bytes()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Create uploads directory if it doesn't exist
        let uploads_dir = PathBuf::from("uploads");
        if !uploads_dir.exists() {
            fs::create_dir_all(&uploads_dir)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        // Generate ID and filename
        let file_id = Uuid::new_v4();
        let unique_filename = format!("{}-{}", file_id, filename);
        let filepath = uploads_dir.join(&unique_filename);

        // Save file
        let mut file = fs::File::create(&filepath)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.write_all(&data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Create DB record
        let new_file = NewFile {
            id: Some(file_id),
            file_id: file_id.to_string(),
            user_id: user.0.id.clone(),
            chat_id: None,
            filename,
            filepath: filepath.to_string_lossy().to_string(),
            mime_type: content_type.clone(),
            size_bytes: data.len() as i64,
            file_type: if content_type.starts_with("image/") {
                "image".to_string()
            } else {
                "file".to_string()
            },
            text_content: None,
            width: None,
            height: None,
            source: Some("upload".to_string()),
            metadata: None,
            is_temporary: Some(false),
            expires_at: None,
        };

        let created_file = state.file_repository.create(new_file).await.map_err(|e| {
            tracing::error!("Failed to create file record: {}", e);
            // Try to clean up the file if DB fails
            let _ = tokio::fs::remove_file(filepath);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(Json(created_file))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

/// Get file metadata
pub async fn get_file(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<File>, StatusCode> {
    let file = state
        .file_repository
        .get(id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if file.user_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(file))
}

/// Get file content
pub async fn get_file_content(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Response, StatusCode> {
    let file = state
        .file_repository
        .get(id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if file.user_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let filepath = PathBuf::from(&file.filepath);
    let file_handle = fs::File::open(filepath)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let stream = ReaderStream::new(file_handle);
    let body = Body::from_stream(stream);

    Response::builder()
        .header(header::CONTENT_TYPE, file.mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", file.filename),
        )
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(serde::Serialize)]
pub struct DownloadUrlResponse {
    pub url: String,
}

/// Get download URL
pub async fn get_download_url(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<DownloadUrlResponse>, StatusCode> {
    let file = state
        .file_repository
        .get(id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if file.user_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Return the content URL
    Ok(Json(DownloadUrlResponse {
        url: format!("/api/v1/files/{}/content", id),
    }))
}

/// Delete file
pub async fn delete_file(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let file = state
        .file_repository
        .get(id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get file for deletion: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if file.user_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Delete from DB first
    state.file_repository.delete(id).await.map_err(|e| {
        tracing::error!("Failed to delete file record: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Delete from disk
    let filepath = PathBuf::from(&file.filepath);
    if filepath.exists() {
        if let Err(e) = fs::remove_file(filepath).await {
            tracing::error!("Failed to remove file from disk: {}", e);
            // We don't fail the request here because the DB record is already gone
            // This aligns with fix #10894 - better error handling/logging
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
