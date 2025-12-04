# Rust Backend Server

The Rust backend for T3Chat. Built with Axum, async Diesel, and OIDC Authentication. This crate replaces the earlier Node.js Hono server while keeping the same HTTP surface area and data model.

## Status

✅ Works in local development with embedded PostgreSQL  
✅ Production-ready authentication via OIDC/JWKS  
✅ Compatible with external PostgreSQL providers (Supabase, self-hosted, etc.)

## Features

-   **Axum Web Framework**: Fast, ergonomic async web framework
-   **Diesel + diesel_async**: Type-safe PostgreSQL ORM with async pooling
-   **Diesel migrations**: Embedded SQL migrations (`server/migrations`) with optional auto-run
-   **OIDC Authentication**: JWKS-based JWT verification for production
-   **Repository pattern**: Dedicated repositories in `db/repositories` for query encapsulation
-   **Graceful Shutdown**: Handles SIGINT/SIGTERM signals properly
-   **Structured Logging**: `tracing` + daily rotating file appender
-   **Static assets**: Serves `wwwroot` alongside API routes

## API Routes

High-level summary of major routes (see `src/main.rs` and `src/api` for full details).

-   `GET /health` – Service & database health check
-   `GET /swagger-ui` – Swagger UI (development & staging only)

**Authentication (`/api/v1/auth`)**

-   `POST /api/v1/auth/local/login` – Local username/email + password login
-   `POST /api/v1/auth/logout` – Logical logout (JWT is client-managed)
-   `GET /api/v1/auth/config` – Returns `{ oidc_enabled: boolean }`
-   `GET /api/v1/auth/login` – Start OIDC login (only when OIDC is configured)
-   `GET /api/v1/auth/callback` – OIDC callback handler
-   `POST /api/v1/auth/refresh` – Refresh OIDC access token
-   `GET /api/v1/auth/me` – Authenticated user info (combined local/OIDC)

**User & models**

-   `GET /api/v1/me` – Fetch authenticated user profile
-   `PUT /api/v1/me` – Update authenticated user profile
-   `GET /api/v1/models` – List AI models visible to the current user
-   `GET /api/v1/models/all` – List all models in the catalog
-   `GET /api/v1/models/{id}` – Fetch a single AI model
-   `GET /api/v1/models/my` – List models enabled for the current user
-   `POST /api/v1/models/{id}/enable` – Enable a model for the current user
-   `POST /api/v1/models/{id}/disable` – Disable a model for the current user

**Chats & messages (`/api/v1/chats`)**

-   `GET /api/v1/chats` – List chats for the authenticated user
-   `POST /api/v1/chats` – Create a chat
-   `GET /api/v1/chats/{id}` – Retrieve a chat
-   `PUT /api/v1/chats/{id}` – Update a chat
-   `DELETE /api/v1/chats/{id}` – Delete a chat
-   `GET /api/v1/chats/{id}/messages` – List chat messages
-   `POST /api/v1/chats/{id}/messages` – Create a message
-   `DELETE /api/v1/chats/{id}/messages` – Clear all messages for a chat
-   `PUT /api/v1/chats/{chat_id}/messages/{id}` – Update a message
-   `DELETE /api/v1/chats/{chat_id}/messages/{id}` – Delete a single message

**Chat completion (`/api/v1/chat`)**

-   `POST /api/v1/chat` – Synchronous chat completion
-   `POST /api/v1/chat/stream` – Streaming chat completion (Server-Sent Events)

**User API keys & features**

-   `GET /api/v1/keys` – List user API keys
-   `POST /api/v1/keys` – Create a new API key
-   `DELETE /api/v1/keys/{id}` – Delete an API key
-   `GET /api/v1/features` – List feature flags for the current user
-   `PUT /api/v1/features/{feature}` – Toggle a feature flag

**Config & admin**

-   `GET /api/v1/config/startup` – Startup config (used by the UI)
-   `GET /api/v1/config/models` – Model catalog derived from `t3chat.yaml`
-   `GET /api/v1/admin/...` – Admin APIs for users, providers, models, and dashboard metrics

## API Documentation

