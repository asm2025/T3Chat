use crate::{
    db::prelude::*,
    db::repositories::{TUserRepository, UserRepository},
};
use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{request::Parts, Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::{future::Future, sync::Arc};

use crate::auth::SessionManager;
use crate::AppState;

#[derive(Clone)]
pub struct AuthenticatedUser(pub UserModel);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let user = parts.extensions.get::<AuthenticatedUser>().cloned();

        async move { user.ok_or(StatusCode::UNAUTHORIZED) }
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip authentication for OPTIONS requests (CORS preflight)
    if request.method() == Method::OPTIONS {
        return Ok(next.run(request).await);
    }

    // Extract token from Authorization header
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Try to verify as session token first (our JWT)
    let user_id = match verify_session_token(token).await {
        Ok(claims) => claims.sub,
        Err(_) => {
            // If session token fails, try OIDC token (if configured)
            if let (Some(_), Some(jwks_cache)) = (&state.oidc_client, &state.jwks_cache) {
                match verify_oidc_token(token, jwks_cache).await {
                    Ok(claims) => claims.sub,
                    Err(_) => return Err(StatusCode::UNAUTHORIZED),
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    };

    // Get user from database
    let user_repo = UserRepository::new(state.db.clone());
    let user = user_repo
        .get(user_id.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if account is disabled (use model fields directly)
    if user.disabled {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if account is locked (use model fields directly)
    if user.locked_out {
        if let Some(lockout_end) = user.lockout_end {
            if lockout_end > chrono::Utc::now() {
                return Err(StatusCode::FORBIDDEN);
            } else {
                // Lockout expired, unlock account
                let _ = user_repo.unlock_user(&user_id).await;
            }
        } else {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Update last login and reset failed login count
    let _ = user_repo.update_last_login(&user_id).await;
    let _ = user_repo.reset_failed_login(&user_id).await;

    // Add user to request extensions
    request.extensions_mut().insert(AuthenticatedUser(user));

    Ok(next.run(request).await)
}

async fn verify_session_token(
    token: &str,
) -> Result<crate::auth::session::SessionClaims, StatusCode> {
    let session_manager = SessionManager::new(
        crate::env::get_jwt_secret().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        crate::env::get_jwt_expiry_seconds(),
    );

    session_manager
        .verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

async fn verify_oidc_token(
    token: &str,
    jwks_cache: &Arc<crate::auth::JwksCache>,
) -> Result<crate::auth::jwt::UserClaims, StatusCode> {
    // Use cached JWKS from AppState
    jwks_cache
        .verify_token(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)
}
