# T3 Chat - Developer Implementation Guide

This document provides a developer-focused implementation plan with specific files, data structures, and code organization. Work is divided into parallel Frontend and Backend tracks for MVP and Additional Features phases.

## Project Structure Overview

### Existing Infrastructure (Leverage These)
- **Backend**: Rust (Axum) with SeaORM, PostgreSQL
- **Frontend**: React + TypeScript + Vite, Tailwind CSS, ShadCN
- **Authentication**: Firebase Auth (already implemented)
- **Database**: PostgreSQL with embedded local development
- **Patterns**: Repository pattern, migration system, auth middleware

### Key Directories
```
server/src/
├── api.rs                    # API route handlers
├── db/
│   ├── schema/              # Database models (SeaORM entities)
│   └── repositories/        # Repository implementations
├── middleware/
│   └── auth.rs              # Firebase JWT authentication
└── main.rs                  # Application entry point

ui/src/
├── components/              # React components
├── lib/
│   ├── serverComm.ts       # API client utilities
│   └── auth-context.tsx    # Authentication context
└── pages/                  # Page components
```

---

## Phase 1: MVP (Minimum Viable Product)

### MVP Scope
1. Chat with multiple LLMs (OpenAI, Anthropic, Google)
2. Chat history persistence
3. Basic UI for chat interface
4. Model selection

---

## Phase 1: Backend Development

### 1.1 Database Schema & Migrations

#### Files to Create:
- `server/migration/src/m20250101_000002_create_ai_models_table.rs`
- `server/migration/src/m20250101_000003_create_user_api_keys_table.rs`
- `server/migration/src/m20250101_000004_create_chats_table.rs`
- `server/migration/src/m20250101_000005_create_messages_table.rs`

#### Migration: AI Models Table
```rust
// m20250101_000002_create_ai_models_table.rs
pub enum AiModels {
    Table,
    Id,                    // UUID primary key
    Provider,              // Enum: OpenAI, Anthropic, Google, DeepSeek, Ollama
    ModelId,              // String: "gpt-4", "claude-3-opus"
    DisplayName,          // String
    Description,          // Text
    ContextWindow,        // Integer
    SupportsStreaming,    // Boolean
    SupportsImages,       // Boolean
    SupportsFunctions,    // Boolean
    CostPerToken,         // Decimal (optional)
    IsActive,             // Boolean
    CreatedAt,
    UpdatedAt,
}
```

#### Migration: User API Keys Table
```rust
// m20250101_000003_create_user_api_keys_table.rs
pub enum UserApiKeys {
    Table,
    Id,                   // UUID primary key
    UserId,               // String (FK to users.id)
    Provider,             // Enum: OpenAI, Anthropic, Google, DeepSeek, Ollama
    EncryptedKey,         // Text (encrypted)
    IsDefault,            // Boolean
    CreatedAt,
    UpdatedAt,
}
```

#### Migration: Chats Table
```rust
// m20250101_000004_create_chats_table.rs
pub enum Chats {
    Table,
    Id,                   // UUID primary key
    UserId,               // String (FK to users.id)
    Title,                // String (auto-generated from first message)
    ModelProvider,        // Enum
    ModelId,              // String
    CreatedAt,
    UpdatedAt,
    DeletedAt,            // Timestamp (nullable, for soft delete)
}
```

#### Migration: Messages Table
```rust
// m20250101_000005_create_messages_table.rs
pub enum Messages {
    Table,
    Id,                   // UUID primary key
    ChatId,               // UUID (FK to chats.id)
    Role,                 // Enum: user, assistant, system
    Content,              // Text
    Metadata,             // JSONB (for attachments, function calls, etc.)
    ParentMessageId,      // UUID (nullable, for branching)
    SequenceNumber,       // Integer (for ordering)
    CreatedAt,
    TokensUsed,           // Integer (nullable)
    ModelUsed,            // String (nullable)
}
```

#### Update Migration Registry
```rust
// server/migration/src/lib.rs
mod m20250101_000002_create_ai_models_table;
mod m20250101_000003_create_user_api_keys_table;
mod m20250101_000004_create_chats_table;
mod m20250101_000005_create_messages_table;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_users_table::Migration),
            Box::new(m20250101_000002_create_ai_models_table::Migration),
            Box::new(m20250101_000003_create_user_api_keys_table::Migration),
            Box::new(m20250101_000004_create_chats_table::Migration),
            Box::new(m20250101_000005_create_messages_table::Migration),
        ]
    }
}
```

### 1.2 Database Models (SeaORM Entities)

#### Files to Create:
- `server/src/db/schema/ai_model.rs`
- `server/src/db/schema/user_api_key.rs`
- `server/src/db/schema/chat.rs`
- `server/src/db/schema/message.rs`

#### Model: AI Model
```rust
// server/src/db/schema/ai_model.rs
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ai_models")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub provider: AiProvider,  // Custom enum
    #[sea_orm(column_type = "String")]
    pub model_id: String,
    pub display_name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub context_window: i32,
    pub supports_streaming: bool,
    pub supports_images: bool,
    pub supports_functions: bool,
    #[sea_orm(column_type = "Decimal", nullable)]
    pub cost_per_token: Option<Decimal>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum AiProvider {
    #[sea_orm(string_value = "openai")]
    OpenAI,
    #[sea_orm(string_value = "anthropic")]
    Anthropic,
    #[sea_orm(string_value = "google")]
    Google,
    #[sea_orm(string_value = "deepseek")]
    DeepSeek,
    #[sea_orm(string_value = "ollama")]
    Ollama,
}
```

