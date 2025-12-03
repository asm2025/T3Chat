use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{Action, NewAction, NewTool, Tool, UpdateAction, UpdateTool},
    schema::{actions, tools},
    DbPool,
};

pub struct ToolRepository {
    pool: DbPool,
}

impl ToolRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    // ==========================================
    // TOOL OPERATIONS
    // ==========================================

    /// Create a new tool
    pub async fn create_tool(&self, new_tool: NewTool) -> Result<Tool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::insert_into(tools::table)
            .values(&new_tool)
            .get_result(&mut conn)
            .await
            .context("Failed to create tool")
    }

    /// Get tool by ID
    pub async fn get_tool_by_id(&self, id: Uuid) -> Result<Option<Tool>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        tools::table
            .filter(tools::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get tool")
    }

    /// List all active tools
    pub async fn list_active_tools(&self) -> Result<Vec<Tool>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        tools::table
            .filter(tools::is_active.eq(true).or(tools::is_active.is_null()))
            .order(tools::display_name.asc())
            .load(&mut conn)
            .await
            .context("Failed to list tools")
    }

    /// Update tool
    pub async fn update_tool(&self, id: Uuid, update: UpdateTool) -> Result<Tool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(tools::table)
            .filter(tools::id.eq(id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update tool")
    }

    /// Delete tool
    pub async fn delete_tool(&self, id: Uuid) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let deleted = diesel::delete(tools::table)
            .filter(tools::id.eq(id))
            .filter(tools::is_system.eq(false).or(tools::is_system.is_null()))
            .execute(&mut conn)
            .await
            .context("Failed to delete tool")?;

        Ok(deleted > 0)
    }

    // ==========================================
    // ACTION OPERATIONS
    // ==========================================

    /// Create a new action
    pub async fn create_action(&self, new_action: NewAction) -> Result<Action> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::insert_into(actions::table)
            .values(&new_action)
            .get_result(&mut conn)
            .await
            .context("Failed to create action")
    }

    /// Get action by ID
    pub async fn get_action_by_id(&self, id: Uuid) -> Result<Option<Action>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        actions::table
            .filter(actions::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get action")
    }

    /// List actions by user
    pub async fn list_actions_by_user(&self, user_id: &str) -> Result<Vec<Action>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        actions::table
            .filter(actions::user_id.eq(user_id))
            .order(actions::created_at.desc())
            .load(&mut conn)
            .await
            .context("Failed to list actions")
    }

    /// Update action
    pub async fn update_action(&self, id: Uuid, update: UpdateAction) -> Result<Action> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(actions::table)
            .filter(actions::id.eq(id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update action")
    }

    /// Delete action
    pub async fn delete_action(&self, id: Uuid) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let deleted = diesel::delete(actions::table)
            .filter(actions::id.eq(id))
            .execute(&mut conn)
            .await
            .context("Failed to delete action")?;

        Ok(deleted > 0)
    }
}
