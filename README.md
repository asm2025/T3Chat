# Your Volo App

Welcome to your new full-stack application! This project was created with `create-volo-app` and comes pre-configured with a modern tech stack and production-ready architecture.

## 🎯 **Philosophy**

This application provides a highly opinionated, production-ready foundation for building full-stack applications with a decoupled frontend and backend. It's designed to maximize development velocity while adhering to best practices, including clear separation of concerns and secure handling of sensitive credentials.

Many boilerplates offer a rapid 'hello world' experience for local development but often defer critical decisions about authentication, database integration, and production deployment. This template takes a different approach. We believe that the complexities of a true full-stack application - setting up auth, a database, and distinct hosting for UI and API - are largely unavoidable for production use. By addressing these components comprehensively from the start, this template aims to provide a clearer, more predictable path to a robust, deployable application, minimizing 'surprise' hurdles down the line and fostering a deeper understanding of the full stack architecture.

Start with everything running locally on your machine, then progressively connect to production services when you're ready or dive in and connect them all at app creation.

## 🚀 **What You Have**

**Frontend:**

-   ⚛️ React + TypeScript + Vite

-   🎨 Tailwind CSS + ShadCN components

-   🔐 Firebase Authentication (Google Sign-In)

**Backend:**

-   🦀 Rust API backend (Axum)

-   🗄️ PostgreSQL with Diesel

-   🔑 Firebase Authentication (JWKS-based JWT verification)

**Local Development (Default):**

-   ⚡ Runs UI + Server + DB + Auth on your computer

-   🏠 Embedded PostgreSQL database

-   🔧 Firebase Auth emulator

-   ✅ Zero sign-ins or accounts needed

## 🗂 Environment Configuration

-   Create per-environment files for both the backend and frontend:
    -   `server/.env.development`, `server/.env.staging`, `server/.env.release`
    -   `ui/.env.development`, `ui/.env.staging`, `ui/.env.release`
-   The backend resolves configuration in this order: `.env.<APP_ENV>.local`, `.env.<APP_ENV>`, `.env.local`, `.env` (default `APP_ENV=development`)
-   `pnpm run dev -- --env staging` launches the stack with staging configuration and exposes Swagger UI
-   For manual runs: `APP_ENV=staging cargo run` or `APP_ENV=release cargo run --release`
-   Vite automatically loads `ui/.env.<mode>`; the dev script forwards the selected environment via `--mode` so the frontend and backend stay aligned

**Production (when connected):**

-   🌐 Cloudflare Pages deployment ready (frontend)

-   🦀 Standalone Rust server deployment

-   🗄️ Supabase or custom PostgreSQL

-   🔐 Production Firebase Auth

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

Start both frontend and backend (with embedded PostgreSQL database and Firebase emulator):

```bash
pnpm run dev
```

This automatically assigns available ports and displays them on startup:

-   **Frontend**: Usually `http://localhost:3010` (or next available)

-   **Backend API**: Usually `http://localhost:3000` (or next available)

-   **PostgreSQL**: Embedded database on dynamic port (starts from 5433)

The system handles port conflicts automatically. For multiple projects, use separate folders.

> **📋 Port Management**: See `[docs/PORT_HANDLING.md](docs/PORT_HANDLING.md)` for details on running multiple instances and port conflict resolution.

### Individual Commands

```bash
# Frontend only
cd ui && pnpm dev

# Backend only (Rust server)
cd server && cargo run

# Backend with custom port
cd server && cargo run -- --port 8788

# Build frontend
cd ui && pnpm build

# Build Rust server for production
cd server && cargo build --release
```

## 🔗 **Connecting Production Services**

Your app defaults to everything running locally. Connect to production services when you're ready:

### Connect Production Database

```bash
# Choose from available providers
pnpm connect:database

# Or connect to specific provider
pnpm connect:database:supabase  # Supabase PostgreSQL (recommended)
pnpm connect:database:custom    # Custom PostgreSQL

# Note: Neon Database is not supported with Rust backend (use Supabase or standard PostgreSQL)
```

### Connect Production Authentication

```bash
# Set up production Firebase Auth
pnpm connect:auth
```

### Connect Production Deployment

