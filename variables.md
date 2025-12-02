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

### Future Variables (From Development Plan)

These variables are planned for future implementation and are documented here for reference.

#### OIDC Authentication (Phase 1.1)

##### `OIDC_ISSUER_URL`

-   **Description**: OIDC provider issuer URL
-   **Format**: URL
-   **Example**: `https://your-provider.com`
-   **Required**: ❌ No (future)
-   **Default**: None
-   **Notes**: Used for OIDC discovery and token verification.

##### `OIDC_CLIENT_ID`

-   **Description**: OIDC client ID
-   **Format**: String
-   **Example**: `your-client-id`
-   **Required**: ❌ No (future)
-   **Default**: None
-   **Notes**: OAuth2/OIDC client identifier.

##### `OIDC_CLIENT_SECRET`

-   **Description**: OIDC client secret
-   **Format**: String
-   **Example**: `your-client-secret`
-   **Required**: ❌ No (future)
-   **Default**: None
-   **Notes**: OAuth2/OIDC client secret. Keep secure.

##### `OIDC_REDIRECT_URI`

-   **Description**: OIDC callback redirect URI
-   **Format**: URL
-   **Example**: `http://localhost:3000/api/v1/auth/callback`
-   **Required**: ❌ No (future)
-   **Default**: None
-   **Notes**: Must match OIDC provider configuration.

##### `JWT_SECRET`

-   **Description**: Secret key for local JWT token signing
-   **Format**: String (secure random)
-   **Example**: `your-jwt-secret-for-local-tokens`
-   **Required**: ❌ No (future)
-   **Default**: None
-   **Notes**: Used for local token generation (if not using OIDC tokens).

##### `JWT_EXPIRY_SECONDS`

-   **Description**: JWT token expiration time in seconds
-   **Format**: Integer
-   **Example**: `3600`
-   **Required**: ❌ No (future)
-   **Default**: `3600` (1 hour)
-   **Notes**: Access token expiration time.

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
DATABASE_URL=postgresql://postgres:password@localhost:5432/t3chat
CORS_ORIGINS=http://localhost:3010
```

#### Frontend (`ui/.env.development`)

```bash
VITE_API_URL=http://localhost:3000
```

### Development Configuration

#### Backend (`server/.env.development`)

```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/t3chat
CORS_ORIGINS=http://localhost:3010,http://localhost:3000
APP_ENV=development
PORT=3000
DEBUG_ROUTES=false
```

#### Frontend (`ui/.env.development`)

```bash
VITE_API_URL=http://localhost:3000
```

### Staging Configuration

#### Backend (`server/.env.staging`)

```bash
DATABASE_URL=postgresql://user:password@staging-db.example.com:5432/t3chat
CORS_ORIGINS=https://staging.example.com,https://api-staging.example.com
APP_ENV=staging
PORT=3000
```

#### Frontend (`ui/.env.staging`)

```bash
VITE_API_URL=https://api-staging.example.com
```

### Production Configuration

#### Backend (`server/.env.release`)

```bash
DATABASE_URL=postgresql://user:password@prod-db.example.com:5432/t3chat
CORS_ORIGINS=https://app.example.com,https://api.example.com
APP_ENV=release
PORT=3000
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

## Troubleshooting

### Backend Issues

**Error: `DATABASE_URL is not set`**

-   Ensure `.env` file exists in `server/` directory
-   Check that `DATABASE_URL` is defined
-   Verify file loading order

**Error: `No valid CORS origins found in CORS_ORIGINS`**

-   Ensure `CORS_ORIGINS` contains at least one valid URL
-   Check for trailing commas or empty values
-   Verify URLs are properly formatted

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

**Last Updated**: 2025-01-XX  
**Version**: 1.0
