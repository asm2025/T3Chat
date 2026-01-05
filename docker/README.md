# T3Chat Docker Setup

This directory contains Docker configuration files for running T3Chat in a containerized environment.

## Files

- `docker-compose.yml` - Main orchestration file for all services
- `server/Dockerfile` - Multi-stage build for Rust backend (includes frontend build)
- `clients/web/Dockerfile` - Frontend build Dockerfile (optional, used by backend Dockerfile)

## Quick Start

### Prerequisites

1. Docker and Docker Compose installed
2. Create a `.env` file in the project root with required environment variables:

```bash
# Required
JWT_SECRET=your-secure-jwt-secret-here

# Optional - Database (defaults work with docker-compose)
DATABASE_URL=postgresql://postgres:password@postgres:5432/appdata

# Optional - Server
PORT=3000
APP_ENV=production
CORS_ORIGINS=http://localhost:3000,http://localhost:3010

# Optional - OIDC Authentication
OIDC_ISSUER_URL=
OIDC_CLIENT_ID=
OIDC_CLIENT_SECRET=
OIDC_REDIRECT_URI=

# Optional - AI Provider API Keys
OPENAI_API_KEY=
ANTHROPIC_API_KEY=
GOOGLE_API_KEY=
OPENROUTER_API_KEY=
ABACUS_API_KEY=
```

### Building and Running

1. **Build and start all services:**
   ```bash
   docker compose up -d --build
   ```

2. **View logs:**
   ```bash
   docker compose logs -f
   ```

3. **Stop all services:**
   ```bash
   docker compose down
   ```

4. **Stop and remove volumes (clean slate):**
   ```bash
   docker compose down -v
   ```

### Services

- **postgres** - PostgreSQL 18 database
  - Port: `5432`
  - Database: `appdata`
  - User: `postgres`
  - Password: `password`

- **backend** - Rust API server with embedded frontend
  - Port: `3000` (configurable via `PORT` env var)
  - Health check: `http://localhost:3000/health`
  - Serves frontend static files from `/app/wwwroot`

### Volume Configuration

By default, PostgreSQL data is stored in a named Docker volume `postgres_data`. 

To use a bind mount instead (like the original `PostegreSQL.compose.yml`), edit `docker-compose.yml` and uncomment the bind mount line:

```yaml
volumes:
  # Option 2: Use bind mount (uncomment and adjust path for Windows)
  - "D:/Work/db/PostegreSQL:/var/lib/postgresql/data"
```

And comment out the named volume line.

### Development vs Production

- **Development**: Set `APP_ENV=development` in `.env` to enable Swagger UI and debug routes
- **Production**: Set `APP_ENV=production` (default) for optimized settings

### Troubleshooting

1. **Backend won't start:**
   - Check that `JWT_SECRET` is set in `.env`
   - Verify PostgreSQL is healthy: `docker compose ps`
   - Check logs: `docker compose logs backend`

2. **Frontend not loading:**
   - Ensure the backend build completed successfully (frontend is built as part of backend Dockerfile)
   - Check that `wwwroot` directory exists in the backend container

3. **Database connection issues:**
   - Verify PostgreSQL is running: `docker compose ps postgres`
   - Check database logs: `docker compose logs postgres`
   - Ensure `DATABASE_URL` matches the postgres service configuration

### Building Individual Services

**Backend only:**
```bash
docker compose build backend
```

**PostgreSQL only (uses pre-built image):**
```bash
docker compose up postgres
```

### Accessing the Application

Once running, access:
- Frontend/API: `http://localhost:3000`
- Health check: `http://localhost:3000/health`
- Swagger UI (if enabled): `http://localhost:3000/swagger-ui`

### Database Migrations

Database migrations run automatically on backend startup. The backend uses Diesel migrations embedded in the `server/migrations/` directory.

