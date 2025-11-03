# Rust Backend Server

A Rust backend server for volo-app, converted from the Node.js Hono server. Built with Axum, SeaORM, and Firebase Authentication.

## Features

- **Axum Web Framework**: Fast, ergonomic async web framework
- **SeaORM**: Type-safe ORM for PostgreSQL
- **Firebase Authentication**: JWT token verification
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

The server expects:
1. PostgreSQL database running
2. `app` schema created (run the setup script from the Node.js server: `node server/scripts/setup-private-schema.mjs`)
3. Users table created in the `app` schema

## Differences from Node.js Version

- Uses SeaORM instead of Drizzle ORM
- Firebase token verification uses simplified emulator mode (production verification needs proper JWKS implementation)
- Database connection pooling handled by SeaORM
- Structured logging with tracing instead of console.log

## Development Notes

The Firebase token verification in production mode currently uses a simplified approach. For production deployments, consider:
- Implementing proper JWKS key rotation handling
- Caching JWKS keys
- Using a dedicated Firebase Admin SDK Rust library when available

