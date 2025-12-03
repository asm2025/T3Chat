use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

/// OIDC Provider Discovery Document
#[derive(Debug, Clone, Deserialize)]
struct ProviderMetadata {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    jwks_uri: String,
    #[serde(default)]
    scopes_supported: Vec<String>,
}

/// OIDC Client for OpenID Connect authentication
#[derive(Debug, Clone)]
pub struct OidcClient {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    metadata: Arc<RwLock<Option<ProviderMetadata>>>,
    issuer_url: String,
    http_client: Client,
}

/// Token response from OIDC provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponseData {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
}

/// User information from OIDC provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,
}

impl OidcClient {
    /// Create a new OIDC client
    pub async fn new(
        issuer_url: String,
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let client = Self {
            client_id,
            client_secret,
            redirect_uri,
            metadata: Arc::new(RwLock::new(None)),
            issuer_url,
            http_client,
        };

        // Initialize metadata
        client.discover_metadata().await?;

        Ok(client)
    }

    /// Discover OIDC provider metadata
    async fn discover_metadata(&self) -> Result<()> {
        let discovery_url = format!(
            "{}/.well-known/openid-configuration",
            self.issuer_url.trim_end_matches('/')
        );

        tracing::debug!("Discovering OIDC metadata from: {}", discovery_url);

        let metadata: ProviderMetadata = self
            .http_client
            .get(&discovery_url)
            .send()
            .await
            .context("Failed to fetch provider metadata")?
            .json()
            .await
            .context("Failed to parse provider metadata")?;

        tracing::info!(
            "OIDC metadata discovered: issuer={}, auth_endpoint={}",
            metadata.issuer,
            metadata.authorization_endpoint
        );

        *self.metadata.write().await = Some(metadata);

        Ok(())
    }

    /// Get the authorization URL for the OIDC flow
    pub fn get_authorization_url(&self) -> (Url, String) {
        // Generate state token
        let state = generate_random_token(32);

        // Build authorization URL
        let metadata = self.metadata.blocking_read();
        let metadata = metadata.as_ref().expect("Metadata not initialized");

        let mut url = Url::parse(&metadata.authorization_endpoint).expect("Invalid auth endpoint");

        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", "openid email profile")
            .append_pair("state", &state);

        (url, state)
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(&self, code: String, _state: String) -> Result<TokenResponseData> {
        let metadata = self.metadata.read().await;
        let metadata = metadata.as_ref().context("Metadata not initialized")?;

        let params = [
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &self.redirect_uri),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        tracing::debug!("Exchanging authorization code for tokens");

        let response = self
            .http_client
            .post(&metadata.token_endpoint)
            .form(&params)
            .send()
            .await
            .context("Failed to exchange code for tokens")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Token exchange failed: {}", error_text);
            anyhow::bail!("Token exchange failed: {}", error_text);
        }

        let token_response: TokenResponseData = response
            .json()
            .await
            .context("Failed to parse token response")?;

        tracing::info!("Successfully exchanged code for tokens");

        Ok(token_response)
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, refresh_token: String) -> Result<TokenResponseData> {
        let metadata = self.metadata.read().await;
        let metadata = metadata.as_ref().context("Metadata not initialized")?;

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh_token),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        tracing::debug!("Refreshing access token");

        let response = self
            .http_client
            .post(&metadata.token_endpoint)
            .form(&params)
            .send()
            .await
            .context("Failed to refresh token")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Token refresh failed: {}", error_text);
            anyhow::bail!("Token refresh failed: {}", error_text);
        }

        let token_response: TokenResponseData = response
            .json()
            .await
            .context("Failed to parse token refresh response")?;

        tracing::info!("Successfully refreshed access token");

        Ok(token_response)
    }

    /// Get user information using access token
    pub async fn get_user_info(&self, access_token: String) -> Result<UserInfo> {
        let metadata = self.metadata.read().await;
        let metadata = metadata.as_ref().context("Metadata not initialized")?;

        tracing::debug!("Fetching user info from provider");

        let response = self
            .http_client
            .get(&metadata.userinfo_endpoint)
            .bearer_auth(&access_token)
            .send()
            .await
            .context("Failed to fetch user info")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("User info fetch failed: {}", error_text);
            anyhow::bail!("User info fetch failed: {}", error_text);
        }

        let user_info: UserInfo = response.json().await.context("Failed to parse user info")?;

        tracing::info!("Successfully fetched user info for sub: {}", user_info.sub);

        Ok(user_info)
    }

    /// Get the JWKS URI from metadata
    pub async fn get_jwks_uri(&self) -> Result<String> {
        let metadata = self.metadata.read().await;
        let metadata = metadata.as_ref().context("Metadata not initialized")?;
        Ok(metadata.jwks_uri.clone())
    }
}

/// Generate a cryptographically secure random token
fn generate_random_token(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();

    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_token() {
        let token1 = generate_random_token(32);
        let token2 = generate_random_token(32);

        assert_eq!(token1.len(), 32);
        assert_eq!(token2.len(), 32);
        assert_ne!(token1, token2); // Should be different
    }
}
