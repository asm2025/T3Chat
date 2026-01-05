use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Error::Network("Request timeout".to_string())
        } else if err.is_connect() {
            Error::Network("Connection failed".to_string())
        } else if err.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
            Error::Auth("Unauthorized".to_string())
        } else if err.status() == Some(reqwest::StatusCode::FORBIDDEN) {
            Error::Auth("Forbidden".to_string())
        } else if err.status() == Some(reqwest::StatusCode::NOT_FOUND) {
            Error::NotFound("Resource not found".to_string())
        } else if err.status().is_some() {
            Error::Server(format!("HTTP error: {}", err.status().unwrap()))
        } else {
            Error::Network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Unknown(format!("JSON error: {}", err))
    }
}

impl From<keyring::Error> for Error {
    fn from(err: keyring::Error) -> Self {
        Error::Storage(err.to_string())
    }
}

