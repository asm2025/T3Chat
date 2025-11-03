use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use crate::middleware::auth::AuthenticatedUser;
use crate::db::test_database_connection;
use crate::schema::users::Entity;
use sea_orm::EntityTrait;

pub async fn hello() -> Json<Value> {
    Json(json!({
        "message": "Hello from Rust Axum!"
    }))
}

pub async fn db_test(
    State(db): State<sea_orm::DatabaseConnection>,
) -> Result<Json<Value>, StatusCode> {
    // Test database connection
    let is_healthy = test_database_connection(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !is_healthy {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Fetch users
    let users = Entity::find()
        .limit(5)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "message": "Database connection successful!",
        "users": users,
        "connectionHealthy": is_healthy,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })))
}

pub async fn protected_me(
    request: Request,
    State(_db): State<sea_orm::DatabaseConnection>,
) -> Result<Json<Value>, StatusCode> {
    // Extract user from request extensions (set by auth middleware)
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(Json(json!({
        "user": {
            "id": user.0.id,
            "email": user.0.email,
            "display_name": user.0.display_name,
            "photo_url": user.0.photo_url,
            "created_at": user.0.created_at.to_rfc3339(),
            "updated_at": user.0.updated_at.to_rfc3339(),
        },
        "message": "You are authenticated!",
    })))
}

