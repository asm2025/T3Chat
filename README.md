# T3Chat - LibreChat-Inspired Multi-AI Platform

Welcome to T3Chat, a LibreChat-inspired multi-AI chat platform built with Rust and React! This project provides a production-ready foundation for building a chat application that supports multiple AI providers (OpenAI, Anthropic, Google, and more) with a modern, normalized PostgreSQL database schema.

## 🎯 **Philosophy**

This application provides a highly opinionated, production-ready foundation for building full-stack applications with a decoupled frontend and backend. It's designed to maximize development velocity while adhering to best practices, including clear separation of concerns and secure handling of sensitive credentials.

Many boilerplates offer a rapid 'hello world' experience for local development but often defer critical decisions about authentication, database integration, and production deployment. This template takes a different approach. We believe that the complexities of a true full-stack application - setting up auth, a database, and distinct hosting for UI and API - are largely unavoidable for production use. By addressing these components comprehensively from the start, this template aims to provide a clearer, more predictable path to a robust, deployable application, minimizing 'surprise' hurdles down the line and fostering a deeper understanding of the full stack architecture.

Start with everything running locally on your machine, then progressively connect to production services when you're ready or dive in and connect them all at app creation.

## 🚀 **What You Have**

**Frontend:**

-   ⚛️ React + TypeScript + Vite

-   🎨 Tailwind CSS v4 + ShadCN components

-   🔐 OIDC Authentication & Local Authentication

-   💬 Multi-provider chat interface (OpenAI, Anthropic, Google, custom)

**Backend:**

-   🦀 Rust API backend (Axum)

-   🗄️ PostgreSQL with Diesel (fully normalized schema)

-   🔑 OIDC Authentication (JWKS-based JWT verification)

-   🤖 AI Provider abstraction system (trait-based, extensible)

-   📊 Comprehensive database schema for conversations, messages, agents, presets, and more

**Local Development (Default):**

-   ⚡ Runs UI + Server on your computer

-   🔐 Local username/password authentication (primary). OIDC authentication (Google OAuth, Firebase, Auth0, Keycloak) is optional and appears only if configured.

**Key Features:**

-   🔄 Multi-AI provider support (switch between OpenAI, Anthropic, Google, etc.)

-   💾 Normalized PostgreSQL schema with proper relationships

-   🎯 Agent system with tools and conversation starters

-   📝 Preset system for saved conversation configurations

-   🏷️ Tag system for organizing conversations

-   📁 File upload and management

-   🔐 Encrypted API key storage per user/provider

## 🗂 Environment Configuration

Environment is configured via a mix of **`.env` files** and a **YAML application config**. See [`variables.md`](variables.md) for a complete reference.

### Backend `.env` files

-   **Location**: create per-environment files in `server/`:
    -   `server/.env.development`, `server/.env.staging`, `server/.env.release`
-   **Load order (inside `server/`)**:
    -   `.env.<APP_ENV>.local`, `.env.<APP_ENV>`, `.env.local`, `.env` (default `APP_ENV=development`)
-   **Key variables (summary)**:
    -   **Required**: `DATABASE_URL`, `CORS_ORIGINS`, `JWT_SECRET`
    -   **OIDC (optional)**: `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_REDIRECT_URI`
    -   **Frontend redirect (recommended for OIDC)**: `FRONTEND_URL` (e.g. `http://localhost:3010`)
    -   **Config path (optional)**: `T3CHAT_CONFIG` to point at a non-default `t3chat.yaml`

### Application config (`t3chat.yaml`)

-   **Location**: project root (same folder as this `README.md`)
-   **Usage**:
    -   Copy `t3chat.example.yaml` → `t3chat.yaml`
    -   Replace `${OPENAI_API_KEY}`, `${ANTHROPIC_API_KEY}`, `${GOOGLE_API_KEY}`, `${OPENROUTER_API_KEY}`, etc. with real environment variables in your shell or `.env` files
    -   Optionally point to a custom path with `T3CHAT_CONFIG=/absolute/or/relative/path/to/t3chat.yaml`
-   This file controls:
    -   Which providers are enabled
    -   Default and fetched models for each provider
    -   High-level model presets (`model_specs`) shown in the UI

### Frontend API base URL

-   The frontend uses `import.meta.env.VITE_API_URL` as its API base URL, but in this repo it is **set via Vite CLI flags**, not `.env` files.
-   **Local development** (recommended):
    -   `cd ui && pnpm dev -- --api-url http://localhost:3000`
