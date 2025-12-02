# T3Chat Developer Implementation Guide

> **Purpose**: Actionable development tasks separated by frontend and backend for parallel development.

## Quick Start

- **Backend Developers**: Start with [Backend Section](#backend-development)
- **Frontend Developers**: Start with [Frontend Section](#frontend-development)
- **Database Developers**: Start with [Database Schema](#database-schema)

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Database Schema](#database-schema)
- [Backend Development](#backend-development)
- [Frontend Development](#frontend-development)
- [API Contract](#api-contract)
- [Testing Checklist](#testing-checklist)

---

## Prerequisites

### Required Environment Variables

Add these to your `.env` file (backend developers):

```bash
# OIDC Provider Configuration
OIDC_ISSUER_URL=https://your-provider.com
OIDC_CLIENT_ID=your-client-id
OIDC_CLIENT_SECRET=your-client-secret
OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback

# JWT Configuration
JWT_SECRET=your-jwt-secret-for-local-tokens
JWT_EXPIRY_SECONDS=3600

# Rate Limiting Configuration
RATE_LIMIT_GLOBAL_PER_MINUTE=100
RATE_LIMIT_AUTHENTICATED_PER_MINUTE=1000
RATE_LIMIT_ADMIN_PER_MINUTE=5000
RATE_LIMIT_LOGIN_ATTEMPTS=5
RATE_LIMIT_LOGIN_WINDOW_MINUTES=15

# Database (existing)
DATABASE_URL=postgresql://...
```

### Required Dependencies

**Backend** (`server/Cargo.toml`):
```toml
oauth2 = "4"
openidconnect = "2"
tower-governor = "0"
```

**Frontend** (`ui/package.json`):
```json
{
  "dependencies": {
    // Existing dependencies
    // No new dependencies needed initially
  }
}
```

---

## Database Schema

### Migration File

**File**: `migrations/YYYYMMDDHHMMSS_initial_schema/up.sql`

**Task**: Create initial schema migration with all tables.

```sql
-- Users Table (ASP.NET Identity Style)
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    normalized_email TEXT NOT NULL UNIQUE,
    name TEXT,
    username TEXT UNIQUE,
    normalized_username TEXT UNIQUE,
    avatar_url TEXT,
    provider TEXT NOT NULL DEFAULT 'oidc',
    
    -- Account Status
    disabled BOOLEAN NOT NULL DEFAULT false,
    locked_out BOOLEAN NOT NULL DEFAULT false,
    lockout_end TIMESTAMPTZ,
    access_failed_count INTEGER NOT NULL DEFAULT 0,
    
    -- Password (for local auth)
    password_hash TEXT,
    password_changed_at TIMESTAMPTZ,
    
    -- Two-Factor Authentication
    two_factor_enabled BOOLEAN NOT NULL DEFAULT false,
    totp_secret TEXT,
    
    -- Login Tracking
    last_login_at TIMESTAMPTZ,
    login_count INTEGER NOT NULL DEFAULT 0,
    
    -- User Preferences
    preferences JSONB,
    
    -- Terms Acceptance
    terms_accepted BOOLEAN,
    terms_accepted_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_normalized_email ON users(normalized_email);
CREATE INDEX idx_users_disabled ON users(disabled);
CREATE INDEX idx_users_locked_out ON users(locked_out);
CREATE INDEX idx_users_provider ON users(provider);

-- Roles Table
CREATE TABLE roles (
    name TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO roles (name, display_name, description) VALUES
    ('user', 'User', 'Standard user role'),
    ('admin', 'Administrator', 'Full system access'),
    ('moderator', 'Moderator', 'Content moderation access');

-- User Roles Table
CREATE TABLE user_roles (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_name TEXT NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by TEXT REFERENCES users(id),
    PRIMARY KEY (user_id, role_name)
);

CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_name ON user_roles(role_name);

-- AI Providers Table
CREATE TABLE ai_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    base_url TEXT,
    website_url TEXT,
    documentation_url TEXT,
    disabled BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    requires_api_key BOOLEAN NOT NULL DEFAULT true,
    supports_streaming BOOLEAN NOT NULL DEFAULT true,
    supports_images BOOLEAN NOT NULL DEFAULT false,
    supports_functions BOOLEAN NOT NULL DEFAULT false,
    supports_vision BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_providers_provider_id ON ai_providers(provider_id);
CREATE INDEX idx_ai_providers_disabled ON ai_providers(disabled);
CREATE INDEX idx_ai_providers_is_active ON ai_providers(is_active);

-- AI Models Table
CREATE TABLE ai_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES ai_providers(id) ON DELETE CASCADE,
    model_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT,
    
    -- Capabilities
    context_window INTEGER NOT NULL,
    max_output_tokens INTEGER,
    supports_streaming BOOLEAN NOT NULL DEFAULT true,
    supports_images BOOLEAN NOT NULL DEFAULT false,
    supports_functions BOOLEAN NOT NULL DEFAULT false,
    supports_vision BOOLEAN NOT NULL DEFAULT false,
    
    -- Pricing
    cost_per_input_token NUMERIC,
    cost_per_output_token NUMERIC,
    is_paid BOOLEAN NOT NULL DEFAULT true,
    
    -- Status
    disabled BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    deprecated_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(provider_id, model_id)
);

CREATE INDEX idx_ai_models_provider_id ON ai_models(provider_id);
CREATE INDEX idx_ai_models_disabled ON ai_models(disabled);
CREATE INDEX idx_ai_models_is_active ON ai_models(is_active);
```

**Rollback File**: `migrations/YYYYMMDDHHMMSS_initial_schema/down.sql`

```sql
DROP TABLE IF EXISTS ai_models;
DROP TABLE IF EXISTS ai_providers;
DROP TABLE IF EXISTS user_roles;
DROP TABLE IF EXISTS roles;
DROP TABLE IF EXISTS users;
```

**Checklist**:
- [ ] Create migration directory
- [ ] Write `up.sql` with all tables
- [ ] Write `down.sql` rollback script
- [ ] Test migration up
- [ ] Test migration down
- [ ] Verify indexes are created
- [ ] Verify foreign keys are created
- [ ] Verify default roles are inserted

---

## Backend Development

### Phase 1: Authentication System

#### Task 1.1: Install Dependencies

**File**: `server/Cargo.toml`

```toml
[dependencies]
oauth2 = "4"
openidconnect = "2"
tower-governor = "0"
```

**Checklist**:
- [ ] Add dependencies to `Cargo.toml`
- [ ] Run `cargo build` to verify dependencies resolve

---

#### Task 1.2: Create Auth Module Structure

**Files to Create**:
- `server/src/auth/mod.rs`
- `server/src/auth/oidc.rs`
- `server/src/auth/jwt.rs`
- `server/src/auth/session.rs`

**File**: `server/src/auth/mod.rs`

```rust
pub mod oidc;
pub mod jwt;
pub mod session;

pub use oidc::OidcClient;
pub use jwt::verify_token;
pub use session::SessionManager;
```

**Checklist**:
- [ ] Create `auth/mod.rs`
- [ ] Create `auth/oidc.rs` (empty for now)
- [ ] Create `auth/jwt.rs` (empty for now)
- [ ] Create `auth/session.rs` (empty for now)
- [ ] Update `server/src/main.rs` to include `mod auth;`

---

#### Task 1.3: Implement OIDC Client

**File**: `server/src/auth/oidc.rs`

**Implementation Steps**:

1. **Create OIDC Client Struct**:
```rust
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    AuthorizationCode, ClientId, ClientSecret, IssuerUrl, RedirectUrl,
};

pub struct OidcClient {
    client: CoreClient,
    issuer_url: IssuerUrl,
}

impl OidcClient {
    pub async fn new(
        issuer_url: String,
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let issuer = IssuerUrl::new(issuer_url)?;
        let provider_metadata = CoreProviderMetadata::discover(&issuer, async_http_client).await?;
        
        let client = CoreClient::from_provider_metadata(provider_metadata, ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_redirect_uri(RedirectUrl::new(redirect_uri)?);
        
        Ok(Self {
            client,
            issuer_url: issuer,
        })
    }
    
    pub fn get_authorization_url(&self) -> (url::Url, String) {
        // Generate authorization URL and state
        // Return (auth_url, state)
        todo!()
    }
    
    pub async fn exchange_code(&self, code: String, state: String) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        // Exchange authorization code for tokens
        todo!()
    }
    
    pub async fn refresh_token(&self, refresh_token: String) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        // Refresh access token
        todo!()
    }
    
    pub async fn get_user_info(&self, access_token: String) -> Result<UserInfo, Box<dyn std::error::Error>> {
        // Fetch user info from UserInfo endpoint
        todo!()
    }
}
```

2. **Implement Authorization Flow**:
   - Generate authorization URL with PKCE
   - Store state in session/cache
   - Handle callback with code exchange

3. **Implement Token Exchange**:
   - Exchange authorization code for tokens
   - Store refresh token securely
   - Return access token

**Checklist**:
- [ ] Implement `OidcClient::new()` with discovery
- [ ] Implement `get_authorization_url()` with PKCE
- [ ] Implement `exchange_code()` for token exchange
- [ ] Implement `refresh_token()` for token refresh
- [ ] Implement `get_user_info()` for user info fetching
- [ ] Add error handling
- [ ] Add logging

---

#### Task 1.4: Implement JWT Verification

**File**: `server/src/auth/jwt.rs`

**Implementation Steps**:

1. **JWKS Caching**:
```rust
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct JwksCache {
    keys: Arc<RwLock<HashMap<String, DecodingKey>>>,
    issuer: String,
}

impl JwksCache {
    pub async fn new(issuer: String) -> Result<Self, Box<dyn std::error::Error>> {
        let cache = Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            issuer,
        };
        cache.refresh_keys().await?;
        Ok(cache)
    }
    
    pub async fn refresh_keys(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch JWKS from issuer/.well-known/jwks.json
        // Parse and cache keys
        todo!()
    }
    
    pub async fn verify_token(&self, token: &str) -> Result<UserClaims, Box<dyn std::error::Error>> {
        // Decode token header to get kid
        // Find key in cache
        // Verify token signature
        // Extract claims
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub aud: String,
}
```

2. **Token Verification Logic**:
   - Fetch JWKS from issuer
   - Cache JWKS keys
   - Verify token signature
   - Validate expiration, issuer, audience
   - Extract user claims

**Checklist**:
- [ ] Implement JWKS fetching
- [ ] Implement JWKS caching with TTL
- [ ] Implement token verification
- [ ] Implement claims extraction
- [ ] Add validation for exp, iss, aud
- [ ] Add error handling
- [ ] Add logging

---

#### Task 1.5: Create User Models

**Files to Create**:
- `server/src/db/models/user.rs`
- `server/src/db/models/user_role.rs`
- `server/src/db/models/role.rs`

**File**: `server/src/db/models/user.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub email_verified: bool,
    pub normalized_email: String,
    pub name: Option<String>,
    pub username: Option<String>,
    pub normalized_username: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: String,
    
    // Account Status
    pub disabled: bool,
    pub locked_out: bool,
    pub lockout_end: Option<DateTime<Utc>>,
    pub access_failed_count: i32,
    
    // Password
    pub password_hash: Option<String>,
    pub password_changed_at: Option<DateTime<Utc>>,
    
    // 2FA
    pub two_factor_enabled: bool,
    pub totp_secret: Option<String>,
    
    // Login Tracking
    pub last_login_at: Option<DateTime<Utc>>,
    pub login_count: i32,
    
    // Preferences
    pub preferences: Option<serde_json::Value>,
    
    // Terms
    pub terms_accepted: Option<bool>,
    pub terms_accepted_at: Option<DateTime<Utc>>,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub id: String,
    pub email: String,
    pub normalized_email: String,
    pub name: Option<String>,
    pub provider: String,
}
```

**File**: `server/src/db/models/role.rs`

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

**File**: `server/src/db/models/user_role.rs`

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRole {
    pub user_id: String,
    pub role_name: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub assigned_by: Option<String>,
}
```

**Checklist**:
- [ ] Create `User` struct with all fields
- [ ] Create `NewUser` struct
- [ ] Create `Role` struct
- [ ] Create `UserRole` struct
- [ ] Add `FromRow` derives
- [ ] Add `Serialize`/`Deserialize` derives
- [ ] Update `db/models/mod.rs` to export new models

---

#### Task 1.6: Create User Repository

**File**: `server/src/db/repositories/user_repository.rs`

**Required Methods**:

```rust
use crate::db::models::user::{User, NewUser};
use sqlx::PgPool;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    // Basic CRUD
    pub async fn create(&self, new_user: NewUser) -> Result<User, sqlx::Error> {
        todo!()
    }
    
    pub async fn get_by_id(&self, id: &str) -> Result<Option<User>, sqlx::Error> {
        todo!()
    }
    
    pub async fn get_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        // Case-insensitive lookup using normalized_email
        todo!()
    }
    
    pub async fn update(&self, user: &User) -> Result<User, sqlx::Error> {
        todo!()
    }
    
    // Account Management
    pub async fn enable_user(&self, user_id: &str) -> Result<(), sqlx::Error> {
        // Set disabled = false
        todo!()
    }
    
    pub async fn disable_user(&self, user_id: &str) -> Result<(), sqlx::Error> {
        // Set disabled = true
        todo!()
    }
    
    pub async fn lock_user(&self, user_id: &str, duration_minutes: i32) -> Result<(), sqlx::Error> {
        // Set locked_out = true, lockout_end = now + duration
        todo!()
    }
    
    pub async fn unlock_user(&self, user_id: &str) -> Result<(), sqlx::Error> {
        // Set locked_out = false, lockout_end = NULL
        todo!()
    }
    
    // Login Tracking
    pub async fn increment_failed_login(&self, user_id: &str) -> Result<(), sqlx::Error> {
        // Increment access_failed_count
        // Lock if >= MAX_FAILED_ATTEMPTS (5)
        todo!()
    }
    
    pub async fn reset_failed_login(&self, user_id: &str) -> Result<(), sqlx::Error> {
        // Set access_failed_count = 0
        todo!()
    }
    
    pub async fn update_last_login(&self, user_id: &str) -> Result<(), sqlx::Error> {
        // Set last_login_at = now(), increment login_count
        todo!()
    }
    
    // Role Management
    pub async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>, sqlx::Error> {
        todo!()
    }
    
    pub async fn add_role(&self, user_id: &str, role_name: &str, assigned_by: Option<&str>) -> Result<(), sqlx::Error> {
        todo!()
    }
    
    pub async fn remove_role(&self, user_id: &str, role_name: &str) -> Result<(), sqlx::Error> {
        todo!()
    }
    
    pub async fn has_role(&self, user_id: &str, role_name: &str) -> Result<bool, sqlx::Error> {
        todo!()
    }
}
```

**Checklist**:
- [ ] Implement `create()` method
- [ ] Implement `get_by_id()` method
- [ ] Implement `get_by_email()` with case-insensitive lookup
- [ ] Implement `update()` method
- [ ] Implement `enable_user()` / `disable_user()`
- [ ] Implement `lock_user()` / `unlock_user()`
- [ ] Implement `increment_failed_login()` with auto-lock
- [ ] Implement `reset_failed_login()`
- [ ] Implement `update_last_login()`
- [ ] Implement role management methods
- [ ] Add error handling
- [ ] Add unit tests

---

#### Task 1.7: Update Auth Middleware

**File**: `server/src/middleware/auth.rs`

**Implementation**:

```rust
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use crate::auth::jwt::JwksCache;
use crate::db::repositories::user_repository::UserRepository;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Extract token from Authorization header
    let auth_header = request.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));
    
    let token = match auth_header {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // 2. Verify JWT token
    let claims = state.jwks_cache.verify_token(token).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // 3. Get user from database
    let user_repo = UserRepository::new(state.db_pool.clone());
    let user = user_repo.get_by_id(&claims.sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // 4. Check if account is disabled
    if user.disabled {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // 5. Check if account is locked
    if user.locked_out {
        if let Some(lockout_end) = user.lockout_end {
            if lockout_end > chrono::Utc::now() {
                return Err(StatusCode::FORBIDDEN);
            }
            // Lockout expired, unlock account
            user_repo.unlock_user(&user.id).await.ok();
        }
    }
    
    // 6. Update last login
    user_repo.update_last_login(&user.id).await.ok();
    user_repo.reset_failed_login(&user.id).await.ok();
    
    // 7. Add user to request extensions
    request.extensions_mut().insert(user);
    
    // 8. Continue to next handler
    Ok(next.run(request).await)
}
```

**Checklist**:
- [ ] Extract token from Authorization header
- [ ] Verify JWT token using JWKS cache
- [ ] Fetch user from database
- [ ] Check `disabled` field (return 403 if true)
- [ ] Check `locked_out` and `lockout_end` (return 403 if locked)
- [ ] Update `last_login_at` and `login_count` on success
- [ ] Reset `access_failed_count` on success
- [ ] Add user to request extensions
- [ ] Handle errors appropriately
- [ ] Add logging

---

#### Task 1.8: Create Auth Routes

**File**: `server/src/api/v1/auth/mod.rs`

**Endpoints to Implement**:

```rust
use axum::{
    extract::{Query, State},
    response::{Redirect, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

// GET /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
) -> Result<Redirect, StatusCode> {
    let (auth_url, state_token) = state.oidc_client.get_authorization_url();
    // Store state_token in session/cache
    Ok(Redirect::to(auth_url.as_str()))
}

// GET /api/v1/auth/callback?code=...&state=...
#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

pub async fn callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackQuery>,
) -> Result<Redirect, StatusCode> {
    // 1. Verify state token
    // 2. Exchange code for tokens
    let token_response = state.oidc_client.exchange_code(params.code, params.state).await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // 3. Get user info
    let user_info = state.oidc_client.get_user_info(token_response.access_token).await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // 4. Create or update user in database
    let user_repo = UserRepository::new(state.db_pool.clone());
    let user = match user_repo.get_by_id(&user_info.sub).await? {
        Some(u) => u,
        None => {
            // Create new user
            let new_user = NewUser {
                id: user_info.sub.clone(),
                email: user_info.email.clone(),
                normalized_email: user_info.email.to_lowercase(),
                name: user_info.name.clone(),
                provider: "oidc".to_string(),
            };
            user_repo.create(new_user).await?
        }
    };
    
    // 5. Generate session token (JWT)
    let session_token = generate_session_token(&user)?;
    
    // 6. Redirect to frontend with token
    Ok(Redirect::to(&format!("http://localhost:5173/auth/callback?token={}", session_token)))
}

// POST /api/v1/auth/logout
pub async fn logout() -> Result<Json<LogoutResponse>, StatusCode> {
    // Invalidate session/token
    Ok(Json(LogoutResponse { success: true }))
}

// POST /api/v1/auth/refresh
#[derive(Deserialize)]
pub struct RefreshRequest {
    refresh_token: String,
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, StatusCode> {
    let token_response = state.oidc_client.refresh_token(req.refresh_token).await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok(Json(RefreshResponse {
        access_token: token_response.access_token,
        expires_in: token_response.expires_in,
    }))
}

// GET /api/v1/auth/me
pub async fn me(
    // Extract user from request extensions (set by auth middleware)
) -> Result<Json<UserResponse>, StatusCode> {
    // Return current user info
    todo!()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh))
        .route("/me", get(me))
}
```

**Checklist**:
- [ ] Implement `login()` endpoint (redirect to OIDC)
- [ ] Implement `callback()` endpoint (handle OIDC callback)
- [ ] Implement `logout()` endpoint
- [ ] Implement `refresh()` endpoint
- [ ] Implement `me()` endpoint (requires auth middleware)
- [ ] Add error handling
- [ ] Add request/response DTOs
- [ ] Add OpenAPI documentation

---

#### Task 1.9: Implement Rate Limiting

**File**: `server/src/middleware/rate_limit.rs`

**Implementation**:

```rust
use tower_governor::{Governor, GovernorConfigBuilder};
use tower_governor::key_extractor::SmartIpKeyExtractor;

pub fn create_rate_limiter() -> Governor {
    let config = Box::leak(Box::new(
        GovernorConfigBuilder::default()
            .per_second(100) // Global limit
            .burst_size(200)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    ));
    
    Governor::new(config)
}
```

**Checklist**:
- [ ] Install `tower-governor`
- [ ] Create global rate limiter
- [ ] Create authenticated user rate limiter
- [ ] Create admin rate limiter
- [ ] Create login attempt rate limiter
- [ ] Add rate limit headers to responses
- [ ] Test rate limiting

---

### Phase 2: Admin API Endpoints

#### Task 2.1: Create Admin Middleware

**File**: `server/src/middleware/admin.rs`

```rust
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use crate::db::models::user::User;
use crate::db::repositories::user_repository::UserRepository;

pub async fn admin_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract user from request extensions (set by auth middleware)
    let user = request.extensions()
        .get::<User>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Check if user has admin role
    let user_repo = UserRepository::new(state.db_pool.clone());
    let has_admin = user_repo.has_role(&user.id, "admin")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !has_admin {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(request).await)
}
```

**Checklist**:
- [ ] Extract user from request extensions
- [ ] Check if user has "admin" role
- [ ] Return 403 if not admin
- [ ] Add error handling
- [ ] Add logging

---

#### Task 2.2: Create Admin User Management API

**File**: `server/src/api/v1/admin/users/mod.rs`

**Endpoints**:

```rust
// GET /api/v1/admin/users?page=1&limit=20&search=...
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<ListUsersResponse>, StatusCode> {
    todo!()
}

// GET /api/v1/admin/users/{id}
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    todo!()
}

// POST /api/v1/admin/users
pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    todo!()
}

// PUT /api/v1/admin/users/{id}
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    todo!()
}