Interactive Swagger UI is available at `http://localhost:<port>/swagger-ui` when the server runs in the `development` or `staging` environments. The OpenAPI spec is served from `/swagger-ui/openapi.json`.

## Environment Variables

The server reads environment variables from `.env` files. See [`variables.md`](../variables.md) for a complete reference of all environment variables.

### Required Variables

-   `DATABASE_URL` – PostgreSQL connection string
-   `CORS_ORIGINS` – Comma-separated list of allowed CORS origins
-   `JWT_SECRET` – Secret key for local JWT token signing (required for session tokens)

### Optional Variables

-   `PORT` – Overrides the listening port (defaults to 3000 or `--port` CLI argument)
-   `APP_ENV` – Application environment: `development`, `staging`, or `release` (defaults to `development`)
-   `DEBUG_ROUTES` – Enable route debugging middleware (`true`/`false`, defaults to `false`)
-   `JWT_EXPIRY_SECONDS` – Lifetime of JWT session tokens in seconds (defaults to `3600`)
-   `FRONTEND_URL` – Base URL of the frontend used for OIDC redirects (defaults to `http://localhost:5173`)
-   `T3CHAT_CONFIG` – Optional path to a custom `t3chat.yaml` (defaults to `./t3chat.yaml` in the current working directory)
-   `OIDC_ISSUER_URL` – OIDC provider issuer URL (optional - only needed if enabling OIDC authentication)
-   `OIDC_CLIENT_ID` – OIDC client ID (optional - only needed if enabling OIDC authentication)
-   `OIDC_CLIENT_SECRET` – OIDC client secret (optional - only needed if enabling OIDC authentication)
-   `OIDC_REDIRECT_URI` – OIDC callback redirect URI (optional - only needed if enabling OIDC authentication)

### Environment File Loading

Environment variables are loaded from `.env` files based on `APP_ENV` using the following priority order:

1. `server/.env.<APP_ENV>.local` (highest priority)
2. `server/.env.<APP_ENV>`
3. `server/.env.local`
4. `server/.env` (lowest priority)

Create separate files such as `.env.development`, `.env.staging`, and `.env.release` to isolate configuration per environment. When running locally with the provided scripts you can pass `--env staging` (or set `APP_ENV=staging`) to switch to another configuration.

### Example Configuration

**Development (`server/.env.development`):**
```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/t3chat
CORS_ORIGINS=http://localhost:3010,http://localhost:3000
APP_ENV=development
PORT=3000
JWT_SECRET=your-secure-jwt-secret

# OIDC Authentication (optional - only needed if you want OIDC login)
# OIDC_ISSUER_URL=https://your-oidc-provider.com
# OIDC_CLIENT_ID=your-client-id
# OIDC_CLIENT_SECRET=your-client-secret
# OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback
```

📖 **For complete environment variable documentation**, see [`variables.md`](../variables.md)

## Running the Server

### Development

```bash
cd server
cargo run
```

### With custom port

```bash
cargo run -- --port 8788
```

### Production Build

```bash
cd server
cargo build --release
./target/release/t3chat-server
```

## Database Setup

The server uses Diesel + `diesel_async` backed by PostgreSQL. Migrations are embedded from the `migrations/` directory and can run automatically during startup.

> **Windows MSVC:** make sure the PostgreSQL client libraries (`libpq.lib`/`libpq.dll`) are installed. See the root `README.md` for details.

### Auto-migration (default behaviour)

When the server boots it will:

1. Ensure the database defined in `DATABASE_URL` exists (creating it if needed)
2. Run any pending Diesel migrations from `migrations/`

Example `.env` snippet:

```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/t3chat
JWT_SECRET=your-secure-jwt-secret

# OIDC Authentication (optional - only needed if you want OIDC login)
# OIDC_ISSUER_URL=https://your-oidc-provider.com
# OIDC_CLIENT_ID=your-client-id
# OIDC_CLIENT_SECRET=your-client-secret
# OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback
```

Then run `cargo run` from `server/`. Auto-migrate is enabled by the call to `db::connect(&database_url, true)` in `main.rs`. Set the second argument to `false` if you want to manage migrations manually.

### Manual migrations (recommended for CI/CD)

#### Windows: Setup libpq via vcpkg (Required for Diesel CLI)

