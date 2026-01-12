use crate::api::client::ApiClient;
use crate::domain::{Chat, ChatListResponse, ChatWithMessages, Error};
use reqwest::Method;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub model_provider: String,
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
    // reqwest 0.13's RequestBuilder no longer exposes `.query(...)` in our build,
    // so we append the query string directly (safe here because values are numeric).
    let mut path = String::from("/api/chats");
    let mut parts: Vec<String> = Vec::new();
    if let Some(p) = page {
        parts.push(format!("page={}", p));
    }
    if let Some(ps) = page_size {
        parts.push(format!("pageSize={}", ps));
    }
    if !parts.is_empty() {
        path.push('?');
        path.push_str(&parts.join("&"));
    }

    let response = client
        .request(Method::GET, &path)
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
            let _ = client.clear_token().await;
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
            let _ = client.clear_token().await;
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
            let _ = client.clear_token().await;
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
            let _ = client.clear_token().await;
            return Err(Error::Auth("Unauthorized".to_string()));
        }
        return Err(Error::from(response.error_for_status().unwrap_err()));
    }

    Ok(())
}