// DELETE /api/v1/admin/users/{id}
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    todo!()
}

// POST /api/v1/admin/users/{id}/enable
pub async fn enable_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    todo!()
}

// POST /api/v1/admin/users/{id}/disable
pub async fn disable_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    todo!()
}

// POST /api/v1/admin/users/{id}/lock
pub async fn lock_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<LockUserRequest>,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    todo!()
}

// POST /api/v1/admin/users/{id}/unlock
pub async fn unlock_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AdminUserResponse>, StatusCode> {
    todo!()
}

// GET /api/v1/admin/users/{id}/roles
pub async fn get_user_roles(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<String>>, StatusCode> {
    todo!()
}

// POST /api/v1/admin/users/{id}/roles
pub async fn add_user_role(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AddRoleRequest>,
) -> Result<Json<Vec<String>>, StatusCode> {
    todo!()
}

// DELETE /api/v1/admin/users/{id}/roles/{role}
pub async fn remove_user_role(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Path(role): Path<String>,
) -> Result<Json<Vec<String>>, StatusCode> {
    todo!()
}

// GET /api/v1/admin/users/{id}/stats
pub async fn get_user_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UserStatsResponse>, StatusCode> {
    todo!()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/:id/enable", post(enable_user))
        .route("/:id/disable", post(disable_user))
        .route("/:id/lock", post(lock_user))
        .route("/:id/unlock", post(unlock_user))
        .route("/:id/roles", get(get_user_roles).post(add_user_role))
        .route("/:id/roles/:role", delete(remove_user_role))
        .route("/:id/stats", get(get_user_stats))
}
```

**Checklist**:
- [ ] Create DTOs (ListUsersQuery, AdminUserResponse, etc.)
- [ ] Implement `list_users()` with pagination and filters
- [ ] Implement `get_user()`
- [ ] Implement `create_user()`
- [ ] Implement `update_user()`
- [ ] Implement `delete_user()` (soft delete)
- [ ] Implement `enable_user()` / `disable_user()`
- [ ] Implement `lock_user()` / `unlock_user()`
- [ ] Implement role management endpoints
- [ ] Implement `get_user_stats()`
- [ ] Add admin middleware to router
- [ ] Add error handling
- [ ] Add OpenAPI documentation

---

#### Task 2.3: Create Provider Management API

**Files to Create**:
- `server/src/db/models/ai_provider.rs`
- `server/src/db/repositories/ai_provider_repository.rs`
- `server/src/api/v1/admin/providers/mod.rs`

**Similar structure to User Management API**

**Checklist**:
- [ ] Create `AiProvider` model
- [ ] Create `AiProviderRepository` with CRUD methods
- [ ] Implement provider endpoints (list, get, create, update, delete, enable, disable)
- [ ] Add admin middleware
- [ ] Add error handling
- [ ] Add OpenAPI documentation

---

#### Task 2.4: Create Model Management API

**Files to Modify**:
- `server/src/db/models/ai_model.rs` (add new fields)
- `server/src/db/repositories/ai_model_repository.rs` (add methods)
- `server/src/api/v1/admin/models/mod.rs` (create endpoints)

**Checklist**:
- [ ] Update `AiModel` model with new fields (`is_paid`, `disabled`, `provider_id`)
- [ ] Update `AiModelRepository` with enable/disable methods
- [ ] Implement model endpoints (list, get, create, update, delete, enable, disable, deprecate)
- [ ] Add scan placeholder endpoints (return 501)
- [ ] Add admin middleware
- [ ] Add error handling
- [ ] Add OpenAPI documentation

---

#### Task 2.5: Create Dashboard API

**File**: `server/src/api/v1/admin/dashboard/mod.rs`

```rust
// GET /api/v1/admin/dashboard/stats
pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<DashboardStatsResponse>, StatusCode> {
    // Aggregate statistics from all repositories
    todo!()
}
```

**Checklist**:
- [ ] Create `DashboardStatsResponse` DTO
- [ ] Implement statistics aggregation
- [ ] Add admin middleware
- [ ] Add error handling
- [ ] Add OpenAPI documentation

---

### Phase 3: Route Organization

#### Task 3.1: Update Router Structure

**File**: `server/src/main.rs`

**Update router setup**:

```rust
// Public routes
let public_routes = Router::new()
    .route("/", get(api::v1::public::home))
    .route("/about", get(api::v1::public::about))
    .route("/health", get(api::v1::health::health_check));

// Auth routes
let auth_routes = api::v1::auth::router();

// Authenticated user routes
let user_routes = Router::new()
    .route("/me", get(api::v1::user::profile))
    .route("/models", get(api::v1::models::list_models))
    // ... other user routes
    .route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::auth::auth_middleware,
    ));