If you're on Windows and need to install `diesel_cli`, you must first install the PostgreSQL client library (`libpq`) using vcpkg:

```powershell
# Clone vcpkg (one-time setup)
git clone https://github.com/microsoft/vcpkg.git C:\vcpkg
cd C:\vcpkg
.\bootstrap-vcpkg.bat

# Install libpq for x64-windows
.\vcpkg install libpq:x64-windows

# Set environment variables (PowerShell - session only)
$env:PQ_LIB_DIR = "C:\vcpkg\installed\x64-windows\lib"
$env:PATH += ";C:\vcpkg\installed\x64-windows\bin"

# Make environment variables permanent (PowerShell as Administrator)
[System.Environment]::SetEnvironmentVariable("PQ_LIB_DIR", "C:\vcpkg\installed\x64-windows\lib", "User")
$currentPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
[System.Environment]::SetEnvironmentVariable("PATH", "$currentPath;C:\vcpkg\installed\x64-windows\bin", "User")

# Verify installation
dir C:\vcpkg\installed\x64-windows\lib\libpq.lib
dir C:\vcpkg\installed\x64-windows\bin\libpq.dll
```

**Updating vcpkg and packages:**

```powershell
# Update vcpkg itself
cd C:\vcpkg
git pull
.\bootstrap-vcpkg.bat

# Check for package updates
.\vcpkg update

# Update all installed packages
.\vcpkg upgrade --no-dry-run

# Or update a specific package
.\vcpkg upgrade libpq:x64-windows --no-dry-run
```

After setting up vcpkg and libpq, you can install Diesel CLI:

```bash
cd server
cargo install diesel_cli --no-default-features --features postgres # one-time setup
diesel migration run
```

You can also create the database manually if necessary:

```sql
CREATE DATABASE t3chat;
```

### Creating new migrations

```bash
cd server
diesel migration generate add_feature_x
# edit migrations/<timestamp>_add_feature_x/up.sql and down.sql
diesel migration run
```

Update models in `src/db/models/` and, if needed, regenerate the Diesel schema with `diesel print-schema > src/db/schema.rs` (or manually update the file to match your changes).

## Differences from Node.js Version

-   Uses Diesel + `diesel_async` instead of Drizzle ORM
-   OIDC token verification with JWKS-based JWT validation (production ready)
-   Repository pattern replaces direct SQL queries from the Hono server
-   Structured logging with `tracing` instead of console logging
-   Graceful shutdown handling for SIGINT/SIGTERM signals
-   No Cloudflare Workers support (standalone deployment only)
-   No Neon serverless driver support (use standard PostgreSQL providers)

## ✅ Implementation Status

### ✅ Production OIDC Authentication - IMPLEMENTED

Full production token verification with JWKS-based JWT verification is now implemented.

**Status:** ✅ Complete  
**Implementation:**

-   Proper JWKS fetching from OIDC provider
-   Token header kid (key ID) extraction
-   RSA key selection and validation
-   Full JWT verification with issuer and audience validation
-   Comprehensive error logging

**Details:** See `src/middleware/auth.rs`

### ✅ Graceful Shutdown - IMPLEMENTED

Server now properly handles SIGINT/SIGTERM signals for graceful shutdown.

**Status:** ✅ Complete  
**Implementation:**

-   Ctrl+C (SIGINT) handling on all platforms
-   SIGTERM handling on Unix-like systems
-   Proper connection cleanup
-   Informative shutdown logging

**Details:** See `src/main.rs`

## Development Status

| Environment     | Status   | Notes                                              |
| --------------- | -------- | -------------------------------------------------- |
| **Development** | ✅ Ready | Full OIDC authentication support                  |
| **Production**  | ✅ Ready | Full JWKS-based authentication + graceful shutdown |

## 🚨 Known Limitations

### Neon Database Not Supported

The Rust server uses standard PostgreSQL protocol and doesn't support Neon's serverless HTTP-based driver.

**Supported databases:**

-   ✅ Standard PostgreSQL (localhost, cloud instances)
-   ✅ Supabase (uses standard Postgres protocol)
-   ❌ Neon Database (requires Node.js-specific serverless driver)
