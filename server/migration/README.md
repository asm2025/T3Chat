# SeaORM Migration

This directory contains the database migrations for the server-rs project.

## Auto-Migration on Server Startup

The main server application (`server-rs`) automatically creates the database and runs migrations on startup when `auto_migrate` is enabled (which is the default). You don't need to run migrations manually for development.

**What happens automatically:**
1. Extracts database name from `DATABASE_URL` (e.g., `t3chat`)
2. Connects to PostgreSQL server (using the `postgres` database)
3. Checks if target database exists
4. Creates the database if it doesn't exist
5. Connects to the target database
6. Runs all pending migrations
7. Creates tables and indexes in the `public` schema (default)

## Running Migrations Manually

### Setup

Make sure you have the `DATABASE_URL` environment variable set:

```bash
export DATABASE_URL="postgres://username:password@localhost/database_name"
```

Or create a `.env` file in the server-rs directory with:
```
DATABASE_URL=postgres://username:password@localhost/database_name
```

**Note:** For manual migration, the database must already exist. The auto-creation feature is only available when running the main server application.

### Commands

Run all pending migrations:
```bash
cd migration
cargo run
```

Or from the server-rs directory:
```bash
cargo run --manifest-path migration/Cargo.toml
```

Check migration status:
```bash
cargo run -- status
```

Rollback the last migration:
```bash
cargo run -- down
```

Rollback all migrations:
```bash
cargo run -- down -n 999
```

Refresh (rollback all and re-apply):
```bash
cargo run -- fresh
```

Reset (drop all tables and re-apply):
```bash
cargo run -- reset
```

## Creating New Migrations

1. Create a new migration file in `src/` following the naming pattern:
   `m{YYYYMMDD}_{HHMMSS}_{description}.rs`

2. Add the migration module to `src/lib.rs`:
   ```rust
   mod m{YYYYMMDD}_{HHMMSS}_{description};
   ```

3. Add the migration to the `Migrator::migrations()` vector:
   ```rust
   Box::new(m{YYYYMMDD}_{HHMMSS}_{description}::Migration),
   ```

## Integration with Server

You can run migrations programmatically from your server code:

```rust
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;

async fn run_migrations() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::connect("DATABASE_URL").await?;
    Migrator::up(&db, None).await?;
    Ok(())
}
```

