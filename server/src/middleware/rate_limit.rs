use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// Rate limit configuration
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed
    pub max_requests: u32,
    /// Time window for the rate limit
    pub window: Duration,
}

impl RateLimitConfig {
    /// Create a new rate limit configuration
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
        }
    }

    /// Per-second rate limit
    pub fn per_second(requests: u32) -> Self {
        Self::new(requests, Duration::from_secs(1))
    }

    /// Per-minute rate limit
    pub fn per_minute(requests: u32) -> Self {
        Self::new(requests, Duration::from_secs(60))
    }

    /// Per-hour rate limit
    pub fn per_hour(requests: u32) -> Self {
        Self::new(requests, Duration::from_secs(3600))
    }
}

/// Request tracker for an IP address
#[derive(Debug, Clone)]
struct RequestTracker {
    requests: Vec<Instant>,
}

impl RequestTracker {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    /// Check if a request should be allowed
    fn check_and_update(&mut self, config: &RateLimitConfig) -> bool {
        let now = Instant::now();
        let cutoff = now - config.window;

        // Remove old requests outside the time window
        self.requests.retain(|&timestamp| timestamp > cutoff);

        // Check if we're under the limit
        if self.requests.len() < config.max_requests as usize {
            self.requests.push(now);
            true
        } else {
            false
        }
    }

    /// Get remaining requests in current window
    fn remaining(&self, config: &RateLimitConfig) -> u32 {
        let now = Instant::now();
        let cutoff = now - config.window;
        let active_requests = self.requests.iter().filter(|&&t| t > cutoff).count();
        config.max_requests.saturating_sub(active_requests as u32)
    }

    /// Get time until next available request
    fn retry_after(&self, config: &RateLimitConfig) -> Option<Duration> {
        if let Some(&oldest) = self.requests.first() {
            let elapsed = oldest.elapsed();
            if elapsed < config.window {
                Some(config.window - elapsed)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// In-memory rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    trackers: Arc<RwLock<HashMap<String, RequestTracker>>>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            trackers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a request from the given key should be allowed
    async fn check(&self, key: &str) -> Result<RateLimitInfo, RateLimitExceeded> {
        let mut trackers = self.trackers.write().await;
        let tracker = trackers
            .entry(key.to_string())
            .or_insert_with(RequestTracker::new);

        if tracker.check_and_update(&self.config) {
            Ok(RateLimitInfo {
                limit: self.config.max_requests,
                remaining: tracker.remaining(&self.config),
                reset_in: self.config.window,
            })
        } else {
            let retry_after = tracker
                .retry_after(&self.config)
                .unwrap_or(self.config.window);
            Err(RateLimitExceeded {
                limit: self.config.max_requests,
                retry_after,
            })
        }
    }

    /// Start a background task to clean up old trackers
    pub fn start_cleanup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(300)).await; // Clean every 5 minutes

                let mut trackers = self.trackers.write().await;
                let now = Instant::now();
                let cutoff = now - self.config.window - Duration::from_secs(60);

                trackers.retain(|_, tracker| tracker.requests.iter().any(|&t| t > cutoff));

                tracing::debug!("Rate limiter cleanup: {} active trackers", trackers.len());
            }
        });
    }
}

/// Rate limit information returned in headers
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_in: Duration,
}

/// Rate limit exceeded error
#[derive(Debug, Clone)]
pub struct RateLimitExceeded {
    pub limit: u32,
    pub retry_after: Duration,
}

impl IntoResponse for RateLimitExceeded {
    fn into_response(self) -> Response {
        let retry_after_secs = self.retry_after.as_secs();

        let body = serde_json::json!({
            "error": "Too many requests",
            "message": format!("Rate limit of {} requests exceeded. Please retry after {} seconds.",
                self.limit, retry_after_secs),
            "retry_after": retry_after_secs,
        });

        (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("X-RateLimit-Limit", self.limit.to_string()),
                ("X-RateLimit-Remaining", "0".to_string()),
                ("Retry-After", retry_after_secs.to_string()),
                ("Content-Type", "application/json".to_string()),
            ],
            body.to_string(),
        )
            .into_response()
    }
}

/// Extract client IP address from request
fn get_client_ip(request: &Request) -> String {
    // Try to get IP from ConnectInfo first
    if let Some(ConnectInfo(addr)) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        return addr.ip().to_string();
    }

    // Fallback to X-Forwarded-For header
    if let Some(forwarded) = request.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip) = forwarded_str.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    // Fallback to X-Real-IP header
    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // Default fallback
    "unknown".to_string()
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
    limiter: Arc<RateLimiter>,
) -> Result<Response, impl IntoResponse> {
    let client_ip = get_client_ip(&request);

    match limiter.check(&client_ip).await {
        Ok(info) => {
            let mut response = next.run(request).await;
            let headers = response.headers_mut();

            headers.insert("X-RateLimit-Limit", info.limit.to_string().parse().unwrap());
            headers.insert(
                "X-RateLimit-Remaining",
                info.remaining.to_string().parse().unwrap(),
            );
            headers.insert(
                "X-RateLimit-Reset",
                info.reset_in.as_secs().to_string().parse().unwrap(),
            );

            Ok(response)
        }
        Err(exceeded) => Err(exceeded),
    }
}

/// Predefined rate limit configurations
pub mod presets {
    use super::*;

    /// Global rate limit: 100 requests per second
    pub fn global() -> RateLimitConfig {
        RateLimitConfig::per_second(100)
    }

    /// Authenticated users: 1000 requests per second
    pub fn authenticated() -> RateLimitConfig {
        RateLimitConfig::per_second(1000)
    }

    /// Admin users: 5000 requests per second
    pub fn admin() -> RateLimitConfig {
        RateLimitConfig::per_second(5000)
    }

    /// Login attempts: 5 per minute
    pub fn login() -> RateLimitConfig {
        RateLimitConfig::per_minute(5)
    }

    /// API key creation: 10 per hour
    pub fn api_key_creation() -> RateLimitConfig {
        RateLimitConfig::per_hour(10)
    }

    /// Strict rate limit for sensitive operations: 10 per minute
    pub fn strict() -> RateLimitConfig {
        RateLimitConfig::per_minute(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_requests_within_limit() {
        let config = RateLimitConfig::per_second(5);
        let limiter = RateLimiter::new(config);

        for _ in 0..5 {
            assert!(limiter.check("test-ip").await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_excess_requests() {
        let config = RateLimitConfig::per_second(3);
        let limiter = RateLimiter::new(config);

        // First 3 should succeed
        for _ in 0..3 {
            assert!(limiter.check("test-ip").await.is_ok());
        }

        // 4th should fail
        assert!(limiter.check("test-ip").await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_resets_after_window() {
        let config = RateLimitConfig::new(2, Duration::from_millis(100));
        let limiter = RateLimiter::new(config);

        // Use up the limit
        assert!(limiter.check("test-ip").await.is_ok());
        assert!(limiter.check("test-ip").await.is_ok());
        assert!(limiter.check("test-ip").await.is_err());

        // Wait for window to pass
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be allowed again
        assert!(limiter.check("test-ip").await.is_ok());
    }
}
