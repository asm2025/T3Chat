use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Json, Redirect},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing;

use crate::AppState;
use crate::auth::SessionManager;
use crate::db::models::CreateUserDto;
use crate::db::repositories::{TUserRepository, UserRepository};
use crate::env;
use crate::middleware::auth::AuthenticatedUser;
use crate::utils::password::verify_password;

// GET /api/v1/auth/login (OIDC)
pub async fn oidc_login(State(state): State<AppState>) -> Result<Redirect, StatusCode> {
    let oidc_client = state.oidc_client.as_ref().ok_or(StatusCode::NOT_FOUND)?;

    let (auth_url, _state_token) = oidc_client.get_authorization_url();

    // TODO: Store state_token in session/cache for verification
    // For now, we'll verify it in the callback

    Ok(Redirect::to(auth_url.as_str()))
}

// GET /api/v1/auth/callback?code=...&state=...
#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

pub async fn callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackQuery>,
) -> Result<Redirect, StatusCode> {
    let oidc_client = state.oidc_client.as_ref().ok_or(StatusCode::NOT_FOUND)?;

    // TODO: Verify state token from session/cache

    // Exchange code for tokens using cached OIDC client
    let token_response = oidc_client
        .exchange_code(params.code, params.state)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Get user info using cached OIDC client
    let user_info = oidc_client
        .get_user_info(token_response.access_token.clone())
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Create or update user in database
    let user_repo = UserRepository::new(state.db.clone());

    // Normalize email
    let _normalized_email = user_info.email.to_lowercase();

    let user = match user_repo
        .get(user_info.sub.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        Some(u) => {
            // Update existing user
            // TODO: Update user info if needed
            u
        }
        None => {
            // Create new user
            let create_user_dto = CreateUserDto {
                id: user_info.sub.clone(),
                email: user_info.email.clone(),
                email_verified: user_info.email_verified,
                name: user_info.name.clone(),
                username: None,
                avatar_url: user_info.picture.clone(),
                provider: Some("oidc".to_string()),
            };

            // Convert to NewUser for creation
            let new_user: crate::db::models::NewUser = create_user_dto.into();

            user_repo
                .create(new_user)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };

    // Generate session token (JWT)
    let session_manager = SessionManager::new(
        crate::env::get_jwt_secret().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        crate::env::get_jwt_expiry_seconds(),
    );

    let session_token = session_manager
        .generate_token(&user)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Redirect to frontend with token
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    Ok(Redirect::to(&format!(
        "{}/auth/callback?token={}",
        frontend_url, session_token
    )))
}

// POST /api/v1/auth/local/login
#[derive(Deserialize)]
pub struct LocalLoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LocalLoginResponse {
    token: String,
    expires_at: i64,
    user: UserResponse,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LocalLoginRequest>,
) -> Result<Json<LocalLoginResponse>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    // Find user by username or email
    let user = user_repo
        .get_by_username_or_email(&req.username)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, username = %req.username, "Failed to find user by username or email");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if account is disabled
    if user.disabled {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if account is locked
    if user.locked_out {
        if let Some(lockout_end) = user.lockout_end {
            if lockout_end > chrono::Utc::now() {
                return Err(StatusCode::FORBIDDEN);
            } else {
                // Lockout expired, unlock account
                let _ = user_repo.unlock_user(&user.id).await;
            }
        } else {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Check if user has a password hash (local auth enabled)
    let password_hash = user
        .password_hash
        .as_ref()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify password
    let password_valid = verify_password(&req.password, password_hash).map_err(|e| {
        tracing::error!(error = ?e, user_id = %user.id, "Failed to verify password");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !password_valid {
        // Increment failed login count
        let _ = user_repo.increment_failed_login(&user.id).await;
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Password is correct - reset failed login count and update last login
    let _ = user_repo.reset_failed_login(&user.id).await;
    let _ = user_repo.update_last_login(&user.id).await;

    // Generate session token (JWT)
    let jwt_secret = crate::env::get_jwt_secret().map_err(|e| {
        tracing::error!(error = ?e, "Failed to get JWT secret");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let expiry_seconds = crate::env::get_jwt_expiry_seconds();
    let session_manager = SessionManager::new(jwt_secret, expiry_seconds);

    let session_token = session_manager.generate_token(&user).map_err(|e| {
        tracing::error!(error = ?e, user_id = %user.id, "Failed to generate session token");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let expires_at = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + expiry_seconds) as i64;

    // Get user roles
    let roles = user_repo.get_user_roles(&user.id).await.map_err(|e| {
        tracing::error!(error = ?e, user_id = %user.id, "Failed to get user roles");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(LocalLoginResponse {
        token: session_token,
        expires_at,
        user: UserResponse {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified,
            name: user.name,
            username: user.username,
            avatar_url: user.avatar_url,
            roles,
        },
    }))
}

// POST /api/v1/auth/logout
#[derive(Serialize)]
pub struct LogoutResponse {
    success: bool,
}

pub async fn logout() -> Result<Json<LogoutResponse>, StatusCode> {
    // Invalidate session/token (client-side for JWT)
    Ok(Json(LogoutResponse { success: true }))
}

// POST /api/v1/auth/refresh
#[derive(Deserialize)]
pub struct RefreshRequest {
    refresh_token: String,
}

#[derive(Serialize)]
pub struct RefreshResponse {
    access_token: String,
    expires_in: Option<u64>,
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, StatusCode> {
    let oidc_client = state.oidc_client.as_ref().ok_or(StatusCode::NOT_FOUND)?;

    let token_response = oidc_client
        .refresh_token(req.refresh_token)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(RefreshResponse {
        access_token: token_response.access_token,
        expires_in: token_response.expires_in,
    }))
}

// GET /api/v1/auth/me
#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub roles: Vec<String>,
}

pub async fn me(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<UserResponse>, StatusCode> {
    let user_repo = UserRepository::new(state.db.clone());

    // Get user roles
    let roles = user_repo
        .get_user_roles(&user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        email_verified: user.email_verified,
        name: user.name,
        username: user.username,
        avatar_url: user.avatar_url,
        roles,
    }))
}

// GET /api/v1/auth/config
#[derive(Serialize)]
pub struct AuthConfigResponse {
    pub oidc_enabled: bool,
}

pub async fn get_auth_config() -> Json<AuthConfigResponse> {
    Json(AuthConfigResponse {
        oidc_enabled: env::is_oidc_configured(),
    })
}

pub fn router() -> Router<AppState> {
    let mut router = Router::new()
        .route("/local/login", post(login))
        .route("/logout", post(logout))
        .route("/config", get(get_auth_config));

    // Only register OIDC routes if OIDC is configured
    if env::is_oidc_configured() {
        router = router
            .route("/login", get(oidc_login))
            .route("/callback", get(callback))
            .route("/refresh", post(refresh));
    }

    router
}