#### Model: User API Key
```rust
// server/src/db/schema/user_api_key.rs
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_api_keys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: String,  // FK to users.id
    pub provider: AiProvider,
    #[sea_orm(column_type = "Text")]
    pub encrypted_key: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### Model: Chat
```rust
// server/src/db/schema/chat.rs
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chats")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: String,  // FK to users.id
    pub title: String,
    pub model_provider: AiProvider,
    #[sea_orm(column_type = "String")]
    pub model_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sea_orm(nullable)]
    pub deleted_at: Option<DateTime<Utc>>,
}
```

#### Model: Message
```rust
// server/src/db/schema/message.rs
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub chat_id: Uuid,  // FK to chats.id
    pub role: MessageRole,  // Custom enum
    #[sea_orm(column_type = "Text")]
    pub content: String,
    #[sea_orm(column_type = "Json", nullable)]
    pub metadata: Option<Json>,
    #[sea_orm(nullable)]
    pub parent_message_id: Option<Uuid>,
    pub sequence_number: i32,
    pub created_at: DateTime<Utc>,
    #[sea_orm(nullable)]
    pub tokens_used: Option<i32>,
    #[sea_orm(nullable)]
    pub model_used: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum MessageRole {
    #[sea_orm(string_value = "user")]
    User,
    #[sea_orm(string_value = "assistant")]
    Assistant,
    #[sea_orm(string_value = "system")]
    System,
}
```

#### Update Schema Module
```rust
// server/src/db/schema/mod.rs
pub mod user;
pub mod ai_model;
pub mod user_api_key;
pub mod chat;
pub mod message;

pub use user::*;
pub use ai_model::*;
pub use user_api_key::*;
pub use chat::*;
pub use message::*;
```

### 1.3 Repositories

#### Files to Create:
- `server/src/db/repositories/ai_model_repository.rs`
- `server/src/db/repositories/user_api_key_repository.rs`
- `server/src/db/repositories/chat_repository.rs`
- `server/src/db/repositories/message_repository.rs`

#### Repository: AI Model
```rust
// server/src/db/repositories/ai_model_repository.rs
#[async_trait]
pub trait IAiModelRepository: Send + Sync {
    async fn list_active(&self) -> Result<Vec<AiModelModel>>;
    async fn get_by_provider_and_model_id(&self, provider: &AiProvider, model_id: &str) -> Result<Option<AiModelModel>>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<AiModelModel>>;
}
```

#### Repository: User API Key
```rust
// server/src/db/repositories/user_api_key_repository.rs
#[async_trait]
pub trait IUserApiKeyRepository: Send + Sync {
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<UserApiKeyModel>>;
    async fn get_default_for_provider(&self, user_id: &str, provider: &AiProvider) -> Result<Option<UserApiKeyModel>>;
    async fn create(&self, model: CreateUserApiKeyDto) -> Result<UserApiKeyModel>;
    async fn update(&self, id: Uuid, model: UpdateUserApiKeyDto) -> Result<UserApiKeyModel>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn set_default(&self, id: Uuid, user_id: &str) -> Result<()>;
}
```

#### Repository: Chat
```rust
// server/src/db/repositories/chat_repository.rs
#[async_trait]
pub trait IChatRepository: Send + Sync {
    async fn list_by_user(&self, user_id: &str, pagination: Option<Pagination>) -> Result<ResultSet<ChatModel>>;
    async fn get_by_id(&self, id: Uuid, user_id: &str) -> Result<Option<ChatModel>>;
    async fn create(&self, model: CreateChatDto) -> Result<ChatModel>;
    async fn update(&self, id: Uuid, user_id: &str, model: UpdateChatDto) -> Result<ChatModel>;
    async fn delete(&self, id: Uuid, user_id: &str) -> Result<()>;  // Soft delete
}
```

#### Repository: Message
```rust
// server/src/db/repositories/message_repository.rs
#[async_trait]
pub trait IMessageRepository: Send + Sync {
    async fn list_by_chat(&self, chat_id: Uuid, user_id: &str) -> Result<Vec<MessageModel>>;
    async fn create(&self, model: CreateMessageDto) -> Result<MessageModel>;
    async fn get_next_sequence_number(&self, chat_id: Uuid) -> Result<i32>;
    async fn update_tokens_used(&self, id: Uuid, tokens: i32, model: &str) -> Result<()>;
}
```

#### Update Repositories Module
```rust
// server/src/db/repositories/mod.rs
pub mod user_repository;
pub mod ai_model_repository;
pub mod user_api_key_repository;
pub mod chat_repository;
pub mod message_repository;

