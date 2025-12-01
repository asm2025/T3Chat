use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{Project, NewProject, UpdateProject, PromptGroup, NewPromptGroup, Prompt, NewPrompt},
    schema::{projects, prompt_groups, prompts},
    DbPool,
};

pub struct ProjectRepository {
    pool: DbPool,
}

impl ProjectRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    // ==========================================
    // PROJECT OPERATIONS
    // ==========================================

    /// Create a new project
    pub async fn create_project(&self, new_project: NewProject) -> Result<Project> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        diesel::insert_into(projects::table)
            .values(&new_project)
            .get_result(&mut conn)
            .await
            .context("Failed to create project")
    }

    /// Get project by ID
    pub async fn get_project_by_id(&self, id: Uuid) -> Result<Option<Project>> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        projects::table
            .filter(projects::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get project")
    }

    /// List projects by owner
    pub async fn list_projects_by_owner(&self, owner_id: &str) -> Result<Vec<Project>> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        projects::table
            .filter(projects::owner_id.eq(owner_id))
            .order(projects::created_at.desc())
            .load(&mut conn)
            .await
            .context("Failed to list projects")
    }

    /// Update project
    pub async fn update_project(&self, id: Uuid, update: UpdateProject) -> Result<Project> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        diesel::update(projects::table)
            .filter(projects::id.eq(id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update project")
    }

    /// Delete project
    pub async fn delete_project(&self, id: Uuid) -> Result<bool> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        let deleted = diesel::delete(projects::table)
            .filter(projects::id.eq(id))
            .execute(&mut conn)
            .await
            .context("Failed to delete project")?;
        
        Ok(deleted > 0)
    }

    // ==========================================
    // PROMPT GROUP OPERATIONS
    // ==========================================

    /// Create a new prompt group
    pub async fn create_prompt_group(&self, new_group: NewPromptGroup) -> Result<PromptGroup> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        diesel::insert_into(prompt_groups::table)
            .values(&new_group)
            .get_result(&mut conn)
            .await
            .context("Failed to create prompt group")
    }

    /// Get prompt group by ID
    pub async fn get_prompt_group_by_id(&self, id: Uuid) -> Result<Option<PromptGroup>> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        prompt_groups::table
            .filter(prompt_groups::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get prompt group")
    }

    /// List prompt groups by author
    pub async fn list_prompt_groups_by_author(&self, author_id: &str) -> Result<Vec<PromptGroup>> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        prompt_groups::table
            .filter(prompt_groups::author_id.eq(author_id))
            .order(prompt_groups::created_at.desc())
            .load(&mut conn)
            .await
            .context("Failed to list prompt groups")
    }

    // ==========================================
    // PROMPT OPERATIONS
    // ==========================================

    /// Create a new prompt
    pub async fn create_prompt(&self, new_prompt: NewPrompt) -> Result<Prompt> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        diesel::insert_into(prompts::table)
            .values(&new_prompt)
            .get_result(&mut conn)
            .await
            .context("Failed to create prompt")
    }

    /// Get prompt by ID
    pub async fn get_prompt_by_id(&self, id: Uuid) -> Result<Option<Prompt>> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        prompts::table
            .filter(prompts::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get prompt")
    }

    /// List prompts in a group
    pub async fn list_prompts_in_group(&self, group_id: Uuid) -> Result<Vec<Prompt>> {
        let mut conn = self.pool.get().await.context("Failed to get DB connection")?;
        
        prompts::table
            .filter(prompts::group_id.eq(group_id))
            .order(prompts::order_index.asc())
            .load(&mut conn)
            .await
            .context("Failed to list prompts")
    }
}