```bash
# Set up Cloudflare Pages deployment (frontend only)
pnpm connect:deploy

# Note: Rust backend requires standalone deployment (not Cloudflare Workers)
```

### Check Connection Status

```bash
# See what's connected to production vs local
pnpm connection:status
```

**What happens when you connect services:**

-   Your `.env` files are automatically updated

-   A backup of your current config is created

-   You can always revert to local development by restoring the backup

## 📁 **Project Structure**

```
├── ui/    # React frontend
│   ├── src/
│   │   ├── components/    # UI components (ShadCN + custom)
│   │   ├── lib/    # Utilities & Firebase config
│   │   └── pages/    # Route-level components
│   └── package.json
├── server/    # Rust API backend (Axum + Diesel)
│   ├── src/
│   │   ├── main.rs    # Application entry point & router
│   │   ├── api/    # Versioned HTTP handlers
│   │   ├── ai/    # AI provider integrations
│   │   ├── db/    # Diesel models, repositories, schema
│   │   └── middleware/    # Auth and request middleware
│   ├── migrations/    # Diesel SQL migrations (embedded)
│   ├── wwwroot/    # Static files served by the backend
│   ├── Cargo.toml    # Rust dependencies
│   └── .env    # Backend environment variables (local only)
├── database-server/    # Embedded PostgreSQL bootstrap (Node.js)
│   └── src/    # Embedded Postgres lifecycle helpers
├── data/    # Local development data
│   ├── postgres/    # Embedded PostgreSQL data
│   └── firebase-emulator/    # Firebase emulator data (auto-backed up)
└── scripts/    # Workspace automation
    ├── run-dev.js    # Development server runner
    ├── post-setup.js    # Setup automation
    └── periodic-emulator-backup.js # Firebase data backup
```

## 🔧 **Customization**

### Adding API Routes

-   Handlers live under `server/src/api/v1/` (grouped by resource).

-   Routes are registered inside `setup_router` in `server/src/main.rs`.

-   Wrap protected routes with `middleware::auth::auth_middleware` via `route_layer`.

### Database Changes

The backend uses Diesel with async connection pooling. Migrations are SQL files embedded from `server/migrations`.

1. Install Diesel CLI (PostgreSQL) if you haven't: `cargo install diesel_cli --no-default-features --features postgres`

2. Generate a migration inside `server/`: `diesel migration generate add_feature_x`

3. Edit the generated `up.sql` and `down.sql`

4. Run `diesel migration run` (from `server/`) or rely on the server's startup auto-migrate (`AUTO_MIGRATE=true`)

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

> **Note**: Embedded PostgreSQL is for local development only. Production deployments require an external database (configured during setup).

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

-   `DATABASE_URL` - PostgreSQL connection string

-   `FIREBASE_PROJECT_ID` - Firebase project ID

-   `PORT` - Server port (optional, defaults to 3000)

