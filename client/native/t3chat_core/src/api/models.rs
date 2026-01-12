use crate::api::client::ApiClient;
use crate::domain::Error;
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

pub async fn list_models(client: &ApiClient) -> Result<Vec<Model>, Error> {
    let response = client
        .request(Method::GET, "/api/models")
        .await?
        .send()
        .await?;

    if !response.status().is_success() {
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            let _ = client.clear_token().await;
            return Err(Error::Auth("Unauthorized".to_string()));
        }
        return Err(Error::from(response.error_for_status().unwrap_err()));
    }

    let models: Vec<Model> = response.json().await?;
    Ok(models)
}