// Admin routes
let admin_routes = Router::new()
    .route("/dashboard/stats", get(api::v1::admin::dashboard::get_stats))
    .nest("/users", api::v1::admin::users::router())
    .nest("/providers", api::v1::admin::providers::router())
    .nest("/models", api::v1::admin::models::router())
    .route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::rate_limit::rate_limit_middleware,
    ))
    .route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::auth::auth_middleware,
    ))
    .route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::admin::admin_middleware,
    ));

// Combine all routes
let api_router = Router::new()
    .merge(public_routes)
    .nest("/api/v1/auth", auth_routes)
    .nest("/api/v1", user_routes)
    .nest("/api/v1/admin", admin_routes);
```

**Checklist**:
- [ ] Organize routes into public, auth, user, admin
- [ ] Apply appropriate middleware layers
- [ ] Test route organization
- [ ] Verify middleware order

---

## Frontend Development

### Phase 1: Authentication UI

#### Task 1.1: Update Auth Utilities

**File**: `ui/src/lib/auth.ts`

**Remove Firebase, add OIDC**:

```typescript
// Remove Firebase imports
// Add OIDC login flow

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export interface AuthToken {
  access_token: string;
  refresh_token: string;
  expires_at: number;
}

export async function initiateLogin(): Promise<void> {
  // Redirect to backend login endpoint
  window.location.href = `${API_BASE_URL}/api/v1/auth/login`;
}