pub use user_repository::*;
pub use ai_model_repository::*;
pub use user_api_key_repository::*;
pub use chat_repository::*;
pub use message_repository::*;
```

### 1.4 AI Provider Abstraction

#### Files to Create:
- `server/src/ai/mod.rs`
- `server/src/ai/types.rs`
- `server/src/ai/providers/mod.rs`
- `server/src/ai/providers/openai.rs`
- `server/src/ai/providers/anthropic.rs`
- `server/src/ai/providers/google.rs`
- `server/src/ai/manager.rs`

#### Provider Trait
```rust
// server/src/ai/providers/mod.rs
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    async fn stream_chat(&self, request: ChatRequest) -> Result<impl Stream<Item = Result<ChatResponseChunk>>>;
    fn get_model_info(&self, model_id: &str) -> Option<ModelInfo>;
    fn list_models(&self) -> Vec<ModelInfo>;
}
```

#### Types
```rust
// server/src/ai/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: Option<u32>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub context_window: u32,
    pub supports_streaming: bool,
    pub supports_images: bool,
}
```

#### Provider Manager
```rust
// server/src/ai/manager.rs
pub struct ProviderManager {
    providers: HashMap<AiProvider, Arc<dyn AIProvider>>,
}

impl ProviderManager {
    pub fn new(api_keys: HashMap<AiProvider, String>) -> Result<Self>;
    pub fn get_provider(&self, provider: &AiProvider) -> Option<Arc<dyn AIProvider>>;
}
```

### 1.5 API Routes

#### Files to Create:
- `server/src/api/models.rs`
- `server/src/api/chats.rs`
- `server/src/api/messages.rs`
- `server/src/api/user_api_keys.rs`
- `server/src/api/chat.rs` (for chat streaming)

#### API: Models
```rust
// server/src/api/models.rs
// GET /api/v1/models - List all active models (public)
pub async fn list_models(state: State<AppState>) -> Result<Json<Vec<ModelResponse>>, StatusCode>;

// GET /api/v1/models/:id - Get model details (public)
pub async fn get_model(state: State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ModelResponse>, StatusCode>;
```

#### API: Chats
```rust
// server/src/api/chats.rs
// GET /api/v1/chats - List user's chats (authenticated)
pub async fn list_chats(user: AuthenticatedUser, state: State<AppState>, Query(params): Query<PaginationParams>) -> Result<Json<ChatListResponse>, StatusCode>;

// POST /api/v1/chats - Create new chat (authenticated)
pub async fn create_chat(user: AuthenticatedUser, state: State<AppState>, Json(payload): Json<CreateChatRequest>) -> Result<Json<ChatResponse>, StatusCode>;

// GET /api/v1/chats/:id - Get chat with messages (authenticated)
pub async fn get_chat(user: AuthenticatedUser, state: State<AppState>, Path(id): Path<Uuid>) -> Result<Json<ChatWithMessagesResponse>, StatusCode>;

// PUT /api/v1/chats/:id - Update chat (authenticated)
pub async fn update_chat(user: AuthenticatedUser, state: State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<UpdateChatRequest>) -> Result<Json<ChatResponse>, StatusCode>;

// DELETE /api/v1/chats/:id - Delete chat (authenticated, soft delete)
pub async fn delete_chat(user: AuthenticatedUser, state: State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, StatusCode>;
```

#### API: Messages
```rust
// server/src/api/messages.rs
// GET /api/v1/chats/:id/messages - Get messages for a chat (authenticated)
pub async fn get_messages(user: AuthenticatedUser, state: State<AppState>, Path(chat_id): Path<Uuid>) -> Result<Json<Vec<MessageResponse>>, StatusCode>;

// POST /api/v1/chats/:id/messages - Add message to chat (authenticated)
pub async fn create_message(user: AuthenticatedUser, state: State<AppState>, Path(chat_id): Path<Uuid>, Json(payload): Json<CreateMessageRequest>) -> Result<Json<MessageResponse>, StatusCode>;
```

#### API: Chat Streaming
```rust
// server/src/api/chat.rs
// POST /api/v1/chat - Send chat message (non-streaming, authenticated)
pub async fn chat(user: AuthenticatedUser, state: State<AppState>, Json(payload): Json<ChatRequest>) -> Result<Json<ChatResponse>, StatusCode>;

// POST /api/v1/chat/stream - Send chat message (streaming, authenticated)
pub async fn stream_chat(user: AuthenticatedUser, state: State<AppState>, Json(payload): Json<ChatRequest>) -> Result<impl IntoResponse, StatusCode>;
```

#### API: User API Keys
```rust
// server/src/api/user_api_keys.rs
// GET /api/v1/user-api-keys - Get user's API keys (authenticated)
pub async fn list_keys(user: AuthenticatedUser, state: State<AppState>) -> Result<Json<Vec<UserApiKeyResponse>>, StatusCode>;

// POST /api/v1/user-api-keys - Add API key (authenticated)
pub async fn create_key(user: AuthenticatedUser, state: State<AppState>, Json(payload): Json<CreateUserApiKeyRequest>) -> Result<Json<UserApiKeyResponse>, StatusCode>;

// DELETE /api/v1/user-api-keys/:id - Delete API key (authenticated)
pub async fn delete_key(user: AuthenticatedUser, state: State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, StatusCode>;
```

#### Update API Module
```rust
// server/src/api.rs
pub mod me;
pub mod models;
pub mod chats;
pub mod messages;
pub mod chat;
pub mod user_api_keys;
```

#### Update Router
```rust
// server/src/main.rs (update setup_router)
let models_routes = Router::new()
    .route("/", get(api::models::list_models))
    .route("/:id", get(api::models::get_model));

let chats_routes = Router::new()
    .route("/", get(api::chats::list_chats).post(api::chats::create_chat))
    .route("/:id", get(api::chats::get_chat).put(api::chats::update_chat).delete(api::chats::delete_chat))
    .route("/:id/messages", get(api::messages::get_messages).post(api::messages::create_message))
    .route_layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth::auth_middleware));

let chat_routes = Router::new()
    .route("/", post(api::chat::chat))
    .route("/stream", post(api::chat::stream_chat))
    .route_layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth::auth_middleware));

