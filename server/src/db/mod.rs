pub mod repositories;
pub mod schema;

use anyhow::{Context, Result};
use sea_orm::{prelude::*, *};
use sea_orm_migration::prelude::*;
use std::{fs, path::Path, time::Duration};

use migration::{Migrator, MigratorTrait};

pub mod prelude {
    pub use super::repositories::*;
    pub use super::schema::*;
}

/// Ensures the database exists and connects to it with auto-migration
pub async fn connect(db_url: &str, auto_migrate: bool) -> Result<DatabaseConnection> {
    tracing::info!("Connecting to database");

    // Ensure the database exists before connecting
    if auto_migrate {
        ensure_database_exists(db_url).await?;
    }

    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(30))
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(300)) // 5 minutes
        .max_lifetime(Duration::from_secs(1800)); // 30 minutes

    // Connect to the database
    let db = Database::connect(opt).await?;
    tracing::info!("Connected to database at {}", db_url);

    if auto_migrate {
        // Apply migrations
        tracing::info!("Applying migrations...");
        Migrator::up(&db, None).await?;
        tracing::info!("Migrations applied successfully.");
    }

    Ok(db)
}

/// Ensures the target database exists, creates it if it doesn't
async fn ensure_database_exists(db_url: &str) -> Result<()> {
    // Parse the database URL to extract database name
    let db_name = extract_database_name(db_url)?;

    tracing::info!("Checking if database '{}' exists...", db_name);

    let db_url_lower = db_url.to_lowercase();

    /*
     Check if the db_url starts with "sqlite://" case insensitive.
     If it does, we will add some logic to make sure the directory and file exist.
    */
    if db_url_lower.starts_with("sqlite://") {
        let db_path = if let Some(pos) = db_url.find("://") {
            &db_url[pos + 3..]
        } else {
            db_url
        };

        if !Path::new(db_path).exists() {
            // Check if the parent directory exists
            if let Some(parent) = Path::new(db_path).parent() {
                if !parent.as_os_str().is_empty() {
                    // Create the directory if it doesn't exist
                    fs::create_dir_all(parent)?;
                    tracing::info!("Created directory for database: {}", parent.display());
                }
            }

            // Touch the file to ensure it can be created
            fs::File::create(db_path)?;
            tracing::info!("Created database file: {}", db_path);
        }

        return Ok(());
    }

    if db_url_lower.starts_with("postgresql://") {
        // Try connecting to common default databases to check/create the target database
        // Common default database names: postgres, postgresql, template1
        let default_databases = vec!["postgres", "postgresql", "template1"];
        let mut db: Option<DatabaseConnection> = None;

        for default_db in &default_databases {
            let default_url = db_url.replace(&format!("/{}", db_name), &format!("/{}", default_db));

            tracing::debug!("Trying to connect to default database '{}'...", default_db);
            let mut opt = ConnectOptions::new(&default_url);
            opt.connect_timeout(Duration::from_secs(10));

            match Database::connect(opt).await {
                Ok(connection) => {
                    tracing::info!("Connected to default database '{}'", default_db);
                    db = Some(connection);
                    break;
                }
                Err(e) => {
                    tracing::debug!("Failed to connect to '{}': {}", default_db, e);
                    continue;
                }
            }
        }

        let db = db.context(format!(
            "Failed to connect to PostgreSQL server. Tried connecting to default databases: {:?}. \
         Please ensure PostgreSQL is running and accessible.",
            default_databases
        ))?;

        // Check if the database exists
        let query = format!("SELECT 1 FROM pg_database WHERE datname = '{}'", db_name);

        let result = db
            .query_one(Statement::from_string(DbBackend::Postgres, query))
            .await;

        match result {
            Ok(Some(_)) => {
                tracing::info!("Database '{}' already exists", db_name);
            }
            Ok(None) | Err(_) => {
                // Database doesn't exist, create it
                tracing::info!("Database '{}' does not exist. Creating...", db_name);

                let create_db_query = format!("CREATE DATABASE {}", db_name);
                db.execute(Statement::from_string(DbBackend::Postgres, create_db_query))
                    .await
                    .context(format!("Failed to create database '{}'", db_name))?;

                tracing::info!("Database '{}' created successfully", db_name);
            }
        }

        // Close the connection to the default database
        db.close().await?;

        return Ok(());
    }

    anyhow::bail!("Invalid database URL: {}", db_url);
}

/// Extracts the database name from a PostgreSQL connection URL
fn extract_database_name(db_url: &str) -> Result<String> {
    // URL format: postgres://user:password@host:port/database
    let parts: Vec<&str> = db_url.split('/').collect();
    let db_name = parts
        .last()
        .context("Invalid DATABASE_URL: missing database name")?
        .split('?')
        .next()
        .context("Invalid DATABASE_URL format")?
        .to_string();

    if db_name.is_empty() {
        anyhow::bail!("Database name is empty in DATABASE_URL");
    }

    Ok(db_name)
}

pub async fn test_database_connection(db: &DatabaseConnection) -> Result<bool> {
    match db
        .execute(Statement::from_string(
            DbBackend::Postgres,
            "SELECT 1".to_string(),
        ))
        .await
    {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
