pub mod dto;
pub mod models;
pub mod repositories;

// Diesel auto-generated schema
#[allow(unused_imports)]
pub mod schema;

// Re-export commonly used types
pub mod prelude {
    pub use super::models::*;
}

use anyhow::{Context, Result};
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use std::time::Duration;

pub type DbPool = Pool<AsyncPgConnection>;

/// Ensures the database exists, creating it if necessary
async fn ensure_database_exists(db_url: &str) -> Result<()> {
    // Parse the database URL to extract database name and base URL
    let url = url::Url::parse(db_url).context("Invalid DATABASE_URL format")?;

    let db_name = url.path().trim_start_matches('/').to_string();

    if db_name.is_empty() {
        anyhow::bail!("Database name not specified in DATABASE_URL");
    }

    // Create a connection to the 'postgres' database to check/create our database
    let mut base_url = url.clone();
    base_url.set_path("/postgres");
    let postgres_url = base_url.to_string();

    tracing::info!("Checking if database '{}' exists", db_name);

    tokio::task::spawn_blocking(move || -> Result<()> {
        use diesel::prelude::*;
        use diesel::sql_query;
        use diesel::sql_types::BigInt;

        let mut conn = diesel::PgConnection::establish(&postgres_url)
            .context("Failed to connect to postgres database")?;

        // Check if database exists
        #[derive(QueryableByName)]
        struct DatabaseCount {
            #[diesel(sql_type = BigInt)]
            count: i64,
        }

        let result: DatabaseCount = sql_query(format!(
            "SELECT COUNT(*) as count FROM pg_database WHERE datname = '{}'",
            db_name
        ))
        .get_result(&mut conn)
        .context("Failed to check if database exists")?;

        if result.count == 0 {
            tracing::info!("Database '{}' does not exist, creating it...", db_name);

            // Create the database
            sql_query(format!("CREATE DATABASE \"{}\"", db_name))
                .execute(&mut conn)
                .context("Failed to create database")?;

            tracing::info!("Database '{}' created successfully", db_name);
        } else {
            tracing::info!("Database '{}' already exists", db_name);
        }

        Ok(())
    })
    .await
    .context("Database existence check task failed")??;

    Ok(())
}

/// Connects to the database and returns a connection pool
pub async fn connect(db_url: &str, auto_migrate: bool) -> Result<DbPool> {
    // Ensure the database exists before connecting
    ensure_database_exists(db_url).await?;

    tracing::info!("Connecting to database");

    // Configure the connection manager
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);

    // Build the pool
    let pool = Pool::builder(config)
        .max_size(100)
        .wait_timeout(Some(Duration::from_secs(30)))
        .create_timeout(Some(Duration::from_secs(30)))
        .recycle_timeout(Some(Duration::from_secs(300)))
        .build()
        .context("Failed to create database pool")?;

    // Test the connection
    let conn = pool
        .get()
        .await
        .context("Failed to get a connection from pool")?;
    drop(conn);

    tracing::info!("Connected to database at {}", db_url);

    if auto_migrate {
        tracing::info!("Running migrations...");
        run_migrations(&pool).await?;
        tracing::info!("Migrations completed successfully.");
    }

    Ok(pool)
}

/// Run pending migrations
async fn run_migrations(_pool: &DbPool) -> Result<()> {
    use diesel::Connection;
    use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    // We need to use a synchronous connection for migrations
    // Extract the database URL from the pool config
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL must be set for migrations")?;

    // Run migrations on a synchronous connection
    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut conn = diesel::PgConnection::establish(&database_url)
            .context("Failed to establish connection for migrations")?;

        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;

        Ok(())
    })
    .await
    .context("Migration task failed")??;

    Ok(())
}

/// Test database connection
pub async fn test_database_connection(pool: &DbPool) -> Result<bool> {
    match pool.get().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
