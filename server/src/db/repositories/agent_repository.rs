use anyhow::{Context, Result};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbPool,
    models::{Agent, Assistant, NewAgent, NewAssistant, UpdateAgent, UpdateAssistant},
    schema::{agents, assistants},
};

#[async_trait]
pub trait TAgentRepository: Send + Sync {
    // Agent methods
    async fn create(&self, new_agent: NewAgent) -> Result<Agent>;
    async fn get(&self, id: Uuid) -> Result<Option<Agent>>;
    async fn list(&self, author_id: &str) -> Result<Vec<Agent>>;
    async fn update(&self, id: Uuid, update: UpdateAgent) -> Result<Agent>;
    async fn delete(&self, id: Uuid) -> Result<bool>;

    // Assistant methods
    async fn create_assistant(&self, new_assistant: NewAssistant) -> Result<Assistant>;
    async fn get_assistant(&self, id: Uuid) -> Result<Option<Assistant>>;
    async fn list_assistants(&self, user_id: &str) -> Result<Vec<Assistant>>;
    async fn update_assistant(&self, id: Uuid, update: UpdateAssistant) -> Result<Assistant>;
    async fn delete_assistant(&self, id: Uuid) -> Result<bool>;
}

pub struct AgentRepository {
    pool: DbPool,
}

impl AgentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TAgentRepository for AgentRepository {
    // ==========================================
    // AGENT OPERATIONS
    // ==========================================

    /// Create a new agent
    async fn create(&self, new_agent: NewAgent) -> Result<Agent> {
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
    async fn get(&self, id: Uuid) -> Result<Option<Agent>> {
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
    async fn list(&self, author_id: &str) -> Result<Vec<Agent>> {
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
    async fn update(&self, id: Uuid, update: UpdateAgent) -> Result<Agent> {
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
    async fn delete(&self, id: Uuid) -> Result<bool> {
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
    async fn create_assistant(&self, new_assistant: NewAssistant) -> Result<Assistant> {
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
    async fn get_assistant(&self, id: Uuid) -> Result<Option<Assistant>> {
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
    async fn list_assistants(&self, user_id: &str) -> Result<Vec<Assistant>> {
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
    async fn update_assistant(&self, id: Uuid, update: UpdateAssistant) -> Result<Assistant> {
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
    async fn delete_assistant(&self, id: Uuid) -> Result<bool> {
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
