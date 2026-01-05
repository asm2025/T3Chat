use crate::api::client::ApiClient;
use crate::domain::{Chat, ChatListResponse, ChatWithMessages, Error};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct CreateChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "modelProvider")]
    pub model_provider: String,
    #[serde(rename = "modelId")]
    pub model_id: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

pub async fn list_chats(
    client: &ApiClient,
    page: Option<u64>,
    page_size: Option<u64>,
) -> Result<ChatListResponse, Error> {
    let mut query = Vec::new();
    if let Some(p) = page {
        query.push(("page", p.to_string()));
    }
    if let Some(ps) = page_size {
        query.push(("pageSize", ps.to_string()));
    }

    let mut req = client.request(Method::GET, "/api/chats").await?;
    if !query.is_empty() {
        req = req.query(&query);
    }

    let response = req.send().await?;

    if !response.status().is_success() {
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            let _ = client.token_storage().delete_token().await;
            return Err(Error::Auth("Unauthorized".to_string()));
        }
        return Err(Error::from(response.error_for_status().unwrap_err()));
    }

    let data: ChatListResponse = response.json().await?;
    Ok(data)
}

pub async fn create_chat(
    client: &ApiClient,
    request: CreateChatRequest,
) -> Result<Chat, Error> {
    let response = client
        .request_with_body(Method::POST, "/api/chats", &request)
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

    let chat: Chat = response.json().await?;
    Ok(chat)
}

pub async fn get_chat(client: &ApiClient, chat_id: &str) -> Result<ChatWithMessages, Error> {
    let response = client
        .request(Method::GET, &format!("/api/chats/{}", chat_id))
        .await?
        .send()
        .await?;

    if !response.status().is_success() {
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::NotFound(format!("Chat {} not found", chat_id)));
        }
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            let _ = client.token_storage().delete_token().await;
            return Err(Error::Auth("Unauthorized".to_string()));
        }
        return Err(Error::from(response.error_for_status().unwrap_err()));
    }

    let data: ChatWithMessages = response.json().await?;
    Ok(data)
}

pub async fn update_chat(
    client: &ApiClient,
    chat_id: &str,
    request: UpdateChatRequest,
) -> Result<Chat, Error> {
    let response = client
        .request_with_body(Method::PUT, &format!("/api/chats/{}", chat_id), &request)
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

    let chat: Chat = response.json().await?;
    Ok(chat)
}

pub async fn delete_chat(client: &ApiClient, chat_id: &str) -> Result<(), Error> {
    let response = client
        .request(Method::DELETE, &format!("/api/chats/{}", chat_id))
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

    Ok(())
}

