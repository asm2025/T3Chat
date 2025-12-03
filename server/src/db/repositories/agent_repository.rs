use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    models::{Agent, Assistant, NewAgent, NewAssistant, UpdateAgent, UpdateAssistant},
    schema::{agents, assistants},
    DbPool,
};

pub struct AgentRepository {
    pool: DbPool,
}

impl AgentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    // ==========================================
    // AGENT OPERATIONS
    // ==========================================

    /// Create a new agent
    pub async fn create_agent(&self, new_agent: NewAgent) -> Result<Agent> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::insert_into(agents::table)
            .values(&new_agent)
            .get_result(&mut conn)
            .await
            .context("Failed to create agent")
    }

    /// Get agent by ID
    pub async fn get_agent_by_id(&self, id: Uuid) -> Result<Option<Agent>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        agents::table
            .filter(agents::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get agent")
    }

    /// List agents by author
    pub async fn list_agents_by_author(&self, author_id: &str) -> Result<Vec<Agent>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        agents::table
            .filter(agents::author_id.eq(author_id))
            .order(agents::created_at.desc())
            .load(&mut conn)
            .await
            .context("Failed to list agents")
    }

    /// Update agent
    pub async fn update_agent(&self, id: Uuid, update: UpdateAgent) -> Result<Agent> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(agents::table)
            .filter(agents::id.eq(id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update agent")
    }

    /// Delete agent
    pub async fn delete_agent(&self, id: Uuid) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let deleted = diesel::delete(agents::table)
            .filter(agents::id.eq(id))
            .execute(&mut conn)
            .await
            .context("Failed to delete agent")?;

        Ok(deleted > 0)
    }

    // ==========================================
    // ASSISTANT OPERATIONS
    // ==========================================

    /// Create a new assistant
    pub async fn create_assistant(&self, new_assistant: NewAssistant) -> Result<Assistant> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::insert_into(assistants::table)
            .values(&new_assistant)
            .get_result(&mut conn)
            .await
            .context("Failed to create assistant")
    }

    /// Get assistant by ID
    pub async fn get_assistant_by_id(&self, id: Uuid) -> Result<Option<Assistant>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        assistants::table
            .filter(assistants::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .context("Failed to get assistant")
    }

    /// List assistants by user
    pub async fn list_assistants_by_user(&self, user_id: &str) -> Result<Vec<Assistant>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        assistants::table
            .filter(assistants::user_id.eq(user_id))
            .order(assistants::created_at.desc())
            .load(&mut conn)
            .await
            .context("Failed to list assistants")
    }

    /// Update assistant
    pub async fn update_assistant(&self, id: Uuid, update: UpdateAssistant) -> Result<Assistant> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(assistants::table)
            .filter(assistants::id.eq(id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update assistant")
    }

    /// Delete assistant
    pub async fn delete_assistant(&self, id: Uuid) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let deleted = diesel::delete(assistants::table)
            .filter(assistants::id.eq(id))
            .execute(&mut conn)
            .await
            .context("Failed to delete assistant")?;

        Ok(deleted > 0)
    }
}
