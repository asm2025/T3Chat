use anyhow::Result;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, warn};

#[derive(Clone)]
pub struct JwksCache {
    keys: Arc<RwLock<HashMap<String, DecodingKey>>>,
    issuer: String,
    jwks_url: String,
    last_refresh: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub aud: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
    kty: String,
    alg: String,
}

impl JwksCache {
    /// Create a cache that pulls JWKS keys from the provider's advertised JWKS URI.
    /// The issuer is still tracked so we can validate the `iss` claim during verification.
    pub async fn new(issuer: String, jwks_url: String) -> Result<Self> {
        let cache = Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            issuer,
            jwks_url,
            last_refresh: Arc::new(RwLock::new(None)),
        };

        cache.refresh_keys().await?;
        Ok(cache)
    }

    pub async fn refresh_keys(&self) -> Result<()> {
        tracing::debug!("Fetching JWKS from {}", self.jwks_url);

        let response = reqwest::get(&self.jwks_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch JWKS: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch JWKS: HTTP {}",
                response.status()
            ));
        }

        let jwks: JwksResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse JWKS: {}", e))?;

        let mut keys = self.keys.write().await;
        keys.clear();

        for jwk in jwks.keys {
            if jwk.kty != "RSA" {
                warn!("Skipping non-RSA key: {}", jwk.kid);
                continue;
            }

            match DecodingKey::from_rsa_components(&jwk.n, &jwk.e) {
                Ok(decoding_key) => {
                    keys.insert(jwk.kid.clone(), decoding_key);
                    tracing::debug!("Cached JWK for kid: {}", jwk.kid);
                }
                Err(e) => {
                    error!("Failed to create DecodingKey for kid {}: {}", jwk.kid, e);
                }
            }
        }

        *self.last_refresh.write().await = Some(chrono::Utc::now());
        tracing::info!("Refreshed JWKS cache with {} keys", keys.len());

        Ok(())
    }

    pub async fn verify_token(&self, token: &str) -> Result<UserClaims> {
        // Decode header to get kid
        let header = decode_header(token)
            .map_err(|e| anyhow::anyhow!("Failed to decode token header: {}", e))?;

        let kid = header
            .kid
            .ok_or_else(|| anyhow::anyhow!("Token header missing 'kid' field"))?;

        // Check if we need to refresh keys (refresh every hour)
        let should_refresh = {
            let last_refresh = self.last_refresh.read().await;
            last_refresh
                .map(|lr| chrono::Utc::now() - lr > chrono::Duration::hours(1))
                .unwrap_or(true)
        };

        if should_refresh {
            // Try to refresh, but don't fail if it doesn't work
            if let Err(e) = self.refresh_keys().await {
                warn!("Failed to refresh JWKS: {}", e);
            }
        }

        // Get the key
        let keys = self.keys.read().await;
        let decoding_key = keys
            .get(&kid)
            .ok_or_else(|| anyhow::anyhow!("No key found for kid: {}", kid))?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        // Note: audience validation should be set based on your OIDC client ID
        // For now, we'll skip audience validation or make it configurable

        // Verify and decode token
        let token_data = decode::<UserClaims>(token, decoding_key, &validation)
            .map_err(|e| anyhow::anyhow!("Token verification failed: {}", e))?;

        Ok(token_data.claims)
    }
}
