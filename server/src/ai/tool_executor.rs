use anyhow::Result;
use async_stream::stream;
use futures::Stream;
use serde_json::Value;
use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;

use crate::ai::types::AgentEvent;
use crate::db::models::Tool;
use crate::db::repositories::tool_repository::{TToolRepository, ToolRepository};

#[derive(Clone)]
pub struct ToolExecutor {
    tool_repository: Arc<ToolRepository>,
}

impl ToolExecutor {
    pub fn new(tool_repository: Arc<ToolRepository>) -> Self {
        Self { tool_repository }
    }

    pub async fn get_available_tools(
        &self,
        agent_id: Option<Uuid>,
        assistant_id: Option<Uuid>,
    ) -> Result<Vec<Tool>> {
        if let Some(agent_id) = agent_id {
            return self.tool_repository.list_active_for_agent(agent_id).await;
        }

        if let Some(assistant_id) = assistant_id {
            return self
                .tool_repository
                .list_active_for_assistant(assistant_id)
                .await;
        }

        Ok(Vec::new())
    }

    /// Execute a tool and stream events. Currently returns a failed tool end for unknown tools.
    pub fn execute_tool(
        &self,
        tool_name: &str,
        arguments: Value,
        tool_call_id: String,
    ) -> Pin<Box<dyn Stream<Item = AgentEvent> + Send>> {
        let tool_name = tool_name.to_string();
        let tool_type = "function".to_string();
        Box::pin(stream! {
            yield AgentEvent::ToolStart {
                tool_call_id: tool_call_id.clone(),
                tool_name: tool_name.clone(),
                tool_type: tool_type.clone(),
                arguments: arguments.clone(),
                metadata: None,
            };

            let error = format!("Tool '{}' is not implemented", tool_name);
            yield AgentEvent::ToolEnd {
                tool_call_id,
                tool_name,
                status: "failed".to_string(),
                error: Some(error),
                result: None,
            };
        })
    }
}
