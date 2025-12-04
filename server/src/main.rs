#![allow(dead_code)]

use anyhow::Result;
use axum::{
    Json, Router,
    body::Body,
    extract::Path,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, post, put},
};
use emix::env::{get_env, get_port_or};
use std::{net::SocketAddr, sync::Arc};
use tokio::task::spawn_blocking;
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};
use utoipa::OpenApi;

mod ai;
mod api;
mod auth;
mod config;
mod db;
mod docs;
mod env;
pub mod middleware;
mod utils;

/// ONLY use concrete types in app state because of the heap allocation requirements of trait objects.
/// NEVER derive Debug or Display for AppState.
#[derive(Clone)]
pub struct AppState {
    pub db: db::DbPool,
    pub user_repository: Arc<db::repositories::UserRepository>,
    pub ai_model_repository: Arc<db::repositories::AiModelRepository>,
    pub ai_provider_repository: Arc<db::repositories::AiProviderRepository>,
    pub user_api_key_repository: Arc<db::repositories::UserApiKeyRepository>,
    pub chat_repository: Arc<db::repositories::ChatRepository>,
    pub user_feature_repository: Arc<db::repositories::UserFeatureRepository>,
    pub oidc_client: Option<Arc<auth::OidcClient>>,
    pub jwks_cache: Option<Arc<auth::JwksCache>>,
    pub t3_config: Arc<config::t3chat::T3ChatConfig>,
    pub model_catalog: Arc<ai::model_catalog::ModelCatalog>,
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

    tracing::info!("Loading T3Chat configuration");
    let t3_config = Arc::new(
        config::load_config()
            .map_err(|err| anyhow::anyhow!("failed to load t3chat.yaml: {}", err))?,
    );
    tracing::info!("Building model catalog");
    let model_catalog = Arc::new(
        ai::model_catalog::ModelCatalog::build(t3_config.clone())
            .await
            .map_err(|err| anyhow::anyhow!("failed to build model catalog: {}", err))?,
    );

    let database_url =
        get_env("DATABASE_URL").ok_or_else(|| anyhow::anyhow!("DATABASE_URL is not set"))?;
    if database_url.is_empty() {
        tracing::error!("DATABASE_URL is empty");
        return Err(anyhow::anyhow!("DATABASE_URL is empty"));
    }
    let pool = db::connect(&database_url, true).await?;

    // Initialize repositories
    tracing::info!("Initializing repositories...");

    let user_repository = Arc::new(db::repositories::UserRepository::new(pool.clone()));
    let ai_model_repository = Arc::new(db::repositories::AiModelRepository::new(pool.clone()));
    let ai_provider_repository =
        Arc::new(db::repositories::AiProviderRepository::new(pool.clone()));
    let user_api_key_repository =
        Arc::new(db::repositories::UserApiKeyRepository::new(pool.clone()));
    let chat_repository = Arc::new(db::repositories::ChatRepository::new(pool.clone()));
    let user_feature_repository =
        Arc::new(db::repositories::UserFeatureRepository::new(pool.clone()));