**Note**: Cloudflare Workers deployment is not supported for the Rust backend. For serverless deployment, consider platforms like [Fly.io](http://Fly.io), Railway, or Render that support Rust applications.

### Frontend (Cloudflare Pages)

1. **Connect to Git**: Link your repository to [Cloudflare Pages](https://dash.cloudflare.com/pages)

2. **Build Settings**:

-   Build command: `pnpm run build`

-   Build output: `ui/dist`

1. **Deploy**: Automatic on every git push

### Environment Variables (Production)

**Backend Server Environment Variables:**

-   `DATABASE_URL` - Your database connection string

-   `FIREBASE_PROJECT_ID` - Firebase project ID

-   `PORT` - Server port (optional, defaults to 3000)

-   `NODE_ENV` - Set to `production` for production mode

**Pages Environment Variables (Frontend):**

-   `VITE_API_URL` - Your deployed backend server URL (e.g., `https://api.yourdomain.com`)

### Post-Deployment Setup

1. **Update Firebase authorized domains**:

-   Go to [Firebase Console](https://console.firebase.google.com) > Authentication > Settings

-   Add your Pages domain (e.g., `your-app.pages.dev`)

1. **Test your deployment**:

```bash
curl https://api.yourdomain.com/api/v1/hello
```

## 🔐 **Authentication Flow**

Your app includes a complete authentication system that works in both local and production modes:

### Local Mode (Default)

1. **Sign in**: Use any email/password combination in the UI

2. **Storage**: User data stored in local Firebase emulator

3. **API calls**: Authenticated requests work normally

4. **Development**: No external accounts needed

### Production Mode (After `pnpm connect:auth`)

1. **Login**: Users sign in with Google (or other configured providers)

2. **Token**: Frontend gets Firebase ID token

3. **API calls**: Token sent in `Authorization: Bearer <token>` header

4. **Verification**: Backend verifies token and creates/finds user in database

5. **Protection**: Protected routes automatically have user context

### Example API Call

```typescript
// Frontend (already implemented in lib/serverComm.ts)
const response = await api.getCurrentUser();
console.log(response.user);
```

## 🗄️ **Database**

The backend uses Diesel with async pooling (`diesel_async`) and repository helpers. Migrations are plain SQL embedded from `server/migrations`.

### Schema & Models

-   Diesel generates schema definitions in `server/src/db/schema.rs` (via `diesel print-schema`)

-   Domain models live in `server/src/db/models/`

-   Data access is encapsulated in `server/src/db/repositories/`

-   Auto-migrations run on startup when `AUTO_MIGRATE=true` (default for local dev)

### Adding New Tables

1. `cd server`

2. `diesel migration generate add_users_table`

3. Edit the generated `up.sql` / `down.sql` in `server/migrations/<timestamp>_*`

4. Add or update models in `server/src/db/models/`

5. Extend repositories in `server/src/db/repositories/` as needed

6. Run `diesel migration run` (or restart the server with auto-migrate enabled)

For detailed instructions, see `[server/README.md](server/README.md)`.

## 📚 **Learning Resources**

-   **React**: [react.dev](https://react.dev)

-   **Rust**: [rust-lang.org](https://www.rust-lang.org)

-   **Axum**: [github.com/tokio-rs/axum](https://github.com/tokio-rs/axum)

-   **Diesel**: [diesel.rs](https://diesel.rs)

-   **Tailwind CSS**: [tailwindcss.com](https://tailwindcss.com)

-   **ShadCN/UI**: [ui.shadcn.com](https://ui.shadcn.com)

-   **Cloudflare Pages**: [developers.cloudflare.com/pages](https://developers.cloudflare.com/pages)

-   **Firebase Auth**: [firebase.google.com/docs/auth](https://firebase.google.com/docs/auth)

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

-   Firebase emulator should start automatically with `pnpm dev`

-   Try signing in with any email/password combination

-   Check `data/firebase-emulator/` for persisted data

-   **Data Protection**: Emulator data is automatically backed up every 60 seconds and on clean shutdown to prevent data loss during crashes

**Production Mode:**

1. **Check Firebase config**: `ui/src/lib/firebase-config.json`

2. **Verify environment variables**: `server/.env`

3. **Check authorized domains** in Firebase Console

### Deployment Issues

1. **Verify build succeeds locally**

-   Frontend: `cd ui && pnpm build`

-   Backend: `cd server && cargo build --release`

1. **Check environment variables** for both frontend (Cloudflare Pages) and backend (your hosting platform)

2. **Review logs** in your hosting platform's dashboard

3. **Test backend endpoints** independently before connecting frontend

## 🎯 **Next Steps**

1. **Explore the code**: Start with `ui/src/App.tsx` and `server/src/main.rs`

2. **Customize the UI**: Modify components and styling

3. **Add features**: Build your app logic in both frontend and backend

4. **Deploy**: Deploy frontend to Cloudflare Pages and backend to your preferred hosting platform

---

## 🦀 **Rust Backend Details**

For detailed information about the Rust backend, including:

-   API routes and endpoints

-   Authentication implementation

-   Database migrations

-   Deployment options

-   Differences from the Node.js version

See `[server/README.md](server/README.md)` for comprehensive documentation.

---

**Happy coding!** 🚀

Need help? Check the detailed documentation in each workspace (`server/README.md`, `ui/README.md`) or visit the [community discussions](https://github.com/VoloBuilds/create-volo-app/discussions).
