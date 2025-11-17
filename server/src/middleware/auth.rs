use crate::{db::prelude::*, db::repositories::TUserRepository, env::get_firebase_project_id};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{Method, Request, StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use emix::env::{get_allow_anonymous_users, is_development};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
struct FirebaseTokenPayload {
    sub: String,
    email: Option<String>,
    aud: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
    kty: String,
    alg: String,
}

#[derive(Clone)]
pub struct AuthenticatedUser(pub UserModel);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)
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
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    // Verify token
    let firebase_user = verify_firebase_token(auth_header)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    // Check if anonymous users are allowed
    let allow_anonymous = get_allow_anonymous_users();
    let is_anonymous = firebase_user.email.is_none();

    if !allow_anonymous && is_anonymous {
        return Err(StatusCode::FORBIDDEN);
    }

    // Try to find existing user first - only create if they don't exist
    let user = match state
        .user_repository
        .get(firebase_user.id.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        Some(existing_user) => {
            // User exists, use their existing data without overwriting
            existing_user
        }
        None => {
            // User doesn't exist, create new user
            let create_user_dto = CreateUserDto {
                id: firebase_user.id,
                email: firebase_user.email,
                display_name: None,
                image_url: None,
            };
            state
                .user_repository
                .upsert(create_user_dto)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };

    // Add user to request extensions
    request.extensions_mut().insert(AuthenticatedUser(user));

    Ok(next.run(request).await)
}

#[derive(Debug)]
struct FirebaseUser {
    id: String,
    email: Option<String>,
}

async fn verify_firebase_token(token: &str) -> Result<FirebaseUser, String> {
    let project_id = get_firebase_project_id().map_err(|e| e.to_string())?;

    if is_development() {
        // In development/emulator mode, decode token without verification
        verify_emulator_token(token, &project_id)
    } else {
        // In production, verify with JWKS
        verify_production_token(token, &project_id).await
    }
}

fn verify_emulator_token(token: &str, project_id: &str) -> Result<FirebaseUser, String> {
    // Decode base64 payload
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid token format".to_string());
    }

    // Decode payload (handle URL-safe base64)
    let payload_b64 = parts[1].replace('-', "+").replace('_', "/");
    // Add padding if needed
    let padding = (4 - payload_b64.len() % 4) % 4;
    let payload_b64_padded = format!("{}{}", payload_b64, "=".repeat(padding));
    let payload_bytes = STANDARD
        .decode(&payload_b64_padded)
        .map_err(|_| "Invalid token payload encoding".to_string())?;

    let payload: FirebaseTokenPayload = serde_json::from_slice(&payload_bytes)
        .map_err(|_| "Invalid token payload JSON".to_string())?;

    // Validate payload
    if payload.sub.is_empty() || payload.aud != project_id {
        return Err("Invalid token payload".to_string());
    }

    Ok(FirebaseUser {
        id: payload.sub,
        email: payload.email,
    })
}

async fn verify_production_token(token: &str, project_id: &str) -> Result<FirebaseUser, String> {
    tracing::debug!("Verifying production Firebase token");

    // Step 1: Decode token header to get the kid (key ID)
    let header = decode_header(token).map_err(|e| {
        tracing::error!("Failed to decode token header: {}", e);
        format!("Invalid token header: {}", e)
    })?;

    let kid = header.kid.ok_or_else(|| {
        tracing::error!("Token header missing 'kid' field");
        "Token missing kid field".to_string()
    })?;

    tracing::debug!("Token kid: {}", kid);

    // Step 2: Fetch JWKS from Google
    let jwks_url =
        "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";

    tracing::debug!("Fetching JWKS from Google");
    let jwks_response = reqwest::get(jwks_url).await.map_err(|e| {
        tracing::error!("Failed to fetch JWKS: {}", e);
        format!("Failed to fetch JWKS: {}", e)
    })?;

    let jwks: JwksResponse = jwks_response.json().await.map_err(|e| {
        tracing::error!("Failed to parse JWKS: {}", e);
        format!("Failed to parse JWKS: {}", e)
    })?;

    // Step 3: Find the matching key from JWKS
    let jwk = jwks.keys.iter().find(|key| key.kid == kid).ok_or_else(|| {
        tracing::error!("No matching key found for kid: {}", kid);
        format!("No matching key found for kid: {}", kid)
    })?;

    tracing::debug!("Found matching JWK for kid: {}", kid);

    // Step 4: Convert JWK to DecodingKey
    // The JWK contains n (modulus) and e (exponent) in base64url format
    let decoding_key = jwk_to_decoding_key(jwk).map_err(|e| {
        tracing::error!("Failed to convert JWK to DecodingKey: {}", e);
        format!("Failed to convert JWK to DecodingKey: {}", e)
    })?;

    // Step 5: Set up validation parameters
    let issuer = format!("https://securetoken.google.com/{}", project_id);
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[project_id]);

    // Step 6: Verify and decode the token
    let token_data =
        decode::<FirebaseTokenPayload>(token, &decoding_key, &validation).map_err(|e| {
            tracing::error!("Token verification failed: {}", e);
            format!("Token verification failed: {}", e)
        })?;

    tracing::debug!(
        "Token verified successfully for user: {}",
        token_data.claims.sub
    );

    Ok(FirebaseUser {
        id: token_data.claims.sub,
        email: token_data.claims.email,
    })
}

fn jwk_to_decoding_key(jwk: &Jwk) -> Result<DecodingKey, String> {
    // Validate that this is an RSA key
    if jwk.kty != "RSA" {
        return Err(format!("Unsupported key type: {}", jwk.kty));
    }

    // Create DecodingKey from RSA components (n and e are base64url-encoded)
    // The jsonwebtoken crate handles the base64 decoding internally
    DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|e| format!("Failed to create DecodingKey: {}", e))
}
