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
struct LocalLoginResponse {
    token: String,
    #[serde(rename = "expiresAt")]
    expires_at: i64,
    user: User,
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

    let data: LocalLoginResponse = response.json().await?;
    
    // Save token
    client.token_storage().save_token(&data.token).await?;

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
            let _ = client.token_storage().delete_token().await;
            return Err(Error::Auth("Unauthorized".to_string()));
        }
        return Err(Error::from(response.error_for_status().unwrap_err()));
    }

    let user: User = response.json().await?;
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

    let config: AuthConfig = response.json().await?;
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
    client.token_storage().delete_token().await?;
    Ok(())
}