-   **Production builds** (e.g. Cloudflare Pages):
    -   `cd ui && pnpm run build -- --api-url https://api.yourdomain.com`

📖 **For complete environment variable documentation**, see [`variables.md`](variables.md). For a step‑by‑step walkthrough, see [`QUICKSTART.md`](QUICKSTART.md).

📖 **For complete environment variable documentation**, see [`variables.md`](variables.md)

🚀 **New to T3Chat?** Start with the [`QUICKSTART.md`](QUICKSTART.md) guide for step-by-step setup instructions!

**Production (when connected):**

-   🌐 Cloudflare Pages deployment ready (frontend)

-   🦀 Standalone Rust server deployment

-   🗄️ Supabase or custom PostgreSQL

-   🔐 Production OIDC Authentication

## 🧰 **Local Prerequisites**

Before running or compiling the Rust server, make sure the following tooling is available on your machine:

-   `pnpm` (workspace package manager)

-   Rust toolchain installed via [rustup](https://rustup.rs)

    -   **Windows**: Use the MSVC toolchain (default)

    -   **Linux/macOS**: Default toolchain works fine

-   PostgreSQL client libraries (`libpq`)

### PostgreSQL Client Library Setup

The Rust server uses the `diesel` crate which requires PostgreSQL client libraries (`libpq`) to compile. **You need these development files even if you only run PostgreSQL in Docker or use the embedded database for local development.**

Choose your platform below for detailed setup instructions:

<details>\
<summary><strong>🪟 Windows (MSVC)</strong></summary>

On Windows, the MSVC linker needs to find `libpq.lib` at compile time and `libpq.dll` at runtime. You have two main options:

#### Option 1: Official PostgreSQL Installer (Recommended for beginners)

1. **Download PostgreSQL 18** from [postgresql.org/download/windows](https://www.postgresql.org/download/windows/)

2. **Run the installer** and ensure you check the **"Command Line Tools"** or **"Development Files"** component during installation

3. **Note the installation path** (typically `C:\Program Files\PostgreSQL\18`)

4. **Set environment variables** (choose one method):

**Method A: Using Diesel-specific variables** (Recommended)

```cmd
setx PQ_LIB_DIR "C:\Program Files\PostgreSQL\18\lib"
setx PQ_INCLUDE_DIR "C:\Program Files\PostgreSQL\18\include"
```

**Method B: Using system linker variables**

-   Open **System Properties** → **Environment Variables**

-   Under **System variables**, find or create `LIB` (the MSVC linker library search path)

-   Add: `C:\Program Files\PostgreSQL\18\lib`

-   Find `Path` and add: `C:\Program Files\PostgreSQL\18\bin` (for runtime DLL)

1. **Restart your terminal** for environment variables to take effect

2. **Verify installation**:

```cmd
where libpq.dll
# Should show: C:\Program Files\PostgreSQL\18\bin\libpq.dll
```

#### Option 2: vcpkg Package Manager

[vcpkg](https://vcpkg.io) is Microsoft's C/C++ package manager that provides pre-built libraries.

1. **Install vcpkg** (if not already installed):

```cmd
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat
```

1. **Add vcpkg to PATH** (optional but recommended):

```cmd
setx PATH "%PATH%;C:\path\to\vcpkg"
```

1. **Install libpq**:

```cmd
vcpkg install libpq:x64-windows
```

This downloads and builds PostgreSQL client libraries (takes 5-10 minutes).

1. **Set environment variables**:

```cmd
setx PQ_LIB_DIR "C:\path\to\vcpkg\installed\x64-windows\lib"
setx PQ_INCLUDE_DIR "C:\path\to\vcpkg\installed\x64-windows\include"
```

1. **Add DLL to PATH** (for runtime):

```cmd
setx PATH "%PATH%;C:\path\to\vcpkg\installed\x64-windows\bin"
```

1. **Restart your terminal** and verify:

```cmd
where libpq.dll
```

**Updating vcpkg and packages:**

If you already have vcpkg installed and want to update it or upgrade installed packages:

```cmd
# Update vcpkg itself
cd C:\path\to\vcpkg
git pull
.\bootstrap-vcpkg.bat

# Check for package updates
vcpkg upgrade

# Update all installed packages
vcpkg upgrade --no-dry-run

# Or update a specific package
vcpkg upgrade libpq:x64-windows --no-dry-run

# Full refresh if needed
vcpkg remove libpq:x64-windows
vcpkg install libpq:x64-windows
```

> 💡 **What is the `LIB` environment variable?**
>
> `LIB` is a Windows-specific environment variable used by the MSVC linker (`link.exe`) to locate `.lib` files during compilation. When you add a directory to `LIB`, any compiled program can link against libraries in that directory. Alternatively, Diesel respects `PQ_LIB_DIR` specifically for PostgreSQL, which is more targeted and won't affect other builds.

#### Option 3: Chocolatey Package Manager

```cmd
choco install postgresql
# Ensure the "Development Files" are included
setx PQ_LIB_DIR "C:\Program Files\PostgreSQL\18\lib"
setx PQ_INCLUDE_DIR "C:\Program Files\PostgreSQL\18\include"
# Restart terminal
```

</details>

<details>\
<summary><strong>🐧 Linux</strong></summary>

On Linux, install the PostgreSQL development package for your distribution:

#### Ubuntu / Debian

```bash
sudo apt update
sudo apt install libpq-dev build-essential
```

#### Fedora / RHEL / CentOS

```bash
sudo dnf install postgresql-devel gcc
# or on older systems:
sudo yum install postgresql-devel gcc
```

#### Arch Linux

```bash
sudo pacman -S postgresql-libs base-devel
```

#### Alpine Linux

```bash
apk add postgresql-dev build-base
```

#### Verify installation

```bash
pkg-config --libs libpq
# Should output: -lpq
```

> 💡 **Note**: On Linux, the PostgreSQL client library path is typically already in the system library search path, so no additional environment variables are needed.

</details>

<details>\
<summary><strong>🍎 macOS</strong></summary>

On macOS, install PostgreSQL via Homebrew:

#### Install PostgreSQL

```bash
brew install postgresql@18
```

#### Link the installation (if needed)

```bash
brew link postgresql@18
```

#### Set environment variables (usually automatic, but if needed)

```bash
# Add to ~/.zshrc or ~/.bash_profile
export PQ_LIB_DIR="$(brew --prefix postgresql@18)/lib"
export PQ_INCLUDE_DIR="$(brew --prefix postgresql@18)/include"
```

#### Reload your shell configuration

```bash
source ~/.zshrc  # or source ~/.bash_profile
```

#### Verify installation

```bash
pg_config --version
# Should show: PostgreSQL 18.x
```

> 💡 **Note**: Homebrew automatically adds PostgreSQL to your library path, so explicit environment variables are usually not needed. If you encounter linking issues, ensure you've run `brew link postgresql@18`.

</details>

### Troubleshooting

**Error: `error: linking with 'link.exe' failed` (Windows)**

-   Ensure `PQ_LIB_DIR` points to the directory containing `libpq.lib`

-   Restart your terminal after setting environment variables

-   Try running `cargo clean` and rebuilding

**Error: `libpq.dll not found` at runtime (Windows)**

-   Add the PostgreSQL `bin` directory (containing `libpq.dll`) to your system `PATH`

-   Or copy `libpq.dll` to the same directory as your compiled executable

**Error: `cannot find -lpq` (Linux/macOS)**

-   Install the PostgreSQL development package (`libpq-dev` or equivalent)

-   Verify with `pkg-config --libs libpq`

**Still having issues?**

-   Check the [Diesel Getting Started guide](https://diesel.rs/guides/getting-started)

-   See [Rust PostgreSQL documentation](https://docs.rs/postgres/latest/postgres/)

-   Visit our [community discussions](https://github.com/VoloBuilds/create-volo-app/discussions)

## 🛠️ **Development**

Run the backend and frontend in separate terminals.

### Backend (Rust server)

```bash
cd server
cargo run
```

By default the backend listens on `http://localhost:3000`. You can override the port:

```bash
cd server
cargo run -- --port 8788
```

### Frontend (React UI)

```bash
cd ui
pnpm dev          # defaults to http://localhost:3010, API http://localhost:3000

# Or override ports / API URL explicitly
pnpm dev -- --port 3010 --api-url http://localhost:3000
```

### Other useful commands

```bash
# Frontend only
cd ui && pnpm dev

# Build frontend
cd ui && pnpm build

# Build Rust server for production
cd server && cargo build --release
```

## 🤖 **AI Provider System**

T3Chat uses a trait-based abstraction system for AI providers, similar to LibreChat's BaseClient pattern.

### Supported Providers

- **OpenAI** - GPT‑4o, GPT‑4 Turbo, and other OpenAI models
- **Anthropic** - Claude 3.5, Claude 3
- **Google** - Gemini 1.5
- **OpenRouter** - Aggregated models via OpenRouter
- **RouteLLM** - Routing profiles defined in config, backed by a single backend-managed key from `t3chat.yaml` (e.g. `ABACUS_API_KEY`)
- **Custom** - Additional OpenAI‑compatible providers via `custom` entries in `t3chat.yaml`

### Architecture

**Backend (Rust):**
- `ai/providers/mod.rs` defines the `AIProvider` trait and concrete providers (OpenAI, Anthropic, Google, OpenRouter, RouteLLM)
- `ai/model_catalog.rs` builds a provider/model catalog from `t3chat.yaml`
- Streaming support via Server-Sent Events (SSE) and typed chat request/response types in `ai/types.rs`

**Frontend (React):**
- Endpoint/model selectors for switching between providers and models
- Settings panel for provider-specific parameters (temperature, max tokens, etc.)
- Streaming message display driven by `/api/v1/chat/stream`

### Adding a New Provider

1. Create a new provider struct in `server/src/ai/providers/`
2. Implement the `AIProvider` trait
3. Add provider to the factory in `server/src/ai/factory.rs`
4. Update frontend endpoint selector if needed

📖 **For detailed implementation guide**, see [`plan.md`](plan.md) Phase 2A

## 📁 **Project Structure**

```
├── ui/                 # React frontend
│   ├── src/
│   │   ├── components/ # Chat UI, endpoints, presets, agents, files, ShadCN components
│   │   ├── lib/        # API client, auth helpers, utilities
│   │   ├── stores/     # State management (Zustand)
│   │   ├── types/      # TypeScript type definitions
│   │   └── pages/      # Route-level components
│   └── package.json
├── server/             # Rust API backend (Axum + Diesel)
│   ├── src/
│   │   ├── main.rs     # Application entry point & router
│   │   ├── api/        # Versioned HTTP handlers
│   │   │   └── v1/
│   │   │       ├── auth/       # Local + OIDC auth
│   │   │       ├── chat/       # Chat completion endpoints
│   │   │       ├── chats/      # Chat + message CRUD
│   │   │       ├── config/     # Startup/model config based on t3chat.yaml
│   │   │       ├── models/     # AI model catalog
│   │   │       ├── user/       # Profile endpoints
│   │   │       ├── user_api_keys/
│   │   │       └── admin/      # Admin APIs (users/providers/models/dashboard)
│   │   ├── ai/         # AI provider integrations
│   │   │   ├── model_catalog.rs
│   │   │   ├── manager.rs
│   │   │   └── providers/      # OpenAI, Anthropic, Google, OpenRouter, RouteLLM
│   │   ├── db/         # Diesel models, repositories, schema
│   │   │   ├── models/         # Domain models
│   │   │   ├── repositories/   # Data access layer
│   │   │   └── schema.rs       # Generated schema
│   │   └── middleware/ # Auth, admin, rate limiting
│   ├── migrations/     # Diesel SQL migrations (embedded)
│   ├── wwwroot/        # Static files served by the backend
│   ├── Cargo.toml      # Rust dependencies
│   └── .env            # Backend environment variables (local only)
├── t3chat.example.yaml # Example multi-provider/model configuration
├── plan.md             # Development plan and architecture
└── SCHEMA_CHANGES_SUMMARY.md  # Database schema documentation
```

## 🔧 **Customization**

### Adding API Routes

-   Handlers live under `server/src/api/v1/` (grouped by resource).

-   Routes are registered inside `setup_router` in `server/src/main.rs`.

-   Wrap protected routes with `middleware::auth::auth_middleware` via `route_layer`.

### Adding AI Providers

To add a new AI provider:

1. Create a new provider file in `server/src/ai/providers/` (e.g., `custom.rs`)

2. Implement the `AIProvider` trait (see `server/src/ai/mod.rs` for the trait definition)

3. Add provider to the factory in `server/src/ai/factory.rs`

4. Update frontend endpoint selector in `ui/src/components/Endpoints/EndpointSelector.tsx`

📖 **For detailed implementation guide**, see [`plan.md`](plan.md) Phase 2A

### Database Changes

The backend uses Diesel with async connection pooling. Migrations are SQL files embedded from `server/migrations`.

**Important**: Follow the normalization principles in `SCHEMA_CHANGES_SUMMARY.md`:
- Use proper tables and foreign keys for lookup/reference data
- Use JSONB only for truly dynamic/provider-specific fields
- Use junction tables for many-to-many relationships

1. Install Diesel CLI (PostgreSQL) if you haven't: `cargo install diesel_cli --no-default-features --features postgres`

2. Generate a migration inside `server/`: `diesel migration generate add_feature_x`

3. Edit the generated `up.sql` and `down.sql`

4. Add or update models in `server/src/db/models/`

5. Extend repositories in `server/src/db/repositories/` as needed

6. Run `diesel migration run` (from `server/`) or rely on the server's startup auto-migrate (`AUTO_MIGRATE=true`)

See `[server/README.md](server/README.md)` for detailed guidance.

### UI Components

-   Add components in `ui/src/components/`

-   Use ShadCN/UI: Browse components at [ui.shadcn.com](https://ui.shadcn.com)

-   Install new components: `cd ui && npx shadcn-ui@latest add [component]`

### Styling

-   Modify `ui/tailwind.config.js` for custom themes

-   Global styles in `ui/src/index.css`

-   Use Tailwind utility classes throughout

## 🚀 **Deployment**

> **Note**: Use an external PostgreSQL (e.g., Supabase) for production.

### Backend (Rust Server)

The Rust backend runs as a standalone server and can be deployed to any platform that supports running binaries (e.g., AWS EC2, DigitalOcean, Railway, Render, etc.).

**Build for production:**

```bash
cd server
cargo build --release
```

**Run the production binary:**

```bash
./target/release/t3chat-server
# or with custom port
./target/release/t3chat-server --port 3000
```

**Environment variables required:**

-   `DATABASE_URL` - PostgreSQL connection string (required)
-   `CORS_ORIGINS` - Comma-separated list of allowed CORS origins (required)
-   `PORT` - Server port (optional, defaults to 3000)

📖 **For complete environment variable documentation**, see [`variables.md`](variables.md)

**Note**: Cloudflare Workers deployment is not supported for the Rust backend. For serverless deployment, consider platforms like [Fly.io](http://Fly.io), Railway, or Render that support Rust applications.

### Frontend (Cloudflare Pages)

1. **Connect to Git**: Link your repository to [Cloudflare Pages](https://dash.cloudflare.com/pages)

2. **Build Settings**:

-   Build command: `pnpm run build`

-   Build output: `ui/dist`

1. **Deploy**: Automatic on every git push

### Environment Variables (Production)

**Backend Server Environment Variables:**

-   `DATABASE_URL` - Your database connection string (required)
-   `CORS_ORIGINS` - Comma-separated list of allowed CORS origins (required)
-   `PORT` - Server port (optional, defaults to 3000)
-   `APP_ENV` - Application environment: `development`, `staging`, or `release` (optional, defaults to `development`)
-   `FRONTEND_URL` - Frontend base URL used for OIDC redirects (e.g., `https://app.example.com`)

**Frontend build / API URL:**

-   Build with the correct API base URL:
    -   `cd ui && pnpm run build -- --api-url https://api.example.com`

📖 **For complete environment variable documentation**, see [`variables.md`](variables.md)

### Post-Deployment Setup

1. **Configure OIDC provider**:

-   Ensure your OIDC provider is configured with the correct redirect URIs
-   Add your Pages domain to allowed origins (e.g., `your-app.pages.dev`)

1. **Test your deployment**:

```bash
curl https://api.yourdomain.com/api/v1/hello
```

## 🔐 **Authentication Flow**

Your app includes a complete authentication system with **local username/password authentication as the primary method**. OIDC authentication is optional and will only appear in the login form if configured.

### Local Authentication Flow (Primary)

1. **Login**: Users sign in with username/email and password

2. **Verification**: Backend verifies credentials against bcrypt password hash

3. **Token**: Backend generates JWT session token

4. **API calls**: Token sent in `Authorization: Bearer <token>` header

5. **Protection**: Same middleware handles both local and OIDC auth tokens

### OIDC Authentication Flow (Optional)

1. **Login**: Users sign in via OIDC provider (Google, Firebase, Auth0, Keycloak)

2. **Token**: Frontend receives OIDC ID token

3. **API calls**: Token sent in `Authorization: Bearer <token>` header

4. **Verification**: Backend verifies token via JWKS and creates/finds user in database

5. **Protection**: Protected routes automatically have user context


OIDC authentication is optional and only appears in the login form if OIDC is configured. To enable OIDC, set the following environment variables: `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, and `OIDC_REDIRECT_URI`.

### Default Admin User

The database includes a seeded admin user for local authentication:

- **Username**: `admin`
- **Password**: `P@$$w0rd`
- **Email**: `admin@localhost`
- **Role**: Administrator

**⚠️ Security Note**: Change the default admin password immediately after first login!

### Example API Call

```typescript
// Frontend (already implemented in lib/serverComm.ts)
const response = await api.getCurrentUser();
console.log(response.user);
```

## 🗄️ **Database**

The backend uses Diesel with async pooling (`diesel_async`) and repository helpers. The database schema is **fully normalized for PostgreSQL** following relational database best practices.

### Schema Overview

The database includes comprehensive tables for a multi-AI chat platform:

**Core Tables:**
- `users` - User accounts with OIDC authentication
- `conversations` - Chat conversations with multi-provider support
- `messages` - Individual messages with model/endpoint tracking
- `ai_models` - Reference table for AI model metadata
- `user_api_keys` - Encrypted API key storage per user/provider

**Organization & Configuration:**
- `presets` - Saved conversation configurations
- `agents` - AI agent definitions with tools and instructions
- `assistants` - OpenAI Assistants API compatibility
- `tags` - User-defined tags for conversation organization
- `files` - File uploads and attachments

**Relationships (Junction Tables):**
- `conversation_tags_map` - Many-to-many: conversations ↔ tags
- `agent_tools` - Many-to-many: agents ↔ tools with per-agent configuration
- `assistant_tools` - Many-to-many: assistants ↔ tools
- `agent_actions` - Many-to-many: agents ↔ custom actions
- `project_agents` - Many-to-many: projects ↔ agents
- `agent_hierarchy` - Many-to-many: parent agents ↔ sub-agents

**Additional Tables:**
- `tools` - System and user-defined tool catalog
- `actions` - Custom tools/plugins (OpenAPI, functions, webhooks)
- `tool_calls` - Function/tool execution logs
- `transactions` - Token usage tracking for billing/analytics
- `shared_links` - Conversation sharing functionality
- `projects`, `prompt_groups`, `prompts` - Advanced organization features

### Schema Design Principles

✅ **Proper Normalization:**
- All lookup/reference data uses proper tables and foreign keys
- Junction tables for many-to-many relationships
- No redundant fields (e.g., `user_id` removed from messages)

✅ **JSONB for Dynamic Data:**
- `model_parameters` - Provider-specific AI settings (varies by provider)
- `feature_flags` - Optional boolean flags, provider-specific
- `tool_resources` - Provider-specific tool configuration
- `metadata` - Extension points for future features

✅ **Performance Optimizations:**
- Strategic composite indexes for common queries
- Partial indexes for filtered queries
- GIN indexes for arrays and full-text search
- Denormalized `model`/`endpoint` in messages for historical accuracy

📖 **For detailed schema documentation**, see [`SCHEMA_CHANGES_SUMMARY.md`](SCHEMA_CHANGES_SUMMARY.md)

### Schema & Models

-   Diesel generates schema definitions in `server/src/db/schema.rs` (via `diesel print-schema`)

-   Domain models live in `server/src/db/models/`

-   Data access is encapsulated in `server/src/db/repositories/`

-   Auto-migrations run on startup when `AUTO_MIGRATE=true` (default for local dev)

### Adding New Tables

1. `cd server`

2. `diesel migration generate add_feature_x`

3. Edit the generated `up.sql` / `down.sql` in `server/migrations/<timestamp>_*`

4. Add or update models in `server/src/db/models/`

5. Extend repositories in `server/src/db/repositories/` as needed

6. Run `diesel migration run` (or restart the server with auto-migrate enabled)

**Note**: Follow the normalization principles outlined in `SCHEMA_CHANGES_SUMMARY.md` - use proper tables and foreign keys for lookup data, JSONB only for truly dynamic/provider-specific fields.

For detailed instructions, see `[server/README.md](server/README.md)`.

## 📚 **Learning Resources**

-   **React**: [react.dev](https://react.dev)

-   **Rust**: [rust-lang.org](https://www.rust-lang.org)

-   **Axum**: [github.com/tokio-rs/axum](https://github.com/tokio-rs/axum)

-   **Diesel**: [diesel.rs](https://diesel.rs)

-   **Tailwind CSS**: [tailwindcss.com](https://tailwindcss.com)

-   **ShadCN/UI**: [ui.shadcn.com](https://ui.shadcn.com)

-   **Cloudflare Pages**: [developers.cloudflare.com/pages](https://developers.cloudflare.com/pages)

-   **OIDC**: [openid.net/specs/openid-connect-core-1_0.html](https://openid.net/specs/openid-connect-core-1_0.html)

## 🆘 **Troubleshooting**

### Development Issues

**Backend won't start:**

```bash
cd server
# Check environment variables
cat .env
# Rebuild the project
cargo build
# Run the server
cargo run
```

**Database connection errors:**

```bash
cd server
# Check DATABASE_URL in .env file
# Verify database is running and accessible
# Run migrations manually if needed
diesel migration run
```

**Frontend build errors:**

```bash
cd ui
# Clear cache and reinstall
rm -rf node_modules .vite dist
pnpm install
```

### Authentication Issues

**Local Development:**

-   Ensure OIDC provider is configured correctly
-   Check OIDC environment variables in `server/.env`
-   Verify redirect URIs match your OIDC provider configuration

**Production Mode:**

1. **Verify OIDC environment variables**: `server/.env` (OIDC_ISSUER_URL, OIDC_CLIENT_ID, etc.)

2. **Check OIDC provider configuration**: Ensure redirect URIs match your application URLs

3. **Verify JWT_SECRET**: Ensure a secure JWT secret is set for session tokens

### Deployment Issues

1. **Verify build succeeds locally**

-   Frontend: `cd ui && pnpm build`

-   Backend: `cd server && cargo build --release`

1. **Check environment variables** for both frontend (Cloudflare Pages) and backend (your hosting platform)

2. **Review logs** in your hosting platform's dashboard

3. **Test backend endpoints** independently before connecting frontend

## 📋 **Development Plan**

T3Chat follows a phased development approach to transform into a full LibreChat-inspired platform:

**Phase 1: Database Foundation & Core Backend Infrastructure**
- ✅ Database schema design (fully normalized PostgreSQL)
- ⏭️ Database migrations and Rust models
- ⏭️ Frontend project structure and base components

**Phase 2: AI Provider Abstraction & Chat Functionality**
- ⏭️ AI Provider trait system (OpenAI, Anthropic, Google)
- ⏭️ Multi-provider chat interface
- ⏭️ Streaming message support

**Future Phases:**
- Agent system with tools
- Preset management
- File upload and multimodal support
- Advanced features (search, branching, etc.)

📖 **For the complete development plan**, see [`plan.md`](plan.md)

## 🎯 **Next Steps**

1. **Review the architecture**: Read [`plan.md`](plan.md) and [`SCHEMA_CHANGES_SUMMARY.md`](SCHEMA_CHANGES_SUMMARY.md)

2. **Set up the database**: Run migrations to create the normalized schema

3. **Explore the code**: Start with `ui/src/App.tsx` and `server/src/main.rs`

4. **Implement AI providers**: Follow Phase 2A in the development plan

5. **Build the chat interface**: Follow Phase 2B in the development plan

6. **Deploy**: Deploy frontend to Cloudflare Pages and backend to your preferred hosting platform

---

## 🦀 **Rust Backend Details**

For detailed information about the Rust backend, including:

-   API routes and endpoints

-   Authentication implementation

-   Database migrations and normalized schema

-   AI Provider abstraction system

-   Deployment options

-   Differences from the Node.js version

See `[server/README.md](server/README.md)` for comprehensive documentation.

**Key Backend Features:**
- Trait-based AI provider system for extensibility
- Fully normalized PostgreSQL schema with proper relationships
- Repository pattern for data access
- Encrypted API key storage (AES-256-GCM)
- Streaming support via Server-Sent Events (SSE)

---

**Happy coding!** 🚀

Need help? Check the detailed documentation in each workspace (`server/README.md`, `ui/README.md`) or visit the [community discussions](https://github.com/VoloBuilds/create-volo-app/discussions).
