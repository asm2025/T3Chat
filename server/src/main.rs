use anyhow::Result;
use axum::{
    Router,
    http::HeaderValue,
    routing::{delete, get, post, put},
};
use emix::env::{get_env, get_port_or};
use std::{net::SocketAddr, sync::Arc};
use tokio::task::spawn_blocking;
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};
use utoipa::OpenApi;

mod ai;
mod api;
mod db;
mod docs;
mod env;
pub mod middleware;

/// ONLY use concrete types in app state because of the heap allocation requirements of trait objects.
/// NEVER derive Debug or Display for AppState.
#[derive(Clone)]
pub struct AppState {
    pub db: db::DbPool,
    pub user_repository: Arc<db::repositories::UserRepository>,
    pub ai_model_repository: Arc<db::repositories::AiModelRepository>,
    pub user_api_key_repository: Arc<db::repositories::UserApiKeyRepository>,
    pub chat_repository: Arc<db::repositories::ChatRepository>,
    pub user_feature_repository: Arc<db::repositories::UserFeatureRepository>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env::ensure_env_loaded();

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
    let pool = db::connect(&database_url, true).await?;

    // Initialize repositories
    tracing::info!("Initializing repositories...");

    let user_repository = Arc::new(db::repositories::UserRepository::new(pool.clone()));
    let ai_model_repository = Arc::new(db::repositories::AiModelRepository::new(pool.clone()));
    let user_api_key_repository =
        Arc::new(db::repositories::UserApiKeyRepository::new(pool.clone()));
    let chat_repository = Arc::new(db::repositories::ChatRepository::new(pool.clone()));
    let user_feature_repository =
        Arc::new(db::repositories::UserFeatureRepository::new(pool.clone()));

    let state = AppState {
        db: pool,
        user_repository,
        ai_model_repository,
        user_api_key_repository,
        chat_repository,
        user_feature_repository,
    };
    tracing::info!("Database configured successfully.");

    // Build the application
    tracing::info!("Configuring application");
    let app = setup_router(state)?;
    tracing::info!("Application configured successfully.");

    tracing::info!("Starting server");
    // Parse CLI arguments for port
    let port = parse_cli_args().unwrap_or_else(|| get_port_or(3000));
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    // Start the server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;
    // Create shutdown signal handler
    let graceful_shutdown = async {
        let wait_for_ctrl_c = || async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler")
        };

        #[cfg(unix)]
        let wait_for_sigterm = || async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler")
                .recv()
                .await
        };

        #[cfg(not(unix))]
        let wait_for_sigterm = || std::future::pending::<()>();

        loop {
            tokio::select! {
                _ = wait_for_ctrl_c() => {
                    if confirm_shutdown("Terminate server (Y/N)? ").await {
                        tracing::info!("Confirmed Ctrl+C shutdown; shutting down gracefully...");
                        break;
                    } else {
                        tracing::info!("Shutdown canceled; continuing to run.");
                    }
                },
                _ = wait_for_sigterm() => {
                    if confirm_shutdown("Terminate server (Y/N)? ").await {
                        tracing::info!("Confirmed SIGTERM shutdown; shutting down gracefully...");
                        break;
                    } else {
                        tracing::info!("Shutdown canceled; continuing to run.");
                    }
                },
            }
        }
    };

    // Serve with graceful shutdown
    tracing::info!("Server listening on http://localhost:{}", port);
    axum::serve(listener, app)
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

