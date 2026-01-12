// This module is used as the single `rust_input` entrypoint for flutter_rust_bridge codegen.
//
// IMPORTANT:
// flutter_rust_bridge codegen does not reliably pick up `pub use` re-exports as public APIs.
// Therefore, we define thin wrapper functions *in this module* and delegate to the actual
// implementations in `auth_ffi.rs`, `chat_ffi.rs`, and `user_ffi.rs`.

use flutter_rust_bridge::frb;

use crate::ffi::auth_ffi;
use crate::ffi::chat_ffi;
use crate::ffi::types::{
    FfiAuthConfig, FfiChat, FfiChatChunk, FfiChatListResponse, FfiChatWithMessages, FfiError,
    FfiUser, FfiUserAndToken,
};
use crate::ffi::user_ffi;

// -----------------------
// Auth
// -----------------------

#[frb(sync)]
pub fn init(base_url: String) -> Result<(), FfiError> {
    auth_ffi::ffi_init(base_url)
}

#[frb(sync)]
pub fn get_auth_config() -> Result<FfiAuthConfig, FfiError> {
    auth_ffi::ffi_get_auth_config()
}

#[frb(sync)]
pub fn login(username: String, password: String) -> Result<FfiUserAndToken, FfiError> {
    auth_ffi::ffi_login(username, password)
}

#[frb(sync)]
pub fn set_token(token: String) -> Result<(), FfiError> {
    auth_ffi::ffi_set_token(token)
}

#[frb(sync)]
pub fn get_current_user() -> Result<FfiUser, FfiError> {
    auth_ffi::ffi_get_current_user()
}

#[frb(sync)]
pub fn logout() -> Result<(), FfiError> {
    auth_ffi::ffi_logout()
}

// -----------------------
// User
// -----------------------

#[frb(sync)]
pub fn get_profile() -> Result<FfiUser, FfiError> {
    user_ffi::ffi_get_profile()
}

#[frb(sync)]
pub fn update_profile(
    display_name: Option<String>,
    image_url: Option<String>,
) -> Result<FfiUser, FfiError> {
    user_ffi::ffi_update_profile(display_name, image_url)
}

// -----------------------
// Chat
// -----------------------

#[frb(sync)]
pub fn list_chats(
    page: Option<u64>,
    page_size: Option<u64>,
) -> Result<FfiChatListResponse, FfiError> {
    chat_ffi::ffi_list_chats(page, page_size)
}

#[frb(sync)]
pub fn create_chat(
    title: Option<String>,
    model_provider: String,
    model_id: String,
) -> Result<FfiChat, FfiError> {
    chat_ffi::ffi_create_chat(title, model_provider, model_id)
}

#[frb(sync)]
pub fn get_chat(chat_id: String) -> Result<FfiChatWithMessages, FfiError> {
    chat_ffi::ffi_get_chat(chat_id)
}

#[frb(sync)]
pub fn update_chat(chat_id: String, title: Option<String>) -> Result<FfiChat, FfiError> {
    chat_ffi::ffi_update_chat(chat_id, title)
}

#[frb(sync)]
pub fn delete_chat(chat_id: String) -> Result<(), FfiError> {
    chat_ffi::ffi_delete_chat(chat_id)
}

#[frb(sync)]
pub fn stream_chat(
    chat_id: String,
    message: String,
    model_provider: String,
    model_id: String,
) -> Result<Vec<FfiChatChunk>, FfiError> {
    chat_ffi::ffi_stream_chat(chat_id, message, model_provider, model_id)
}

