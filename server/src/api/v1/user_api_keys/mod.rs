use crate::{
    AppState, db::prelude::*, db::repositories::TUserApiKeyRepository,
    middleware::auth::AuthenticatedUser,
};
use axum::{extract::Path, extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct UserApiKeyResponse {
    pub id: Uuid,
    pub user_id: String,
    pub provider: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserApiKeyModel> for UserApiKeyResponse {
    fn from(key: UserApiKeyModel) -> Self {
        Self {
            id: key.id,
            user_id: key.user_id,
            provider: match key.provider {
                AiProvider::OpenAI => "openai".to_string(),
                AiProvider::Anthropic => "anthropic".to_string(),
                AiProvider::Google => "google".to_string(),
                AiProvider::DeepSeek => "deepseek".to_string(),
                AiProvider::Ollama => "ollama".to_string(),
            },
            is_default: key.is_default,
            created_at: key.created_at.to_rfc3339(),
            updated_at: key.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserApiKeyRequest {
    pub provider: String,
    pub api_key: String,
    pub is_default: Option<bool>,
}

/// List all API keys for the authenticated user
#[utoipa::path(
    get,
    path = "/api/v1/user-api-keys",
    tag = "User API Keys",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of API keys", body = [UserApiKeyResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_keys(
    user: AuthenticatedUser,
    state: State<AppState>,
) -> Result<Json<Vec<UserApiKeyResponse>>, StatusCode> {
    let keys = state
        .user_api_key_repository
        .list(&user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        keys.into_iter().map(UserApiKeyResponse::from).collect(),
    ))
}

/// Create a new API key for the authenticated user
#[utoipa::path(
    post,
    path = "/api/v1/user-api-keys",
    tag = "User API Keys",
    security(("bearer_auth" = [])),
    request_body = CreateUserApiKeyRequest,
    responses(
        (status = 200, description = "API key created", body = UserApiKeyResponse),
        (status = 400, description = "Invalid provider"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_key(
    user: AuthenticatedUser,
    state: State<AppState>,
    Json(payload): Json<CreateUserApiKeyRequest>,
) -> Result<Json<UserApiKeyResponse>, StatusCode> {
    let provider = match payload.provider.as_str() {
        "openai" => AiProvider::OpenAI,
        "anthropic" => AiProvider::Anthropic,
        "google" => AiProvider::Google,
        "deepseek" => AiProvider::DeepSeek,
        "ollama" => AiProvider::Ollama,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    // TODO: Encrypt API key
    // For now, store as plaintext (not secure!)
    let encrypted_key = payload.api_key;

    let key = state
        .user_api_key_repository
        .create(CreateUserApiKeyDto {
            user_id: user.0.id.clone(),
            provider,
            encrypted_key,
            is_default: payload.is_default.unwrap_or(false),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // If this is set as default, update it
    if payload.is_default.unwrap_or(false) {
        state
            .user_api_key_repository
            .set_default(key.id, &user.0.id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(UserApiKeyResponse::from(key)))
}

/// Delete an API key
#[utoipa::path(
    delete,
    path = "/api/v1/user-api-keys/{id}",
    tag = "User API Keys",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "API key identifier")
    ),
    responses(
        (status = 204, description = "API key deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "API key not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_key(
    user: AuthenticatedUser,
    state: State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Verify key belongs to user
    let keys = state
        .user_api_key_repository
        .list(&user.0.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !keys.iter().any(|k| k.id == id) {
        return Err(StatusCode::NOT_FOUND);
    }

    state
        .user_api_key_repository
        .delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
