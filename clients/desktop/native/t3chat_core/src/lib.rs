// This file is the entry point for the Rust library
// It exports the FFI bridge and public API

pub mod api;
pub mod domain;
pub mod ffi;
pub mod storage;

// Re-export for convenience
pub use api::client::ApiClient;
pub use domain::errors::Error;
pub use domain::models::*;

// Re-export FFI functions for flutter_rust_bridge
pub use ffi::bridge::*;

