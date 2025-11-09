use axum::response::Json;
use serde_json::{Value, json};

/// Health check endpoint
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "API is running"
    }))
}
