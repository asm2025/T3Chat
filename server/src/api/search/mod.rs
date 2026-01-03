use crate::{
    AppState,
    middleware::auth::AuthenticatedUser,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const MEILISEARCH_INDEX_NAME: &str = "chats_messages";

#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultItem {
    pub id: String,
    pub chat_id: String,
    pub title: Option<String>,
    pub text: Option<String>,
    pub role: Option<String>,
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub created_at: String,
    #[serde(rename = "type")]
    pub r#type: String, // "chat" or "message"
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(search))
}

async fn search(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, StatusCode> {
    // Get MeiliSearch client from environment
    let meili_host = std::env::var("MEILI_HOST")
        .unwrap_or_else(|_| "http://localhost:7700".to_string());
    let meili_master_key = std::env::var("MEILI_MASTER_KEY").ok();

    let client = match Client::new(meili_host, meili_master_key) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to create MeiliSearch client: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let index = client.index(MEILISEARCH_INDEX_NAME);
    
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    // Search with user filter
    let filter_str = format!("user_id = '{}'", user.0.id);
    let search_result: meilisearch_sdk::search::SearchResults<SearchResultItem> = index
        .search()
        .with_query(&params.q)
        .with_limit(limit)
        .with_offset(offset)
        .with_filter(&filter_str)
        .execute::<SearchResultItem>()
        .await
        .map_err(|e| {
            tracing::error!("MeiliSearch error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(SearchResponse {
        results: search_result.hits.into_iter().map(|h| h.result).collect(),
        total: search_result.estimated_total_hits.unwrap_or(0),
        limit,
        offset,
    }))
}

