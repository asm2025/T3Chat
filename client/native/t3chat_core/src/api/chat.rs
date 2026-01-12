use crate::api::client::ApiClient;
use crate::domain::{ChatChunk, ChatCompletionResponse, Error};
use futures::Stream;
use reqwest::Method;
use serde::Serialize;
use std::pin::Pin;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub chat_id: String,
    pub message: String,
    pub model_provider: String,
    pub model_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_flags: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
    pub stream: bool,
}

const STREAM_END_MARKER: &str = "\u{0000}\u{0001}\u{0002}";

pub async fn send_message(
    client: &ApiClient,
    request: ChatRequest,
) -> Result<ChatCompletionResponse, Error> {
    let mut req = request;
    req.stream = false;

    let response = client
        .request_with_body(Method::POST, "/api/chat", &req)
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

    let completion: ChatCompletionResponse = response.json().await?;
    Ok(completion)
}

pub async fn stream_message(
    client: &ApiClient,
    request: ChatRequest,
) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Error>> + Send>>, Error> {
    let mut req = request;
    req.stream = true;

    let response = client
        .request_with_body(Method::POST, "/api/chat/stream", &req)
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

    // Parse SSE stream - collect all data first, then parse
    // Note: For true streaming, we'd need to use a more complex async stream parser
    // This simplified version collects the response and parses it
    let full_text = response.text().await?;
    
    // Parse SSE format: lines starting with "data: "
    let chunks: Vec<Result<ChatChunk, Error>> = full_text
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || !line.starts_with("data: ") {
                return None;
            }

            let data = line.strip_prefix("data: ")?;
            if data == "[DONE]" || data == STREAM_END_MARKER {
                return Some(Ok(ChatChunk {
                    delta: String::new(),
                    done: true,
                }));
            }

            match serde_json::from_str::<serde_json::Value>(data) {
                Ok(json) => {
                    let delta = json["delta"].as_str().unwrap_or("").to_string();
                    let done = json["done"].as_bool().unwrap_or(false);
                    Some(Ok(ChatChunk { delta, done }))
                }
                Err(_) => None, // Skip invalid JSON
            }
        })
        .collect();

    // Convert to stream
    use futures::stream;
    Ok(Box::pin(stream::iter(chunks)))
}

