# Rust Backend Server

The Rust backend for T3Chat. Built with Axum, async Diesel, and Firebase Authentication. This crate replaces the earlier Node.js Hono server while keeping the same HTTP surface area and data model.

## Status

✅ Works in local development with embedded PostgreSQL and Firebase emulator  
✅ Production-ready authentication via JWKS  
✅ Compatible with external PostgreSQL providers (Supabase, self-hosted, etc.)

## Features

-   **Axum Web Framework**: Fast, ergonomic async web framework
-   **Diesel + diesel_async**: Type-safe PostgreSQL ORM with async pooling
-   **Diesel migrations**: Embedded SQL migrations (`server/migrations`) with optional auto-run
-   **Firebase Authentication**: JWKS-based JWT verification for production and emulator support
-   **Repository pattern**: Dedicated repositories in `db/repositories` for query encapsulation
-   **Graceful Shutdown**: Handles SIGINT/SIGTERM signals properly
-   **Structured Logging**: `tracing` + daily rotating file appender
-   **Static assets**: Serves `wwwroot` alongside API routes

## API Routes

-   `GET /` – Service & database health check
-   `GET /api/v1/models` – List AI models (public)
-   `GET /api/v1/models/{id}` – Fetch a single AI model (public)
-   `GET /api/v1/chats` – List chats for the authenticated user
-   `POST /api/v1/chats` – Create a chat
-   `GET /api/v1/chats/{id}` – Retrieve a chat
-   `PUT /api/v1/chats/{id}` – Update a chat
-   `DELETE /api/v1/chats/{id}` – Delete a chat
-   `GET /api/v1/chats/{id}/messages` – List chat messages
-   `POST /api/v1/chats/{id}/messages` – Create a message
-   `POST /api/v1/chat` – Synchronous chat completion
-   `POST /api/v1/chat/stream` – Streaming chat completion (Server-Sent Events)
-   `GET /api/v1/me` – Fetch authenticated user profile
-   `PUT /api/v1/me` – Update authenticated user profile
-   `GET /api/v1/user-api-keys` – List user API keys
-   `POST /api/v1/user-api-keys` – Create a new API key
-   `DELETE /api/v1/user-api-keys/{id}` – Delete an API key

## API Documentation

Interactive Swagger UI is available at `http://localhost:<port>/swagger-ui` when the server runs in the `development` or `staging` environments. The OpenAPI spec is served from `/swagger-ui/openapi.json`.

## Environment Variables

The server reads these environment variables (typically via `.env` during development):

-   `DATABASE_URL` – PostgreSQL connection string (required)
-   `FIREBASE_PROJECT_ID` – Firebase project ID (required)
-   `FIREBASE_AUTH_EMULATOR_HOST` – Host/port for the Firebase Auth emulator (optional)
-   `PORT` – Overrides the listening port (otherwise defaults to 3000 or `--port`)
-   `CORS_ORIGINS` – Comma-separated list of allowed origins (defaults to `http://localhost`)
-   `APP_ENV` – Optional override for the active environment (`development`, `staging`, or `release`); defaults to `development`

### Environment files

Environment variables are loaded from `.env` files based on `APP_ENV` using the following priority order:

1. `server/.env.<APP_ENV>.local`
2. `server/.env.<APP_ENV>`
3. `server/.env.local`
4. `server/.env`

Create separate files such as `.env.development`, `.env.staging`, and `.env.release` to isolate configuration per environment. When running locally with the provided scripts you can pass `--env staging` (or set `APP_ENV=staging`) to switch to another configuration.

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
FIREBASE_PROJECT_ID=t3chat-dev
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
-   Firebase token verification with JWKS-based JWT validation (production ready)
-   Repository pattern replaces direct SQL queries from the Hono server
-   Structured logging with `tracing` instead of console logging
-   Graceful shutdown handling for SIGINT/SIGTERM signals
-   No Cloudflare Workers support (standalone deployment only)
-   No Neon serverless driver support (use standard PostgreSQL providers)

## ✅ Implementation Status

### ✅ Production Firebase Authentication - IMPLEMENTED

Full production token verification with JWKS-based JWT verification is now implemented.

**Status:** ✅ Complete  
**Implementation:**

-   Proper JWKS fetching from Google's Firebase public keys
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
| **Development** | ✅ Ready | Full compatibility with Firebase emulator          |
| **Production**  | ✅ Ready | Full JWKS-based authentication + graceful shutdown |

## 🚨 Known Limitations

### Neon Database Not Supported

The Rust server uses standard PostgreSQL protocol and doesn't support Neon's serverless HTTP-based driver.

**Supported databases:**

-   ✅ Standard PostgreSQL (localhost, cloud instances)
-   ✅ Supabase (uses standard Postgres protocol)
-   ❌ Neon Database (requires Node.js-specific serverless driver)
