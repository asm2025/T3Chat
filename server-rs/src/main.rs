use anyhow::Result;
use axum::{
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod api;
mod db;
mod env;
pub mod middleware;
mod schema;

use db::get_database;
use env::{get_env, get_port};

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server_rs=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    info!("🚀 Starting Rust backend server...");

    // Connect to database
    let database_url = get_env("DATABASE_URL")
        .unwrap_or_else(|| "postgresql://postgres:password@localhost:5502/postgres".to_string());
    
    info!("🔗 Connecting to database...");
    let db = get_database(&database_url).await?;
    info!("✅ Database connection established");

    let app_state = AppState { db };

    // Build the application
    let app = create_router(app_state).await;

    // Parse CLI arguments for port
    let port = parse_cli_args().unwrap_or_else(|| get_port());
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("🌐 Server listening on http://{}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn parse_cli_args() -> Option<u16> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(port_index) = args.iter().position(|arg| arg == "--port") {
        if let Some(port_str) = args.get(port_index + 1) {
            return port_str.parse().ok();
        }
    }
    None
}

async fn create_router(state: AppState) -> Router {
    let protected_routes = Router::new()
        .route("/me", get(api::protected_me))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    Router::new()
        .route("/", get(health_check))
        .route("/api/v1/hello", get(api::hello))
        .route("/api/v1/db-test", get(api::db_test))
        .nest("/api/v1/protected", protected_routes)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                }),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "API is running"
    }))
}