    // Initialize OIDC client and JWKS cache (optional)
    let (oidc_client, jwks_cache) = if env::is_oidc_configured() {
        tracing::info!("OIDC configuration detected. Initializing OIDC client and JWKS cache...");
        let oidc_issuer_url = env::get_oidc_issuer_url()
            .map_err(|e| anyhow::anyhow!("Failed to get OIDC_ISSUER_URL: {}", e))?;
        let oidc_client_id = env::get_oidc_client_id()
            .map_err(|e| anyhow::anyhow!("Failed to get OIDC_CLIENT_ID: {}", e))?;
        let oidc_client_secret = env::get_oidc_client_secret()
            .map_err(|e| anyhow::anyhow!("Failed to get OIDC_CLIENT_SECRET: {}", e))?;
        let oidc_redirect_uri = env::get_oidc_redirect_uri()
            .map_err(|e| anyhow::anyhow!("Failed to get OIDC_REDIRECT_URI: {}", e))?;

        let client = Arc::new(
            auth::OidcClient::new(
                oidc_issuer_url.clone(),
                oidc_client_id,
                oidc_client_secret,
                oidc_redirect_uri,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create OIDC client: {}", e))?,
        );

        let jwks_uri = client
            .get_jwks_uri()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to resolve JWKS URI: {}", e))?;

        let cache = Arc::new(
            auth::JwksCache::new(oidc_issuer_url, jwks_uri)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create JWKS cache: {}", e))?,
        );

        tracing::info!("OIDC client and JWKS cache initialized successfully.");
        (Some(client), Some(cache))
    } else {
        tracing::info!("OIDC not configured.");
        (None, None)
    };

    let state = AppState {
        db: pool,
        user_repository,
        ai_model_repository,
        ai_provider_repository,
        user_api_key_repository,
        chat_repository,
        user_feature_repository,
        oidc_client,
        jwks_cache,
        t3_config,
        model_catalog,
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

    let config_routes = Router::new()
        .route(
            "/startup",
            get(api::v1::config::startup::get_startup_config),
        )
        .route("/models", get(api::v1::config::models::list_models));

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

    let user_routes = Router::new()
        .route(
            "/me",
            get(api::v1::user::profile).put(api::v1::user::update_profile),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    // Auth routes - /me requires authentication
    let auth_me_route = Router::new()
        .route("/me", get(api::v1::auth::me))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    // Admin routes
    let admin_routes = Router::new()
        .nest("/users", api::v1::admin::users::router())
        .nest("/providers", api::v1::admin::providers::router())
        .nest("/models", api::v1::admin::models::router())
        .nest("/dashboard", api::v1::admin::dashboard::router())
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::admin::admin_middleware,
        ));

    let api_router = Router::new()
        .route("/health", get(api::v1::health::health_check))
        .nest("/api/v1/auth", api::v1::auth::router().merge(auth_me_route))
        .nest("/api/v1/models", models_routes)
        .nest("/api/v1/chats", chats_routes)
        .nest("/api/v1/chat", chat_routes)
        .nest("/api/v1/keys", user_api_keys_routes)
        .nest("/api/v1/features", features_routes)
        .nest("/api/v1", user_routes)
        .nest("/api/v1/admin", admin_routes)
        .nest("/api/v1/config", config_routes);

    let index_html = static_path.join("index.html");
    let static_files_service = ServeDir::new(static_path)
        .append_index_html_on_directories(true)
        .not_found_service(ServeFile::new(index_html));

    let mut router = api_router
        .fallback_service(static_files_service)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
        )
        .layer(cors);

    // Add debug middleware if enabled
    if env::is_debug_routes_enabled() {
        tracing::warn!("🔍 Route debugging enabled");
        router = router.layer(axum::middleware::from_fn(debug_route_middleware));
    }

    if env::is_swagger_enabled() {
        tracing::info!("📚 Swagger UI enabled at /swagger-ui");
        router = router.merge(swagger_docs_router());
    }

    let router = router.with_state(state);

    Ok(router)
}

fn swagger_docs_router() -> Router<AppState> {
    let open_api = Arc::new(docs::ApiDoc::openapi());
    let swagger_config: Arc<utoipa_swagger_ui::Config<'static>> =
        Arc::new(utoipa_swagger_ui::Config::from("/openapi.json"));

    let openapi_route = Router::<AppState>::new().route(
        "/openapi.json",
        get({
            let open_api = open_api.clone();
            move || {
                let open_api = open_api.clone();
                async move { Json((*open_api).clone()) }
            }
        }),
    );

    let ui_router = Router::<AppState>::new()
        .route(
            "/swagger-ui",
            get(|| async { Redirect::temporary("/swagger-ui/") }),
        )
        .route(
            "/swagger-ui/",
            get({
                let config = swagger_config.clone();
                move || {
                    let config = config.clone();
                    async move { serve_swagger_ui_response("", config) }
                }
            }),
        )
        .route(
            "/swagger-ui/{*path}",
            get({
                let config = swagger_config.clone();
                move |Path(path): Path<String>| {
                    let config = config.clone();
                    async move { serve_swagger_ui_response(&path, config) }
                }
            }),
        );

    openapi_route.merge(ui_router)
}

fn serve_swagger_ui_response(
    path: &str,
    config: Arc<utoipa_swagger_ui::Config<'static>>,
) -> Response {
    let relative_path = path.trim_start_matches('/');
    let request_path = if relative_path.is_empty() {
        "/"
    } else {
        relative_path
    };

    match utoipa_swagger_ui::serve(request_path, config) {
        Ok(Some(file)) => {
            let body = Body::from(file.bytes.into_owned());
            match Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, file.content_type)
                .body(body)
            {
                Ok(response) => response,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

async fn debug_route_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();
    let has_auth = request.headers().get("authorization").is_some();

    tracing::info!(
        "🔍 {} {} | Auth: {}",
        method,
        path,
        if has_auth { "✓" } else { "✗" }
    );

    let response = next.run(request).await;
    let status = response.status();

    if !status.is_success() {
        tracing::warn!(
            "🔍 {} {} → {} {}",
            method,
            path,
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        );
        if let Some(allow) = response.headers().get("allow") {
            tracing::warn!("🔍   Allowed methods: {:?}", allow);
        }
    } else {
        tracing::info!("🔍 {} {} → {}", method, path, status.as_u16());
    }

    response
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
