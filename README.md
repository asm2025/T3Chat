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
-   🗄️ PostgreSQL with SeaORM
-   🔑 Firebase Authentication (JWKS-based JWT verification)

**Local Development (Default):**

-   ⚡ Runs UI + Server + DB + Auth on your computer
-   🏠 Embedded PostgreSQL database
-   🔧 Firebase Auth emulator
-   ✅ Zero sign-ins or accounts needed

**Production (when connected):**

-   🌐 Cloudflare Pages deployment ready (frontend)
-   🦀 Standalone Rust server deployment
-   🗄️ Supabase or custom PostgreSQL
-   🔐 Production Firebase Auth

## 🛠️ **Development**

Start both frontend and backend (with embedded PostgreSQL database and Firebase emulator):

```bash
pnpm run dev
```

This automatically assigns available ports and displays them on startup:

-   **Frontend**: Usually `http://localhost:5173` (or next available)
-   **Backend API**: Usually `http://localhost:8787` (or next available)
-   **PostgreSQL**: Embedded database on dynamic port (starts from 5433)

The system handles port conflicts automatically. For multiple projects, use separate folders.

> **📋 Port Management**: See [`docs/PORT_HANDLING.md`](docs/PORT_HANDLING.md) for details on running multiple instances and port conflict resolution.

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
├── ui/                    # React frontend
│   ├── src/
│   │   ├── components/    # UI components (ShadCN)
│   │   ├── lib/          # Utilities & Firebase config
│   │   └── App.tsx       # Main app component
│   └── package.json
├── server/               # Rust API backend (Axum)
│   ├── src/
│   │   ├── api.rs        # API routes
│   │   ├── main.rs       # Application entry point
│   │   ├── db/           # Database layer
│   │   │   ├── repositories/  # Data access layer
│   │   │   └── schema/        # Database schema (SeaORM entities)
│   │   ├── middleware/   # Auth & other middleware
│   │   └── env.rs        # Environment configuration
│   ├── migration/        # SeaORM migrations
│   ├── Cargo.toml        # Rust dependencies
│   └── .env              # Your environmental variables
├── database-server/      # Embedded PostgreSQL server (Node.js)
│   └── src/              # Database server implementation
├── data/                 # Local development data
│   ├── postgres/         # Embedded PostgreSQL data
│   └── firebase-emulator/ # Firebase emulator data (auto-backed up)
└── scripts/
    ├── post-setup.js     # Setup automation
    ├── run-dev.js        # Development server runner
    └── periodic-emulator-backup.js # Firebase data backup (runs automatically)
```

## 🔧 **Customization**

### Adding API Routes

Edit `server/src/api.rs`:

```rust
// Add to the existing router
router = router.route("/your-route", get(|| async {
    Json(json!({ "message": "Hello!" }))
}));

// For protected routes, use the auth middleware:
router = router.route("/private-route", get(
    require_auth(|| async move |user: AuthenticatedUser| {
        Json(json!({ "user": user }))
    })
));
```

### Database Changes

1. Create a new migration: `cd server/migration && cargo run`
2. Edit the generated migration file in `server/migration/src/`
3. Apply migration: The server auto-migrates on startup, or run manually with `cd server/migration && cargo run`

For detailed migration instructions, see [`server/README.md`](server/README.md) and [`server/migration/README.md`](server/migration/README.md).

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
./target/release/t3chat-server --port 8787
```

**Environment variables required:**

-   `DATABASE_URL` - PostgreSQL connection string
-   `FIREBASE_PROJECT_ID` - Firebase project ID
-   `PORT` - Server port (optional, defaults to 8787)

**Note**: Cloudflare Workers deployment is not supported for the Rust backend. For serverless deployment, consider platforms like Fly.io, Railway, or Render that support Rust applications.

### Frontend (Cloudflare Pages)

1. **Connect to Git**: Link your repository to [Cloudflare Pages](https://dash.cloudflare.com/pages)
2. **Build Settings**:
    - Build command: `pnpm run build`
    - Build output: `ui/dist`
3. **Deploy**: Automatic on every git push

### Environment Variables (Production)

**Backend Server Environment Variables:**

-   `DATABASE_URL` - Your database connection string
-   `FIREBASE_PROJECT_ID` - Firebase project ID
-   `PORT` - Server port (optional, defaults to 8787)
-   `NODE_ENV` - Set to `production` for production mode

**Pages Environment Variables (Frontend):**

-   `VITE_API_URL` - Your deployed backend server URL (e.g., `https://api.yourdomain.com`)

### Post-Deployment Setup

1. **Update Firebase authorized domains**:

    - Go to [Firebase Console](https://console.firebase.google.com) > Authentication > Settings
    - Add your Pages domain (e.g., `your-app.pages.dev`)

2. **Test your deployment**:
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

Your database is set up with SeaORM and works the same whether local or production:

### User Schema (included)

The database uses SeaORM entities located in `server/src/db/schema/`. The server automatically runs migrations on startup when `AUTO_MIGRATE=true` is set in the environment.

### Adding New Tables

1. Create a new migration: `cd server/migration && cargo run`
2. Edit the generated migration file in `server/migration/src/`
3. Create corresponding SeaORM entity in `server/src/db/schema/`
4. The migration will run automatically on next server startup, or run manually: `cd server/migration && cargo run`

For detailed instructions, see [`server/README.md`](server/README.md) and [`server/migration/README.md`](server/migration/README.md).

## 📚 **Learning Resources**

-   **React**: [react.dev](https://react.dev)
-   **Rust**: [rust-lang.org](https://www.rust-lang.org)
-   **Axum**: [github.com/tokio-rs/axum](https://github.com/tokio-rs/axum)
-   **SeaORM**: [sea-ql.org/SeaORM](https://www.sea-ql.org/SeaORM)
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
cd migration && cargo run
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
    - Frontend: `cd ui && pnpm build`
    - Backend: `cd server && cargo build --release`
2. **Check environment variables** for both frontend (Cloudflare Pages) and backend (your hosting platform)
3. **Review logs** in your hosting platform's dashboard
4. **Test backend endpoints** independently before connecting frontend

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

See [`server/README.md`](server/README.md) for comprehensive documentation.

---

**Happy coding!** 🚀

Need help? Check the detailed documentation in each workspace (`server/README.md`, `ui/README.md`) or visit the [community discussions](https://github.com/VoloBuilds/create-volo-app/discussions).
