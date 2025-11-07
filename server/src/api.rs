use axum::{extract::State, http::StatusCode, response::Json};
use chrono::Utc;
use emixdb::prelude::{Pagination, ResultSet};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    AppState,
    db::{prelude::*, test_database_connection},
    middleware::auth::AuthenticatedUser,
};

#[derive(Debug, Serialize)]
pub struct HelloResponse {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DbTestResponse {
    pub message: String,
    pub users: ResultSet<UserModel>,
    #[serde(rename = "connectionHealthy")]
    pub connection_healthy: bool,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub image_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserModel> for UserResponse {
    fn from(user: UserModel) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            image_url: user.image_url,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user: UserResponse,
    pub message: String,
}

pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "API is running"
    }))
}

pub async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello from Rust Axum!".to_string(),
    })
}

pub async fn db_test(State(state): State<AppState>) -> Result<Json<DbTestResponse>, StatusCode> {
    // Test database connection
    let is_healthy = test_database_connection(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !is_healthy {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Fetch users using repository
    let users = state
        .user_repository
        .list(
            None,
            Some(Pagination {
                page: 1,
                page_size: 5,
            }),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(DbTestResponse {
        message: "Database connection successful!".to_string(),
        users,
        connection_healthy: is_healthy,
        timestamp: Utc::now().to_rfc3339(),
    }))
}

pub mod protected {
    use super::*;

    pub mod profile {
        use super::*;

        pub async fn me(user: AuthenticatedUser) -> Result<Json<MeResponse>, StatusCode> {
            Ok(Json(MeResponse {
                user: UserResponse::from(user.0),
                message: "You are authenticated!".to_string(),
            }))
        }
    }
}
