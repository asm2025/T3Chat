use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

const MEILISEARCH_INDEX_NAME: &str = "chats_messages";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeiliDocument {
    pub id: String, // Composite: "chat:{uuid}" or "message:{uuid}"
    pub user_id: String,
    pub chat_id: String,
    pub r#type: String, // "chat" or "message"
    pub title: Option<String>,
    pub text: Option<String>,
    pub role: Option<String>,
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct MeiliSearchService {
    client: Arc<Mutex<Option<Arc<Client>>>>,
    index_name: String,
}

impl MeiliSearchService {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            index_name: MEILISEARCH_INDEX_NAME.to_string(),
        }
    }

    async fn get_client(&self) -> Option<Arc<Client>> {
        let mut client_guard = self.client.lock().await;
        if client_guard.is_none() {
            let meili_host = std::env::var("MEILI_HOST")
                .unwrap_or_else(|_| "http://localhost:7700".to_string());
            let meili_master_key = std::env::var("MEILI_MASTER_KEY").ok();

            match Client::new(meili_host, meili_master_key) {
                Ok(client) => {
                    *client_guard = Some(Arc::new(client));
                }
                Err(e) => {
                    tracing::error!("Failed to create MeiliSearch client: {:?}", e);
                    return None;
                }
            }
        }
        client_guard.clone()
    }

    pub async fn index_chat(&self, doc: MeiliDocument) -> Result<(), String> {
        let client = self.get_client().await.ok_or("MeiliSearch not configured")?;
        let index = client.index(&self.index_name);

        index
            .add_documents(&[doc], Some("id"))
            .await
            .map_err(|e| format!("Failed to index chat: {:?}", e))?;

        Ok(())
    }

    pub async fn index_message(&self, doc: MeiliDocument) -> Result<(), String> {
        let client = self.get_client().await.ok_or("MeiliSearch not configured")?;
        let index = client.index(&self.index_name);

        index
            .add_documents(&[doc], Some("id"))
            .await
            .map_err(|e| format!("Failed to index message: {:?}", e))?;

        Ok(())
    }

    pub async fn delete_chat(&self, chat_id: &str) -> Result<(), String> {
        let client = self.get_client().await.ok_or("MeiliSearch not configured")?;
        let index = client.index(&self.index_name);

        let doc_id = format!("chat:{}", chat_id);
        index
            .delete_document(&doc_id)
            .await
            .map_err(|e| format!("Failed to delete chat from index: {:?}", e))?;

        // Also delete all messages for this chat
        // Note: MeiliSearch doesn't support bulk delete by filter easily,
        // so we'd need to track message IDs or use a different approach
        // For now, we'll delete messages individually when they're deleted

        Ok(())
    }

    pub async fn delete_message(&self, message_id: &str) -> Result<(), String> {
        let client = self.get_client().await.ok_or("MeiliSearch not configured")?;
        let index = client.index(&self.index_name);

        let doc_id = format!("message:{}", message_id);
        index
            .delete_document(&doc_id)
            .await
            .map_err(|e| format!("Failed to delete message from index: {:?}", e))?;

        Ok(())
    }

    pub async fn ensure_index_exists(&self) -> Result<(), String> {
        let client = self.get_client().await.ok_or("MeiliSearch not configured")?;
        let index = client.index(&self.index_name);

        // Try to get index settings to check if it exists
        if let Err(e) = index.get_settings().await {
            if e.to_string().contains("index_not_found") {
                // Index doesn't exist, create it
                tracing::info!("Creating MeiliSearch index: {}", self.index_name);
            } else {
                return Err(format!("Failed to check index: {:?}", e));
            }
        }

        // Set searchable attributes
        index
            .set_searchable_attributes(&["title", "text"])
            .await
            .map_err(|e| format!("Failed to set searchable attributes: {:?}", e))?;

        // Set filterable attributes
        index
            .set_filterable_attributes(&["user_id", "chat_id", "type", "role", "model", "endpoint"])
            .await
            .map_err(|e| format!("Failed to set filterable attributes: {:?}", e))?;

        Ok(())
    }
}

impl Default for MeiliSearchService {
    fn default() -> Self {
        Self::new()
    }
}

