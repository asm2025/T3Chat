# T3Chat Authentication & Admin System Development Plan

## Executive Summary

This document outlines the development plan for replacing Firebase authentication with a claim-based authentication system supporting OIDC, and implementing a comprehensive admin dashboard for user and model management. The plan is divided into Frontend and Backend tracks to enable parallel development by multiple developers.

---

## 1. Authentication System Selection

### Recommended Library: `oauth2` + `openidconnect`

**Primary Choice: `oauth2` crate**

-   **Repository**: https://github.com/ramosbugs/oauth2-rs
-   **Status**: Production-ready, actively maintained
-   **Features**:
    -   Full OAuth 2.0 implementation
    -   Async/await support
    -   Framework-agnostic (works with Axum)
    -   Well-documented and widely used
    -   Supports all OAuth2 flows (authorization code, client credentials, etc.)

**For OIDC Support: `openidconnect` crate**

-   **Repository**: https://github.com/ramosbugs/openidconnect-rs
-   **Status**: Production-ready, built on top of `oauth2`
-   **Features**:
    -   Strongly-typed OIDC implementation
    -   JWKS support for token verification
    -   UserInfo endpoint support
    -   Discovery document support

**Why not `yup-oauth2`?**

-   More Google-specific (though it can work with other providers)
-   Less flexible for general OIDC implementations
-   `oauth2` + `openidconnect` provides better OIDC support

**Additional Dependencies Needed:**

-   `jsonwebtoken` (already in use) - for JWT verification
-   `reqwest` (already in use) - for HTTP requests
-   `serde` + `serde_json` (already in use) - for serialization
-   `tower-governor` or `tower-ratelimit` - for rate limiting (recommended: `tower-governor`)

---

## 2. Application Structure Overview

### 2.1 Route Organization

#### Public Area (No Authentication Required)

-   `GET /` - Home page
-   `GET /about` - About page
-   `GET /health` - Health check endpoint (already exists at `/api/v1/health`)

#### Authenticated Users Area (Requires Authentication)

-   `GET /me` - User profile (already exists at `/api/v1/me`)
-   `GET /models` - List available models (already exists at `/api/v1/models`)
-   `GET /chats` - List user's chats (already exists at `/api/v1/chats`)
-   `GET /chat` - Chat interface (already exists at `/api/v1/chat`)
-   `GET /keys` - API keys management (already exists at `/api/v1/keys`)
-   `GET /features` - User features (already exists at `/api/v1/features`)

#### Administrators Area (Requires Admin Role)

-   `GET /admin/` - Admin dashboard
-   `GET /admin/models` - Model management
-   `GET /admin/users` - User management

---

## 3. Database Schema Design

**Note**: Since this is a brand new project, we'll create a fresh, clean schema from scratch rather than modifying existing tables. This minimizes migrations and ensures a clean foundation.

### 3.1 Users Table (ASP.NET Identity Style)

**Design Philosophy**: Model after ASP.NET Core Identity's `IdentityUser` with lockout support and separate role management.

**Schema**:

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY, -- OIDC subject ID or generated UUID
    email TEXT NOT NULL UNIQUE,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    normalized_email TEXT NOT NULL UNIQUE, -- For case-insensitive lookups
    name TEXT,
    username TEXT UNIQUE,
    normalized_username TEXT UNIQUE, -- For case-insensitive lookups
    avatar_url TEXT,
    provider TEXT NOT NULL DEFAULT 'oidc', -- "oidc", "local", "google", "github", etc.

    -- Account Status (ASP.NET Identity style)
    disabled BOOLEAN NOT NULL DEFAULT false, -- Account disabled flag (false = enabled)
    locked_out BOOLEAN NOT NULL DEFAULT false, -- Account locked due to failed attempts
    lockout_end TIMESTAMPTZ, -- When lockout expires (NULL if not locked)
    access_failed_count INTEGER NOT NULL DEFAULT 0, -- Failed login attempts

    -- Password (for local auth)
    password_hash TEXT, -- Hashed password (if using local auth)
    password_changed_at TIMESTAMPTZ, -- Last password change

    -- Two-Factor Authentication
    two_factor_enabled BOOLEAN NOT NULL DEFAULT false,
    totp_secret TEXT, -- Encrypted TOTP secret

    -- Login Tracking
    last_login_at TIMESTAMPTZ, -- Last successful login
    login_count INTEGER NOT NULL DEFAULT 0, -- Total successful logins

    -- User Preferences
    preferences JSONB, -- UI settings, default models, etc.

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
```

### 3.2 User Roles Table (ASP.NET Identity Style)

**Purpose**: Separate role management table, similar to ASP.NET Identity's `IdentityUserRole`.

**Schema**:

```sql
CREATE TABLE user_roles (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_name TEXT NOT NULL, -- "admin", "user", "moderator", etc.
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by TEXT REFERENCES users(id), -- Who assigned this role
    PRIMARY KEY (user_id, role_name)
);

CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_name ON user_roles(role_name);
```

### 3.3 Roles Table (ASP.NET Identity Style)

**Purpose**: Define available roles in the system.

**Schema**:

```sql
CREATE TABLE roles (
    name TEXT PRIMARY KEY, -- "admin", "user", "moderator", etc.
    display_name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default roles
INSERT INTO roles (name, display_name, description) VALUES
    ('user', 'User', 'Standard user role'),
    ('admin', 'Administrator', 'Full system access'),
    ('moderator', 'Moderator', 'Content moderation access');
```

### 3.4 AI Providers Table

**Purpose**: Separate table for AI service providers to enable provider-level management.

**Schema**:

```sql
CREATE TABLE ai_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id TEXT NOT NULL UNIQUE, -- "openai", "anthropic", "google", etc.
    display_name TEXT NOT NULL,
    description TEXT,
    base_url TEXT, -- API base URL
    website_url TEXT,
    documentation_url TEXT,
    disabled BOOLEAN NOT NULL DEFAULT false, -- Admin can disable (false = enabled)
    is_active BOOLEAN NOT NULL DEFAULT true, -- Provider is active/available
    requires_api_key BOOLEAN NOT NULL DEFAULT true,
    supports_streaming BOOLEAN NOT NULL DEFAULT true,
    supports_images BOOLEAN NOT NULL DEFAULT false,
    supports_functions BOOLEAN NOT NULL DEFAULT false,
    supports_vision BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB, -- Additional provider-specific config
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_providers_provider_id ON ai_providers(provider_id);
CREATE INDEX idx_ai_providers_disabled ON ai_providers(disabled);
CREATE INDEX idx_ai_providers_is_active ON ai_providers(is_active);
```

### 3.5 AI Models Table

**Schema**:

```sql
CREATE TABLE ai_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES ai_providers(id) ON DELETE CASCADE,
    model_id TEXT NOT NULL, -- "gpt-4-turbo", "claude-3-opus", "gemini-pro", etc.
    display_name TEXT NOT NULL,
    description TEXT,

    -- Capabilities
    context_window INTEGER NOT NULL,
    max_output_tokens INTEGER,
    supports_streaming BOOLEAN NOT NULL DEFAULT true,
    supports_images BOOLEAN NOT NULL DEFAULT false,
    supports_functions BOOLEAN NOT NULL DEFAULT false,
    supports_vision BOOLEAN NOT NULL DEFAULT false,

    -- Pricing (per 1M tokens)
    cost_per_input_token NUMERIC,
    cost_per_output_token NUMERIC,
    is_paid BOOLEAN NOT NULL DEFAULT true, -- true = paid model, false = free model

    -- Status
    disabled BOOLEAN NOT NULL DEFAULT false, -- Admin can disable (false = enabled)
    is_active BOOLEAN NOT NULL DEFAULT true, -- Model is active/available
    deprecated_at TIMESTAMPTZ, -- When model was deprecated

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(provider_id, model_id)
);

CREATE INDEX idx_ai_models_provider_id ON ai_models(provider_id);
CREATE INDEX idx_ai_models_disabled ON ai_models(disabled);
CREATE INDEX idx_ai_models_is_active ON ai_models(is_active);
```

**Migration File**: `migrations/YYYYMMDDHHMMSS_initial_schema/up.sql`

**Note**: Since this is a new project, create a single comprehensive migration file with all tables (users, roles, user_roles, ai_providers, ai_models).

### 3.2 AI Providers Table (New)

**Purpose**: Separate table for AI service providers to enable provider-level management.

**Schema**:

```sql
CREATE TABLE ai_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id TEXT NOT NULL UNIQUE, -- "openai", "anthropic", "google", etc.
    display_name TEXT NOT NULL,
    description TEXT,
    base_url TEXT, -- API base URL
    website_url TEXT,
    documentation_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    disabled BOOLEAN NOT NULL DEFAULT false, -- Admin can disable (false = enabled)
    requires_api_key BOOLEAN NOT NULL DEFAULT true,
    supports_streaming BOOLEAN NOT NULL DEFAULT true,
    supports_images BOOLEAN NOT NULL DEFAULT false,
    supports_functions BOOLEAN NOT NULL DEFAULT false,
    supports_vision BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB, -- Additional provider-specific config
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_providers_provider_id ON ai_providers(provider_id);
CREATE INDEX idx_ai_providers_is_active ON ai_providers(is_active);
CREATE INDEX idx_ai_providers_disabled ON ai_providers(disabled);
```

**Migration File**: `migrations/YYYYMMDDHHMMSS_create_ai_providers_table/up.sql`

---

## 4. Backend Development Plan

### Phase 1: Authentication System Replacement

#### 4.1.1 OAuth2/OIDC Integration

**Files to Create/Modify:**

-   `server/src/auth/mod.rs` - New authentication module
-   `server/src/auth/oidc.rs` - OIDC client implementation
-   `server/src/auth/jwt.rs` - JWT token verification
-   `server/src/auth/session.rs` - Session management
-   `server/src/middleware/auth.rs` - Update to use new auth system

**Tasks:**

1. **Install Dependencies**:

    ```toml
    oauth2 = "4"
    openidconnect = "2"
    tower-governor = "0" # Rate limiting
    ```

2. **Create OIDC Client** (`server/src/auth/oidc.rs`):

    - Implement OIDC discovery document fetching
    - Implement authorization code flow
    - Implement token exchange
    - Implement user info endpoint calls
    - Handle refresh tokens

3. **Create JWT Verification** (`server/src/auth/jwt.rs`):

    - JWKS fetching and caching
    - Token verification with claims extraction
    - Support for multiple issuers (if needed)
    - Token expiration handling

4. **Update Auth Middleware** (`server/src/middleware/auth.rs`):

    - Replace Firebase token verification with OIDC JWT verification
    - Extract user claims from JWT
    - Check `disabled` field before allowing login (must be false)
    - Check `locked_out` field and `lockout_end` before allowing login
    - Update `last_login_at` and `login_count` on successful auth
    - Reset `access_failed_count` on successful login
    - Increment `access_failed_count` on failed login
    - Lock account if `access_failed_count` exceeds threshold

5. **Create Auth Routes** (`server/src/api/v1/auth/mod.rs`):
    - `GET /api/v1/auth/login` - Initiate OIDC login
    - `GET /api/v1/auth/callback` - Handle OIDC callback
    - `POST /api/v1/auth/logout` - Logout endpoint
    - `POST /api/v1/auth/refresh` - Refresh token endpoint
    - `GET /api/v1/auth/me` - Get current user (move from `/api/v1/me`)

**Dependencies:**

-   Requires database access for user lookup/creation
-   Requires environment variables for OIDC configuration

**Environment Variables Needed:**

```bash
# OIDC Provider Configuration
OIDC_ISSUER_URL=https://your-provider.com
OIDC_CLIENT_ID=your-client-id
OIDC_CLIENT_SECRET=your-client-secret
OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback

