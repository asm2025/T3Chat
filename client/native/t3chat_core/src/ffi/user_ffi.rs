use crate::api::user;
use crate::ffi::auth_ffi::get_client;
use crate::ffi::types::{FfiError, FfiUser};
use flutter_rust_bridge::frb;

#[frb(sync)]
pub fn ffi_get_profile() -> Result<FfiUser, FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        user::get_profile(&client)
            .await
            .map(crate::ffi::types::FfiUser::from)
            .map_err(FfiError::from)
    })
}

#[frb(sync)]
pub fn ffi_update_profile(
    display_name: Option<String>,
    image_url: Option<String>,
) -> Result<FfiUser, FfiError> {
    let client = get_client()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| FfiError::Unknown(e.to_string()))?;
    rt.block_on(async {
        let request = user::UpdateUserRequest {
            display_name,
            image_url,
        };
        user::update_profile(&client, request)
            .await
            .map(crate::ffi::types::FfiUser::from)
            .map_err(FfiError::from)
    })
}

