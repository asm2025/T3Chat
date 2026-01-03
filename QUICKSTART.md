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

## Step 2: Configure Authentication

T3Chat uses **local username/password authentication by default**. OIDC authentication (Google OAuth, Firebase, Auth0, Keycloak) is optional and can be enabled by configuring the OIDC environment variables below.

### Option A: Local Authentication Only (Default - No Additional Setup Required)

Local authentication works out of the box with the default admin user. Skip to Step 3 if you only want local authentication.

### Option B: Enable OIDC Authentication (Optional)

If you want to enable OIDC authentication, follow these steps. Google OAuth is the quickest to set up:

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

## Step 3: Configure AI Providers & Models (`t3chat.yaml`)

The Rust backend reads provider and model configuration from a YAML file.

1. From the project root, copy the example config:

    ```bash
    cp t3chat.example.yaml t3chat.yaml
    ```

2. Set provider API key environment variables in your shell or OS (only the providers you care about are required):

    ```bash
    # OpenAI (for GPT-4o, GPT-4 Turbo, etc.)
    export OPENAI_API_KEY=your-openai-key

    # Anthropic (for Claude 3.5 / Claude 3)
    export ANTHROPIC_API_KEY=your-anthropic-key

    # Google Gemini
    export GOOGLE_API_KEY=your-gemini-key

    # OpenRouter (optional)
    export OPENROUTER_API_KEY=your-openrouter-key

    # ChatLLM (optional, backend-managed key shared by all users)
    export ABACUS_API_KEY=your-chatllm-api-key
    ```

3. If you want to keep the config file elsewhere, set `T3CHAT_CONFIG` to point to it:

    ```bash
    export T3CHAT_CONFIG=/absolute/path/to/t3chat.yaml
    ```

> 💡 You can comment out or remove providers from `t3chat.yaml` if you don't have keys for them yet.

### ChatLLM (optional)

If you want to use ChatLLM (an OpenAI-compatible provider) as a backend-managed provider:

-   Ensure your `t3chat.yaml` has a custom endpoint named `ChatLLM` with:
    -   `apiKey: "${ABACUS_API_KEY}"`
-   `baseURL` set directly in the YAML.
-   Restart the backend so the provider appears in the UI.

## Step 4: Configure Environment Variables

Create a `.env` file in the `server/` directory:

```bash
cd server
cat > .env << 'EOF'
# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/t3chat

# CORS (must include your frontend URL)
CORS_ORIGINS=http://localhost:3010

# Frontend URL used for OIDC redirect
FRONTEND_URL=http://localhost:3010

# JWT Session Token Secret (generate with: openssl rand -base64 32)
JWT_SECRET=your-super-secure-random-jwt-secret-key-min-32-chars

# Optional: OIDC Authentication (only needed if you want OIDC login)
# Leave these commented out if you only want local authentication
# OIDC_ISSUER_URL=https://accounts.google.com
# OIDC_CLIENT_ID=YOUR_CLIENT_ID.apps.googleusercontent.com
# OIDC_CLIENT_SECRET=YOUR_CLIENT_SECRET
# OIDC_REDIRECT_URI=http://localhost:3000/api/auth/callback

# Optional: Development settings
APP_ENV=development
PORT=3000
DEBUG_ROUTES=false
EOF
```

**Note**: If you want to enable OIDC authentication, uncomment and fill in the OIDC variables above. Otherwise, local authentication will work without them.

Generate a secure JWT secret:

```bash
# Using OpenSSL
openssl rand -base64 32

# Or using Node.js
node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"
```

Copy the generated secret and replace `your-super-secure-random-jwt-secret-key-min-32-chars` in the `.env` file.

## Step 5: Install Dependencies

### Backend

```bash
cd server
cargo build
```

### Frontend

```bash
cd client
pnpm install
```

## Step 6: Run the Application

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
cd client
pnpm dev -- --api-url http://localhost:3000
```

The UI will start on `http://localhost:3010` and talk to the backend at `http://localhost:3000/api`.

## Step 7: Log In

### Local Login (Default - Username & Password)

1. Open your browser to `http://localhost:3010`
2. The login form will show username/password fields by default
3. Use the default admin credentials:
    - **Username**: `admin`
    - **Password**: `P@$$w0rd`
4. You'll be logged in immediately

