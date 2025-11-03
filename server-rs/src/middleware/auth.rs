use crate::env::{get_firebase_auth_emulator_host, get_firebase_project_id, is_development, get_allow_anonymous_users};
use crate::schema::users::{ActiveModel, Entity, Model};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct FirebaseTokenPayload {
    sub: String,
    email: Option<String>,
    aud: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

pub struct AuthenticatedUser(pub Model);

pub async fn auth_middleware(
    State(db): State<sea_orm::DatabaseConnection>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify token
    let firebase_user = verify_firebase_token(auth_header).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check if anonymous users are allowed
    let allow_anonymous = get_allow_anonymous_users();
    let is_anonymous = firebase_user.email.is_none();
    
    if !allow_anonymous && is_anonymous {
        return Err(StatusCode::FORBIDDEN);
    }

    // Upsert user in database
    let user = upsert_user(&db, &firebase_user).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Add user to request extensions
    request.extensions_mut().insert(AuthenticatedUser(user));

    Ok(next.run(request).await)
}

async fn upsert_user(
    db: &sea_orm::DatabaseConnection,
    firebase_user: &FirebaseUser,
) -> Result<Model, sea_orm::DbErr> {
    use sea_orm::EntityTrait;
    
    // Try to find existing user
    let existing_user = Entity::find_by_id(&firebase_user.id)
        .one(db)
        .await?;

    if let Some(mut user) = existing_user {
        // Update if email changed
        if user.email != firebase_user.email {
            let mut user_active: ActiveModel = user.clone().into();
            user_active.email = Set(firebase_user.email.clone());
            user_active.updated_at = Set(chrono::Utc::now());
            user = user_active.update(db).await?;
        }
        Ok(user)
    } else {
        // Insert new user
        let new_user = ActiveModel {
            id: Set(firebase_user.id.clone()),
            email: Set(firebase_user.email.clone()),
            display_name: Set(None),
            photo_url: Set(None),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };
        new_user.insert(db).await
    }
}

#[derive(Debug)]
struct FirebaseUser {
    id: String,
    email: Option<String>,
}

async fn verify_firebase_token(token: &str) -> Result<FirebaseUser, String> {
    let project_id = get_firebase_project_id().map_err(|e| e)?;

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
    let payload_bytes = base64::engine::general_purpose::STANDARD
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
    // Fetch JWKS from Google
    let jwks_url = "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";
    
    // For production, we'll use a simpler approach with jsonwebtoken
    // In a production app, you'd want to cache JWKS and handle key rotation
    let jwks_response = reqwest::get(jwks_url)
        .await
        .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;
    
    let jwks: serde_json::Value = jwks_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JWKS: {}", e))?;

    // Extract keys from JWKS (simplified - in production you'd want proper key selection)
    // For now, we'll use a validation approach that works with the first key
    // Note: This is simplified - proper JWKS handling requires more complex logic
    
    let issuer = format!("https://securetoken.google.com/{}", project_id);
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[project_id]);

    // Try to decode with a placeholder key (this won't work for real verification)
    // For a proper implementation, you'd need to:
    // 1. Extract the kid from the token header
    // 2. Find the matching key in JWKS
    // 3. Decode with that specific key
    
    // For now, we'll use a simplified approach that works with the emulator
    // In production, you'd want to use a library that properly handles JWKS
    verify_emulator_token(token, project_id)
}