fn setup_router(state: AppState) -> Result<Router> {
    tracing::info!("Configuring router");

    let curdir = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
    let static_path = curdir.join("wwwroot");

    let cors_origins_str = std::env::var("CORS_ORIGINS")
        .map_err(|e| anyhow::anyhow!("Failed to get CORS_ORIGINS: {}", e))?;

    let origins: Vec<HeaderValue> = cors_origins_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<HeaderValue>()
                .map_err(|e| anyhow::anyhow!("Invalid CORS origin '{}': {}", s, e))
        })
        .collect::<Result<Vec<_>>>()?;

    if origins.is_empty() {
        return Err(anyhow::anyhow!(
            "No valid CORS origins found in CORS_ORIGINS"
        ));
    }

    tracing::info!(
        "Allowed CORS origins: {:?}",
        origins
            .iter()
            .map(|v| v.to_str().unwrap_or("<invalid>"))
            .collect::<Vec<_>>()
    );

    // Clone origins for the closure
    let allowed_origins = origins.clone();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(
            move |origin: &HeaderValue, _request: &_| allowed_origins.contains(origin),
        ))
        .allow_methods(AllowMethods::list([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::HEAD,
            axum::http::Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ]))
        .allow_credentials(true);
    let models_routes = Router::new()
        .route("/", get(api::v1::models::list_models))
        .route("/all", get(api::v1::models::list_all_models))
        .route("/{id}", get(api::v1::models::get_model))
        .route("/my", get(api::v1::models::list_my_models))
        .route("/{id}/enable", post(api::v1::models::enable_model))
        .route("/{id}/disable", post(api::v1::models::disable_model))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    let chats_routes = Router::new()
        .route(
            "/",
            get(api::v1::chats::list_chats).post(api::v1::chats::create_chat),
        )
        .route(
            "/{id}",
            get(api::v1::chats::get_chat)
                .put(api::v1::chats::update_chat)
                .delete(api::v1::chats::delete_chat),
        )
        .route(
            "/{id}/messages",
            get(api::v1::chats::messages::get_messages)
                .post(api::v1::chats::messages::create_message)
                .delete(api::v1::chats::messages::clear_messages),
        )
        .route(
            "/{chat_id}/messages/{id}",
            put(api::v1::chats::messages::update_message)
                .delete(api::v1::chats::messages::delete_message),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    let chat_routes = Router::new()
        .route("/", post(api::v1::chat::chat))
        .route("/stream", post(api::v1::chat::stream_chat))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    let user_api_keys_routes = Router::new()
        .route(
            "/",
            get(api::v1::user_api_keys::list_keys).post(api::v1::user_api_keys::create_key),
        )
        .route("/{id}", delete(api::v1::user_api_keys::delete_key))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    let features_routes = Router::new()
        .route("/", get(api::v1::features::list_features))
        .route("/{feature}", put(api::v1::features::update_feature))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    let authenticated_routes = Router::new()
        .route(
            "/me",
            get(api::v1::user::profile).put(api::v1::user::update_profile),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    let router = Router::new()
        .route("/health", get(api::v1::health::health_check))
        .nest("/api/v1/models", models_routes)
        .nest("/api/v1/chats", chats_routes)
        .nest("/api/v1/chat", chat_routes)
        .nest("/api/v1/user-api-keys", user_api_keys_routes)
        .nest("/api/v1/features", features_routes)
        .nest("/api/v1", authenticated_routes)
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
        .with_state(state);

    if env::is_swagger_enabled() {
        Ok(router.merge(swagger_docs_router()))
    } else {
        Ok(router)
    }
}

fn swagger_docs_router() -> Router {
    let open_api = docs::ApiDoc::openapi();
    let swagger: Router =
        Into::<Router>::into(utoipa_swagger_ui::SwaggerUi::new("/").url("/openapi.json", open_api));
    swagger
}

async fn confirm_shutdown(prompt: &str) -> bool {
    // Use a blocking read in a background thread to avoid stalling the async runtime
    let prompt_owned = prompt.to_owned();
    match spawn_blocking(move || {
        use std::io::{Write, stdin, stdout};
        let prompt = prompt_owned;
        print!("{prompt}");
        let _ = stdout().flush();
        let mut input = String::new();
        if stdin().read_line(&mut input).is_ok() {
            let first = input.trim().chars().next().unwrap_or('n');
            first == 'y' || first == 'Y'
        } else {
            false
        }
    })
    .await
    {
        Ok(confirm) => confirm,
        Err(_) => false,
    }
}
