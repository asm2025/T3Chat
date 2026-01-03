use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize)]
struct RagUploadRequest {
    file_id: String,
    user_id: String,
    filepath: String,
    filename: String,
    mime_type: String,
}

#[derive(Debug, Deserialize)]
struct RagUploadResponse {
    success: bool,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct RagQueryRequest {
    query: String,
    file_ids: Vec<String>,
    entity_id: String,
    top_k: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct RagQueryResponse {
    results: Vec<Vec<(RagDocumentInfo, f64)>>,
}

#[derive(Debug, Deserialize)]
struct RagDocumentInfo {
    page_content: String,
    metadata: RagMetadata,
}

#[derive(Debug, Deserialize)]
struct RagMetadata {
    source: String,
    page: Option<usize>,
}

pub struct RagService {
    client: Arc<Mutex<Option<Client>>>,
    base_url: String,
}

impl RagService {
    pub fn new() -> Self {
        let base_url = std::env::var("RAG_API_URL")
            .unwrap_or_else(|_| "http://localhost:8000".to_string());
        
        Self {
            client: Arc::new(Mutex::new(None)),
            base_url,
        }
    }

    async fn get_client(&self) -> Client {
        let mut client_guard = self.client.lock().await;
        if client_guard.is_none() {
            *client_guard = Some(Client::new());
        }
        client_guard.as_ref().unwrap().clone()
    }

    /// Upload/ingest a file into RAG
    pub async fn ingest_file(
        &self,
        file_id: &str,
        user_id: &str,
        filepath: &str,
        filename: &str,
        mime_type: &str,
    ) -> Result<(), String> {
        let client = self.get_client().await;
        
        let url = format!("{}/upload", self.base_url);
        let payload = RagUploadRequest {
            file_id: file_id.to_string(),
            user_id: user_id.to_string(),
            filepath: filepath.to_string(),
            filename: filename.to_string(),
            mime_type: mime_type.to_string(),
        };

        let response = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send RAG upload request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("RAG API error ({}): {}", status, text));
        }

        let result: RagUploadResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse RAG response: {}", e))?;

        if !result.success {
            return Err(result.message.unwrap_or_else(|| "Unknown error".to_string()));
        }

        Ok(())
    }

    /// Query RAG for relevant file content
    pub async fn query_files(
        &self,
        query: &str,
        file_ids: Vec<String>,
        user_id: &str,
        top_k: Option<usize>,
    ) -> Result<Vec<(String, String, f64)>, String> {
        let client = self.get_client().await;
        
        let url = format!("{}/query", self.base_url);
        let payload = RagQueryRequest {
            query: query.to_string(),
            file_ids,
            entity_id: user_id.to_string(),
            top_k: top_k.or(Some(10)),
        };

        let response = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send RAG query request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("RAG API error ({}): {}", status, text));
        }

        let result: RagQueryResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse RAG response: {}", e))?;

        // Flatten results: Vec<Vec<(doc, distance)>> -> Vec<(filename, content, relevance)>
        let mut formatted: Vec<(String, String, f64)> = Vec::new();
        for file_results in result.results {
            for (doc, distance) in file_results {
                let filename = doc.metadata.source.split('/').last().unwrap_or("unknown").to_string();
                let relevance = 1.0 - distance;
                formatted.push((filename, doc.page_content, relevance));
            }
        }

        // Sort by relevance (descending)
        formatted.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        Ok(formatted)
    }

    /// Delete a file from RAG index
    pub async fn delete_file(&self, file_id: &str, user_id: &str) -> Result<(), String> {
        let client = self.get_client().await;
        
        let url = format!("{}/delete/{}", self.base_url, file_id);
        
        let response = client
            .delete(&url)
            .header("X-User-Id", user_id)
            .send()
            .await
            .map_err(|e| format!("Failed to send RAG delete request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("RAG API error ({}): {}", status, text));
        }

        Ok(())
    }
}

impl Default for RagService {
    fn default() -> Self {
        Self::new()
    }
}

