use crate::domain::Error;
use crate::storage::TokenStorage;
use reqwest::{Client, Method, RequestBuilder};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ApiClient {
    client: Client,
    base_url: String,
    token_storage: Arc<TokenStorage>,
    /// In-memory cache to avoid repeated keyring reads and to ensure the token is
    /// immediately available after login within the same process.
    token_cache: RwLock<Option<String>>,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token_storage: Arc::new(TokenStorage::new()),
            token_cache: RwLock::new(None),
        }
    }

    pub async fn request(&self, method: Method, path: &str) -> Result<RequestBuilder, Error> {
        let url = format!("{}{}", self.base_url, path);
        let mut builder = self.client.request(method, &url);

        // Add auth token if available
        if let Some(token) = self.token_cache.read().await.clone() {
            builder = builder.bearer_auth(token);
            return Ok(builder);
        }

        if let Some(token) = self.token_storage.get_token().await? {
            *self.token_cache.write().await = Some(token.clone());
            builder = builder.bearer_auth(token);
        }

        Ok(builder)
    }

    pub async fn request_with_body<T: serde::Serialize>(
        &self,
        method: Method,
        path: &str,
        body: &T,
    ) -> Result<RequestBuilder, Error> {
        let mut req = self.request(method, path).await?;
        req = req.json(body);
        Ok(req)
    }

    pub fn token_storage(&self) -> &Arc<TokenStorage> {
        &self.token_storage
    }

    pub async fn set_token(&self, token: &str) -> Result<(), Error> {
        self.token_storage.save_token(token).await?;
        *self.token_cache.write().await = Some(token.to_string());
        Ok(())
    }

    pub async fn clear_token(&self) -> Result<(), Error> {
        self.token_storage.delete_token().await?;
        *self.token_cache.write().await = None;
        Ok(())
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

