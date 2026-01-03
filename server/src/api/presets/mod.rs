use crate::{
    AppState,
    db::{
        models::{NewPreset, Preset, UpdatePreset},
        repositories::preset_repository::TPresetRepository,
    },
    middleware::auth::AuthenticatedUser,
};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePresetRequest {
    pub preset_id: Option<String>,
    pub title: String,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub model_label: Option<String>,
    pub chat_id: Option<String>,
    pub prompt_prefix: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub context_window: Option<i32>,
    pub tools: Option<JsonValue>,
    pub options: Option<JsonValue>,
    pub metadata: Option<JsonValue>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePresetRequest {
    pub title: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub model_label: Option<String>,
    pub prompt_prefix: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub context_window: Option<i32>,
    pub tools: Option<JsonValue>,
    pub options: Option<JsonValue>,
    pub metadata: Option<JsonValue>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReorderPresetsRequest {
    pub preset_ids: Vec<Uuid>,
}

/// List all presets for the user
#[utoipa::path(
    get,
    path = "/api/presets",
    tag = "Presets",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of presets", body = Vec<Preset>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_presets(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Preset>>, StatusCode> {
    let presets = state
        .preset_repository
        .list(&user.0.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list presets: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(presets))
}

/// Get a preset by ID
#[utoipa::path(
    get,
    path = "/api/presets/{id}",
    tag = "Presets",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Preset ID")
    ),
    responses(
        (status = 200, description = "Preset details", body = Preset),
        (status = 404, description = "Preset not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_preset(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Preset>, StatusCode> {
    let preset = state
        .preset_repository
        .get(id, &user.0.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get preset: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verify ownership
    if preset.user_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(preset))
}

/// Create a new preset
#[utoipa::path(
    post,
    path = "/api/presets",
    tag = "Presets",
    security(("bearer_auth" = [])),
    request_body = CreatePresetRequest,
    responses(
        (status = 201, description = "Preset created", body = Preset),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_preset(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreatePresetRequest>,
) -> Result<Json<Preset>, StatusCode> {
    // If setting as default, clear existing defaults first
    if payload.is_default == Some(true) {
        let all_presets = state
            .preset_repository
            .list(&user.0.id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to list presets: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for p in all_presets {
            if p.is_default == Some(true) {
                let update = UpdatePreset {
                    is_default: Some(false),
                    title: None,
                    endpoint: None,
                    model: None,
                    model_label: None,
                    model_parameters: None,
                    system_message: None,
                    instructions: None,
                    feature_flags: None,
                    agent_id: None,
                    agent_options: None,
                    updated_at: chrono::Utc::now(),
                    order_index: None,
                };
                let _ = state
                    .preset_repository
                    .update(p.id, &user.0.id, update)
                    .await;
            }
        }
    }

    // Build model_parameters JSON from individual fields
    let mut model_parameters = serde_json::Map::new();
    if let Some(temp) = payload.temperature {
        model_parameters.insert(
            "temperature".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(temp as f64).unwrap()),
        );
    }
    if let Some(top_p) = payload.top_p {
        model_parameters.insert(
            "top_p".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(top_p as f64).unwrap()),
        );
    }
    if let Some(freq_penalty) = payload.frequency_penalty {
        model_parameters.insert(
            "frequency_penalty".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(freq_penalty as f64).unwrap()),
        );
    }
    if let Some(pres_penalty) = payload.presence_penalty {
        model_parameters.insert(
            "presence_penalty".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(pres_penalty as f64).unwrap()),
        );
    }
    if let Some(ctx_window) = payload.context_window {
        model_parameters.insert(
            "context_window".to_string(),
            serde_json::Value::Number(serde_json::Number::from(ctx_window)),
        );
    }
    if let Some(tools) = payload.tools {
        model_parameters.insert("tools".to_string(), tools);
    }
    let model_parameters_value = if model_parameters.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(model_parameters))
    };

    // Build feature_flags JSON from options and metadata
    let mut feature_flags = serde_json::Map::new();
    if let Some(options) = payload.options {
        feature_flags.insert("options".to_string(), options);
    }
    if let Some(metadata) = payload.metadata {
        feature_flags.insert("metadata".to_string(), metadata);
    }
    let feature_flags_value = if feature_flags.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(feature_flags))
    };

    let new_preset = NewPreset {
        id: None,
        preset_id: payload
            .preset_id
            .unwrap_or_else(|| Uuid::new_v4().to_string()),
        user_id: user.0.id.clone(),
        title: payload.title,
        endpoint: payload.endpoint.unwrap_or_default(),
        model: payload.model.unwrap_or_default(),
        model_label: payload.model_label,
        model_parameters: model_parameters_value,
        system_message: payload.prompt_prefix,
        instructions: None,
        feature_flags: feature_flags_value,
        agent_id: None,
        agent_options: None,
        is_default: payload.is_default,
        order_index: None,
    };

    let preset = state
        .preset_repository
        .create(new_preset)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create preset: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(preset))
}