### OIDC Login (Optional - Only if Configured)

If you configured OIDC in Step 2, you'll see an "Or" separator and a "Login with OIDC" button:

1. Click "Login with OIDC" in the login form
2. Complete the OAuth flow
3. You'll be redirected back to T3Chat

**⚠️ Security Note**: The default admin password should be changed immediately after first login!

## What's Next?

-   **Change Admin Password**: If using local auth, change the default admin password immediately
-   **Add AI Provider Keys**: Go to Settings to add your OpenAI, Anthropic, or Google AI API keys
-   **Start Chatting**: Create a new chat and select an AI model
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
-   Make sure the OIDC redirect URI matches exactly: `http://localhost:3000/api/auth/callback`

### PostgreSQL Connection Failed

-   Verify PostgreSQL is running: `pg_isready` or check Docker: `docker ps`
-   Test connection: `psql -U postgres -h localhost -p 5432 -d t3chat`
-   Check the `DATABASE_URL` format matches: `postgresql://user:password@host:port/database`

## Authentication Options

T3Chat supports two authentication methods:

1. **Local Authentication** (Primary - Default):

    - Uses username/password stored in the database
    - Default admin user: `admin` / `P@$$w0rd`
    - No external services required
    - Works out of the box - no additional configuration needed
    - Best for development, testing, and self-hosted deployments

2. **OIDC Authentication** (Optional):
    - Supports Google OAuth, Firebase, Auth0, Keycloak, and other OIDC providers
    - Only appears in the login form if configured
    - Enable by setting OIDC environment variables (see Step 4)
    - See configuration examples below

## Alternative OIDC Providers

### Auth0

1. Sign up at [auth0.com](https://auth0.com/)
2. Create a "Regular Web Application"
3. Configure:
    - Allowed Callback URLs: `http://localhost:3000/api/auth/callback`
    - Allowed Logout URLs: `http://localhost:3010`
    - Allowed Web Origins: `http://localhost:3010`
4. Update `.env`:

```bash
OIDC_ISSUER_URL=https://YOUR_DOMAIN.auth0.com
OIDC_CLIENT_ID=your-auth0-client-id
OIDC_CLIENT_SECRET=your-auth0-client-secret
```

### Firebase (Google Cloud Identity Platform)

Firebase Authentication can be used via Google Cloud Identity Platform, which supports OIDC:

1. **Create a Firebase Project**:

    - Go to [Firebase Console](https://console.firebase.google.com/)
    - Create a new project or select an existing one
    - Note your project ID

2. **Enable Google Cloud Identity Platform**:

    - In Firebase Console, go to "Authentication" > "Providers"
    - Enable "Google Cloud Identity Platform" (if not already enabled)
    - This enables OIDC support for your Firebase project

3. **Configure OAuth Consent Screen** (in Google Cloud Console):

    - Go to [Google Cloud Console](https://console.cloud.google.com/)
    - Select your Firebase project
    - Navigate to "APIs & Services" > "OAuth consent screen"
    - Configure the consent screen (use "External" for testing)

4. **Create OAuth 2.0 Client**:

    - Navigate to "APIs & Services" > "Credentials"
    - Click "Create Credentials" > "OAuth 2.0 Client ID"
    - Choose "Web application"
    - Add authorized redirect URI: `http://localhost:3000/api/auth/callback`
    - Add authorized JavaScript origin: `http://localhost:3010`
    - Copy the Client ID and Client Secret

5. **Update `.env`**:

```bash
OIDC_ISSUER_URL=https://securetoken.google.com/YOUR_PROJECT_ID
OIDC_CLIENT_ID=your-oauth-client-id.apps.googleusercontent.com
OIDC_CLIENT_SECRET=your-oauth-client-secret
OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback
```

**Note**: Replace `YOUR_PROJECT_ID` with your Firebase project ID. The issuer URL format may vary depending on your Firebase configuration. You can verify the correct issuer URL by checking the `.well-known/openid-configuration` endpoint.

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
    - Valid redirect URIs: `http://localhost:3000/api/auth/callback`
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
-   Check [`client/README.md`](client/README.md) for frontend-specific details
-   Review the main [`README.md`](README.md) for architecture and deployment information

---

**Last Updated**: 2025-12-02  
**Version**: 1.0
