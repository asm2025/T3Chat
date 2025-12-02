# Integration Guide: OIDC & Rate Limiting

This guide shows how to integrate the rewritten OIDC authentication and rate limiting modules into your application.

## Table of Contents

1. [OIDC Integration](#oidc-integration)
2. [Rate Limiting Integration](#rate-limiting-integration)
3. [Combined Example](#combined-example)
4. [Removing Old Dependencies](#removing-old-dependencies)

## OIDC Integration

The OIDC module is already integrated in your `main.rs`. No changes needed! The rewritten version maintains the same API but with better error handling and simpler implementation.

### Current Integration (Already Working)

```rust
// In main.rs - already implemented
let oidc_client = Arc::new(
    auth::OidcClient::new(
        oidc_issuer_url.clone(),
        oidc_client_id,
        oidc_client_secret,
        oidc_redirect_uri,
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to create OIDC client: {}", e))?,
);
```

### Auth Routes (Already Working)

```rust
// In src/api/v1/auth/mod.rs - already implemented
pub async fn login(State(state): State<AppState>) -> Result<Redirect, StatusCode> {
    let (auth_url, _state_token) = state.oidc_client.get_authorization_url();
    Ok(Redirect::to(auth_url.as_str()))
}

pub async fn callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackQuery>,
) -> Result<Redirect, StatusCode> {
    let token_response = state.oidc_client.exchange_code(params.code, params.state).await?;
    let user_info = state.oidc_client.get_user_info(token_response.access_token).await?;
    // ... rest of your logic
}
```

## Rate Limiting Integration

The new rate limiting module needs to be integrated into your router. Here's how:

### Step 1: Update AppState

Add rate limiters to your AppState in `main.rs`:

```rust
use crate::middleware::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub db: db::DbPool,
    // ... existing fields ...
    pub oidc_client: Arc<auth::OidcClient>,
    pub jwks_cache: Arc<auth::JwksCache>,
    
    // Add these rate limiters
    pub global_rate_limiter: Arc<RateLimiter>,
    pub login_rate_limiter: Arc<RateLimiter>,
    pub api_rate_limiter: Arc<RateLimiter>,
}
```

### Step 2: Initialize Rate Limiters

In your `run()` function in `main.rs`, initialize the rate limiters:

```rust
use crate::middleware::rate_limit::{RateLimiter, presets};

async fn run() -> Result<()> {
    // ... existing database and repository initialization ...
    
    // Initialize rate limiters
    tracing::info!("Initializing rate limiters...");
    
    let global_rate_limiter = Arc::new(RateLimiter::new(presets::global()));
    global_rate_limiter.clone().start_cleanup_task();
    
    let login_rate_limiter = Arc::new(RateLimiter::new(presets::login()));
    login_rate_limiter.clone().start_cleanup_task();
    
    let api_rate_limiter = Arc::new(RateLimiter::new(presets::authenticated()));
    api_rate_limiter.clone().start_cleanup_task();
    
    tracing::info!("Rate limiters initialized successfully.");
    
    let state = AppState {
        db: pool,
        // ... existing fields ...
        global_rate_limiter,
        login_rate_limiter,
        api_rate_limiter,
    };
    
    // ... rest of your code ...
}
```

### Step 3: Apply Rate Limiting to Routes

Update your `setup_router()` function to apply rate limiting:

```rust
use crate::middleware::rate_limit::rate_limit_middleware;

fn setup_router(state: AppState) -> Result<Router> {
    // ... existing CORS and other setup ...
    
    // Auth routes with login rate limiter
    let auth_public_routes = Router::new()
        .route("/login", get(api::v1::auth::login))
        .route("/callback", get(api::v1::auth::callback))
        .route("/refresh", post(api::v1::auth::refresh))
        .layer({
            let limiter = state.login_rate_limiter.clone();
            axum::middleware::from_fn(move |req, next| {
                rate_limit_middleware(req, next, limiter.clone())
            })
        });
    
    let auth_me_route = Router::new()
        .route("/me", get(api::v1::auth::me))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));
    
    // API routes with API rate limiter
    let models_routes = Router::new()
        .route("/", get(api::v1::models::list_models))
        .route("/all", get(api::v1::models::list_all_models))
        // ... other routes ...
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ))
        .layer({
            let limiter = state.api_rate_limiter.clone();
            axum::middleware::from_fn(move |req, next| {
                rate_limit_middleware(req, next, limiter.clone())
            })
        });
    
    // Apply similar pattern to other route groups...
    
    // Combine routes
    let api_router = Router::new()
        .route("/health", get(api::v1::health::health_check))
        .nest("/api/v1/auth", auth_public_routes.merge(auth_me_route))
        .nest("/api/v1/models", models_routes)
        // ... rest of your routes ...
        ;
    
    // Apply global rate limiter to everything
    let router = api_router
        .fallback_service(ServeDir::new(static_path).append_index_html_on_directories(true))
        .layer({
            let limiter = state.global_rate_limiter.clone();
            axum::middleware::from_fn(move |req, next| {
                rate_limit_middleware(req, next, limiter.clone())
            })
        })
        .layer(TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
            )
        }))
        .layer(cors)
        .with_state(state);
    
    // ... rest of your setup ...
    
    Ok(router)
}
```

## Combined Example

Here's a complete minimal example showing both modules working together:

```rust
use axum::{routing::get, Router};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize OIDC
    let oidc_client = Arc::new(
        auth::OidcClient::new(
            std::env::var("OIDC_ISSUER_URL")?,
            std::env::var("OIDC_CLIENT_ID")?,
            std::env::var("OIDC_CLIENT_SECRET")?,
            std::env::var("OIDC_REDIRECT_URI")?,
        )
        .await?,
    );
    
    // Initialize rate limiters
    let global_limiter = Arc::new(RateLimiter::new(presets::global()));
    global_limiter.clone().start_cleanup_task();
    
    let login_limiter = Arc::new(RateLimiter::new(presets::login()));
    login_limiter.clone().start_cleanup_task();
    
    // Build state
    let state = AppState {
        oidc_client,
        global_rate_limiter: global_limiter,
        login_rate_limiter: login_limiter,
        // ... other fields
    };
    
    // Build routes with rate limiting
    let auth_routes = Router::new()
        .route("/login", get(login_handler))
        .layer({
            let limiter = state.login_rate_limiter.clone();
            axum::middleware::from_fn(move |req, next| {
                rate_limit_middleware(req, next, limiter.clone())
            })
        });
    
    let app = Router::new()
        .nest("/auth", auth_routes)
        .layer({
            let limiter = state.global_rate_limiter.clone();
            axum::middleware::from_fn(move |req, next| {
                rate_limit_middleware(req, next, limiter.clone())
            })
        })
        .with_state(state);
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

## Removing Old Dependencies

### Optional: Clean Up Cargo.toml

The new rate limiting implementation doesn't use `tower_governor` or `governor`. If you want to remove them (optional, won't hurt to keep):

```toml
# In Cargo.toml - you can optionally remove these lines:
# tower_governor = "0.4"
# governor = "0.6"
```

Then run:
```bash
cargo build
```

Note: Only remove these if they're not used elsewhere in your codebase!

### Testing the Integration

1. **Test OIDC Flow**:
   - Navigate to `/api/v1/auth/login`
   - Should redirect to your OIDC provider
   - After authentication, should redirect back with user info

2. **Test Rate Limiting**:
   ```bash
   # Should work for first few requests
   for i in {1..10}; do curl http://localhost:3000/api/v1/auth/login; done
   
   # Should return 429 Too Many Requests after limit exceeded
   ```

3. **Check Headers**:
   ```bash
   curl -I http://localhost:3000/api/v1/data
   
   # Should include:
   # X-RateLimit-Limit: 100
   # X-RateLimit-Remaining: 99
   # X-RateLimit-Reset: 1
   ```

## Troubleshooting

### Rate Limiting Not Working

- Ensure rate limiters are initialized and added to AppState
- Check that middleware is applied to routes (use .layer())
- Verify cleanup tasks are started
- Check logs for rate limiting events

### OIDC Errors

- Verify environment variables are set correctly
- Check provider's `.well-known/openid-configuration` is accessible
- Ensure redirect URI matches exactly in provider settings
- Review logs for detailed error messages

### Performance Issues

- Monitor memory usage of rate limiter
- Adjust cleanup interval if needed
- Consider Redis for distributed deployments
- Profile your application to identify bottlenecks

## Next Steps

1. Review the README files in `src/auth/README_OIDC.md` and `src/middleware/README_RATE_LIMIT.md`
2. Adjust rate limits based on your application's needs
3. Add monitoring and alerting for rate limit violations
4. Consider implementing user-specific rate limits for API keys
5. Test thoroughly in staging before deploying to production

## Support

For issues or questions:
- Check the module README files for detailed documentation
- Review unit tests for usage examples
- Enable debug logging to see detailed operation traces

