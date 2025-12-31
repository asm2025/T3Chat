// API module - all API handlers organized by version and hierarchy
pub mod admin;
pub mod auth;
pub mod chat;
pub mod chats;
pub mod common;
pub mod config;
pub mod errors;
pub mod features;
pub mod files;
pub mod health;
pub mod models;
pub mod user;
pub mod user_api_keys;

// Re-export commonly used types
pub use common::*;
