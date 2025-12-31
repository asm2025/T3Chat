use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};
use serde::Serialize;

use crate::AppState;
use crate::middleware::auth::AuthenticatedUser;

#[derive(Debug, Serialize)]
pub struct DashboardStatsResponse {
    pub total_users: u64,
    pub active_users: u64,
    pub disabled_users: u64,
    pub locked_users: u64,
    pub total_providers: u64,
    pub active_providers: u64,
    pub disabled_providers: u64,
    pub total_models: u64,
    pub active_models: u64,
    pub disabled_models: u64,
}

pub async fn get_stats(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<DashboardStatsResponse>, StatusCode> {
    use diesel::QueryableByName;
    use diesel_async::RunQueryDsl;

    #[derive(QueryableByName)]
    struct UserStatsRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        disabled: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        locked: i64,
    }

    #[derive(QueryableByName)]
    struct ProviderStatsRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        disabled: i64,
    }

    #[derive(QueryableByName)]
    struct ModelStatsRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        disabled: i64,
    }

    let mut conn = state
        .db
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get user stats
    let user_stats: Vec<UserStatsRow> = diesel::sql_query(
        r#"
        SELECT 
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE disabled = false AND locked_out = false) as active,
            COUNT(*) FILTER (WHERE disabled = true) as disabled,
            COUNT(*) FILTER (WHERE locked_out = true) as locked
        FROM users
        "#,
    )
    .load(&mut conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get provider stats
    let provider_stats: Vec<ProviderStatsRow> = diesel::sql_query(
        r#"
        SELECT 
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE disabled = false AND is_active = true) as active,
            COUNT(*) FILTER (WHERE disabled = true) as disabled
        FROM ai_providers
        "#,
    )
    .load(&mut conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get model stats
    let model_stats: Vec<ModelStatsRow> = diesel::sql_query(
        r#"
        SELECT 
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE disabled = false AND (is_active IS NULL OR is_active = true)) as active,
            COUNT(*) FILTER (WHERE disabled = true) as disabled
        FROM ai_models
        "#
    )
    .load(&mut conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_stats = user_stats
        .into_iter()
        .next()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let provider_stats = provider_stats
        .into_iter()
        .next()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let model_stats = model_stats
        .into_iter()
        .next()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(DashboardStatsResponse {
        total_users: user_stats.total as u64,
        active_users: user_stats.active as u64,
        disabled_users: user_stats.disabled as u64,
        locked_users: user_stats.locked as u64,
        total_providers: provider_stats.total as u64,
        active_providers: provider_stats.active as u64,
        disabled_providers: provider_stats.disabled as u64,
        total_models: model_stats.total as u64,
        active_models: model_stats.active as u64,
        disabled_models: model_stats.disabled as u64,
    }))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/stats", get(get_stats))
}
