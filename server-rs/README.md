# Rust Backend Server

A Rust backend server for T3Chat, converted from the Node.js Hono server. Built with Axum, SeaORM, and Firebase Authentication.

## 📋 Review & Compatibility Documentation

**NEW:** Comprehensive comparison with Node.js server available!

- **🎯 [Quick Summary](REVIEW_SUMMARY.md)** - Start here! Overview of compatibility status
- **📊 [Compatibility Matrix](COMPATIBILITY_MATRIX.md)** - Detailed feature-by-feature comparison
- **📝 [Full Comparison Report](COMPARISON_REPORT.md)** - In-depth technical analysis
- **🔧 [Action Plan](ACTION_PLAN.md)** - Step-by-step guide to fix remaining issues

**TL;DR:** ✅ Works perfectly in development | ✅ Production-ready with JWKS-based Firebase auth

## Features

- **Axum Web Framework**: Fast, ergonomic async web framework
- **SeaORM**: Type-safe ORM for PostgreSQL
- **Firebase Authentication**: Full JWKS-based JWT token verification for production
- **Graceful Shutdown**: Handles SIGINT/SIGTERM signals properly
- **Structured Logging**: Using tracing and tracing-subscriber
- **CORS Support**: Enabled for cross-origin requests
- **Health Checks**: Database connectivity and API health endpoints

## API Routes

- `GET /` - Health check endpoint
- `GET /api/v1/hello` - Hello world endpoint
- `GET /api/v1/db-test` - Database connection test
- `GET /api/v1/protected/me` - Get authenticated user info (requires Bearer token)

## Environment Variables

The server expects the following environment variables (can be set via `.env` file):

- `DATABASE_URL` - PostgreSQL connection string (defaults to `postgresql://postgres:password@localhost:5502/postgres`)
- `PORT` - Server port (defaults to `8787`)
- `FIREBASE_PROJECT_ID` - Firebase project ID (required)
- `FIREBASE_AUTH_EMULATOR_HOST` - Firebase Auth emulator host (for development, optional)
- `NODE_ENV` - Set to `development` for emulator mode
- `ALLOW_ANONYMOUS_USERS` - Allow anonymous Firebase users (defaults to `true`)
- `AUTO_MIGRATE` - Set to `true` to automatically run migrations on startup (requires `with-migration` feature)

## Running the Server

### Development

```bash
cd server-rs
cargo run
```

### With custom port

```bash
cargo run -- --port 8788
```

### Production Build

```bash
cargo build --release
./target/release/server-rs
```

## Database Setup

The server uses SeaORM migrations with automatic database creation and schema management.

### Auto-Migration (Recommended for Development)

The server will automatically:
1. Create the database if it doesn't exist (e.g., `t3chat`)
2. Run all pending migrations
3. Create tables and indexes in the `public` schema (default PostgreSQL schema)

Simply set your `DATABASE_URL` in `.env` and run:

```bash
# Example .env file
DATABASE_URL=postgresql://postgres:password@localhost:5432/t3chat

# Run the server (auto_migrate is enabled by default in main.rs)
cargo run
```

The auto-migration runs on startup when `auto_migrate` is `true` in the `db::connect()` call (enabled by default).

### Manual Migration (For Production or Manual Control)

If you prefer to run migrations manually:

```bash
cd migration
cargo run
```

**Note:** Manual migration requires the database to already exist. You can create it with:
```sql
CREATE DATABASE t3chat;
```

### Creating New Migrations

See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for detailed instructions on:
- Running migrations
- Creating new migrations
- Best practices
- Common patterns

### Schema Information

- **Schema**: `public` (default PostgreSQL schema, no custom schema required)
- **Database Name**: Extracted from `DATABASE_URL` (e.g., `t3chat`)
- **Tables**: Created via SeaORM migrations with proper indexes

## Differences from Node.js Version

- Uses SeaORM instead of Drizzle ORM
- Firebase token verification with proper JWKS-based JWT verification for production
- Database connection pooling handled by SeaORM
- Structured logging with tracing instead of console.log
- Graceful shutdown handling for SIGINT/SIGTERM signals
- No Cloudflare Workers support (standalone deployment only)
- No Neon database support (use standard PostgreSQL or Supabase)

## ✅ Implementation Status

### ✅ Production Firebase Authentication - IMPLEMENTED
Full production token verification with JWKS-based JWT verification is now implemented.

**Status:** ✅ Complete  
**Implementation:**
- Proper JWKS fetching from Google's Firebase public keys
- Token header kid (key ID) extraction
- RSA key selection and validation
- Full JWT verification with issuer and audience validation
- Comprehensive error logging

**Details:** See `src/middleware/auth.rs`

### ✅ Graceful Shutdown - IMPLEMENTED
Server now properly handles SIGINT/SIGTERM signals for graceful shutdown.

**Status:** ✅ Complete  
**Implementation:**
- Ctrl+C (SIGINT) handling on all platforms
- SIGTERM handling on Unix-like systems
- Proper connection cleanup
- Informative shutdown logging

**Details:** See `src/main.rs`

## Development Status

| Environment | Status | Notes |
|-------------|--------|-------|
| **Development** | ✅ Ready | Full compatibility with Firebase emulator |
| **Production** | ✅ Ready | Full JWKS-based authentication + graceful shutdown |

## 🚨 Known Limitations

### Neon Database Not Supported
The Rust server uses standard PostgreSQL protocol and doesn't support Neon's serverless HTTP-based driver.

**Supported databases:**
- ✅ Standard PostgreSQL (localhost, cloud instances)
- ✅ Supabase (uses standard Postgres protocol)
- ❌ Neon Database (requires Node.js-specific serverless driver)

**For detailed compatibility analysis, see [REVIEW_SUMMARY.md](REVIEW_SUMMARY.md)**

