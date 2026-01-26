use anyhow::{Context, Result};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbPool,
    models::{Action, NewAction, NewTool, Tool, UpdateAction, UpdateTool},
    schema::{actions, agent_tools, assistant_tools, tools},
};

#[async_trait]
pub trait TToolRepository: Send + Sync {
    // Tool methods
    async fn create(&self, new_tool: NewTool) -> Result<Tool>;
    async fn get(&self, id: Uuid) -> Result<Option<Tool>>;
    async fn list_active(&self) -> Result<Vec<Tool>>;
    async fn list_active_for_agent(&self, agent_id: Uuid) -> Result<Vec<Tool>>;
    async fn list_active_for_assistant(&self, assistant_id: Uuid) -> Result<Vec<Tool>>;
    async fn update(&self, id: Uuid, update: UpdateTool) -> Result<Tool>;
    async fn delete(&self, id: Uuid) -> Result<bool>;

    // Action methods
    async fn create_action(&self, new_action: NewAction) -> Result<Action>;
    async fn get_action(&self, id: Uuid) -> Result<Option<Action>>;
    async fn list_actions(&self, user_id: &str) -> Result<Vec<Action>>;
    async fn update_action(&self, id: Uuid, update: UpdateAction) -> Result<Action>;
    async fn delete_action(&self, id: Uuid) -> Result<bool>;
}

pub struct ToolRepository {
    pool: DbPool,
}

impl ToolRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TToolRepository for ToolRepository {
    // ==========================================
    // TOOL OPERATIONS
    // ==========================================

    /// Create a new tool
    async fn create(&self, new_tool: NewTool) -> Result<Tool> {
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
    async fn get(&self, id: Uuid) -> Result<Option<Tool>> {
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
    async fn list_active(&self) -> Result<Vec<Tool>> {
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

    /// List active tools available for a specific agent
    async fn list_active_for_agent(&self, agent_id: Uuid) -> Result<Vec<Tool>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        tools::table
            .inner_join(agent_tools::table.on(agent_tools::tool_id.eq(tools::id)))
            .filter(agent_tools::agent_id.eq(agent_id))
            .filter(agent_tools::is_enabled.eq(true).or(agent_tools::is_enabled.is_null()))
            .filter(tools::is_active.eq(true).or(tools::is_active.is_null()))
            .select(tools::all_columns)
            .order(tools::display_name.asc())
            .load(&mut conn)
            .await
            .context("Failed to list agent tools")
    }

    /// List active tools available for a specific assistant
    async fn list_active_for_assistant(&self, assistant_id: Uuid) -> Result<Vec<Tool>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        tools::table
            .inner_join(assistant_tools::table.on(assistant_tools::tool_id.eq(tools::id)))
            .filter(assistant_tools::assistant_id.eq(assistant_id))
            .filter(tools::is_active.eq(true).or(tools::is_active.is_null()))
            .select(tools::all_columns)
            .order(tools::display_name.asc())
            .load(&mut conn)
            .await
            .context("Failed to list assistant tools")
    }

    /// Update tool
    async fn update(&self, id: Uuid, update: UpdateTool) -> Result<Tool> {
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
    async fn delete(&self, id: Uuid) -> Result<bool> {
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
    async fn create_action(&self, new_action: NewAction) -> Result<Action> {
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
    async fn get_action(&self, id: Uuid) -> Result<Option<Action>> {
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
    async fn list_actions(&self, user_id: &str) -> Result<Vec<Action>> {
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
    async fn update_action(&self, id: Uuid, update: UpdateAction) -> Result<Action> {
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
    async fn delete_action(&self, id: Uuid) -> Result<bool> {
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
