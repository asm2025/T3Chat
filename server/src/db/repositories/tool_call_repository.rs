use anyhow::{Context, Result};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbPool,
    models::{NewToolCall, ToolCall, UpdateToolCall},
    schema::tool_calls,
};

#[async_trait]
pub trait TToolCallRepository: Send + Sync {
    async fn create(&self, new_tool_call: NewToolCall) -> Result<ToolCall>;
    async fn update_by_tool_call_id(
        &self,
        message_id: Uuid,
        tool_call_id: &str,
        update: UpdateToolCall,
    ) -> Result<ToolCall>;
}

pub struct ToolCallRepository {
    pool: DbPool,
}

impl ToolCallRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TToolCallRepository for ToolCallRepository {
    async fn create(&self, new_tool_call: NewToolCall) -> Result<ToolCall> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::insert_into(tool_calls::table)
            .values(&new_tool_call)
            .get_result(&mut conn)
            .await
            .context("Failed to create tool call")
    }

    async fn update_by_tool_call_id(
        &self,
        message_id: Uuid,
        tool_call_id: &str,
        update: UpdateToolCall,
    ) -> Result<ToolCall> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        diesel::update(tool_calls::table)
            .filter(tool_calls::message_id.eq(message_id))
            .filter(tool_calls::tool_call_id.eq(tool_call_id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .context("Failed to update tool call")
    }
}