# JWT Configuration
JWT_SECRET=your-jwt-secret-for-local-tokens
JWT_EXPIRY_SECONDS=3600
```

#### 4.1.2 User Account Management (ASP.NET Identity Style)

**Files to Create/Modify:**

-   `server/src/db/models/user.rs` - Create new user model (ASP.NET Identity style)
-   `server/src/db/models/user_role.rs` - User role relationship model
-   `server/src/db/models/role.rs` - Role model
-   `server/src/db/repositories/user_repository.rs` - User repository with lockout support
-   `server/src/db/repositories/user_role_repository.rs` - User role repository
-   `server/src/db/repositories/role_repository.rs` - Role repository
-   `server/src/api/v1/user/mod.rs` - Update user endpoints

**Tasks:**

1. **Create User Model** (ASP.NET Identity style):

    - `disabled` field (Boolean, default: false) - Account disabled flag
    - `locked_out` field (Boolean, default: false) - Account locked flag
    - `lockout_end` field (DateTime<Utc>, nullable) - Lockout expiration
    - `access_failed_count` field (Integer, default: 0) - Failed login attempts
    - `last_login_at` field (DateTime<Utc>, nullable) - Last successful login
    - `login_count` field (Integer, default: 0) - Total successful logins
    - `normalized_email` field (String) - For case-insensitive lookups
    - `normalized_username` field (String, nullable) - For case-insensitive lookups
    - Remove `role` field (roles are now in separate table)

2. **Create Role Models**:

    - `Role` model - System roles
    - `UserRole` model - User-role relationship

3. **Update User Repository**:

    - `enable_user(user_id)` - Set `disabled = false`
    - `disable_user(user_id)` - Set `disabled = true`
    - `lock_user(user_id, duration)` - Lock account for specified duration
    - `unlock_user(user_id)` - Unlock account
    - `increment_failed_login(user_id)` - Increment failed count, lock if threshold reached
    - `reset_failed_login(user_id)` - Reset failed count on successful login
    - `update_last_login(user_id)` - Update last login timestamp and count
    - `get_by_email(email)` - Case-insensitive email lookup
    - `get_user_roles(user_id)` - Get all roles for a user
    - `add_role(user_id, role_name)` - Add role to user
    - `remove_role(user_id, role_name)` - Remove role from user
    - `has_role(user_id, role_name)` - Check if user has role

4. **Update Auth Middleware**:
    - Check `disabled` field (must be false)
    - Check `locked_out` field and `lockout_end` (must not be locked or lockout expired)
    - Return 403 Forbidden if account is disabled or locked
    - Increment `access_failed_count` on failed authentication
    - Lock account if `access_failed_count >= MAX_FAILED_ATTEMPTS` (e.g., 5)
    - Reset `access_failed_count` on successful authentication

**Dependencies:**

-   Requires fresh database schema (Phase 3.1)

### Phase 1.3: Rate Limiting

#### 4.1.3 Rate Limiting Implementation

**Files to Create:**

-   `server/src/middleware/rate_limit.rs` - Rate limiting middleware
-   `server/src/config/rate_limit.rs` - Rate limit configuration

**Tasks:**

1. **Install Rate Limiting Library**:

    - Use `tower-governor` (recommended) or `tower-ratelimit`
    - Configure rate limiters per route/endpoint

2. **Create Rate Limit Middleware**:

    - Global rate limiter (e.g., 100 requests/minute per IP)
    - Authenticated user rate limiter (e.g., 1000 requests/minute per user)
    - Admin rate limiter (e.g., 5000 requests/minute per admin)
    - Per-endpoint rate limiters for sensitive operations
    - Rate limit for login attempts (e.g., 5 attempts per 15 minutes per IP)

3. **Rate Limit Configuration**:

    - Configurable via environment variables
    - Different limits for different environments
    - IP-based limiting for public endpoints
    - User-based limiting for authenticated endpoints

4. **Rate Limit Headers**:
    - Return `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` headers
    - Return 429 Too Many Requests when limit exceeded

**Environment Variables Needed:**

```bash
# Rate Limiting Configuration
RATE_LIMIT_GLOBAL_PER_MINUTE=100
RATE_LIMIT_AUTHENTICATED_PER_MINUTE=1000
RATE_LIMIT_ADMIN_PER_MINUTE=5000
RATE_LIMIT_LOGIN_ATTEMPTS=5
RATE_LIMIT_LOGIN_WINDOW_MINUTES=15
```

**Dependencies:**

-   Can be implemented in parallel with Phase 1.1

### Phase 2: Admin API Endpoints

#### 4.2.1 Admin Middleware

**Files to Create:**

-   `server/src/middleware/admin.rs` - Admin authorization middleware

**Tasks:**

1. **Create Admin Middleware**:
    - Check if user has "admin" role (via `user_roles` table)
    - Return 403 Forbidden if not admin
    - Extract admin user from request extensions

**Usage Pattern:**

```rust
.route_layer(axum::middleware::from_fn_with_state(
    state.clone(),
    middleware::auth::auth_middleware, // First check auth
))
.route_layer(axum::middleware::from_fn_with_state(
    state.clone(),
    middleware::admin::admin_middleware, // Then check admin role
))
```

#### 4.2.2 User Management API

**Files to Create:**

-   `server/src/api/v1/admin/users/mod.rs` - Admin user management endpoints

**Endpoints to Implement:**

-   `GET /api/v1/admin/users` - List all users (with pagination)
    -   Query params: `page`, `limit`, `search`, `role`, `disabled`, `locked_out`
    -   Response: List of users with metadata
-   `GET /api/v1/admin/users/{id}` - Get user details
-   `POST /api/v1/admin/users` - Create new user (admin only)
-   `PUT /api/v1/admin/users/{id}` - Update user
-   `DELETE /api/v1/admin/users/{id}` - Delete user (soft delete)
-   `POST /api/v1/admin/users/{id}/enable` - Enable user account (set `disabled = false`)
-   `POST /api/v1/admin/users/{id}/disable` - Disable user account (set `disabled = true`)
-   `POST /api/v1/admin/users/{id}/lock` - Lock user account
-   `POST /api/v1/admin/users/{id}/unlock` - Unlock user account
-   `GET /api/v1/admin/users/{id}/roles` - Get user roles
-   `POST /api/v1/admin/users/{id}/roles` - Add role to user
-   `DELETE /api/v1/admin/users/{id}/roles/{role}` - Remove role from user
-   `GET /api/v1/admin/users/{id}/stats` - Get user statistics (login count, failed attempts, etc.)

**Tasks:**

1. **Create User DTOs**:

    - `AdminUserListDto` - For list endpoint (includes roles)
    - `AdminCreateUserDto` - For user creation
    - `AdminUpdateUserDto` - For user updates
    - `AdminUserStatsDto` - For user statistics
    - `UserRoleDto` - For role management

2. **Implement Repository Methods**:

    - `list_users(filters, pagination)` - List with filters (include roles)
    - `get_user_by_id(id)` - Get user details (include roles)
    - `create_user(dto)` - Create user
    - `update_user(id, dto)` - Update user
    - `soft_delete_user(id)` - Soft delete
    - `get_user_stats(id)` - Get statistics
    - `get_user_roles(id)` - Get user roles
    - `add_user_role(id, role)` - Add role
    - `remove_user_role(id, role)` - Remove role

3. **Implement Endpoints**:
    - All endpoints require admin middleware
    - Proper error handling and validation
    - OpenAPI documentation
    - Include role management endpoints

**Dependencies:**

-   Requires admin middleware (Phase 2.1)
-   Requires updated user repository (Phase 1.2)

#### 4.2.3 AI Provider Management API

**Files to Create:**

-   `server/src/db/models/ai_provider.rs` - AI Provider model
-   `server/src/db/repositories/ai_provider_repository.rs` - Provider repository
-   `server/src/api/v1/admin/providers/mod.rs` - Provider management endpoints

**Endpoints to Implement:**

-   `GET /api/v1/admin/providers` - List all providers
-   `GET /api/v1/admin/providers/{id}` - Get provider details
-   `POST /api/v1/admin/providers` - Create new provider
-   `PUT /api/v1/admin/providers/{id}` - Update provider
-   `DELETE /api/v1/admin/providers/{id}` - Delete provider
-   `POST /api/v1/admin/providers/{id}/enable` - Enable provider (set `disabled = false`)
-   `POST /api/v1/admin/providers/{id}/disable` - Disable provider (set `disabled = true`)
-   `POST /api/v1/admin/providers/{id}/scan` - Scan for new models (placeholder)

**Tasks:**

1. **Create Provider Model**:

    - Define `AiProvider` struct
    - Define `NewAiProvider` struct
    - Define `UpdateAiProvider` struct

2. **Create Provider Repository**:

    - CRUD operations
    - Enable/disable methods
    - List with filters

3. **Implement Endpoints**:
    - All CRUD operations
    - Enable/disable endpoints
    - Scan endpoint (placeholder - returns 501 Not Implemented)

**Dependencies:**

-   Requires fresh database schema (Phase 3.1)
-   Requires admin middleware (Phase 2.1)

#### 4.2.4 AI Model Management API

**Files to Create/Modify:**

-   `server/src/db/models/ai_model.rs` - Update model with new fields
-   `server/src/db/repositories/ai_model_repository.rs` - Update repository
-   `server/src/api/v1/admin/models/mod.rs` - Admin model management endpoints

**Endpoints to Implement:**

-   `GET /api/v1/admin/models` - List all models (with filters)
    -   Query params: `provider_id`, `disabled`, `is_paid`, `is_active`
-   `GET /api/v1/admin/models/{id}` - Get model details
-   `POST /api/v1/admin/models` - Create new model
-   `PUT /api/v1/admin/models/{id}` - Update model
-   `DELETE /api/v1/admin/models/{id}` - Delete model
-   `POST /api/v1/admin/models/{id}/enable` - Enable model (set `disabled = false`)
-   `POST /api/v1/admin/models/{id}/disable` - Disable model (set `disabled = true`)
-   `POST /api/v1/admin/models/{id}/deprecate` - Mark as deprecated
-   `POST /api/v1/admin/models/scan` - Scan all providers for new models (placeholder)
-   `POST /api/v1/admin/models/scan/{provider_id}` - Scan specific provider (placeholder)

**Tasks:**

1. **Update Model**:

    - Add new fields: `is_paid`, `disabled`
    - Add `provider_id` foreign key (required, not nullable)

2. **Update Repository**:

    - Add methods for enable/disable (using `disabled` field)
    - Add methods for deprecation
    - Add scan placeholder methods

3. **Implement Endpoints**:
    - All CRUD operations
    - Enable/disable/deprecate endpoints
    - Scan endpoints (placeholders)

**Placeholder Implementation for Scan:**

```rust
pub async fn scan_provider_models(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
) -> Result<Json<ScanResult>, StatusCode> {
    // TODO: Implement model scanning
    // This will:
    // 1. Fetch available models from provider API
    // 2. Compare with existing models in database
    // 3. Add new models, mark deprecated ones
    // 4. Detect paid vs free models (based on pricing information)

    Err(StatusCode::NOT_IMPLEMENTED)
}
```

**Dependencies:**

-   Requires fresh database schema (Phase 3.1)
-   Requires admin middleware (Phase 2.1)
-   Requires provider repository (Phase 2.3)

#### 4.2.5 Admin Dashboard API

**Files to Create:**

-   `server/src/api/v1/admin/dashboard/mod.rs` - Dashboard statistics endpoint

**Endpoints to Implement:**

-   `GET /api/v1/admin/dashboard/stats` - Get dashboard statistics
    -   Response: Total users, active users, disabled users, total models, active models, etc.

**Tasks:**

1. **Create Statistics DTO**:

    - `DashboardStatsDto` with all metrics

2. **Implement Repository Methods**:

    - `get_user_count()` - Total users
    - `get_active_user_count()` - Active users (where `disabled = false`)
    - `get_disabled_user_count()` - Disabled users (where `disabled = true`)
    - `get_locked_user_count()` - Locked users (where `locked_out = true`)
    - `get_model_count()` - Total models
    - `get_active_model_count()` - Active models (where `disabled = false`)
    - `get_provider_count()` - Total providers
    - `get_active_provider_count()` - Active providers (where `disabled = false`)

3. **Implement Endpoint**:
    - Aggregate all statistics
    - Return comprehensive dashboard data

**Dependencies:**

-   Requires admin middleware (Phase 2.1)
-   Requires user repository (Phase 1.2)
-   Requires model repository (Phase 2.4)
-   Requires provider repository (Phase 2.3)

### Phase 3: Database Schema & Route Organization

#### 4.3.1 Create Fresh Database Schema

**Files to Create:**

-   `migrations/YYYYMMDDHHMMSS_initial_schema/up.sql` - Complete initial schema
-   `migrations/YYYYMMDDHHMMSS_initial_schema/down.sql` - Rollback script

**Tasks:**

1. **Create Initial Migration**:

    - Include all tables: `users`, `roles`, `user_roles`, `ai_providers`, `ai_models`
    - Include all indexes
    - Include default role data
    - Use clean, minimal schema (no legacy fields)

2. **Schema Validation**:
    - Ensure all foreign keys are properly defined
    - Ensure all indexes are created
    - Ensure default values are set correctly

**Dependencies:**

-   None (foundational)

### Phase 4: Route Organization

#### 4.4.1 Update Router Structure

**Files to Modify:**

-   `server/src/main.rs` - Update router setup

**Tasks:**

1. **Organize Routes**:

    ```rust
    // Public routes
    let public_routes = Router::new()
        .route("/", get(api::v1::public::home))
        .route("/about", get(api::v1::public::about))
        .route("/health", get(api::v1::health::health_check));

    // Auth routes (public but special)
    let auth_routes = Router::new()
        .route("/login", get(api::v1::auth::login))
        .route("/callback", get(api::v1::auth::callback))
        .route("/logout", post(api::v1::auth::logout))
        .route("/refresh", post(api::v1::auth::refresh));

    // Authenticated user routes
    let user_routes = Router::new()
        .route("/me", get(api::v1::user::profile).put(api::v1::user::update_profile))
        .route("/models", get(api::v1::models::list_models))
        .route("/chats", get(api::v1::chats::list_chats).post(api::v1::chats::create_chat))
        // ... other user routes
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    // Admin routes
    let admin_routes = Router::new()
        .route("/dashboard/stats", get(api::v1::admin::dashboard::stats))
        .nest("/users", admin_users_routes)
        .nest("/providers", admin_providers_routes)
        .nest("/models", admin_models_routes)
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::rate_limit::rate_limit_middleware, // Rate limiting
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware, // Authentication
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::admin::admin_middleware, // Admin authorization
        ));

    // Combine all routes
    let api_router = Router::new()
        .merge(public_routes)
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1", user_routes)
        .nest("/api/v1/admin", admin_routes);
    ```

**Dependencies:**

-   Requires all previous phases

---

## 5. Frontend Development Plan

### Phase 1: Authentication UI Updates

#### 5.1.1 OIDC Login Flow

**Files to Create/Modify:**

-   `ui/src/lib/auth.ts` - Update authentication utilities
-   `ui/src/components/login-form.tsx` - Update login form
-   `ui/src/lib/auth-context.tsx` - Update auth context

**Tasks:**

1. **Update Auth Utilities**:

    - Remove Firebase SDK dependencies
    - Add OIDC login flow
    - Implement token storage (localStorage/sessionStorage)
    - Implement token refresh logic

2. **Update Login Form**:

    - Replace Firebase login with OIDC redirect
    - Handle OIDC callback
    - Display error messages

3. **Update Auth Context**:
    - Remove Firebase auth state
    - Add OIDC token state
    - Add user claims extraction
    - Add logout functionality

**Dependencies:**

-   Requires backend auth endpoints (Backend Phase 1.1)

#### 5.1.2 Protected Route Components

**Files to Create:**

-   `ui/src/components/protected-route.tsx` - Route protection component
-   `ui/src/components/admin-route.tsx` - Admin route protection

**Tasks:**

1. **Create Protected Route**:

    - Check authentication status
    - Redirect to login if not authenticated
    - Show loading state

2. **Create Admin Route**:
    - Check authentication status
    - Check admin role
    - Redirect to home if not admin

**Dependencies:**

-   Requires updated auth context (Phase 1.1)

### Phase 2: Public Area Pages

#### 5.2.1 Home Page

**Files to Create:**

-   `ui/src/pages/home.tsx` - Home page component

**Tasks:**

1. **Design Home Page**:
    - Landing page with app description
    - Call-to-action buttons
    - Feature highlights
    - Responsive design

**Dependencies:**

-   None (public page)

#### 5.2.2 About Page

**Files to Create:**

-   `ui/src/pages/about.tsx` - About page component

**Tasks:**

1. **Design About Page**:
    - Project information
    - Technology stack
    - Team/contributors
    - Links to documentation

**Dependencies:**

-   None (public page)

#### 5.2.3 Health Page

**Files to Create:**

-   `ui/src/pages/health.tsx` - Health status page

**Tasks:**

1. **Design Health Page**:
    - Display system health status
    - API connectivity status
    - Database status
    - Uptime information

**Dependencies:**

-   Requires `/api/v1/health` endpoint (already exists)

### Phase 3: Authenticated User Area

#### 5.3.1 Profile Page

**Files to Modify:**

-   `ui/src/pages/profile.tsx` - User profile page (may already exist)

**Tasks:**

1. **Update Profile Page**:
    - Display user information
    - Edit profile form
    - Change password (if local auth)
    - Account settings

**Dependencies:**

-   Requires `/api/v1/me` endpoint (already exists)

#### 5.3.2 Models Page

**Files to Modify:**

-   `ui/src/pages/models.tsx` - Models listing page (may already exist)

**Tasks:**

1. **Update Models Page**:
    - List available models
    - Filter by provider
    - Search functionality
    - Model details view

**Dependencies:**

-   Requires `/api/v1/models` endpoint (already exists)

#### 5.3.3 Chats Page

**Files to Modify:**

-   `ui/src/pages/chats.tsx` - Chats listing page (may already exist)

**Tasks:**

1. **Update Chats Page**:
    - List user's chats
    - Create new chat
    - Delete chat
    - Search/filter chats

**Dependencies:**

-   Requires `/api/v1/chats` endpoint (already exists)

#### 5.3.4 Chat Page

**Files to Modify:**

-   `ui/src/pages/chat.tsx` - Chat interface (may already exist)

**Tasks:**

1. **Update Chat Page**:
    - Chat interface
    - Message history
    - Model selection
    - Streaming support

**Dependencies:**

-   Requires `/api/v1/chat` endpoint (already exists)

#### 5.3.5 API Keys Page

**Files to Modify:**

-   `ui/src/pages/keys.tsx` - API keys management (may already exist)

**Tasks:**

1. **Update API Keys Page**:
    - List user's API keys
    - Create new API key
    - Delete API key
    - Show key usage stats

**Dependencies:**

-   Requires `/api/v1/keys` endpoint (already exists)

#### 5.3.6 Features Page

**Files to Modify:**

-   `ui/src/pages/features.tsx` - Features page (may already exist)

**Tasks:**

1. **Update Features Page**:
    - List available features
    - Enable/disable features
    - Feature descriptions

**Dependencies:**

-   Requires `/api/v1/features` endpoint (already exists)

### Phase 4: Admin Area

#### 5.4.1 Admin Dashboard

**Files to Create:**

-   `ui/src/pages/admin/dashboard.tsx` - Admin dashboard

**Tasks:**

1. **Design Dashboard**:
    - Statistics cards (users, models, providers)
    - Charts/graphs (user growth, model usage)
    - Recent activity feed
    - Quick actions

**Components Needed:**

-   `ui/src/components/admin/stats-card.tsx` - Statistics card component
-   `ui/src/components/admin/activity-feed.tsx` - Activity feed component

**Dependencies:**

-   Requires `/api/v1/admin/dashboard/stats` endpoint (Backend Phase 2.5)

#### 5.4.2 Admin Users Management

**Files to Create:**

-   `ui/src/pages/admin/users.tsx` - User management page
-   `ui/src/components/admin/user-list.tsx` - User list component
-   `ui/src/components/admin/user-form.tsx` - User create/edit form
-   `ui/src/components/admin/user-details.tsx` - User details modal

**Tasks:**

1. **Design Users Page**:
    - User list table with pagination
    - Search and filter functionality
    - Create user button
    - Edit user action
    - Enable/disable user action
    - Delete user action
    - User details modal

**Features:**

-   Pagination
-   Search by email/name
-   Filter by role, enabled status
-   Sort by various columns
-   Bulk actions (enable/disable multiple users)

**Dependencies:**

-   Requires `/api/v1/admin/users` endpoints (Backend Phase 2.2)

#### 5.4.3 Admin Providers Management

**Files to Create:**

-   `ui/src/pages/admin/providers.tsx` - Provider management page
-   `ui/src/components/admin/provider-list.tsx` - Provider list component
-   `ui/src/components/admin/provider-form.tsx` - Provider create/edit form
-   `ui/src/components/admin/provider-details.tsx` - Provider details modal

**Tasks:**

1. **Design Providers Page**:
    - Provider list table
    - Create provider form
    - Edit provider form
    - Enable/disable provider
    - Delete provider
    - Scan for models button (placeholder)

**Features:**

-   Provider configuration
-   API endpoint configuration
-   Capability flags (streaming, images, etc.)
-   Metadata editor (JSON)

**Dependencies:**

-   Requires `/api/v1/admin/providers` endpoints (Backend Phase 2.3)

#### 5.4.4 Admin Models Management

**Files to Create:**

-   `ui/src/pages/admin/models.tsx` - Model management page
-   `ui/src/components/admin/model-list.tsx` - Model list component
-   `ui/src/components/admin/model-form.tsx` - Model create/edit form
-   `ui/src/components/admin/model-details.tsx` - Model details modal

**Tasks:**

1. **Design Models Page**:
    - Model list table with filters
    - Filter by provider, enabled status, paid/free
    - Create model form
    - Edit model form
    - Enable/disable model
    - Deprecate model
    - Delete model
    - Scan for models button (placeholder)

**Features:**

-   Model configuration
-   Pricing configuration
-   Capability flags
-   Context window settings
-   Auto-discovery indicator

**Dependencies:**

-   Requires `/api/v1/admin/models` endpoints (Backend Phase 2.4)

#### 5.4.5 Admin Navigation

**Files to Create:**

-   `ui/src/components/admin/admin-nav.tsx` - Admin navigation component
-   `ui/src/layouts/admin-layout.tsx` - Admin layout wrapper

**Tasks:**

1. **Design Admin Navigation**:

    - Sidebar navigation
    - Dashboard link
    - Users link
    - Providers link
    - Models link
    - Logout button

2. **Create Admin Layout**:
    - Wrapper component for admin pages
    - Includes navigation
    - Includes header
    - Responsive design

**Dependencies:**

-   Requires admin route protection (Phase 1.2)

---

## 6. Development Dependencies & Order

### Backend Dependencies

```
Phase 1.1 (OAuth2/OIDC) → Phase 1.2 (User Management - ASP.NET Identity Style)
Phase 1.3 (Rate Limiting) - Can be done in parallel
    ↓
