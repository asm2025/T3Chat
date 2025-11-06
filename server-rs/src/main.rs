use anyhow::Result;
use axum::{Router, http::HeaderValue, routing::get};
use dotenvy::dotenv;
use emix::env::{get_env, get_port_or};
use sea_orm::prelude::*;
use sea_orm_migration::prelude::*;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

mod api;
mod db;
mod env;
pub mod middleware;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub user_repository: Arc<Box<dyn db::repositories::IUserRepository>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let app_name = env::APP_INFO.name.to_string();
    setup_tracing(&app_name).unwrap_or_else(|e| {
        tracing::error!("Failed to setup tracing: {}", e);
        std::process::exit(1);
    });
    tracing::info!("Starting {app_name}...");

    let result = run().await;

    if let Err(e) = result {
        tracing::error!("{app_name} error: {e}");
        std::process::exit(1);
    }

    tracing::info!("{app_name} shutdown.");
    Ok(())
}

async fn run() -> Result<()> {
    // Connect to database
    tracing::info!("Configuring database");

    let database_url = get_env("DATABASE_URL").ok_or_else(|| {
        tracing::error!("DATABASE_URL is not set.");
        std::process::exit(1);
    })?;
    let connection = db::connect(&database_url, true).await?;

    // Initialize repositories
    tracing::info!("Initializing repositories...");

    let user_repository: Arc<Box<dyn db::repositories::IUserRepository>> = Arc::new(Box::new(
        db::repositories::UserRepository::new(connection.clone()),
    ));
    let state = AppState {
        db: connection,
        user_repository,
    };
    tracing::info!("Database configured successfully.");

    // Build the application
    tracing::info!("Configuring application");
    let app = setup_router(state);
    tracing::info!("Application configured successfully.");

    tracing::info!("Starting server");
    // Parse CLI arguments for port
    let port = parse_cli_args().unwrap_or_else(|| get_port_or(3000));
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    // Start the server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;
    // Create shutdown signal handler
    let graceful_shutdown = async {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!("Received Ctrl+C signal, shutting down gracefully...");
            },
            _ = terminate => {
                tracing::info!("Received SIGTERM signal, shutting down gracefully...");
            },
        }
    };

    // Serve with graceful shutdown
    tracing::info!("Server listening on http://{}", addr);
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(graceful_shutdown)
        .await?;
    tracing::info!("Server shutdown complete");

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

// Setup
fn setup_tracing(name: &str) -> Result<()> {
    // Create a directory for logs if it doesn't exist
    std::fs::create_dir_all("_logs")?;

    // Setup file appender for logging
    let log_filename = name.to_owned();
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "_logs", &log_filename);
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::TRACE
    } else {
        LevelFilter::INFO
    };
    let filter = EnvFilter::from_default_env()
        .add_directive("sqlx::query=off".parse()?)
        .add_directive("sqlx_core=off".parse()?)
        .add_directive(log_level.into());

    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .compact()
                .with_file(true)
                .with_line_number(true)
                .with_thread_names(true)
                .with_target(false),
        )
        .with(
            fmt::layer().with_writer(file_appender).with_ansi(false), // No color codes in file
        )
        .init();

    Ok(())
}

fn setup_router(state: AppState) -> Router {
    tracing::info!("Configuring router");

    let curdir = std::env::current_dir().unwrap();
    let static_path = curdir.join("wwwroot");
    let origins = std::env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost".to_string())
        .split(',')
        .map(|s| s.trim().parse::<HeaderValue>().unwrap())
        .collect::<Vec<_>>();
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers(Any);
    let profile_routes = Router::new()
        .route("/me", get(api::protected::profile::me))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));
    Router::new()
        .route("/", get(api::health_check))
        .route("/api/v1/hello", get(api::hello))
        .route("/api/v1/db-test", get(api::db_test))
        .nest("/api/v1/profile", profile_routes)
        .fallback_service(ServeDir::new(static_path).append_index_html_on_directories(true))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
        )
        .layer(cors)
        .with_state(state)
}
