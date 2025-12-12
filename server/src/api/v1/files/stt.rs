use crate::{
    AppState,
    ai::audio::openai_stt::OpenAISttClient,
    db::{models::AiProvider, repositories::user_api_key_repository::TUserApiKeyRepository},
    middleware::auth::AuthenticatedUser,
    utils::encryption,
};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SttResponse {
    pub text: String,
}

/// Speech-to-text (OpenAI Whisper).
///
/// Expects `multipart/form-data` with:
/// - `file`: the audio file
/// - `language` (optional): locale string like "en-US" or "en" (invalid values are ignored)
/// - `model` (optional): OpenAI transcription model (defaults to "whisper-1")
pub async fn transcribe_audio(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<SttResponse>, Response> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut mime_type: Option<String> = None;
    let mut language: Option<String> = None;
    let mut model: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST.into_response())?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                filename = Some(field.file_name().unwrap_or("audio").to_string());
                mime_type = Some(
                    field
                        .content_type()
                        .unwrap_or("application/octet-stream")
                        .to_string(),
                );
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST.into_response())?
                        .to_vec(),
                );
            }
            "language" => {
                language = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST.into_response())?,
                );
            }
            "model" => {
                model = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST.into_response())?,
                );
            }
            _ => {
                // ignore unknown fields
            }
        }
    }

    let audio_bytes = file_bytes.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing 'file' field" })),
        )
            .into_response()
    })?;

    let filename = filename.unwrap_or_else(|| "audio".to_string());
    let mime_type = mime_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let model = model.unwrap_or_else(|| "whisper-1".to_string());

    // Get user's OpenAI API key
    let api_key = state
        .user_api_key_repository
        .get_default_for_provider(&user.0.id, &AiProvider::OpenAI)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching OpenAI API key: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        })?
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Missing API key for provider 'openai'. Please add one in Settings > API Keys.",
                    "code": "missing_api_key",
                    "provider": "openai"
                })),
            )
                .into_response()
        })?;

    let decrypted_key = encryption::decrypt(&api_key.encrypted_key).map_err(|e| {
        tracing::error!("Failed to decrypt OpenAI API key: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?;

    let stt_client = OpenAISttClient::new(decrypted_key);
    let text = stt_client
        .transcribe(audio_bytes, filename, mime_type, language, model)
        .await
        .map_err(|e| {
            tracing::error!("STT error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "STT request failed" })),
            )
                .into_response()
        })?;

    Ok(Json(SttResponse { text }))
}