let user_api_keys_routes = Router::new()
    .route("/", get(api::user_api_keys::list_keys).post(api::user_api_keys::create_key))
    .route("/:id", delete(api::user_api_keys::delete_key))
    .route_layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth::auth_middleware));

Router::new()
    .route("/", get(api::health_check))
    .nest("/api/v1/models", models_routes)
    .nest("/api/v1/chats", chats_routes)
    .nest("/api/v1/chat", chat_routes)
    .nest("/api/v1/user-api-keys", user_api_keys_routes)
    .nest("/api/v1", authenticated_routes)
    // ... rest of router
```

### 1.6 Update AppState
```rust
// server/src/main.rs
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub user_repository: Arc<Box<dyn db::repositories::IUserRepository>>,
    pub ai_model_repository: Arc<Box<dyn db::repositories::IAiModelRepository>>,
    pub user_api_key_repository: Arc<Box<dyn db::repositories::IUserApiKeyRepository>>,
    pub chat_repository: Arc<Box<dyn db::repositories::IChatRepository>>,
    pub message_repository: Arc<Box<dyn db::repositories::IMessageRepository>>,
    pub provider_manager: Arc<ai::manager::ProviderManager>,
}
```

### 1.7 Dependencies to Add
```toml
# server/Cargo.toml
[dependencies]
uuid = { version = "1", features = ["v4", "serde"] }
rust_decimal = { version = "1", features = ["serde"] }
tokio-stream = "0"
futures = "0"
aes-gcm = "0"  # For API key encryption
```

---

## Phase 1: Frontend Development

### 1.1 Type Definitions

#### Files to Create:
- `ui/src/types/chat.ts`
- `ui/src/types/model.ts`
- `ui/src/types/api.ts`

#### Types: Chat
```typescript
// ui/src/types/chat.ts
export interface Chat {
  id: string;
  user_id: string;
  title: string;
  model_provider: AiProvider;
  model_id: string;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
}

export interface Message {
  id: string;
  chat_id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  metadata?: Record<string, any>;
  parent_message_id?: string;
  sequence_number: number;
  created_at: string;
  tokens_used?: number;
  model_used?: string;
}

export interface ChatWithMessages extends Chat {
  messages: Message[];
}

export type AiProvider = 'openai' | 'anthropic' | 'google' | 'deepseek' | 'ollama';
```

#### Types: Model
```typescript
// ui/src/types/model.ts
export interface AIModel {
  id: string;
  provider: AiProvider;
  model_id: string;
  display_name: string;
  description?: string;
  context_window: number;
  supports_streaming: boolean;
  supports_images: boolean;
  supports_functions: boolean;
  cost_per_token?: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}
```

#### Types: API
```typescript
// ui/src/types/api.ts
export interface CreateChatRequest {
  model_provider: AiProvider;
  model_id: string;
  title?: string;
}

export interface CreateMessageRequest {
  content: string;
  role?: 'user' | 'system';
}

export interface ChatRequest {
  chat_id: string;
  message: string;
  model_provider: AiProvider;
  model_id: string;
  temperature?: number;
  max_tokens?: number;
  stream?: boolean;
}

export interface ChatResponse {
  content: string;
  model: string;
  tokens_used?: number;
  finish_reason?: string;
}

