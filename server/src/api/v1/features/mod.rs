use crate::{
    AppState, db::models::Feature, db::repositories::TUserFeatureRepository,
    middleware::auth::AuthenticatedUser,
};
use axum::{extract::Path, extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct UserFeatureResponse {
    pub feature: String,
    pub enabled: bool,
}

impl From<(Feature, bool)> for UserFeatureResponse {
    fn from((feature, enabled): (Feature, bool)) -> Self {
        Self {
            feature: feature.as_str().to_string(),
            enabled,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserFeaturesResponse {
    pub features: Vec<UserFeatureResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFeatureRequest {
    pub enabled: bool,
}

/// List all features for the authenticated user
#[utoipa::path(
    get,
    path = "/api/v1/features",
    tag = "Features",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of user features", body = UserFeaturesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_features(
    user: AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<Json<UserFeaturesResponse>, StatusCode> {
    let features = app_state
        .user_feature_repository
        .list(&user.0.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list features: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Build a map of enabled features
    let mut feature_map: std::collections::HashMap<Feature, bool> = std::collections::HashMap::new();
    for feature_model in &features {
        feature_map.insert(feature_model.feature, feature_model.enabled);
    }

    // Return all possible features with their enabled status (default to false if not set)
    let all_features = vec![Feature::WebSearch];
    let response_features: Vec<UserFeatureResponse> = all_features
        .into_iter()
        .map(|f| {
            let enabled = feature_map.get(&f).copied().unwrap_or(false);
            UserFeatureResponse::from((f, enabled))
        })
        .collect();

    Ok(Json(UserFeaturesResponse {
        features: response_features,
    }))
}

/// Update a specific feature for the authenticated user
#[utoipa::path(
    put,
    path = "/api/v1/features/{feature}",
    tag = "Features",
    security(("bearer_auth" = [])),
    params(
        ("feature" = String, Path, description = "Feature name (e.g., 'web_search')")
    ),
    request_body = UpdateFeatureRequest,
    responses(
        (status = 200, description = "Updated feature", body = UserFeatureResponse),
        (status = 400, description = "Invalid feature name"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_feature(
    user: AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(feature_str): Path<String>,
    Json(payload): Json<UpdateFeatureRequest>,
) -> Result<Json<UserFeatureResponse>, StatusCode> {
    let feature = Feature::from_str(&feature_str).ok_or_else(|| {
        tracing::warn!("Invalid feature name: {}", feature_str);
        StatusCode::BAD_REQUEST
    })?;

    let create_dto = crate::db::models::CreateUserFeatureDto {
        user_id: user.0.id.clone(),
        feature,
        enabled: payload.enabled,
    };

    let updated_feature = app_state
        .user_feature_repository
        .upsert(create_dto)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update feature: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(UserFeatureResponse::from((updated_feature.feature, updated_feature.enabled))))
}