/// Update a preset
#[utoipa::path(
    put,
    path = "/api/presets/{id}",
    tag = "Presets",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Preset ID")
    ),
    request_body = UpdatePresetRequest,
    responses(
        (status = 200, description = "Preset updated", body = Preset),
        (status = 404, description = "Preset not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_preset(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePresetRequest>,
) -> Result<Json<Preset>, StatusCode> {
    let preset = state
        .preset_repository
        .get(id, &user.0.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get preset: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if preset.user_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    // If setting as default, clear existing defaults first
    if payload.is_default == Some(true) {
        let all_presets = state
            .preset_repository
            .list(&user.0.id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to list presets: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        for p in all_presets {
            if p.is_default == Some(true) && p.id != id {
                let update = UpdatePreset {
                    is_default: Some(false),
                    title: None,
                    endpoint: None,
                    model: None,
                    model_label: None,
                    model_parameters: None,
                    system_message: None,
                    instructions: None,
                    feature_flags: None,
                    agent_id: None,
                    agent_options: None,
                    updated_at: chrono::Utc::now(),
                    order_index: None,
                };
                let _ = state
                    .preset_repository
                    .update(p.id, &user.0.id, update)
                    .await;
            }
        }
    }

    // Build model_parameters JSON from individual fields
    let mut model_parameters = preset
        .model_parameters
        .clone()
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    if let Some(temp) = payload.temperature {
        model_parameters.insert(
            "temperature".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(temp as f64).unwrap()),
        );
    }
    if let Some(top_p) = payload.top_p {
        model_parameters.insert(
            "top_p".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(top_p as f64).unwrap()),
        );
    }
    if let Some(freq_penalty) = payload.frequency_penalty {
        model_parameters.insert(
            "frequency_penalty".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(freq_penalty as f64).unwrap()),
        );
    }
    if let Some(pres_penalty) = payload.presence_penalty {
        model_parameters.insert(
            "presence_penalty".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(pres_penalty as f64).unwrap()),
        );
    }
    if let Some(ctx_window) = payload.context_window {
        model_parameters.insert(
            "context_window".to_string(),
            serde_json::Value::Number(serde_json::Number::from(ctx_window)),
        );
    }
    if let Some(tools) = payload.tools {
        model_parameters.insert("tools".to_string(), tools);
    }
    let model_parameters_value = if model_parameters.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(model_parameters))
    };

    // Build feature_flags JSON from options and metadata
    let mut feature_flags = preset
        .feature_flags
        .clone()
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    if let Some(options) = payload.options {
        feature_flags.insert("options".to_string(), options);
    }
    if let Some(metadata) = payload.metadata {
        feature_flags.insert("metadata".to_string(), metadata);
    }
    let feature_flags_value = if feature_flags.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(feature_flags))
    };

    let update = UpdatePreset {
        title: payload.title,
        endpoint: payload.endpoint,
        model: payload.model,
        model_label: payload.model_label.map(Some),
        model_parameters: model_parameters_value,
        system_message: payload.prompt_prefix.map(Some),
        instructions: None,
        feature_flags: feature_flags_value,
        agent_id: None,
        agent_options: None,
        is_default: payload.is_default,
        updated_at: chrono::Utc::now(),
        order_index: None,
    };

    let updated = state
        .preset_repository
        .update(id, &user.0.id, update)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update preset: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(updated))
}

/// Delete a preset
#[utoipa::path(
    delete,
    path = "/api/presets/{id}",
    tag = "Presets",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Preset ID")
    ),
    responses(
        (status = 204, description = "Preset deleted"),
        (status = 404, description = "Preset not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_preset(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let preset = state
        .preset_repository
        .get(id, &user.0.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get preset: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if preset.user_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    state
        .preset_repository
        .delete(id, &user.0.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete preset: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Reorder presets
#[utoipa::path(
    post,
    path = "/api/presets/reorder",
    tag = "Presets",
    security(("bearer_auth" = [])),
    request_body = ReorderPresetsRequest,
    responses(
        (status = 200, description = "Presets reordered"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn reorder_presets(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<ReorderPresetsRequest>,
) -> Result<StatusCode, StatusCode> {
    // This assumes the repo has a batch update or we loop
    // For MVP, we might skip implementation details or loop updates
    for (index, id) in payload.preset_ids.iter().enumerate() {
        let update = UpdatePreset {
            order_index: Some(index as i32),
            title: None,
            endpoint: None,
            model: None,
            model_label: None,
            model_parameters: None,
            system_message: None,
            instructions: None,
            feature_flags: None,
            agent_id: None,
            agent_options: None,
            is_default: None,
            updated_at: chrono::Utc::now(),
        };

        let _ = state
            .preset_repository
            .update(*id, &user.0.id, update)
            .await;
    }

    Ok(StatusCode::OK)
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_presets).post(create_preset))
        .route(
            "/{id}",
            get(get_preset).put(update_preset).delete(delete_preset),
        )
        .route("/reorder", post(reorder_presets))
}
