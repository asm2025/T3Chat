use anyhow::Result;
use sea_orm::{Database, DatabaseConnection};

pub async fn get_database(url: &str) -> Result<DatabaseConnection> {
    // Create connection - SeaORM manages pooling internally
    let db = Database::connect(url).await?;
    Ok(db)
}

pub async fn test_database_connection(db: &DatabaseConnection) -> Result<bool> {
    use sea_orm::{DbBackend, Statement};
    
    match db.execute(Statement::from_string(
        DbBackend::Postgres,
        "SELECT 1".to_string(),
    )).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