Phase 3.1 (Database Schema) - Fresh schema creation
    ↓
Phase 2.1 (Admin Middleware) → Phase 2.2 (User Management API)
    ↓                          → Phase 2.3 (Provider Management API)
    ↓                          → Phase 2.4 (Model Management API)
    ↓                          → Phase 2.5 (Dashboard API)
    ↓
Phase 4.1 (Route Organization)
```

### Frontend Dependencies

```
Phase 1.1 (OIDC Login) → Phase 1.2 (Protected Routes)
    ↓
Phase 2 (Public Pages) - Can be done in parallel
    ↓
Phase 3 (User Area) - Can be done in parallel with Phase 4
    ↓
Phase 4.1 (Admin Dashboard) → Phase 4.2-4.4 (Admin Management Pages)
    ↓
Phase 4.5 (Admin Navigation)
```

### Cross-Dependencies

-   **Frontend Phase 1** depends on **Backend Phase 1.1**
-   **Frontend Phase 4** depends on **Backend Phase 2**
-   **Frontend Phase 3** can work with existing endpoints (already implemented)

---

## 7. Testing Strategy

### Backend Testing

1. **Unit Tests**:

    - OIDC client functions
    - JWT verification
    - Repository methods
    - Middleware functions

2. **Integration Tests**:

    - Auth flow end-to-end
    - Admin endpoints
    - User enable/disable flow

3. **API Tests**:
    - All endpoints with proper authentication
    - Error cases
    - Edge cases

### Frontend Testing

1. **Component Tests**:

    - Login form
    - Protected routes
    - Admin components

2. **Integration Tests**:

    - Auth flow
    - Admin workflows
    - User management flows

3. **E2E Tests**:
    - Complete user journeys
    - Admin workflows

---

## 8. Security Considerations

### Authentication Security

1. **Token Storage**:

    - Use httpOnly cookies for refresh tokens (recommended)
    - Or secure localStorage with XSS protection
    - Implement CSRF protection

2. **Token Validation**:

    - Always verify JWT signatures
    - Check token expiration
    - Validate issuer and audience
    - Implement token refresh before expiration

3. **OIDC Configuration**:
    - Use HTTPS in production
    - Validate redirect URIs
    - Implement PKCE for public clients

### Authorization Security

1. **Role-Based Access Control**:

    - Always check roles server-side
    - Never trust client-side role checks
    - Implement middleware for role checks

2. **Admin Endpoints**:

    - Require admin middleware on all admin routes
    - Log all admin actions
    - Implement audit trail

3. **User Account Security**:
    - Prevent disabled users from logging in (`disabled = true`)
    - Prevent locked users from logging in (`locked_out = true` and `lockout_end` not expired)
    - Implement account lockout after failed attempts (increment `access_failed_count`)
    - Lock account when `access_failed_count >= MAX_FAILED_ATTEMPTS` (e.g., 5)
    - Set `lockout_end` to current time + lockout duration (e.g., 15 minutes)
    - Reset `access_failed_count` on successful login
    - Log all authentication attempts

---

## 9. Migration Strategy

**Note**: Since this is a brand new project, we're creating a fresh schema. No data migration is needed.

### Schema Initialization

1. **Initial Migration**:

    - Create single comprehensive migration with all tables
    - Include default roles (user, admin, moderator)
    - Set up indexes and constraints
    - No legacy data to migrate

2. **First Admin User**:
    - Create first admin user manually via database script or admin signup flow
    - Assign "admin" role via `user_roles` table
    - Or implement special first-user-admin flow during initial setup

### Feature Flags

1. **Gradual Rollout**:

    - Use feature flags to enable/disable OIDC auth
    - Monitor error rates during rollout
    - Implement proper logging for debugging

2. **Rollback Plan**:
    - Keep migration rollback scripts ready
    - Implement feature flags for easy disable
    - Monitor error rates during migration

---

## 10. Documentation Requirements

### API Documentation

1. **OpenAPI/Swagger**:

    - Document all new endpoints
    - Include authentication requirements
    - Include request/response examples

2. **Authentication Guide**:
    - OIDC setup instructions
    - Environment variable documentation
    - Token refresh flow documentation

### User Documentation

1. **Admin Guide**:

    - How to manage users
    - How to manage providers
    - How to manage models
    - How to use scan features (when implemented)

2. **Developer Guide**:
    - How to add new OIDC providers
    - How to implement model scanning
    - How to extend admin functionality

---

## 11. Future Enhancements (Placeholders)

### Model Scanning Implementation

**Phase**: Future (after MVP)

**Tasks**:

1. **Provider-Specific Scanners**:

    - OpenAI API scanner
    - Anthropic API scanner
    - Google API scanner
    - Generic OAuth2 API scanner

2. **Scan Logic**:

    - Fetch available models from provider API
    - Compare with database models
    - Detect new models
    - Detect deprecated models
    - Detect pricing changes
    - Detect capability changes

3. **Scheduled Scans**:
    - Implement background job for periodic scans
    - Configurable scan intervals per provider
    - Email notifications for significant changes

### Additional Admin Features

1. **Audit Logging**:

    - Log all admin actions
    - User action history
    - Model change history

2. **Analytics Dashboard**:

    - User growth charts
    - Model usage statistics
    - Provider usage statistics
    - Cost analysis

3. **Bulk Operations**:
    - Bulk user enable/disable
    - Bulk model enable/disable
    - Bulk provider operations

---

## 12. Estimated Timeline

### Backend Development

-   **Phase 1**: Authentication System (2-3 weeks)
    -   OAuth2/OIDC integration: 1-2 weeks
    -   User management (ASP.NET Identity style): 3-5 days
    -   Rate limiting: 2-3 days
-   **Phase 3**: Database Schema (1-2 days)
    -   Create fresh schema migration
-   **Phase 2**: Admin APIs (2-3 weeks)

    -   Admin middleware: 1 day
    -   User management API: 1 week
    -   Provider management API: 3-5 days
    -   Model management API: 1 week
    -   Dashboard API: 2-3 days

-   **Phase 4**: Route organization (2-3 days)

**Total Backend**: 4-6 weeks

### Frontend Development

-   **Phase 1**: Authentication UI (1 week)
-   **Phase 2**: Public Pages (3-5 days)
-   **Phase 3**: User Area Updates (1 week)
-   **Phase 4**: Admin Area (2-3 weeks)
    -   Dashboard: 3-5 days
    -   User management: 1 week
    -   Provider management: 3-5 days
    -   Model management: 1 week
    -   Navigation: 2-3 days

**Total Frontend**: 4-5 weeks

### Parallel Development

With proper coordination, Frontend and Backend can be developed in parallel:

-   **Frontend Phase 1** waits for **Backend Phase 1.1**
-   **Frontend Phase 4** waits for **Backend Phase 2**
-   Other phases can proceed independently

**Total Project Timeline**: 5-6 weeks with parallel development

---

## 13. Success Criteria

### Authentication System

-   [ ] Users can log in via OIDC
-   [ ] JWT tokens are properly verified
-   [ ] Token refresh works correctly
-   [ ] Disabled users cannot log in (`disabled = true`)
-   [ ] Locked users cannot log in (`locked_out = true`)
-   [ ] Failed login attempts increment `access_failed_count`
-   [ ] Accounts lock automatically after max failed attempts
-   [ ] Rate limiting works correctly for all endpoints
-   [ ] Admin users can manage accounts

### Admin System

-   [ ] Admin dashboard displays statistics
-   [ ] Admins can list/create/edit/delete users
-   [ ] Admins can enable/disable user accounts (`disabled` field)
-   [ ] Admins can lock/unlock user accounts (`locked_out` field)
-   [ ] Admins can manage user roles (add/remove roles)
-   [ ] Admins can manage AI providers
-   [ ] Admins can manage AI models
-   [ ] Admin routes are properly protected
-   [ ] Non-admin users cannot access admin routes

### User Experience

-   [ ] Public pages are accessible without auth
-   [ ] User pages require authentication
-   [ ] Admin pages require admin role
-   [ ] All pages are responsive
-   [ ] Error messages are clear and helpful

---

## 14. Notes & Considerations

### Research Findings

1. **LibreChat**: Has basic admin features but no comprehensive admin dashboard. First account is admin. Admin panel is planned for 2025.

2. **T3 Chat**: Uses T3 stack but doesn't have documented admin implementation. The T3 stack provides good foundation for building admin features.

3. **OAuth2 Libraries**: `oauth2` crate is the most popular and robust choice. `openidconnect` crate provides OIDC-specific features.

### Technical Decisions

1. **Token Storage**: Recommend httpOnly cookies for refresh tokens, localStorage for access tokens (or httpOnly cookies for both).

2. **Database Design**:

    - Separate `ai_providers` table allows for better provider management and future extensibility.
    - Separate `roles` and `user_roles` tables (ASP.NET Identity style) allows flexible role management.
    - Using `disabled` field (default false) instead of `is_enabled` follows negative flag pattern.

3. **Account Management**:

    - Using `disabled` field (default false) allows admins to disable accounts without deleting them.
    - Using `locked_out` and `lockout_end` fields implements account lockout similar to ASP.NET Identity.
    - Using `access_failed_count` tracks failed login attempts for automatic lockout.

4. **Rate Limiting**: Using `tower-governor` provides flexible rate limiting per endpoint and user type.

5. **Model Scanning**: Placeholder implementation allows for future enhancement without blocking current development.

6. **Crate Versions**: Using major versions only (e.g., `oauth2 = "4"`) allows automatic patch updates while maintaining compatibility.

### Open Questions

1. **OIDC Provider Selection**: Which OIDC provider will be used? (Auth0, Keycloak, Okta, custom, etc.)
2. **Token Expiry**: What should be the access token and refresh token expiry times?
3. **First Admin**: How should the first admin account be created? (Manual DB insert, special signup flow, etc.)
4. **Model Scanning Frequency**: How often should model scanning run? (Manual, scheduled, on-demand)
5. **Lockout Configuration**: What should be the maximum failed attempts before lockout? (Default: 5)
6. **Lockout Duration**: How long should accounts be locked? (Default: 15 minutes)
7. **Rate Limit Values**: What should be the default rate limits for each endpoint type?

---

## 15. Next Steps

1. **Review and Approve Plan**: Team review of this plan
2. **Set Up OIDC Provider**: Configure OIDC provider (Auth0, Keycloak, etc.)
3. **Create Initial Database Migration**: Create fresh schema migration with all tables
4. **Start Backend Phase 1**: Begin OAuth2/OIDC integration
5. **Start Frontend Phase 1**: Begin authentication UI updates
6. **Set Up Development Environment**: Ensure all developers have access to OIDC provider for testing

---

**Document Version**: 1.0  
**Last Updated**: 2025-01-XX  
**Author**: Development Team  
**Status**: Draft - Pending Review
