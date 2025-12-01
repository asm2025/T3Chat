use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// API error response format
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Application error types
#[derive(Debug)]
pub enum AppError {
    // Authentication errors
    Unauthorized(String),
    Forbidden(String),

    // Validation errors
    BadRequest(String),
    InvalidInput(String, Option<serde_json::Value>),

    // Resource errors
    NotFound(String),
    Conflict(String),

    // Provider errors
    ProviderError { provider: String, message: String },
    ProviderUnavailable(String),
    InvalidApiKey(String),

    // Database errors
    DatabaseError(String),
    
    // Rate limiting
    RateLimitExceeded { retry_after: Option<u64> },

    // Generic errors
    InternalError(String),
    ServiceUnavailable(String),
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::InvalidInput(_, _) => "INVALID_INPUT",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::ProviderError { .. } => "PROVIDER_ERROR",
            AppError::ProviderUnavailable(_) => "PROVIDER_UNAVAILABLE",
            AppError::InvalidApiKey(_) => "INVALID_API_KEY",
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            AppError::InternalError(_) => "INTERNAL_ERROR",
            AppError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) | AppError::InvalidInput(_, _) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::ProviderError { .. } => StatusCode::BAD_GATEWAY,
            AppError::ProviderUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::InvalidApiKey(_) => StatusCode::BAD_REQUEST,
            AppError::DatabaseError(_) | AppError::InternalError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    pub fn message(&self) -> String {
        match self {
            AppError::Unauthorized(msg) => msg.clone(),
            AppError::Forbidden(msg) => msg.clone(),
            AppError::BadRequest(msg) => msg.clone(),
            AppError::InvalidInput(msg, _) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::Conflict(msg) => msg.clone(),
            AppError::ProviderError { provider, message } => {
                format!("{} error: {}", provider, message)
            }
            AppError::ProviderUnavailable(msg) => msg.clone(),
            AppError::InvalidApiKey(msg) => msg.clone(),
            AppError::DatabaseError(msg) => format!("Database error: {}", msg),
            AppError::RateLimitExceeded { retry_after } => {
                if let Some(seconds) = retry_after {
                    format!("Rate limit exceeded. Retry after {} seconds", seconds)
                } else {
                    "Rate limit exceeded".to_string()
                }
            }
            AppError::InternalError(msg) => msg.clone(),
            AppError::ServiceUnavailable(msg) => msg.clone(),
        }
    }

    pub fn details(&self) -> Option<serde_json::Value> {
        match self {
            AppError::InvalidInput(_, details) => details.clone(),
            AppError::RateLimitExceeded { retry_after } => retry_after.map(|seconds| {
                serde_json::json!({
                    "retry_after": seconds
                })
            }),
            _ => None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        let message = self.message();
        let details = self.details();

        // Log errors (except client errors like 400/401/403/404)
        if status.is_server_error() {
            tracing::error!(
                error_code = error_code,
                status = %status,
                message = %message,
                "API error occurred"
            );
        } else if status == StatusCode::TOO_MANY_REQUESTS {
            tracing::warn!(
                error_code = error_code,
                status = %status,
                message = %message,
                "Rate limit exceeded"
            );
        }

        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: error_code.to_string(),
                message,
                details,
            },
        };

        let mut response = Json(error_response).into_response();
        *response.status_mut() = status;

        // Add retry-after header for rate limiting
        if let AppError::RateLimitExceeded {
            retry_after: Some(seconds),
        } = self
        {
            if let Ok(header_value) = axum::http::HeaderValue::from_str(&seconds.to_string()) {
                response
                    .headers_mut()
                    .insert(axum::http::header::RETRY_AFTER, header_value);
            }
        }

        response
    }
}

/// Convert anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

/// Convert diesel errors to AppError
impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => AppError::NotFound("Resource not found".to_string()),
            diesel::result::Error::DatabaseError(kind, info) => {
                AppError::DatabaseError(format!("{:?}: {:?}", kind, info))
            }
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

/// Validation helper functions
pub mod validation {
    use super::*;

    /// Validate model parameters
    pub fn validate_model_parameters(
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        top_p: Option<f32>,
        top_k: Option<u32>,
    ) -> Result<(), AppError> {
        if let Some(temp) = temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(AppError::InvalidInput(
                    "Temperature must be between 0.0 and 2.0".to_string(),
                    Some(serde_json::json!({ "field": "temperature", "value": temp })),
                ));
            }
        }

        if let Some(tokens) = max_tokens {
            if tokens == 0 || tokens > 1_000_000 {
                return Err(AppError::InvalidInput(
                    "Max tokens must be between 1 and 1,000,000".to_string(),
                    Some(serde_json::json!({ "field": "max_tokens", "value": tokens })),
                ));
            }
        }

        if let Some(p) = top_p {
            if !(0.0..=1.0).contains(&p) {
                return Err(AppError::InvalidInput(
                    "Top P must be between 0.0 and 1.0".to_string(),
                    Some(serde_json::json!({ "field": "top_p", "value": p })),
                ));
            }
        }

        if let Some(k) = top_k {
            if k == 0 || k > 10000 {
                return Err(AppError::InvalidInput(
                    "Top K must be between 1 and 10,000".to_string(),
                    Some(serde_json::json!({ "field": "top_k", "value": k })),
                ));
            }
        }

        Ok(())
    }

    /// Validate message content
    pub fn validate_message(message: &str) -> Result<(), AppError> {
        if message.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "Message cannot be empty".to_string(),
                None,
            ));
        }

        if message.len() > 1_000_000 {
            return Err(AppError::InvalidInput(
                "Message too long (max 1,000,000 characters)".to_string(),
                Some(serde_json::json!({ "length": message.len() })),
            ));
        }

        Ok(())
    }

    /// Validate provider name
    pub fn validate_provider(provider: &str) -> Result<(), AppError> {
        const VALID_PROVIDERS: &[&str] = &["openai", "anthropic", "google", "deepseek", "ollama"];

        if !VALID_PROVIDERS.contains(&provider) {
            return Err(AppError::InvalidInput(
                format!("Invalid provider: {}", provider),
                Some(serde_json::json!({
                    "valid_providers": VALID_PROVIDERS
                })),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(
            AppError::Unauthorized("test".to_string()).error_code(),
            "UNAUTHORIZED"
        );
        assert_eq!(
            AppError::NotFound("test".to_string()).error_code(),
            "NOT_FOUND"
        );
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(
            AppError::Unauthorized("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn test_validation() {
        use validation::*;

        // Valid parameters
        assert!(validate_model_parameters(Some(0.7), Some(1000), Some(0.9), Some(40)).is_ok());

        // Invalid temperature
        assert!(validate_model_parameters(Some(3.0), None, None, None).is_err());

        // Invalid top_p
        assert!(validate_model_parameters(None, None, Some(1.5), None).is_err());

        // Valid message
        assert!(validate_message("Hello world").is_ok());

        // Empty message
        assert!(validate_message("").is_err());
        assert!(validate_message("   ").is_err());
    }
}

