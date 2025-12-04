use anyhow::{Context, Result};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbPool,
    models::{File, NewFile, UpdateFile},
    schema::files,
};

#[async_trait]
pub trait TFileRepository: Send + Sync {
    async fn create(&self, new_file: NewFile) -> Result<File>;
    async fn get(&self, id: Uuid) -> Result<Option<File>>;
    async fn list(&self, user_id: &str) -> Result<Vec<File>>;
    async fn update(&self, id: Uuid, update: UpdateFile) -> Result<File>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
}

pub struct FileRepository {
    pool: DbPool,
}

impl FileRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TFileRepository for FileRepository {
    /// Create a new file
    async fn create(&self, new_file: NewFile) -> Result<File> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::insert_into(files::table)
            .values(&new_file)
            .get_result(&mut conn)
            .await
            .context("Failed to create file")
    }

    /// Get file by ID
    async fn get(&self, id: Uuid) -> Result<Option<File>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        files::table
            .filter(files::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get file")
    }

    /// List files by user
    async fn list(&self, user_id: &str) -> Result<Vec<File>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        files::table
            .filter(files::user_id.eq(user_id))
            .order(files::created_at.desc())
            .load(&mut conn)
            .await
            .context("Failed to list files")
    }

    /// Update file
    async fn update(&self, id: Uuid, update: UpdateFile) -> Result<File> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(files::table)
            .filter(files::id.eq(id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update file")
    }

    /// Delete file
    async fn delete(&self, id: Uuid) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let deleted = diesel::delete(files::table)
            .filter(files::id.eq(id))
            .execute(&mut conn)
            .await
            .context("Failed to delete file")?;

        Ok(deleted > 0)
    }
}