export interface UserApiKey {
  id: string;
  user_id: string;
  provider: AiProvider;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateUserApiKeyRequest {
  provider: AiProvider;
  api_key: string;
  is_default?: boolean;
}
```

### 1.2 API Client

#### Files to Create:
- `ui/src/lib/api/models.ts`
- `ui/src/lib/api/chats.ts`
- `ui/src/lib/api/messages.ts`
- `ui/src/lib/api/chat.ts`
- `ui/src/lib/api/userApiKeys.ts`

#### API Client: Models
```typescript
// ui/src/lib/api/models.ts
import { fetchWithAuth } from '../serverComm';
import type { AIModel } from '@/types/model';

export async function listModels(): Promise<AIModel[]> {
  const response = await fetchWithAuth('/api/v1/models');
  return response.json();
}

export async function getModel(id: string): Promise<AIModel> {
  const response = await fetchWithAuth(`/api/v1/models/${id}`);
  return response.json();
}
```

#### API Client: Chats
```typescript
// ui/src/lib/api/chats.ts
import { fetchWithAuth } from '../serverComm';
import type { Chat, ChatWithMessages } from '@/types/chat';
import type { CreateChatRequest } from '@/types/api';

export async function listChats(page = 1, pageSize = 20): Promise<{ data: Chat[]; total: number }> {
  const response = await fetchWithAuth(`/api/v1/chats?page=${page}&page_size=${pageSize}`);
  return response.json();
}

export async function getChat(id: string): Promise<ChatWithMessages> {
  const response = await fetchWithAuth(`/api/v1/chats/${id}`);
  return response.json();
}

export async function createChat(data: CreateChatRequest): Promise<Chat> {
  const response = await fetchWithAuth('/api/v1/chats', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return response.json();
}

export async function updateChat(id: string, data: { title?: string }): Promise<Chat> {
  const response = await fetchWithAuth(`/api/v1/chats/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return response.json();
}

export async function deleteChat(id: string): Promise<void> {
  await fetchWithAuth(`/api/v1/chats/${id}`, {
    method: 'DELETE',
  });
}
```

#### API Client: Messages
```typescript
// ui/src/lib/api/messages.ts
import { fetchWithAuth } from '../serverComm';
import type { Message } from '@/types/chat';
import type { CreateMessageRequest } from '@/types/api';

export async function getMessages(chatId: string): Promise<Message[]> {
  const response = await fetchWithAuth(`/api/v1/chats/${chatId}/messages`);
  return response.json();
}

export async function createMessage(chatId: string, data: CreateMessageRequest): Promise<Message> {
  const response = await fetchWithAuth(`/api/v1/chats/${chatId}/messages`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return response.json();
}
```

#### API Client: Chat
```typescript
// ui/src/lib/api/chat.ts
import { fetchWithAuth } from '../serverComm';
import type { ChatRequest, ChatResponse } from '@/types/api';

export async function sendChat(data: ChatRequest): Promise<ChatResponse> {
  const response = await fetchWithAuth('/api/v1/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return response.json();
}

export async function* streamChat(data: ChatRequest): AsyncGenerator<string, void, unknown> {
  const response = await fetchWithAuth('/api/v1/chat/stream', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ ...data, stream: true }),
  });

  if (!response.body) {
    throw new Error('Response body is null');
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const chunk = decoder.decode(value, { stream: true });
      const lines = chunk.split('\n').filter(line => line.trim());

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = line.slice(6);
          if (data === '[DONE]') return;
          try {
            const parsed = JSON.parse(data);
            yield parsed.content || '';
          } catch (e) {
            // Skip invalid JSON
          }
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}
```

#### API Client: User API Keys
```typescript
// ui/src/lib/api/userApiKeys.ts
import { fetchWithAuth } from '../serverComm';
import type { UserApiKey, CreateUserApiKeyRequest } from '@/types/api';

export async function listUserApiKeys(): Promise<UserApiKey[]> {
  const response = await fetchWithAuth('/api/v1/user-api-keys');
  return response.json();
}

export async function createUserApiKey(data: CreateUserApiKeyRequest): Promise<UserApiKey> {
  const response = await fetchWithAuth('/api/v1/user-api-keys', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  return response.json();
}

export async function deleteUserApiKey(id: string): Promise<void> {
  await fetchWithAuth(`/api/v1/user-api-keys/${id}`, {
    method: 'DELETE',
  });
}
```

#### Update serverComm.ts
```typescript
// ui/src/lib/serverComm.ts (add exports)
export * from './api/models';
export * from './api/chats';
export * from './api/messages';
export * from './api/chat';
export * from './api/userApiKeys';
```

### 1.3 React Hooks

#### Files to Create:
- `ui/src/hooks/useChats.ts`
- `ui/src/hooks/useChat.ts`
- `ui/src/hooks/useModels.ts`
- `ui/src/hooks/useStreamingChat.ts`
- `ui/src/hooks/useUserApiKeys.ts`

#### Hook: useChats
```typescript
// ui/src/hooks/useChats.ts
import { useState, useEffect } from 'react';
import * as api from '@/lib/serverComm';
import type { Chat } from '@/types/chat';

export function useChats() {
  const [chats, setChats] = useState<Chat[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const loadChats = async () => {
    try {
      setLoading(true);
      const result = await api.listChats();
      setChats(result.data);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadChats();
  }, []);

  return { chats, loading, error, refresh: loadChats };
}
```

#### Hook: useChat
```typescript
// ui/src/hooks/useChat.ts
import { useState, useEffect } from 'react';
import * as api from '@/lib/serverComm';
import type { ChatWithMessages } from '@/types/chat';

export function useChat(chatId: string | null) {
  const [chat, setChat] = useState<ChatWithMessages | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!chatId) {
      setChat(null);
      setLoading(false);
      return;
    }

    const loadChat = async () => {
      try {
        setLoading(true);
        const data = await api.getChat(chatId);
        setChat(data);
      } catch (err) {
        setError(err as Error);
      } finally {
        setLoading(false);
      }
    };

    loadChat();
  }, [chatId]);

  return { chat, loading, error };
}
```

#### Hook: useModels
```typescript
// ui/src/hooks/useModels.ts
import { useState, useEffect } from 'react';
import * as api from '@/lib/serverComm';
import type { AIModel } from '@/types/model';

export function useModels() {
  const [models, setModels] = useState<AIModel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const loadModels = async () => {
      try {
        setLoading(true);
        const data = await api.listModels();
        setModels(data);
      } catch (err) {
        setError(err as Error);
      } finally {
        setLoading(false);
      }
    };

    loadModels();
  }, []);

  return { models, loading, error };
}
```

#### Hook: useStreamingChat
```typescript
// ui/src/hooks/useStreamingChat.ts
import { useState, useCallback } from 'react';
import * as api from '@/lib/serverComm';
import type { ChatRequest } from '@/types/api';

export function useStreamingChat() {
  const [streaming, setStreaming] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const sendMessage = useCallback(async (
    request: ChatRequest,
    onChunk: (chunk: string) => void,
    onComplete: () => void
  ) => {
    try {
      setStreaming(true);
      setError(null);

      for await (const chunk of api.streamChat(request)) {
        onChunk(chunk);
      }

      onComplete();
    } catch (err) {
      setError(err as Error);
    } finally {
      setStreaming(false);
    }
  }, []);

  return { sendMessage, streaming, error };
}
```

#### Hook: useUserApiKeys
```typescript
// ui/src/hooks/useUserApiKeys.ts
import { useState, useEffect } from 'react';
import * as api from '@/lib/serverComm';
import type { UserApiKey, CreateUserApiKeyRequest } from '@/types/api';

export function useUserApiKeys() {
  const [keys, setKeys] = useState<UserApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const loadKeys = async () => {
    try {
      setLoading(true);
      const data = await api.listUserApiKeys();
      setKeys(data);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadKeys();
  }, []);

  const createKey = async (data: CreateUserApiKeyRequest) => {
    const newKey = await api.createUserApiKey(data);
    setKeys([...keys, newKey]);
    return newKey;
  };

  const deleteKey = async (id: string) => {
    await api.deleteUserApiKey(id);
    setKeys(keys.filter(k => k.id !== id));
  };

  return { keys, loading, error, createKey, deleteKey, refresh: loadKeys };
}
```

### 1.4 Chat Components

#### Files to Create:
- `ui/src/components/chat/ChatView.tsx`
- `ui/src/components/chat/ChatList.tsx`
- `ui/src/components/chat/MessageList.tsx`
- `ui/src/components/chat/MessageInput.tsx`
- `ui/src/components/chat/MessageBubble.tsx`
- `ui/src/components/model/ModelSelector.tsx`
- `ui/src/components/model/ModelInfo.tsx`

#### Component: ChatView
```typescript
// ui/src/components/chat/ChatView.tsx
import { useState, useEffect, useRef } from 'react';
import { useChat } from '@/hooks/useChat';
import { useStreamingChat } from '@/hooks/useStreamingChat';
import { MessageList } from './MessageList';
import { MessageInput } from './MessageInput';
import { ModelSelector } from '@/components/model/ModelSelector';
import { useModels } from '@/hooks/useModels';
import * as api from '@/lib/serverComm';
import type { AIModel } from '@/types/model';

export function ChatView({ chatId }: { chatId: string | null }) {
  const { chat, loading } = useChat(chatId);
  const { models } = useModels();
  const [selectedModel, setSelectedModel] = useState<AIModel | null>(null);
  const { sendMessage, streaming } = useStreamingChat();
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (chat && models.length > 0) {
      const model = models.find(m => 
        m.provider === chat.model_provider && m.model_id === chat.model_id
      );
      setSelectedModel(model || models[0]);
    }
  }, [chat, models]);

  const handleSendMessage = async (content: string) => {
    if (!chatId || !selectedModel) return;

    // Add user message
    await api.createMessage(chatId, { content, role: 'user' });

    // Stream assistant response
    let assistantContent = '';
    await sendMessage(
      {
        chat_id: chatId,
        message: content,
        model_provider: selectedModel.provider,
        model_id: selectedModel.model_id,
        stream: true,
      },
      (chunk) => {
        assistantContent += chunk;
        // Update message in real-time (optimistic update)
      },
      async () => {
        // Save complete message
        await api.createMessage(chatId, {
          content: assistantContent,
          role: 'assistant',
        });
      }
    );
  };

  if (loading) return <div>Loading...</div>;
  if (!chat) return <div>Select a chat or create a new one</div>;

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-4">
        <ModelSelector
          models={models}
          selectedModel={selectedModel}
          onSelect={setSelectedModel}
        />
      </div>
      <MessageList messages={chat.messages} />
      <MessageInput onSend={handleSendMessage} disabled={streaming} />
    </div>
  );
}
```

#### Component: ChatList
```typescript
// ui/src/components/chat/ChatList.tsx
import { useChats } from '@/hooks/useChats';
import { useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import * as api from '@/lib/serverComm';

export function ChatList() {
  const { chats, loading, refresh } = useChats();
  const navigate = useNavigate();

  const handleNewChat = async () => {
    // Create new chat with default model
    const newChat = await api.createChat({
      model_provider: 'openai',
      model_id: 'gpt-3.5-turbo',
    });
    navigate(`/chat/${newChat.id}`);
  };

  if (loading) return <div>Loading chats...</div>;

  return (
    <div className="flex flex-col h-full">
      <Button onClick={handleNewChat} className="m-4">
        New Chat
      </Button>
      <div className="flex-1 overflow-y-auto">
        {chats.map(chat => (
          <div
            key={chat.id}
            onClick={() => navigate(`/chat/${chat.id}`)}
            className="p-4 border-b cursor-pointer hover:bg-accent"
          >
            <div className="font-medium">{chat.title}</div>
            <div className="text-sm text-muted-foreground">
              {new Date(chat.updated_at).toLocaleDateString()}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
```

#### Component: MessageList
```typescript
// ui/src/components/chat/MessageList.tsx
import { useEffect, useRef } from 'react';
import { MessageBubble } from './MessageBubble';
import type { Message } from '@/types/chat';

export function MessageList({ messages }: { messages: Message[] }) {
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  return (
    <div className="flex-1 overflow-y-auto p-4 space-y-4">
      {messages.map(message => (
        <MessageBubble key={message.id} message={message} />
      ))}
      <div ref={messagesEndRef} />
    </div>
  );
}
```

#### Component: MessageBubble
```typescript
// ui/src/components/chat/MessageBubble.tsx
import type { Message } from '@/types/chat';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';

export function MessageBubble({ message }: { message: Message }) {
  const isUser = message.role === 'user';

  return (
    <div className={`flex gap-3 ${isUser ? 'flex-row-reverse' : 'flex-row'}`}>
      <Avatar>
        <AvatarFallback>
          {isUser ? 'U' : 'AI'}
        </AvatarFallback>
      </Avatar>
      <div className={`flex-1 ${isUser ? 'text-right' : 'text-left'}`}>
        <div className={`inline-block p-3 rounded-lg ${
          isUser ? 'bg-primary text-primary-foreground' : 'bg-muted'
        }`}>
          {message.content}
        </div>
      </div>
    </div>
  );
}
```

#### Component: MessageInput
```typescript
// ui/src/components/chat/MessageInput.tsx
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';

export function MessageInput({ 
  onSend, 
  disabled 
}: { 
  onSend: (content: string) => void;
  disabled?: boolean;
}) {
  const [content, setContent] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (content.trim() && !disabled) {
      onSend(content);
      setContent('');
    }
  };

  return (
    <form onSubmit={handleSubmit} className="border-t p-4">
      <div className="flex gap-2">
        <Textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder="Type your message..."
          disabled={disabled}
          rows={3}
          className="flex-1"
        />
        <Button type="submit" disabled={disabled || !content.trim()}>
          Send
        </Button>
      </div>
    </form>
  );
}
```

#### Component: ModelSelector
```typescript
// ui/src/components/model/ModelSelector.tsx
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import type { AIModel } from '@/types/model';

export function ModelSelector({
  models,
  selectedModel,
  onSelect,
}: {
  models: AIModel[];
  selectedModel: AIModel | null;
  onSelect: (model: AIModel) => void;
}) {
  return (
    <Select
      value={selectedModel?.id}
      onValueChange={(value) => {
        const model = models.find(m => m.id === value);
        if (model) onSelect(model);
      }}
    >
      <SelectTrigger className="w-full">
        <SelectValue placeholder="Select a model" />
      </SelectTrigger>
      <SelectContent>
        {models.map(model => (
          <SelectItem key={model.id} value={model.id}>
            {model.display_name} ({model.provider})
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
```

### 1.5 Pages

#### Files to Create:
- `ui/src/pages/Chat.tsx`

#### Update App.tsx
```typescript
// ui/src/App.tsx (add route)
<Route path="/chat/:chatId?" element={<Chat />} />
```

#### Page: Chat
```typescript
// ui/src/pages/Chat.tsx
import { useParams } from 'react-router-dom';
import { ChatView } from '@/components/chat/ChatView';

export function Chat() {
  const { chatId } = useParams<{ chatId?: string }>();
  return <ChatView chatId={chatId || null} />;
}
```

### 1.6 Update Sidebar
```typescript
// ui/src/components/appSidebar.tsx (add Chat link)
<NavItem title="Chat" url="/chat" />
```

---

## Phase 2: Additional Features

### Phase 2 Scope
1. Syntax highlighting
2. Markdown rendering
3. Attachment support (images)
4. Chat branching
5. Chat sharing
6. BYOK (Bring Your Own Key) UI
7. Resumable streams
8. Web search integration

---

## Phase 2: Backend Development

### 2.1 Database Schema & Migrations

#### Files to Create:
- `server/migration/src/m20250101_000006_create_chat_branches_table.rs`
- `server/migration/src/m20250101_000007_create_attachments_table.rs`
- `server/migration/src/m20250101_000008_create_shared_chats_table.rs`

#### Migration: Chat Branches
```rust
// m20250101_000006_create_chat_branches_table.rs
pub enum ChatBranches {
    Table,
    Id,                   // UUID primary key
    ChatId,               // UUID (FK to chats.id)
    ParentMessageId,      // UUID (FK to messages.id)
    BranchName,           // String (optional)
    CreatedAt,
}
```

#### Migration: Attachments
```rust
// m20250101_000007_create_attachments_table.rs
pub enum Attachments {
    Table,
    Id,                   // UUID primary key
    MessageId,            // UUID (FK to messages.id)
    FileName,             // String
    FileType,             // Enum: image, pdf, other
    MimeType,             // String
    FileSize,             // Integer (bytes)
    StoragePath,          // String
    ThumbnailPath,        // String (nullable)
    CreatedAt,
}
```

#### Migration: Shared Chats
```rust
// m20250101_000008_create_shared_chats_table.rs
pub enum SharedChats {
    Table,
    Id,                   // UUID primary key
    ChatId,               // UUID (FK to chats.id)
    ShareToken,           // String (unique)
    ShareType,            // Enum: public, unlisted, private
    ExpiresAt,            // Timestamp (nullable)
    CreatedAt,
    ViewCount,            // Integer
    AllowComments,        // Boolean
}
```

### 2.2 Database Models

#### Files to Create:
- `server/src/db/schema/chat_branch.rs`
- `server/src/db/schema/attachment.rs`
- `server/src/db/schema/shared_chat.rs`

### 2.3 Repositories

#### Files to Create:
- `server/src/db/repositories/chat_branch_repository.rs`
- `server/src/db/repositories/attachment_repository.rs`
- `server/src/db/repositories/shared_chat_repository.rs`

### 2.4 API Routes

#### Files to Create/Update:
- `server/src/api/branches.rs`
- `server/src/api/attachments.rs`
- `server/src/api/shared_chats.rs`
- `server/src/api/search.rs` (for web search)

### 2.5 File Storage

#### Files to Create:
- `server/src/storage/mod.rs`
- `server/src/storage/local.rs`
- `server/src/storage/s3.rs` (optional for production)

---

## Phase 2: Frontend Development

### 2.1 Markdown & Syntax Highlighting

#### Dependencies to Add:
```json
// ui/package.json
{
  "dependencies": {
    "react-markdown": "^9",
    "remark-gfm": "^4",
    "react-syntax-highlighter": "^15",
    "@types/react-syntax-highlighter": "^15"
  }
}
```

#### Files to Create:
- `ui/src/components/chat/MarkdownRenderer.tsx`
- `ui/src/components/chat/CodeBlock.tsx`

### 2.2 Attachment Components

#### Files to Create:
- `ui/src/components/chat/FileUpload.tsx`
- `ui/src/components/chat/AttachmentPreview.tsx`
- `ui/src/components/chat/ImageViewer.tsx`

### 2.3 Branching Components

#### Files to Create:
- `ui/src/components/chat/BranchSelector.tsx`
- `ui/src/components/chat/BranchTree.tsx`

### 2.4 Sharing Components

#### Files to Create:
- `ui/src/components/chat/ShareDialog.tsx`
- `ui/src/pages/SharedChat.tsx`

### 2.5 Settings Components

#### Files to Update:
- `ui/src/pages/Settings.tsx` (add API key management UI)

---

## Implementation Order

### Week 1-2: MVP Backend
1. Database migrations (1.1)
2. Database models (1.2)
3. Repositories (1.3)
4. AI provider abstraction (1.4)
5. API routes (1.5)

### Week 1-2: MVP Frontend (Parallel)
1. Type definitions (1.1)
2. API client (1.2)
3. React hooks (1.3)
4. Chat components (1.4)
5. Pages and routing (1.5-1.6)

### Week 3: Integration & Testing
1. Connect frontend to backend
2. End-to-end testing
3. Bug fixes
4. UI polish

### Week 4+: Additional Features
1. Implement Phase 2 features incrementally
2. Test each feature before moving to next

---

## Key Implementation Notes

1. **Authentication**: Use existing `AuthenticatedUser` extractor from `middleware::auth`
2. **Database**: Follow existing SeaORM patterns from `user.rs` schema
3. **Repositories**: Follow existing `UserRepository` pattern
4. **API Routes**: Use existing `api.rs` structure and route organization
5. **Frontend API**: Extend existing `serverComm.ts` pattern
6. **Components**: Use existing ShadCN components from `ui/src/components/ui/`

---

## Environment Variables

### Backend (.env)
```bash
DATABASE_URL=postgresql://user:password@localhost:5432/t3chat
FIREBASE_PROJECT_ID=your-project-id
CORS_ORIGINS=http://localhost:5173,http://localhost:3000
# API keys for default providers (optional)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GOOGLE_API_KEY=...
```

### Frontend (.env)
```bash
VITE_API_URL=http://localhost:8787
VITE_FIREBASE_API_KEY=...
VITE_FIREBASE_AUTH_DOMAIN=...
VITE_FIREBASE_PROJECT_ID=...
VITE_ALLOW_ANONYMOUS_USERS=true
```

---

## Testing Checklist

### MVP Testing
- [ ] Create new chat
- [ ] Send message and receive response
- [ ] Switch between models
- [ ] View chat history
- [ ] Delete chat
- [ ] Add/remove API keys
- [ ] Streaming chat works

### Additional Features Testing
- [ ] Syntax highlighting works
- [ ] Markdown renders correctly
- [ ] File upload works
- [ ] Image preview works
- [ ] Create branch from message
- [ ] Share chat link works
- [ ] Web search integration works

