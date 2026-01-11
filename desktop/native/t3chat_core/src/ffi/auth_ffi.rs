use crate::api::{auth, client::ApiClient};
use crate::ffi::types::{FfiAuthConfig, FfiError, FfiUser, FfiUserAndToken};
use flutter_rust_bridge::frb;
use std::sync::Arc;
use tokio::sync::RwLock;

// Global API client instance
static API_CLIENT: std::sync::OnceLock<Arc<RwLock<Option<Arc<ApiClient>>>>> =
    std::sync::OnceLock::new();

fn get_client() -> Result<Arc<ApiClient>, FfiError> {
    let client_store = API_CLIENT.get_or_init(|| Arc::new(RwLock::new(None)));
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        let guard = client_store.read().await;
        guard.clone().ok_or_else(|| FfiError::Unknown("Client not initialized".to_string()))
    })
}

#[frb(sync)]
pub fn ffi_init(base_url: String) -> Result<(), FfiError> {
    let client = Arc::new(ApiClient::new(base_url));
    let client_store = API_CLIENT.get_or_init(|| Arc::new(RwLock::new(None)));
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        let mut guard = client_store.write().await;
        *guard = Some(client);
        Ok(())
    })
}

#[frb(sync)]
pub fn ffi_login(username: String, password: String) -> Result<FfiUserAndToken, FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        let result = auth::login(&client, username, password).await;
        result.map(|ut| FfiUserAndToken {
            user: FfiUser::from(ut.user),
            token: ut.token,
            expires_at: ut.expires_at,
        })
        .map_err(FfiError::from)
    })
}

#[frb(sync)]
pub fn ffi_get_current_user() -> Result<FfiUser, FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        auth::get_current_user(&client)
            .await
            .map(FfiUser::from)
            .map_err(FfiError::from)
    })
}

#[frb(sync)]
pub fn ffi_get_auth_config() -> Result<FfiAuthConfig, FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        auth::get_auth_config(&client)
            .await
            .map(|config| FfiAuthConfig {
                oidc_enabled: config.oidc_enabled,
            })
            .map_err(FfiError::from)
    })
}

#[frb(sync)]
pub fn ffi_logout() -> Result<(), FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async { auth::logout(&client).await.map_err(FfiError::from) })
}

