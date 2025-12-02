# Rate Limiting Middleware

This module provides a flexible, in-memory rate limiting solution for the server using a sliding window algorithm.

## Features

- **Sliding Window Algorithm**: More accurate than fixed windows
- **In-Memory Tracking**: Fast, no external dependencies required
- **Flexible Configuration**: Easy to customize limits and time windows
- **IP-Based Limiting**: Tracks requests per client IP address
- **Standard Headers**: Returns RFC-compliant rate limit headers
- **Automatic Cleanup**: Background task removes old tracking data
- **Multiple Extractors**: Supports X-Forwarded-For, X-Real-IP, and ConnectInfo
- **Async/Await**: Fully asynchronous with minimal overhead

## Quick Start

### Basic Usage

```rust
use crate::middleware::rate_limit::{RateLimiter, RateLimitConfig, rate_limit_middleware};
use std::sync::Arc;

// Create a rate limiter: 100 requests per second
let config = RateLimitConfig::per_second(100);
let limiter = Arc::new(RateLimiter::new(config));

// Start cleanup task (optional but recommended)
limiter.clone().start_cleanup_task();

// Apply to router
let app = Router::new()
    .route("/api/data", get(handler))
    .layer(axum::middleware::from_fn(move |req, next| {
        rate_limit_middleware(req, next, limiter.clone())
    }));
```

## Configuration Options

### Pre-Defined Presets

```rust
use crate::middleware::rate_limit::presets;

// Global rate limit: 100 req/s
let global = presets::global();

// Authenticated users: 1000 req/s
let authenticated = presets::authenticated();

// Admin users: 5000 req/s
let admin = presets::admin();

// Login attempts: 5 per minute
let login = presets::login();

// API key creation: 10 per hour
let api_keys = presets::api_key_creation();

// Strict for sensitive ops: 10 per minute
let strict = presets::strict();
```

### Custom Configuration

```rust
use std::time::Duration;

// 50 requests per 30 seconds
let custom = RateLimitConfig::new(50, Duration::from_secs(30));

// Or use convenience methods
let per_sec = RateLimitConfig::per_second(100);
let per_min = RateLimitConfig::per_minute(60);
let per_hour = RateLimitConfig::per_hour(1000);
```

## Integration Examples

### Global Rate Limit

```rust
// Apply to all routes
let limiter = Arc::new(RateLimiter::new(presets::global()));
limiter.clone().start_cleanup_task();

let app = Router::new()
    .route("/api/v1/data", get(handler))
    .layer(axum::middleware::from_fn(move |req, next| {
        rate_limit_middleware(req, next, limiter.clone())
    }));
```

### Different Limits for Different Routes

```rust
// Login routes - strict limit
let login_limiter = Arc::new(RateLimiter::new(presets::login()));
login_limiter.clone().start_cleanup_task();

let login_routes = Router::new()
    .route("/api/v1/auth/login", post(login_handler))
    .layer(axum::middleware::from_fn(move |req, next| {
        rate_limit_middleware(req, next, login_limiter.clone())
    }));

// API routes - normal limit
let api_limiter = Arc::new(RateLimiter::new(presets::authenticated()));
api_limiter.clone().start_cleanup_task();

let api_routes = Router::new()
    .route("/api/v1/data", get(data_handler))
    .layer(axum::middleware::from_fn(move |req, next| {
        rate_limit_middleware(req, next, api_limiter.clone())
    }));

// Combine routes
let app = Router::new()
    .merge(login_routes)
    .merge(api_routes);
```

### Per-User Rate Limiting (Advanced)

For user-specific rate limits, you can extend the middleware to extract user ID from JWT:

```rust
// Custom rate limiting by user ID
pub async fn user_rate_limit_middleware(
    request: Request,
    next: Next,
    limiter: Arc<RateLimiter>,
) -> Result<Response, impl IntoResponse> {
    // Extract user ID from JWT or session
    let user_id = extract_user_id(&request).unwrap_or_else(|| "anonymous".to_string());
    
    // Check rate limit using user ID as key
    match limiter.check(&user_id).await {
        Ok(info) => {
            let mut response = next.run(request).await;
            // Add headers...
            Ok(response)
        }
        Err(exceeded) => Err(exceeded),
    }
}
```

## Response Headers

When rate limiting is active, the following headers are added to responses:

- `X-RateLimit-Limit`: Maximum number of requests allowed
- `X-RateLimit-Remaining`: Number of requests remaining in current window
- `X-RateLimit-Reset`: Seconds until the rate limit window resets

When the limit is exceeded (429 response):

- `Retry-After`: Seconds to wait before retrying
- Response body includes JSON with error details

## Error Response Format

```json
{
  "error": "Too many requests",
  "message": "Rate limit of 100 requests exceeded. Please retry after 45 seconds.",
  "retry_after": 45
}
```

## Key Improvements Over Previous Implementation

1. **Actually Works**: The previous implementation using `tower_governor` was defined but never used
2. **Simpler**: No need for complex governor configuration and key extractors
3. **More Flexible**: Easy to customize and extend for different use cases
4. **Better Testing**: Includes comprehensive unit tests
5. **Cleaner Integration**: Simple function-based middleware instead of layers
6. **Memory Management**: Automatic cleanup of old tracking data
7. **Better Errors**: Detailed error messages with retry information

## Algorithm Details

### Sliding Window

This implementation uses a **sliding window** algorithm:

1. Each request timestamp is stored
2. When checking limits, old requests outside the time window are removed
3. If remaining count < limit, request is allowed
4. Otherwise, request is rejected with retry-after information

**Advantages**:
- More accurate than fixed windows
- No "burst" problems at window boundaries
- Fair distribution of requests over time

**Memory Usage**:
- Each tracked IP stores up to `max_requests` timestamps
- Timestamps are ~8 bytes each
- Example: 100 req/s limit = 800 bytes per IP
- Cleanup task removes inactive IPs every 5 minutes

## Performance Considerations

- **Lock Contention**: Uses RwLock for minimal blocking
- **Memory**: Automatically cleans up inactive trackers
- **CPU**: Very lightweight - just timestamp comparisons
- **Scalability**: For high-traffic applications, consider using Redis-based rate limiting

## Testing

The module includes comprehensive unit tests:

```bash
cargo test --package server --lib middleware::rate_limit
```

Tests cover:
- Requests within limits
- Blocking excess requests
- Window reset behavior

## Production Recommendations

1. **Cleanup Task**: Always start the cleanup task to prevent memory leaks
2. **Monitoring**: Log rate limit violations for security monitoring
3. **Scaling**: For distributed systems, use Redis instead of in-memory
4. **Tuning**: Adjust limits based on your application's capacity
5. **Exemptions**: Consider exempting health checks and monitoring endpoints

## Future Enhancements

Possible improvements for future versions:

- Redis backend for distributed rate limiting
- Token bucket algorithm option
- Configurable key extractors (by API key, user ID, etc.)
- Rate limit burst allowances
- Per-endpoint configuration via attributes
- Metrics and Prometheus integration

