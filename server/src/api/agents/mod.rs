use crate::{
    AppState,
    db::{
        models::{Agent, AgentChatStarter, NewAgent, NewAgentChatStarter, UpdateAgent},
        repositories::agent_repository::TAgentRepository,
        schema::{agent_chat_starters, agent_tools},
    },
    middleware::auth::AuthenticatedUser,
};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub model: String,
    pub provider: String,
    pub model_parameters: Option<Value>,
    pub avatar: Option<Value>, // { filepath, source }
    pub project_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub model_parameters: Option<Value>,
    pub avatar: Option<Value>,
    pub project_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddToolsRequest {
    pub tool_ids: Vec<Uuid>,
    pub configuration: Option<Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoveToolsRequest {
    pub tool_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddChatStartersRequest {
    pub starters: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChatStartersRequest {
    pub starters: Vec<ChatStarterItem>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChatStarterItem {
    pub id: Option<String>,
    pub text: String,
    pub order_index: i32,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_agents).post(create_agent))
        .route(
            "/:id",
            get(get_agent).put(update_agent).delete(delete_agent),
        )
        .route("/:id/tools", post(add_tools).delete(remove_tools))
        .route(
            "/:id/chat-starters",
            post(add_chat_starters).put(update_chat_starters),
        )
}

// ... Implement CRUD handlers ...

async fn list_agents(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Agent>>, StatusCode> {
    let agents = state.agent_repository.list(&user.0.id).await.map_err(|e| {
        tracing::error!("Failed to list agents: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(agents))
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentWithDetails {
    #[serde(flatten)]
    pub agent: Agent,
    pub tools: Option<Vec<crate::db::models::Tool>>,
    pub chat_starters: Option<Vec<AgentChatStarter>>,
}

async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentWithDetails>, StatusCode> {
    let agent = state
        .agent_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Fetch tools and chat starters
    let mut conn = state
        .db
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    use crate::db::schema::{agent_chat_starters, agent_tools, tools};
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let agent_tool_ids: Vec<Uuid> = agent_tools::table
        .filter(agent_tools::agent_id.eq(id))
        .filter(
            agent_tools::is_enabled
                .eq(true)
                .or(agent_tools::is_enabled.is_null()),
        )
        .select(agent_tools::tool_id)
        .load::<Uuid>(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let agent_tools_list: Vec<crate::db::models::Tool> = if agent_tool_ids.is_empty() {
        Vec::new()
    } else {
        tools::table
            .filter(tools::id.eq_any(agent_tool_ids))
            .load::<crate::db::models::Tool>(&mut conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let chat_starters_list: Vec<AgentChatStarter> = agent_chat_starters::table
        .filter(agent_chat_starters::agent_id.eq(id))
        .order(agent_chat_starters::order_index.asc())
        .load::<AgentChatStarter>(&mut conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AgentWithDetails {
        agent,
        tools: Some(agent_tools_list),
        chat_starters: Some(chat_starters_list),
    }))
}

async fn create_agent(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateAgentRequest>,
) -> Result<Json<Agent>, StatusCode> {
    let avatar_filepath = payload
        .avatar
        .as_ref()
        .and_then(|v| v.get("filepath"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let avatar_source = payload
        .avatar
        .as_ref()
        .and_then(|v| v.get("source"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let new_agent = NewAgent {
        id: Some(Uuid::new_v4()),
        agent_id: Uuid::new_v4().to_string(), // Or generate friendly ID
        author_id: user.0.id.clone(),
        name: payload.name,
        description: payload.description,
        instructions: payload.instructions,
        avatar_filepath,
        avatar_source,
        provider: payload.provider,
        model: payload.model,
        model_parameters: payload.model_parameters,
        access_level: Some(1), // Private by default
        recursion_limit: None,
        hide_sequential_outputs: None,
        end_after_tools: None,
        is_collaborative: None,
        tool_resources: None,
    };

    let agent = state
        .agent_repository
        .create(new_agent)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create agent: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(agent))
}

async fn update_agent(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAgentRequest>,
) -> Result<Json<Agent>, StatusCode> {
    // Verify ownership (TODO: or admin)
    let existing = state
        .agent_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if existing.author_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let avatar_filepath = payload
        .avatar
        .as_ref()
        .and_then(|v| v.get("filepath"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .map(Some); // Wrap for UpdateAgent Option<Option<String>>

    let avatar_source = payload
        .avatar
        .as_ref()
        .and_then(|v| v.get("source"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .map(Some);

    let update = UpdateAgent {
        name: payload.name,
        description: payload.description.map(Some),
        instructions: payload.instructions.map(Some),
        avatar_filepath,
        avatar_source,
        provider: payload.provider,
        model: payload.model,
        model_parameters: payload.model_parameters,
        access_level: None,
        recursion_limit: None,
        hide_sequential_outputs: None,
        end_after_tools: None,
        is_collaborative: None,
        tool_resources: None,
        updated_at: chrono::Utc::now(),
    };

    let agent = state
        .agent_repository
        .update(id, update)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update agent: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(agent))
}

async fn delete_agent(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Verify ownership
    let existing = state
        .agent_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if existing.author_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    state
        .agent_repository
        .delete(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

async fn add_tools(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AddToolsRequest>,
) -> Result<StatusCode, StatusCode> {
    // Verify ownership
    let agent = state
        .agent_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if agent.author_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut conn = state
        .db
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for tool_id in payload.tool_ids {
        diesel::insert_into(agent_tools::table)
            .values((
                agent_tools::agent_id.eq(id),
                agent_tools::tool_id.eq(tool_id),
                agent_tools::configuration.eq(payload.configuration.clone()),
                agent_tools::is_enabled.eq(true),
            ))
            .on_conflict((agent_tools::agent_id, agent_tools::tool_id))
            .do_update()
            .set((
                agent_tools::configuration.eq(payload.configuration.clone()),
                agent_tools::is_enabled.eq(true),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                tracing::error!("Failed to add tool: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    Ok(StatusCode::OK)
}

// Note: DELETE with body is non-standard, but client sends it
// Using axum's Json extractor which works for DELETE
async fn remove_tools(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    axum::extract::Json(payload): axum::extract::Json<RemoveToolsRequest>,
) -> Result<StatusCode, StatusCode> {
    // Verify ownership
    let agent = state
        .agent_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if agent.author_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut conn = state
        .db
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(agent_tools::table)
        .filter(agent_tools::agent_id.eq(id))
        .filter(agent_tools::tool_id.eq_any(payload.tool_ids))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            tracing::error!("Failed to remove tools: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

async fn add_chat_starters(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AddChatStartersRequest>,
) -> Result<StatusCode, StatusCode> {
    // Verify ownership
    let agent = state
        .agent_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if agent.author_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut conn = state
        .db
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for (index, text) in payload.starters.iter().enumerate() {
        let new_starter = NewAgentChatStarter {
            id: Some(Uuid::new_v4()),
            agent_id: id,
            text: text.clone(),
            order_index: Some(index as i32),
        };

        diesel::insert_into(agent_chat_starters::table)
            .values(&new_starter)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                tracing::error!("Failed to add chat starter: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    Ok(StatusCode::OK)
}

async fn update_chat_starters(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateChatStartersRequest>,
) -> Result<StatusCode, StatusCode> {
    // Verify ownership
    let agent = state
        .agent_repository
        .get(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if agent.author_id != user.0.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut conn = state
        .db
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Delete existing starters
    diesel::delete(agent_chat_starters::table)
        .filter(agent_chat_starters::agent_id.eq(id))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete chat starters: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Insert new starters
    for item in payload.starters {
        let starter_id = item
            .id
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or_else(Uuid::new_v4);

        let new_starter = NewAgentChatStarter {
            id: Some(starter_id),
            agent_id: id,
            text: item.text,
            order_index: Some(item.order_index),
        };

        diesel::insert_into(agent_chat_starters::table)
            .values(&new_starter)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                tracing::error!("Failed to add chat starter: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    Ok(StatusCode::OK)
}
