use crate::api::client::ApiClient;
use crate::domain::{AuthConfig, Error, User, UserAndToken};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct LocalLoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalLoginResponse {
    token: String,
    #[serde(alias = "expires_at")]
    expires_at: i64,
    user: User,
}

fn redact_token_in_body(body: &str) -> String {
    // Best-effort: if body is JSON, redact `token` at the top level.
    // Otherwise, return a short snippet.
    if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(obj) = value.as_object_mut() {
            if obj.contains_key("token") {
                obj.insert(
                    "token".to_string(),
                    serde_json::Value::String("<redacted>".to_string()),
                );
            }
        }
        return value.to_string().chars().take(1000).collect();
    }

    body.chars().take(500).collect()
}

pub async fn login(
    client: &ApiClient,
    username: String,
    password: String,
) -> Result<UserAndToken, Error> {
    let request = LocalLoginRequest { username, password };
    let response = client
        .request_with_body(Method::POST, "/api/auth/local/login", &request)
        .await?
        .send()
        .await?;

    if !response.status().is_success() {
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::Auth("Invalid credentials".to_string()));
        }
        return Err(Error::from(response.error_for_status().unwrap_err()));
    }

    // Decode manually so we can surface the real serde error (missing field, wrong casing, etc.)
    // and show a redacted body snippet for debugging.
    let body = response.text().await?;
    let data: LocalLoginResponse = serde_json::from_str(&body).map_err(|e| {
        let redacted = redact_token_in_body(&body);
        Error::Unknown(format!(
            "Failed to decode login response JSON: {}. Body: {}",
            e, redacted
        ))
    })?;
    
    // Save token (persist + in-memory cache)
    client.set_token(&data.token).await?;

    Ok(UserAndToken {
        user: data.user,
        token: data.token,
        expires_at: data.expires_at,
    })
}

pub async fn get_current_user(client: &ApiClient) -> Result<User, Error> {
    let response = client
        .request(Method::GET, "/api/auth/me")
        .await?
        .send()
        .await?;

    if !response.status().is_success() {
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            // Clear invalid token
            let _ = client.clear_token().await;
            return Err(Error::Auth("Unauthorized".to_string()));
        }
        return Err(Error::from(response.error_for_status().unwrap_err()));
    }

    let body = response.text().await?;
    let user: User = serde_json::from_str(&body).map_err(|e| {
        Error::Unknown(format!("Failed to decode /api/auth/me JSON: {}. Body: {}", e, body))
    })?;
    Ok(user)
}

pub async fn get_auth_config(client: &ApiClient) -> Result<AuthConfig, Error> {
    let response = client
        .request(Method::GET, "/api/auth/config")
        .await?
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Error::from(response.error_for_status().unwrap_err()));
    }

    let body = response.text().await?;
    let config: AuthConfig = serde_json::from_str(&body).map_err(|e| {
        Error::Unknown(format!(
            "Failed to decode /api/auth/config JSON: {}. Body: {}",
            e, body
        ))
    })?;
    Ok(config)
}

pub async fn logout(client: &ApiClient) -> Result<(), Error> {
    // Try to call logout endpoint (may fail if already logged out)
    let _ = client
        .request(Method::POST, "/api/auth/logout")
        .await?
        .send()
        .await;

    // Always clear local token
    client.clear_token().await?;
    Ok(())
}

