# T3Chat Quick Start Guide

This guide will help you get T3Chat up and running in under 10 minutes.

## Prerequisites

-   [Node.js](https://nodejs.org/) (v18 or later)
-   [pnpm](https://pnpm.io/) (`npm install -g pnpm`)
-   [Rust](https://rustup.rs/) (latest stable)
-   [PostgreSQL](https://www.postgresql.org/) (v14 or later, or use Docker)

## Step 1: Set Up PostgreSQL

### Option A: Use Docker (Easiest)

```bash
docker run --name postgres -e POSTGRES_PASSWORD=password -e POSTGRES_DB=t3chat -p 5432:5432 -d
```

### Option B: Use Local PostgreSQL

Create a database named `t3chat`:

```bash
psql -U postgres
CREATE DATABASE t3chat;
\q
```

## Step 2: Set Up OIDC Authentication

T3Chat requires an OIDC provider for authentication. Google OAuth is the quickest to set up:

### Using Google OAuth (Recommended)

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Navigate to "APIs & Services" > "Credentials"
4. Click "Create Credentials" > "OAuth 2.0 Client ID"
5. Configure the OAuth consent screen (use "External" for testing)
6. Choose "Web application" as the application type
7. Add these authorized redirect URIs:
    - `http://localhost:3000/api/v1/auth/callback`
8. Add these authorized JavaScript origins:
    - `http://localhost:3010`
9. Copy the "Client ID" and "Client secret"

## Step 3: Configure Environment Variables

Create a `.env` file in the `server/` directory:

```bash
cd server
cat > .env << 'EOF'
# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/t3chat

# CORS (must include your frontend URL)
CORS_ORIGINS=http://localhost:3010

# OIDC Authentication (Google)
OIDC_ISSUER_URL=https://accounts.google.com
OIDC_CLIENT_ID=YOUR_CLIENT_ID.apps.googleusercontent.com
OIDC_CLIENT_SECRET=YOUR_CLIENT_SECRET
OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback

# JWT Session Token Secret (generate with: openssl rand -base64 32)
JWT_SECRET=your-super-secure-random-jwt-secret-key-min-32-chars

# Optional: Development settings
APP_ENV=development
PORT=3000
DEBUG_ROUTES=false
EOF
```

**Important**: Replace `YOUR_CLIENT_ID` and `YOUR_CLIENT_SECRET` with your actual Google OAuth credentials!

Generate a secure JWT secret:

```bash
# Using OpenSSL
openssl rand -base64 32

# Or using Node.js
node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"
```

Copy the generated secret and replace `your-super-secure-random-jwt-secret-key-min-32-chars` in the `.env` file.

## Step 4: Install Dependencies

### Backend

```bash
cd server
cargo build
```

### Frontend

```bash
cd ui
pnpm install
```

## Step 5: Run the Application

### Start Backend (from project root)

```bash
cd server
cargo run
```

The server will:

-   Auto-create the database if needed
-   Run all migrations automatically
-   Start listening on `http://localhost:3000`

### Start Frontend (in a new terminal)

```bash
cd ui
pnpm dev
```

The UI will start on `http://localhost:3010`

## Step 6: Log In

You can log in using either OIDC or local authentication:

### Option A: OIDC Login (Google OAuth)

1. Open your browser to `http://localhost:3010`
2. Click "Login with OIDC"
3. Complete the OAuth flow
4. You'll be redirected back to T3Chat

### Option B: Local Login (Username & Password)

1. Open your browser to `http://localhost:3010`
2. Click "Login with Username & Password"
3. Use the default admin credentials:
   - **Username**: `admin`
   - **Password**: `P@$$w0rd`
4. You'll be logged in immediately

**⚠️ Security Note**: The default admin password should be changed immediately after first login!

## What's Next?

-   **Change Admin Password**: If using local auth, change the default admin password immediately
-   **Add AI Provider Keys**: Go to Settings to add your OpenAI, Anthropic, or Google AI API keys
-   **Start Chatting**: Create a new conversation and select an AI model
-   **Explore Features**: Try agents, presets, file uploads, and more

## Troubleshooting

### Error: "Failed to get OIDC_ISSUER_URL"

-   Make sure you created the `server/.env` file
-   Verify all OIDC variables are set correctly
-   Check that there are no extra spaces or quotes around the values

### Error: "DATABASE_URL is not set"

-   Ensure the `.env` file is in the `server/` directory (not the project root)
-   Verify PostgreSQL is running

### Error: "CORS error" when logging in

-   Check that `CORS_ORIGINS` includes `http://localhost:3010`
-   Verify the backend is running on port 3000
-   Make sure the OIDC redirect URI matches exactly: `http://localhost:3000/api/v1/auth/callback`

### PostgreSQL Connection Failed

-   Verify PostgreSQL is running: `pg_isready` or check Docker: `docker ps`
-   Test connection: `psql -U postgres -h localhost -p 5432 -d t3chat`
-   Check the `DATABASE_URL` format matches: `postgresql://user:password@host:port/database`

## Authentication Options

T3Chat supports two authentication methods:

1. **Local Authentication** (simplest for testing):
   - Uses username/password stored in the database
   - Default admin user: `admin` / `P@$$w0rd`
   - No external services required
   - Best for development and testing

2. **OIDC Authentication** (recommended for production):
   - Supports Google OAuth, Auth0, Keycloak, and other OIDC providers
   - More secure for production environments
   - See configuration examples below

## Alternative OIDC Providers

### Auth0

1. Sign up at [auth0.com](https://auth0.com/)
2. Create a "Regular Web Application"
3. Configure:
    - Allowed Callback URLs: `http://localhost:3000/api/v1/auth/callback`
    - Allowed Logout URLs: `http://localhost:3010`
    - Allowed Web Origins: `http://localhost:3010`
4. Update `.env`:

```bash
OIDC_ISSUER_URL=https://YOUR_DOMAIN.auth0.com
OIDC_CLIENT_ID=your-auth0-client-id
OIDC_CLIENT_SECRET=your-auth0-client-secret
```

### Keycloak (Self-Hosted)

1. Run Keycloak:

```bash
docker run -p 8080:8080 -e KEYCLOAK_ADMIN=admin -e KEYCLOAK_ADMIN_PASSWORD=admin quay.io/keycloak/keycloak:latest start-dev
```

2. Access `http://localhost:8080` and login with admin/admin
3. Create a realm (e.g., "t3chat")
4. Create a client:
    - Client ID: `t3chat-client`
    - Client authentication: ON
    - Valid redirect URIs: `http://localhost:3000/api/v1/auth/callback`
    - Web origins: `http://localhost:3010`
5. Get the client secret from the "Credentials" tab
6. Update `.env`:

```bash
OIDC_ISSUER_URL=http://localhost:8080/realms/t3chat
OIDC_CLIENT_ID=t3chat-client
OIDC_CLIENT_SECRET=your-keycloak-client-secret
```

## Need More Help?

-   See [`variables.md`](variables.md) for complete environment variable documentation
-   Check [`server/README.md`](server/README.md) for backend-specific details
-   Check [`ui/README.md`](ui/README.md) for frontend-specific details
-   Review the main [`README.md`](README.md) for architecture and deployment information

---

**Last Updated**: 2025-12-02  
**Version**: 1.0
