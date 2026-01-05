use crate::api::client::ApiClient;
use crate::domain::{Error, User};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct UpdateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "imageUrl")]
    pub image_url: Option<String>,
}

pub async fn get_profile(client: &ApiClient) -> Result<User, Error> {
    let response = client
        .request(Method::GET, "/api/me")
        .await?
        .send()
        .await?;

    if !response.status().is_success() {
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            let _ = client.token_storage().delete_token().await;
            return Err(Error::Auth("Unauthorized".to_string()));
        }
        return Err(Error::from(response.error_for_status().unwrap_err()));
    }

    let user: User = response.json().await?;
    Ok(user)
}

pub async fn update_profile(
    client: &ApiClient,
    request: UpdateUserRequest,
) -> Result<User, Error> {
    let response = client
        .request_with_body(Method::PUT, "/api/me", &request)
        .await?
        .send()
        .await?;

    if !response.status().is_success() {
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            let _ = client.token_storage().delete_token().await;
            return Err(Error::Auth("Unauthorized".to_string()));
        }
        return Err(Error::from(response.error_for_status().unwrap_err()));
    }

    let user: User = response.json().await?;
    Ok(user)
}

