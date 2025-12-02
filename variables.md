# Environment Variables Reference

This document provides a comprehensive reference for all environment variables used in T3Chat across backend and frontend applications.

## 📋 Table of Contents

-   [Backend (Server) Variables](#backend-server-variables)
-   [Frontend (UI) Variables](#frontend-ui-variables)
-   [Environment File Loading Order](#environment-file-loading-order)
-   [Quick Reference](#quick-reference)

---

## Backend (Server) Variables

### Required Variables

#### `DATABASE_URL`

-   **Description**: PostgreSQL connection string
-   **Format**: `postgresql://[user]:[password]@[host]:[port]/[database]`
-   **Example**: `postgresql://postgres:password@localhost:5432/t3chat`
-   **Required**: ✅ Yes
-   **Default**: None
-   **Notes**: Used for database connections. Supports standard PostgreSQL and Supabase.

#### `CORS_ORIGINS`

-   **Description**: Comma-separated list of allowed CORS origins
-   **Format**: Comma-separated URLs
-   **Example**: `http://localhost:3010,http://localhost:3000`
-   **Required**: ✅ Yes
-   **Default**: `http://localhost`
-   **Notes**: Must include at least one valid origin. Used for CORS middleware configuration.

#### `OIDC_ISSUER_URL`

-   **Description**: OIDC provider issuer URL
-   **Format**: URL
-   **Example**: `https://accounts.google.com`
-   **Required**: ✅ Yes
-   **Default**: None
-   **Notes**:
    -   Used for OIDC discovery and token verification
    -   Must support OpenID Connect Discovery (`.well-known/openid-configuration`)
    -   For Google: `https://accounts.google.com`
    -   For Firebase (GCIP): `https://securetoken.google.com/YOUR_PROJECT_ID`
    -   For Auth0: `https://YOUR_DOMAIN.auth0.com`
    -   For Keycloak: `https://your-keycloak.com/realms/YOUR_REALM`

##### `OIDC_CLIENT_ID`

-   **Description**: OIDC client ID
-   **Format**: String
-   **Example**: `your-client-id.apps.googleusercontent.com`
-   **Required**: ✅ Yes
-   **Default**: None
-   **Notes**: OAuth2/OIDC client identifier obtained from your OIDC provider.

##### `OIDC_CLIENT_SECRET`

-   **Description**: OIDC client secret
-   **Format**: String (secure)
-   **Example**: `GOCSPX-xxxxxxxxxxxxxxxxxxxx`
-   **Required**: ✅ Yes
-   **Default**: None
-   **Notes**:
    -   OAuth2/OIDC client secret. **Keep secure!**
    -   Never commit this to version control
    -   Obtain from your OIDC provider's console

##### `OIDC_REDIRECT_URI`

-   **Description**: OIDC callback redirect URI
-   **Format**: URL
-   **Example**: `http://localhost:3000/api/v1/auth/callback`
-   **Required**: ✅ Yes
-   **Default**: None
-   **Notes**:
    -   Must **exactly match** the redirect URI configured in your OIDC provider
    -   For local development: `http://localhost:3000/api/v1/auth/callback`
    -   For production: `https://your-domain.com/api/v1/auth/callback`

##### `JWT_SECRET`

-   **Description**: Secret key for local JWT session token signing
-   **Format**: String (secure random, minimum 32 characters)
-   **Example**: `your-super-secure-random-jwt-secret-key-min-32-chars`
-   **Required**: ✅ Yes
-   **Default**: None
-   **Notes**:
    -   Used for signing session tokens issued after OIDC authentication
    -   Generate with: `openssl rand -base64 32` or `node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"`
    -   **Keep secure!** Never commit to version control

### Optional Variables

#### `PORT`

-   **Description**: Server listening port
-   **Format**: Integer (1-65535)
-   **Example**: `3000`
-   **Required**: ❌ No
-   **Default**: `3000` (or `--port` CLI argument)
-   **Notes**: Can be overridden via `--port` CLI argument.

#### `APP_ENV`

-   **Description**: Application environment
-   **Format**: `development` | `staging` | `release`
-   **Example**: `development`
-   **Required**: ❌ No
-   **Default**: `development`
-   **Notes**:
    -   Controls which `.env` files are loaded
    -   `development` and `staging` enable Swagger UI
    -   Affects logging levels and debug features

#### `DEBUG_ROUTES`

-   **Description**: Enable route debugging middleware
-   **Format**: `true` | `false`
-   **Example**: `true`
-   **Required**: ❌ No
-   **Default**: `false`
-   **Notes**: When enabled, logs all HTTP requests with method, path, and status.

#### `JWT_EXPIRY_SECONDS`

-   **Description**: JWT session token expiration time in seconds
-   **Format**: Integer
-   **Example**: `3600`
-   **Required**: ❌ No
-   **Default**: `3600` (1 hour)
-   **Notes**: Controls how long session tokens remain valid after OIDC authentication.

### Future Variables (From Development Plan)

These variables are planned for future implementation and are documented here for reference.

#### Rate Limiting (Phase 1.3)

##### `RATE_LIMIT_GLOBAL_PER_MINUTE`

-   **Description**: Global rate limit per IP address
-   **Format**: Integer
-   **Example**: `100`
-   **Required**: ❌ No (future)
-   **Default**: `100`
-   **Notes**: Maximum requests per minute for unauthenticated endpoints.

##### `RATE_LIMIT_AUTHENTICATED_PER_MINUTE`

-   **Description**: Rate limit for authenticated users
-   **Format**: Integer
-   **Example**: `1000`
-   **Required**: ❌ No (future)
-   **Default**: `1000`
-   **Notes**: Maximum requests per minute per authenticated user.

##### `RATE_LIMIT_ADMIN_PER_MINUTE`

-   **Description**: Rate limit for admin users
-   **Format**: Integer
-   **Example**: `5000`
-   **Required**: ❌ No (future)
-   **Default**: `5000`
-   **Notes**: Maximum requests per minute per admin user.

##### `RATE_LIMIT_LOGIN_ATTEMPTS`

-   **Description**: Maximum failed login attempts before lockout
-   **Format**: Integer
-   **Example**: `5`
-   **Required**: ❌ No (future)
-   **Default**: `5`
-   **Notes**: Number of failed attempts before account lockout.

##### `RATE_LIMIT_LOGIN_WINDOW_MINUTES`

-   **Description**: Time window for login attempt rate limiting
-   **Format**: Integer
-   **Example**: `15`
-   **Required**: ❌ No (future)
-   **Default**: `15`
-   **Notes**: Time window in minutes for counting login attempts.

---

## Frontend (UI) Variables

### Optional Variables

#### `VITE_API_URL`

-   **Description**: Backend API base URL
-   **Format**: URL
-   **Example**: `http://localhost:3000`
-   **Required**: ❌ No
-   **Default**: `http://localhost:3000`
-   **Notes**:
    -   Must be prefixed with `VITE_` for Vite to expose it
    -   Used by API client for all backend requests
    -   Should match backend server URL

---

## Environment File Loading Order

### Backend (Server)

The backend loads environment variables from multiple `.env` files in the following priority order (first match wins):

1. `server/.env.<APP_ENV>.local` (highest priority)
2. `server/.env.<APP_ENV>`
3. `server/.env.local`
4. `server/.env` (lowest priority)

**Example for `APP_ENV=development`:**

1. `server/.env.development.local`
2. `server/.env.development`
3. `server/.env.local`
4. `server/.env`

**Example for `APP_ENV=staging`:**

1. `server/.env.staging.local`
2. `server/.env.staging`
3. `server/.env.local`
4. `server/.env`

### Frontend (UI)

Vite automatically loads environment variables from `.env` files based on the `--mode` flag:

-   **Development**: `ui/.env.development`
-   **Staging**: `ui/.env.staging`
-   **Production**: `ui/.env.release` or `ui/.env.production`

**Note**: Only variables prefixed with `VITE_` are exposed to the frontend code.

---

## Quick Reference

### Minimum Required Configuration

#### Backend (`server/.env`)

```bash
# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/t3chat

# CORS (must include your frontend URL)
CORS_ORIGINS=http://localhost:3010

# OIDC Authentication (REQUIRED)
# Choose one of the following OIDC provider configurations:

# Option 1: Google OAuth (recommended for quick start)
OIDC_ISSUER_URL=https://accounts.google.com
OIDC_CLIENT_ID=your-client-id.apps.googleusercontent.com
OIDC_CLIENT_SECRET=your-client-secret
OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback

# Option 2: Firebase (Google Cloud Identity Platform)
# OIDC_ISSUER_URL=https://securetoken.google.com/YOUR_PROJECT_ID
# OIDC_CLIENT_ID=your-oauth-client-id.apps.googleusercontent.com
# OIDC_CLIENT_SECRET=your-oauth-client-secret
# OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback

# Option 3: Auth0
# OIDC_ISSUER_URL=https://YOUR_DOMAIN.auth0.com
# OIDC_CLIENT_ID=your-auth0-client-id
# OIDC_CLIENT_SECRET=your-auth0-client-secret
# OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback

# Option 4: Keycloak (for self-hosted)
# OIDC_ISSUER_URL=https://your-keycloak.com/realms/YOUR_REALM
# OIDC_CLIENT_ID=your-keycloak-client-id
# OIDC_CLIENT_SECRET=your-keycloak-client-secret
# OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback

# JWT Session Token Secret (generate with: openssl rand -base64 32)
JWT_SECRET=your-super-secure-random-jwt-secret-key-min-32-chars
```

#### Frontend (`ui/.env.development`)

```bash
VITE_API_URL=http://localhost:3000
```

### Development Configuration

#### Backend (`server/.env.development`)

```bash
# Environment
APP_ENV=development
PORT=3000
DEBUG_ROUTES=false

# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/t3chat

# CORS
CORS_ORIGINS=http://localhost:3010,http://localhost:3000

# OIDC Authentication (example with Google)
OIDC_ISSUER_URL=https://accounts.google.com
OIDC_CLIENT_ID=your-client-id.apps.googleusercontent.com
OIDC_CLIENT_SECRET=your-client-secret
OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback

# JWT Session Tokens
JWT_SECRET=dev-secret-change-in-production-min-32-chars-long
JWT_EXPIRY_SECONDS=3600
```

#### Frontend (`ui/.env.development`)

```bash
VITE_API_URL=http://localhost:3000
```

### Staging Configuration

#### Backend (`server/.env.staging`)

```bash
# Environment
APP_ENV=staging
PORT=3000

# Database
DATABASE_URL=postgresql://user:password@staging-db.example.com:5432/t3chat

# CORS
CORS_ORIGINS=https://staging.example.com,https://api-staging.example.com

# OIDC Authentication
OIDC_ISSUER_URL=https://accounts.google.com
OIDC_CLIENT_ID=staging-client-id.apps.googleusercontent.com
OIDC_CLIENT_SECRET=staging-client-secret
OIDC_REDIRECT_URI=https://api-staging.example.com/api/v1/auth/callback

# JWT Session Tokens
JWT_SECRET=staging-secret-generate-with-openssl-rand-base64-32
JWT_EXPIRY_SECONDS=3600
```

#### Frontend (`ui/.env.staging`)

```bash
VITE_API_URL=https://api-staging.example.com
```

### Production Configuration

#### Backend (`server/.env.release`)

```bash
# Environment
APP_ENV=release
PORT=3000

# Database
DATABASE_URL=postgresql://user:password@prod-db.example.com:5432/t3chat

# CORS
CORS_ORIGINS=https://app.example.com,https://api.example.com

# OIDC Authentication
OIDC_ISSUER_URL=https://accounts.google.com
OIDC_CLIENT_ID=production-client-id.apps.googleusercontent.com
OIDC_CLIENT_SECRET=production-client-secret-keep-secure
OIDC_REDIRECT_URI=https://api.example.com/api/v1/auth/callback

# JWT Session Tokens
JWT_SECRET=production-secret-generate-with-openssl-rand-base64-32
JWT_EXPIRY_SECONDS=3600
```

#### Frontend (`ui/.env.release`)

```bash
VITE_API_URL=https://api.example.com
```

---

## Environment-Specific Notes

### Development

-   Swagger UI enabled at `/swagger-ui`
-   Debug routes enabled with `DEBUG_ROUTES=true`
-   More verbose logging

### Staging

-   Swagger UI enabled at `/swagger-ui`
-   Production-like configuration
-   Useful for testing before production deployment

### Release (Production)

-   Swagger UI disabled
-   Minimal logging
-   Production OIDC authentication
-   External PostgreSQL database

---

## Security Best Practices

1. **Never commit `.env` files** - Add them to `.gitignore`
2. **Use `.env.local`** for local overrides that shouldn't be committed
3. **Rotate secrets regularly** - Especially `OIDC_CLIENT_SECRET` and `JWT_SECRET`
4. **Use different values per environment** - Never reuse production secrets in development
5. **Restrict CORS origins** - Only include trusted domains in `CORS_ORIGINS`
6. **Use strong secrets** - Generate secure random strings for secrets
7. **Limit database access** - Use read-only users where possible
8. **Monitor environment variables** - Log which environment is active (not the values)

---

## Authentication Configuration

T3Chat supports two authentication methods:

### 1. Local Authentication (Username & Password)

Local authentication is enabled by default and requires no additional configuration. The database includes a seeded admin user:

- **Username**: `admin`
- **Email**: `admin@localhost`
- **Password**: `P@$$w0rd`
- **Role**: Administrator

**⚠️ Security Note**: Change the default admin password immediately after first login!

Local authentication is ideal for:
- Development and testing environments
- Self-hosted deployments without external identity providers
- Quick setup without OIDC configuration

### 2. OIDC Authentication (OpenID Connect)

T3Chat also supports OpenID Connect (OIDC) for integration with external identity providers. This is recommended for production deployments.

### Option 1: Google OAuth (Recommended for Quick Start)

Google OAuth is the easiest to set up for local development:

1. **Create a Google Cloud Project**:

    - Go to [Google Cloud Console](https://console.cloud.google.com/)
    - Create a new project or select an existing one

2. **Enable Google OAuth**:

    - Navigate to "APIs & Services" > "Credentials"
    - Click "Create Credentials" > "OAuth 2.0 Client ID"
    - Configure the OAuth consent screen if prompted
    - Choose "Web application" as the application type

3. **Configure Redirect URIs**:

    - Add `http://localhost:3000/api/v1/auth/callback` to "Authorized redirect URIs"
    - For frontend: Add `http://localhost:3010` to "Authorized JavaScript origins"

4. **Get Your Credentials**:
    - Copy the "Client ID" and "Client secret"
    - Add them to your `.env` file:

```bash
OIDC_ISSUER_URL=https://accounts.google.com
OIDC_CLIENT_ID=your-client-id.apps.googleusercontent.com
OIDC_CLIENT_SECRET=GOCSPX-your-client-secret
OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback
JWT_SECRET=generate-with-openssl-rand-base64-32
```

### Option 2: Auth0 (Easy Managed Service)

Auth0 provides a free tier and easy setup:

1. Sign up at [auth0.com](https://auth0.com/)
2. Create a new "Regular Web Application"
3. Configure:
    - Allowed Callback URLs: `http://localhost:3000/api/v1/auth/callback`
    - Allowed Logout URLs: `http://localhost:3010`
    - Allowed Web Origins: `http://localhost:3010`
4. Get credentials from "Settings" tab
5. Add to your `.env`:

```bash
OIDC_ISSUER_URL=https://YOUR_DOMAIN.auth0.com
OIDC_CLIENT_ID=your-auth0-client-id
OIDC_CLIENT_SECRET=your-auth0-client-secret
OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback
JWT_SECRET=generate-with-openssl-rand-base64-32
```

### Option 3: Keycloak (Self-Hosted)

For complete control, use Keycloak:

1. **Run Keycloak with Docker**:

```bash
docker run -p 8080:8080 -e KEYCLOAK_ADMIN=admin -e KEYCLOAK_ADMIN_PASSWORD=admin quay.io/keycloak/keycloak:latest start-dev
```

2. **Access Keycloak**: Open `http://localhost:8080` and login with admin/admin

3. **Create a Realm**: Create a new realm (e.g., "t3chat")

4. **Create a Client**:

    - Client ID: `t3chat-client`
    - Client authentication: ON
    - Valid redirect URIs: `http://localhost:3000/api/v1/auth/callback`
    - Web origins: `http://localhost:3010`

5. **Get Client Secret**: From the "Credentials" tab

6. **Add to your `.env`**:

```bash
OIDC_ISSUER_URL=http://localhost:8080/realms/t3chat
OIDC_CLIENT_ID=t3chat-client
OIDC_CLIENT_SECRET=your-keycloak-client-secret
OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback
JWT_SECRET=generate-with-openssl-rand-base64-32
```

### Generating JWT Secret

Generate a secure JWT secret using one of these commands:

```bash
# Using OpenSSL
openssl rand -base64 32

# Using Node.js
node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"

# Using Python
python -c "import secrets; print(secrets.token_urlsafe(32))"
```

---

## Troubleshooting

### Backend Issues

**Error: `Failed to get OIDC_ISSUER_URL` (or other OIDC variables)**

-   **Cause**: OIDC environment variables are required but not set
-   **Solution**:
    -   Create a `server/.env` file with all required OIDC variables
    -   See [Setting Up OIDC Authentication](#setting-up-oidc-authentication) section above
    -   Minimum required: `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_REDIRECT_URI`, `JWT_SECRET`

**Error: `DATABASE_URL is not set`**

-   Ensure `.env` file exists in `server/` directory
-   Check that `DATABASE_URL` is defined
-   Verify file loading order

**Error: `No valid CORS origins found in CORS_ORIGINS`**

-   Ensure `CORS_ORIGINS` contains at least one valid URL
-   Check for trailing commas or empty values
-   Verify URLs are properly formatted

**Error: OIDC Provider Discovery Fails**

-   Check that `OIDC_ISSUER_URL` is correct and accessible
-   Verify the provider supports `.well-known/openid-configuration` endpoint
-   Test by visiting: `{OIDC_ISSUER_URL}/.well-known/openid-configuration` in your browser
-   Check network connectivity and firewall rules

### Frontend Issues

**API calls fail with CORS errors**

-   Verify `VITE_API_URL` matches backend server URL
-   Check backend `CORS_ORIGINS` includes frontend URL
-   Ensure both are using same protocol (http/https)

**Environment variable not accessible**

-   Frontend variables must be prefixed with `VITE_`
-   Restart dev server after adding new variables
-   Check that variable is in correct `.env` file for current mode

---

## Related Documentation

-   [Backend README](server/README.md) - Backend-specific documentation
-   [Frontend README](ui/README.md) - Frontend-specific documentation
-   [Development Plan](plan.md) - Complete development plan with future variables

---

---

## Database Seeded Data

The database migration includes the following seeded data:

### Default Admin User

The database includes a seeded admin user for local authentication:

-   **User ID**: `system-admin-00000000`
-   **Email**: `admin@localhost`
-   **Username**: `admin`
-   **Password**: `P@$$w0rd` (bcrypt hashed)
-   **Role**: Administrator
-   **System User**: Yes (cannot be deleted or disabled)

**⚠️ Security Note**: Change the default admin password immediately after first login!

**Authentication Methods**:
- **Local Auth**: Login with username `admin` and password `P@$$w0rd`
- **OIDC Auth**: Users created via OIDC login with their OIDC `sub` as the user ID

### AI Providers and Models

The migration also seeds:

-   **9 AI Providers**: OpenAI, Anthropic, Google, Meta, DeepSeek, Mistral, xAI, Cohere, Alibaba
-   **35+ AI Models**: Including GPT-4o, Claude 3.5, Gemini 2.0, Llama 3.3, and more

---

**Last Updated**: 2025-12-02  
**Version**: 2.0