export async function handleCallback(token: string): Promise<void> {
  // Store token in localStorage
  localStorage.setItem('auth_token', token);
}

export async function logout(): Promise<void> {
  localStorage.removeItem('auth_token');
  await fetch(`${API_BASE_URL}/api/v1/auth/logout`, { method: 'POST' });
  window.location.href = '/';
}

export function getToken(): string | null {
  return localStorage.getItem('auth_token');
}

export async function refreshToken(): Promise<string | null> {
  const refreshToken = localStorage.getItem('refresh_token');
  if (!refreshToken) return null;
  
  const response = await fetch(`${API_BASE_URL}/api/v1/auth/refresh`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ refresh_token: refreshToken }),
  });
  
  if (!response.ok) {
    logout();
    return null;
  }
  
  const data = await response.json();
  localStorage.setItem('auth_token', data.access_token);
  return data.access_token;
}

export async function getCurrentUser(): Promise<User | null> {
  const token = getToken();
  if (!token) return null;
  
  const response = await fetch(`${API_BASE_URL}/api/v1/auth/me`, {
    headers: { 'Authorization': `Bearer ${token}` },
  });
  
  if (!response.ok) {
    if (response.status === 401) {
      logout();
    }
    return null;
  }
  
  return response.json();
}
```

**Checklist**:
- [ ] Remove Firebase SDK code
- [ ] Implement `initiateLogin()` (redirect to backend)
- [ ] Implement `handleCallback()` (store token)
- [ ] Implement `logout()` (clear token, call backend)
- [ ] Implement `getToken()` (read from localStorage)
- [ ] Implement `refreshToken()` (call backend refresh endpoint)
- [ ] Implement `getCurrentUser()` (call backend /me endpoint)
- [ ] Add token expiration checking
- [ ] Add automatic token refresh

---

#### Task 1.2: Update Auth Context

**File**: `ui/src/lib/auth-context.tsx`

```typescript
import { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { getCurrentUser, logout as authLogout, getToken } from './auth';

interface User {
  id: string;
  email: string;
  name?: string;
  roles?: string[];
}

interface AuthContextType {
  user: User | null;
  loading: boolean;
  isAuthenticated: boolean;
  isAdmin: boolean;
  login: () => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadUser();
  }, []);

  async function loadUser() {
    const token = getToken();
    if (!token) {
      setLoading(false);
      return;
    }

    try {
      const currentUser = await getCurrentUser();
      setUser(currentUser);
    } catch (error) {
      console.error('Failed to load user:', error);
    } finally {
      setLoading(false);
    }
  }

  function login() {
    // Redirect to backend login
    window.location.href = '/api/v1/auth/login';
  }

  async function logout() {
    await authLogout();
    setUser(null);
  }

  const isAuthenticated = !!user;
  const isAdmin = user?.roles?.includes('admin') ?? false;

  return (
    <AuthContext.Provider
      value={{
        user,
        loading,
        isAuthenticated,
        isAdmin,
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
```

**Checklist**:
- [ ] Remove Firebase auth state
- [ ] Add OIDC token state
- [ ] Implement `loadUser()` function
- [ ] Implement `login()` function (redirect)
- [ ] Implement `logout()` function
- [ ] Add `isAdmin` computed property
- [ ] Add loading state
- [ ] Handle token expiration
- [ ] Update provider component

---

#### Task 1.3: Update Login Form

**File**: `ui/src/components/login-form.tsx`

```typescript
import { useAuth } from '@/lib/auth-context';

export function LoginForm() {
  const { login } = useAuth();

  return (
    <div className="flex flex-col items-center justify-center min-h-screen">
      <div className="w-full max-w-md p-8 bg-white rounded-lg shadow">
        <h1 className="text-2xl font-bold mb-6">Login</h1>
        <button
          onClick={login}
          className="w-full px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
        >
          Login with OIDC
        </button>
      </div>
    </div>
  );
}
```

**Checklist**:
- [ ] Remove Firebase login code
- [ ] Add OIDC login button
- [ ] Handle callback route (`/auth/callback`)
- [ ] Display error messages
- [ ] Add loading state

---

#### Task 1.4: Create Protected Route Component

**File**: `ui/src/components/protected-route.tsx`

```typescript
import { ReactNode } from 'react';
import { Navigate } from 'react-router-dom';
import { useAuth } from '@/lib/auth-context';

export function ProtectedRoute({ children }: { children: ReactNode }) {
  const { isAuthenticated, loading } = useAuth();

  if (loading) {
    return <div>Loading...</div>;
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}
```

**Checklist**:
- [ ] Check authentication status
- [ ] Show loading state
- [ ] Redirect to login if not authenticated
- [ ] Render children if authenticated

---

#### Task 1.5: Create Admin Route Component

**File**: `ui/src/components/admin-route.tsx`

```typescript
import { ReactNode } from 'react';
import { Navigate } from 'react-router-dom';
import { useAuth } from '@/lib/auth-context';

export function AdminRoute({ children }: { children: ReactNode }) {
  const { isAuthenticated, isAdmin, loading } = useAuth();

  if (loading) {
    return <div>Loading...</div>;
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  if (!isAdmin) {
    return <Navigate to="/" replace />;
  }

  return <>{children}</>;
}
```

**Checklist**:
- [ ] Check authentication status
- [ ] Check admin role
- [ ] Show loading state
- [ ] Redirect to login if not authenticated
- [ ] Redirect to home if not admin
- [ ] Render children if admin

---

### Phase 2: Public Pages

#### Task 2.1: Create Home Page

**File**: `ui/src/pages/home.tsx`

**Checklist**:
- [ ] Create landing page component
- [ ] Add app description
- [ ] Add call-to-action buttons
- [ ] Add feature highlights
- [ ] Make responsive
- [ ] Add routing

---

#### Task 2.2: Create About Page

**File**: `ui/src/pages/about.tsx`

**Checklist**:
- [ ] Create about page component
- [ ] Add project information
- [ ] Add technology stack
- [ ] Add team/contributors section
- [ ] Add links to documentation
- [ ] Make responsive

---

#### Task 2.3: Create Health Page

**File**: `ui/src/pages/health.tsx`

**Checklist**:
- [ ] Create health page component
- [ ] Fetch health status from `/api/v1/health`
- [ ] Display system health status
- [ ] Display API connectivity status
- [ ] Display database status
- [ ] Add auto-refresh
- [ ] Make responsive

---

### Phase 3: Authenticated User Area

#### Task 3.1: Update Profile Page

**File**: `ui/src/pages/profile.tsx`

**Checklist**:
- [ ] Fetch user data from `/api/v1/me`
- [ ] Display user information
- [ ] Add edit profile form
- [ ] Add account settings
- [ ] Add logout button
- [ ] Make responsive

---

#### Task 3.2: Update Models Page

**File**: `ui/src/pages/models.tsx`

**Checklist**:
- [ ] Fetch models from `/api/v1/models`
- [ ] Display model list
- [ ] Add filter by provider
- [ ] Add search functionality
- [ ] Add model details view
- [ ] Make responsive

---

#### Task 3.3-3.6: Update Other User Pages

**Similar updates for**:
- `ui/src/pages/chats.tsx`
- `ui/src/pages/chat.tsx`
- `ui/src/pages/keys.tsx`
- `ui/src/pages/features.tsx`

**Checklist**:
- [ ] Update to use new auth system
- [ ] Add ProtectedRoute wrapper
- [ ] Update API calls
- [ ] Handle authentication errors
- [ ] Make responsive

---

### Phase 4: Admin Area

#### Task 4.1: Create Admin Layout

**File**: `ui/src/layouts/admin-layout.tsx`

```typescript
import { ReactNode } from 'react';
import { AdminNav } from '@/components/admin/admin-nav';

export function AdminLayout({ children }: { children: ReactNode }) {
  return (
    <div className="flex min-h-screen">
      <AdminNav />
      <main className="flex-1 p-8">
        {children}
      </main>
    </div>
  );
}
```

**Checklist**:
- [ ] Create layout wrapper
- [ ] Include AdminNav component
- [ ] Add header
- [ ] Make responsive
- [ ] Add AdminRoute wrapper

---

#### Task 4.2: Create Admin Navigation

**File**: `ui/src/components/admin/admin-nav.tsx`

**Checklist**:
- [ ] Create sidebar navigation
- [ ] Add Dashboard link
- [ ] Add Users link
- [ ] Add Providers link
- [ ] Add Models link
- [ ] Add Logout button
- [ ] Highlight active route
- [ ] Make responsive (mobile menu)

---

#### Task 4.3: Create Admin Dashboard

**File**: `ui/src/pages/admin/dashboard.tsx`

**Checklist**:
- [ ] Fetch stats from `/api/v1/admin/dashboard/stats`
- [ ] Create statistics cards
- [ ] Add charts/graphs (optional)
- [ ] Add recent activity feed (placeholder)
- [ ] Add quick actions
- [ ] Make responsive

**Components**:
- [ ] `ui/src/components/admin/stats-card.tsx`
- [ ] `ui/src/components/admin/activity-feed.tsx`

---

#### Task 4.4: Create Admin Users Page

**File**: `ui/src/pages/admin/users.tsx`

**Checklist**:
- [ ] Fetch users from `/api/v1/admin/users`
- [ ] Display user list table
- [ ] Add pagination
- [ ] Add search functionality
- [ ] Add filters (role, disabled, locked)
- [ ] Add create user button
- [ ] Add edit user action
- [ ] Add enable/disable action
- [ ] Add lock/unlock action
- [ ] Add delete action
- [ ] Add user details modal
- [ ] Add bulk actions
- [ ] Make responsive

**Components**:
- [ ] `ui/src/components/admin/user-list.tsx`
- [ ] `ui/src/components/admin/user-form.tsx`
- [ ] `ui/src/components/admin/user-details.tsx`

---

#### Task 4.5: Create Admin Providers Page

**File**: `ui/src/pages/admin/providers.tsx`

**Similar to Users page**

**Checklist**:
- [ ] Fetch providers from `/api/v1/admin/providers`
- [ ] Display provider list
- [ ] Add create/edit forms
- [ ] Add enable/disable actions
- [ ] Add delete action
- [ ] Add scan button (placeholder)
- [ ] Make responsive

---

#### Task 4.6: Create Admin Models Page

**File**: `ui/src/pages/admin/models.tsx`

**Similar to Users page**

**Checklist**:
- [ ] Fetch models from `/api/v1/admin/models`
- [ ] Display model list with filters
- [ ] Add create/edit forms
- [ ] Add enable/disable actions
- [ ] Add deprecate action
- [ ] Add delete action
- [ ] Add scan button (placeholder)
- [ ] Make responsive

---

## API Contract

### Authentication Endpoints

**Base URL**: `http://localhost:3000/api/v1/auth`

| Method | Endpoint | Auth Required | Description |
|--------|----------|---------------|-------------|
| GET | `/login` | No | Redirects to OIDC provider |
| GET | `/callback` | No | Handles OIDC callback |
| POST | `/logout` | Yes | Logs out user |
| POST | `/refresh` | No | Refreshes access token |
| GET | `/me` | Yes | Gets current user |

### Admin Endpoints

**Base URL**: `http://localhost:3000/api/v1/admin`

| Method | Endpoint | Auth Required | Description |
|--------|----------|---------------|-------------|
| GET | `/dashboard/stats` | Admin | Gets dashboard statistics |
| GET | `/users` | Admin | Lists users (paginated) |
| GET | `/users/{id}` | Admin | Gets user details |
| POST | `/users` | Admin | Creates user |
| PUT | `/users/{id}` | Admin | Updates user |
| DELETE | `/users/{id}` | Admin | Deletes user |
| POST | `/users/{id}/enable` | Admin | Enables user |
| POST | `/users/{id}/disable` | Admin | Disables user |
| POST | `/users/{id}/lock` | Admin | Locks user |
| POST | `/users/{id}/unlock` | Admin | Unlocks user |
| GET | `/users/{id}/roles` | Admin | Gets user roles |
| POST | `/users/{id}/roles` | Admin | Adds role to user |
| DELETE | `/users/{id}/roles/{role}` | Admin | Removes role from user |
| GET | `/users/{id}/stats` | Admin | Gets user statistics |

**Similar endpoints for `/providers` and `/models`**

---

## Testing Checklist

### Backend Testing

- [ ] Unit tests for OIDC client
- [ ] Unit tests for JWT verification
- [ ] Unit tests for user repository
- [ ] Unit tests for admin middleware
- [ ] Integration tests for auth flow
- [ ] Integration tests for admin endpoints
- [ ] API tests for all endpoints

### Frontend Testing

- [ ] Component tests for login form
- [ ] Component tests for protected routes
- [ ] Component tests for admin components
- [ ] Integration tests for auth flow
- [ ] Integration tests for admin workflows
- [ ] E2E tests for user journeys

---

## Development Workflow

### For Backend Developers

1. Start with Database Schema (Task: Database Schema)
2. Implement Authentication System (Phase 1)
3. Implement Admin APIs (Phase 2)
4. Organize Routes (Phase 3)

### For Frontend Developers

1. Start with Public Pages (Phase 2) - No dependencies
2. Wait for Backend Phase 1.1, then implement Auth UI (Phase 1)
3. Update User Area (Phase 3) - Can use existing endpoints
4. Wait for Backend Phase 2, then implement Admin Area (Phase 4)

### Parallel Development Tips

- **Backend**: Use mock data/stubs for testing
- **Frontend**: Use mock API responses for development
- **Communication**: Use API contract document for coordination
- **Integration**: Test integration points frequently

---

**Document Version**: 1.0  
**Last Updated**: 2025-01-XX  
**Status**: Active Development Guide

