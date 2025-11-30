# T3Chat Development Plan: LibreChat-Inspired Multi-AI Platform

## Executive Summary

This document outlines the development plan to transform T3Chat into a LibreChat-inspired multi-AI chat platform, maintaining the current tech stack (Rust/Axum backend, React/Vite/pnpm frontend) while implementing LibreChat's core functionality for supporting multiple AI providers.

### PostgreSQL vs MongoDB Compatibility Analysis

**Can PostgreSQL be used instead of MongoDB?**

✅ **YES** - PostgreSQL can fully support all LibreChat features with the following considerations:

**Advantages of PostgreSQL for this use case:**
1. **Structured data**: Most of LibreChat's data is relatively structured (users, conversations, messages, agents)
2. **Strong typing**: Diesel provides compile-time query validation
3. **JSONB support**: PostgreSQL's JSONB can handle the flexible/dynamic fields (metadata, tool outputs, agent options)
4. **Better relationships**: Foreign keys and referential integrity for users, conversations, messages
5. **Full-text search**: PostgreSQL's built-in full-text search can replace Meilisearch for message/conversation search
6. **Performance**: Better query optimization for relational queries (e.g., user's conversations with messages)

**Fields that will use JSONB in PostgreSQL:**
- Message `content` (array of content blocks for multimodal messages)
- Message `attachments` (flexible file attachments)
- Message `plugin` data
- Conversation `agent_options` (dynamic configuration)
- Agent `model_parameters` (provider-specific settings)
- Agent `tool_resources` (flexible tool configuration)
- Preset `agent_options`
- File `metadata`

**No features are lost** - All LibreChat functionality can be implemented with PostgreSQL + JSONB.

---

## Architecture Overview

### LibreChat's Key Abstractions

1. **BaseClient Pattern**: All AI providers inherit from `BaseClient` with common interface
2. **Endpoint System**: Routes handle different AI providers (OpenAI, Anthropic, Google, etc.)
3. **Flexible Schema**: Conversations/messages support multiple providers with optional fields
4. **Agent System**: Pre-configured AI assistants with specific tools and instructions
5. **Preset System**: Saved conversation configurations
6. **Tool/Plugin System**: Extensible tools that agents can use

### Our Implementation Strategy

**Backend (Rust):**
- Trait-based abstraction for AI providers (similar to BaseClient)
- Axum routes for different endpoints
- Diesel ORM with JSONB for flexible fields
- Repository pattern for data access

**Frontend (React):**
- Reuse LibreChat's React component patterns
- Adapt state management to our existing approach
- Maintain Tailwind CSS v4 styling
- Use ShadCN UI components

---

## Phase 1: Database Foundation & Core Backend Infrastructure

### Phase 1A: Backend - Database Schema & Migrations

**Timeline:** Week 1-2  
**Developer:** Backend Developer

#### Tasks:

1. **Delete Old Migrations**
   - Remove all existing migrations in `server/migrations/` (except diesel_initial_setup)
   - Clean slate for new schema

2. **Create New Migration Structure**
   
   Create a single comprehensive migration: `2025-01-01-000001_librechat_schema`

   **Tables to create:**

   a. **users** (Enhanced)
   ```sql
   - id: TEXT PRIMARY KEY (Firebase UID)
   - email: TEXT UNIQUE NOT NULL
   - email_verified: BOOLEAN DEFAULT FALSE
   - name: TEXT
   - username: TEXT UNIQUE
   - password_hash: TEXT (for local auth, optional)
   - avatar_url: TEXT
   - provider: TEXT NOT NULL DEFAULT 'firebase'
   - role: TEXT DEFAULT 'user'
   - plugins: JSONB
   - two_factor_enabled: BOOLEAN DEFAULT FALSE
   - totp_secret: TEXT
   - refresh_tokens: JSONB (array of tokens)
   - expires_at: TIMESTAMPTZ
   - terms_accepted: BOOLEAN DEFAULT FALSE
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

   b. **conversations**
   ```sql
   - id: UUID PRIMARY KEY
   - conversation_id: TEXT UNIQUE NOT NULL (for API compatibility)
   - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - title: TEXT DEFAULT 'New Chat'
   - endpoint: TEXT NOT NULL (openai, anthropic, google, custom, etc.)
   - endpoint_type: TEXT
   - model: TEXT
   - model_label: TEXT
   - icon_url: TEXT
   -
   - # AI Parameters (all optional, provider-specific)
   - temperature: DOUBLE PRECISION
   - top_p: DOUBLE PRECISION
   - top_k: INTEGER
   - max_tokens: INTEGER
   - max_output_tokens: INTEGER
   - max_context_tokens: INTEGER
   - presence_penalty: DOUBLE PRECISION
   - frequency_penalty: DOUBLE PRECISION
   - stop_sequences: TEXT[] (array)
   - reasoning_effort: TEXT
   - 
   - # System/Instructions
   - prompt_prefix: TEXT
   - system_message: TEXT
   - instructions: TEXT
   - 
   - # Features
   - resend_files: BOOLEAN
   - resend_images: BOOLEAN
   - image_detail: TEXT
   - prompt_cache: BOOLEAN (Anthropic)
   - thinking: BOOLEAN
   - thinking_budget: INTEGER
   - 
   - # Agent/Assistant references
   - agent_id: TEXT
   - assistant_id: TEXT
   - agent_options: JSONB
   - 
   - # Metadata
   - tags: TEXT[] (array)
   - tools: TEXT[] (array)
   - is_archived: BOOLEAN DEFAULT FALSE
   - greeting: TEXT
   - spec: TEXT
   - 
   - # Timestamps
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   - expired_at: TIMESTAMPTZ (for temporary conversations)
   ```

   c. **messages**
   ```sql
   - id: UUID PRIMARY KEY
   - message_id: TEXT UNIQUE NOT NULL (for API compatibility)
   - conversation_id: UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE
   - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - 
   - # Message basics
   - parent_message_id: TEXT (references another message_id)
   - sender: TEXT NOT NULL
   - text: TEXT
   - is_created_by_user: BOOLEAN NOT NULL
   - 
   - # AI/Model info
   - model: TEXT
   - endpoint: TEXT
   - 
   - # Content (for multimodal messages)
   - content: JSONB (array of content blocks)
   - 
   - # Completion info
   - token_count: INTEGER
   - summary_token_count: INTEGER
   - finish_reason: TEXT
   - unfinished: BOOLEAN DEFAULT FALSE
   - error: BOOLEAN DEFAULT FALSE
   - 
   - # Attachments & plugins
   - files: JSONB (array of file references)
   - attachments: JSONB (array of attachment objects)
   - plugin_data: JSONB
   - plugins: JSONB
   - 
   - # Metadata
   - icon_url: TEXT
   - thread_id: TEXT (for assistants API)
   - 
   - # Timestamps
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   - expired_at: TIMESTAMPTZ
   ```

   d. **presets**
   ```sql
   - id: UUID PRIMARY KEY
   - preset_id: TEXT UNIQUE NOT NULL
   - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - title: TEXT NOT NULL
   - default_preset: BOOLEAN DEFAULT FALSE
   - order_index: INTEGER
   - 
   - # All conversation fields (same as conversations table)
   - endpoint: TEXT NOT NULL
   - endpoint_type: TEXT
   - model: TEXT
   - temperature: DOUBLE PRECISION
   - top_p: DOUBLE PRECISION
   - top_k: INTEGER
   - max_tokens: INTEGER
   - # ... (all the same fields as conversations)
   - 
   - agent_options: JSONB
   - 
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

   e. **agents**
   ```sql
   - id: UUID PRIMARY KEY
   - agent_id: TEXT UNIQUE NOT NULL
   - author_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - 
   - # Basic info
   - name: TEXT
   - description: TEXT
   - instructions: TEXT
   - author_name: TEXT
   - 
   - # Avatar
   - avatar_filepath: TEXT
   - avatar_source: TEXT
   - 
   - # Model configuration
   - provider: TEXT NOT NULL
   - model: TEXT NOT NULL
   - model_parameters: JSONB
   - 
   - # Behavior
   - artifacts: TEXT
   - access_level: INTEGER
   - recursion_limit: INTEGER
   - hide_sequential_outputs: BOOLEAN
   - end_after_tools: BOOLEAN
   - is_collaborative: BOOLEAN
   - 
   - # Tools & Actions
   - tools: TEXT[] (array)
   - tool_kwargs: JSONB
   - actions: TEXT[] (array)
   - tool_resources: JSONB
   - 
   - # Collaboration
   - agent_ids: TEXT[] (array of sub-agents)
   - project_ids: UUID[] (array)
   - 
   - # UI
   - conversation_starters: TEXT[] (array)
   - 
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

   f. **assistants**
   ```sql
   - id: UUID PRIMARY KEY
   - assistant_id: TEXT UNIQUE NOT NULL
   - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - 
   - avatar_filepath: TEXT
   - avatar_source: TEXT
   - conversation_starters: TEXT[] (array)
   - access_level: INTEGER
   - file_ids: TEXT[] (array)
   - actions: TEXT[] (array)
   - append_current_datetime: BOOLEAN DEFAULT FALSE
   - 
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

   g. **files**
   ```sql
   - id: UUID PRIMARY KEY
   - file_id: TEXT UNIQUE NOT NULL
   - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - conversation_id: UUID REFERENCES conversations(id) ON DELETE CASCADE
   - 
   - # File info
   - filename: TEXT NOT NULL
   - filepath: TEXT NOT NULL
   - file_type: TEXT NOT NULL
   - bytes: BIGINT NOT NULL
   - 
   - # Content
   - text_content: TEXT
   - embedded: BOOLEAN
   - 
   - # Metadata
   - source: TEXT DEFAULT 'local'
   - context: TEXT
   - usage_count: INTEGER DEFAULT 0
   - model: TEXT
   - width: INTEGER
   - height: INTEGER
   - metadata: JSONB
   - 
   - # Temporary files
   - temp_file_id: TEXT
   - expires_at: TIMESTAMPTZ
   - 
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

   h. **projects**
   ```sql
   - id: UUID PRIMARY KEY
   - name: TEXT UNIQUE NOT NULL
   - prompt_group_ids: UUID[] (array)
   - agent_ids: TEXT[] (array)
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

   i. **prompt_groups**
   ```sql
   - id: UUID PRIMARY KEY
   - name: TEXT NOT NULL
   - author_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - project_ids: UUID[] (array)
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

   j. **prompts**
   ```sql
   - id: UUID PRIMARY KEY
   - group_id: UUID NOT NULL REFERENCES prompt_groups(id) ON DELETE CASCADE
   - author_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - prompt: TEXT NOT NULL
   - prompt_type: TEXT NOT NULL CHECK (prompt_type IN ('text', 'chat'))
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

   k. **conversation_tags**
   ```sql
   - id: UUID PRIMARY KEY
   - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - tag: TEXT NOT NULL
   - description: TEXT
   - position: INTEGER
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   - UNIQUE(user_id, tag)
   ```

   l. **actions**
   ```sql
   - id: UUID PRIMARY KEY
   - action_id: TEXT UNIQUE NOT NULL
   - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - action_type: TEXT DEFAULT 'action_prototype'
   - settings: JSONB
   - agent_id: TEXT
   - assistant_id: TEXT
   - 
   - # Metadata
   - domain: TEXT NOT NULL
   - api_key: TEXT
   - auth_type: TEXT
   - auth_config: JSONB
   - privacy_policy_url: TEXT
   - raw_spec: TEXT
   - oauth_client_id: TEXT
   - oauth_client_secret: TEXT
   - 
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

   m. **tool_calls**
   ```sql
   - id: UUID PRIMARY KEY
   - conversation_id: UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE
   - message_id: UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE
   - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - tool_id: TEXT NOT NULL
   - result: JSONB
   - attachments: JSONB
   - block_index: INTEGER
   - part_index: INTEGER
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

   n. **balances** (for tracking token usage)
   ```sql
   - id: UUID PRIMARY KEY
   - user_id: TEXT UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - token_credit_balance: BIGINT DEFAULT 0
   - token_credit_consumed: BIGINT DEFAULT 0
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

   o. **transactions** (for tracking individual token usage)
   ```sql
   - id: UUID PRIMARY KEY
   - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - conversation_id: UUID REFERENCES conversations(id) ON DELETE CASCADE
   - model: TEXT
   - endpoint: TEXT
   - token_type: TEXT (prompt, completion, total)
   - token_count: INTEGER NOT NULL
   - rate: DOUBLE PRECISION
   - raw_amount: BIGINT
   - context: TEXT
   - created_at: TIMESTAMPTZ NOT NULL
   ```

   p. **shared_links** (for conversation sharing)
   ```sql
   - id: UUID PRIMARY KEY
   - share_id: TEXT UNIQUE NOT NULL
   - conversation_id: UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE
   - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
   - is_public: BOOLEAN DEFAULT FALSE
   - is_anonymous: BOOLEAN DEFAULT FALSE
   - title: TEXT
   - created_at: TIMESTAMPTZ NOT NULL
   - updated_at: TIMESTAMPTZ NOT NULL
   ```

3. **Create Indexes**
   ```sql
   -- Performance indexes
   CREATE INDEX idx_conversations_user_id ON conversations(user_id);
   CREATE INDEX idx_conversations_updated_at ON conversations(updated_at DESC);
   CREATE INDEX idx_conversations_user_archived ON conversations(user_id, is_archived);
   CREATE INDEX idx_conversations_tags ON conversations USING GIN(tags);
   
   CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
   CREATE INDEX idx_messages_user_id ON messages(user_id);
   CREATE INDEX idx_messages_created_at ON messages(created_at);
   CREATE INDEX idx_messages_parent ON messages(parent_message_id);
   
   CREATE INDEX idx_files_user_id ON files(user_id);
   CREATE INDEX idx_files_conversation_id ON files(conversation_id);
   
   CREATE INDEX idx_agents_author_id ON agents(author_id);
   CREATE INDEX idx_agents_project_ids ON agents USING GIN(project_ids);
   
   CREATE INDEX idx_tool_calls_conversation_id ON tool_calls(conversation_id);
   CREATE INDEX idx_tool_calls_message_id ON tool_calls(message_id);
   
   CREATE INDEX idx_transactions_user_id ON transactions(user_id);
   CREATE INDEX idx_transactions_created_at ON transactions(created_at);
   
   -- Full-text search indexes (for conversation/message search)
   CREATE INDEX idx_conversations_title_fts ON conversations USING GIN(to_tsvector('english', title));
   CREATE INDEX idx_messages_text_fts ON messages USING GIN(to_tsvector('english', COALESCE(text, '')));
   ```

4. **Update Diesel Schema**
   - Run `diesel migration run`
   - Update `src/db/schema.rs` with new tables

5. **Create Rust Models**
   - Update `src/db/models/` with structs for all new tables
   - Use `#[derive(Queryable, Insertable, AsChangeset)]` appropriately
   - Use `serde_json::Value` for JSONB fields

**Deliverables:**
- ✅ Clean migration structure (single comprehensive migration)
- ✅ All tables created with proper constraints
- ✅ Indexes for performance
- ✅ Updated schema.rs
- ✅ Rust model structs

---

### Phase 1B: Frontend - Project Structure & Base Components

**Timeline:** Week 1-2  
**Developer:** Frontend Developer

#### Tasks:

1. **Study LibreChat Frontend Structure**
   - Analyze `d:/Work/nodejs/LibreChat/LibreChat/client/src/`
   - Understand component hierarchy
   - Identify reusable patterns

2. **Set Up Type Definitions**
   
   Create `ui/src/types/librechat.ts`:
   ```typescript
   // Endpoint types
   export type Endpoint = 'openai' | 'anthropic' | 'google' | 'custom' | 'bedrock';
   
   // Conversation types
   export interface Conversation {
     id: string;
     conversationId: string;
     title: string;
     endpoint: Endpoint;
     model?: string;
     temperature?: number;
     maxTokens?: number;
     agentId?: string;
     tags?: string[];
     isArchived: boolean;
     createdAt: string;
     updatedAt: string;
   }
   
   // Message types
   export interface Message {
     id: string;
     messageId: string;
     conversationId: string;
     sender: string;
     text?: string;
     content?: any[];
     isCreatedByUser: boolean;
     model?: string;
     tokenCount?: number;
     finishReason?: string;
     error?: boolean;
     files?: any[];
     attachments?: any[];
     createdAt: string;
   }
   
   // Preset types
   export interface Preset {
     id: string;
     presetId: string;
     title: string;
     endpoint: Endpoint;
     model?: string;
     temperature?: number;
     maxTokens?: number;
     defaultPreset?: boolean;
   }
   
   // Agent types
   export interface Agent {
     id: string;
     agentId: string;
     name?: string;
     description?: string;
     provider: string;
     model: string;
     instructions?: string;
     tools?: string[];
     avatarFilepath?: string;
   }
   
   // Endpoint configuration
   export interface EndpointOption {
     endpoint: Endpoint;
     model?: string;
     temperature?: number;
     maxTokens?: number;
     topP?: number;
     topK?: number;
     presencePenalty?: number;
     frequencyPenalty?: number;
     stopSequences?: string[];
     systemMessage?: string;
   }
   ```

3. **Create Store Structure**
   
   Update `ui/src/stores/appStore.ts` to include:
   ```typescript
   // Add to existing store
   interface AppState {
     // ... existing fields
     
     // Conversations
     conversations: Conversation[];
     currentConversation: Conversation | null;
     
     // Messages
     messages: Message[];
     
     // Presets
     presets: Preset[];
     
     // Agents
     agents: Agent[];
     
     // Endpoint options
     endpointOptions: EndpointOption | null;
     
     // Actions
     setConversations: (conversations: Conversation[]) => void;
     setCurrentConversation: (conversation: Conversation | null) => void;
     setMessages: (messages: Message[]) => void;
     addMessage: (message: Message) => void;
     updateMessage: (id: string, updates: Partial<Message>) => void;
     setPresets: (presets: Preset[]) => void;
     setAgents: (agents: Agent[]) => void;
     setEndpointOptions: (options: EndpointOption) => void;
   }
   ```

4. **Create API Client Extensions**
   
   Create `ui/src/lib/librechat-client.ts`:
   ```typescript
   import { apiClient } from './api-client';
   import type { Conversation, Message, Preset, Agent } from '@/types/librechat';
   
   export const librechatClient = {
     // Conversations
     conversations: {
       list: () => apiClient.get<Conversation[]>('/api/v1/conversations'),
       get: (id: string) => apiClient.get<Conversation>(`/api/v1/conversations/${id}`),
       create: (data: Partial<Conversation>) => 
         apiClient.post<Conversation>('/api/v1/conversations', data),
       update: (id: string, data: Partial<Conversation>) => 
         apiClient.put<Conversation>(`/api/v1/conversations/${id}`, data),
       delete: (id: string) => apiClient.delete(`/api/v1/conversations/${id}`),
     },
     
     // Messages
     messages: {
       list: (conversationId: string) => 
         apiClient.get<Message[]>(`/api/v1/conversations/${conversationId}/messages`),
       create: (conversationId: string, data: Partial<Message>) =>
         apiClient.post<Message>(`/api/v1/conversations/${conversationId}/messages`, data),
     },
     
     // Presets
     presets: {
       list: () => apiClient.get<Preset[]>('/api/v1/presets'),
       create: (data: Partial<Preset>) => apiClient.post<Preset>('/api/v1/presets', data),
       update: (id: string, data: Partial<Preset>) =>
         apiClient.put<Preset>(`/api/v1/presets/${id}`, data),
       delete: (id: string) => apiClient.delete(`/api/v1/presets/${id}`),
     },
     
     // Agents
     agents: {
       list: () => apiClient.get<Agent[]>('/api/v1/agents'),
       get: (id: string) => apiClient.get<Agent>(`/api/v1/agents/${id}`),
       create: (data: Partial<Agent>) => apiClient.post<Agent>('/api/v1/agents', data),
       update: (id: string, data: Partial<Agent>) =>
         apiClient.put<Agent>(`/api/v1/agents/${id}`, data),
       delete: (id: string) => apiClient.delete(`/api/v1/agents/${id}`),
     },
     
     // Chat
     chat: {
       sendMessage: (data: {
         conversationId?: string;
         message: string;
         endpointOptions: any;
       }) => apiClient.post('/api/v1/chat', data),
       
       streamMessage: (data: {
         conversationId?: string;
         message: string;
         endpointOptions: any;
       }) => {
         // EventSource implementation for SSE
         const params = new URLSearchParams();
         params.append('data', JSON.stringify(data));
         return new EventSource(`/api/v1/chat/stream?${params}`);
       },
     },
   };
   ```

5. **Create Base Component Structure**
   
   Organize components following LibreChat's pattern:
   ```
   ui/src/components/
   ├── Chat/
   │   ├── ConversationList.tsx       (sidebar conversation list)
   │   ├── ConversationItem.tsx       (individual conversation)
   │   ├── MessageList.tsx            (existing, enhance)
   │   ├── MessageInput.tsx           (existing, enhance)
   │   ├── MessageBubble.tsx          (existing, enhance)
   │   └── ChatView.tsx               (existing, enhance)
   ├── Endpoints/
   │   ├── EndpointSelector.tsx       (dropdown to select AI provider)
   │   ├── ModelSelector.tsx          (existing, enhance)
   │   └── EndpointSettings.tsx       (temperature, tokens, etc.)
   ├── Presets/
   │   ├── PresetList.tsx
   │   ├── PresetItem.tsx
   │   └── PresetEditor.tsx
   ├── Agents/
   │   ├── AgentList.tsx
   │   ├── AgentCard.tsx
   │   └── AgentEditor.tsx
   └── Files/
       ├── FileUpload.tsx
       └── FileList.tsx
   ```

6. **Prepare Styling System**
   - Review Tailwind CSS v4 migration guide
   - Set up design tokens matching LibreChat's aesthetic
   - Create utility classes for chat UI
   - Ensure ShadCN components are styled consistently

**Deliverables:**
- ✅ Type definitions for all entities
- ✅ Extended API client
- ✅ Component structure
- ✅ Store with conversation/message state
- ✅ Styling foundation

---

## Phase 2: AI Provider Abstraction & Chat Functionality

### Phase 2A: Backend - AI Provider Trait System

**Timeline:** Week 3-4  
**Developer:** Backend Developer

#### Tasks:

1. **Create Provider Trait**
   
   Create `server/src/ai/mod.rs`:
   ```rust
   use async_trait::async_trait;
   use serde::{Deserialize, Serialize};
   use serde_json::Value;
   use anyhow::Result;
   use futures::Stream;
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ModelParameters {
       pub model: String,
       pub temperature: Option<f64>,
       pub max_tokens: Option<i32>,
       pub top_p: Option<f64>,
       pub top_k: Option<i32>,
       pub presence_penalty: Option<f64>,
       pub frequency_penalty: Option<f64>,
       pub stop: Option<Vec<String>>,
       #[serde(flatten)]
       pub extra: Option<Value>, // Provider-specific fields
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Message {
       pub role: String,
       pub content: String,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub name: Option<String>,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ChatRequest {
       pub messages: Vec<Message>,
       pub parameters: ModelParameters,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub system: Option<String>,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ChatResponse {
       pub message: String,
       pub finish_reason: Option<String>,
       pub usage: Option<TokenUsage>,
       pub model: String,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct TokenUsage {
       pub prompt_tokens: i32,
       pub completion_tokens: i32,
       pub total_tokens: i32,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct StreamChunk {
       pub delta: String,
       pub finish_reason: Option<String>,
   }
   
   #[async_trait]
   pub trait AIProvider: Send + Sync {
       /// Provider name (e.g., "openai", "anthropic")
       fn name(&self) -> &str;
       
       /// Non-streaming chat completion
       async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
       
       /// Streaming chat completion
       async fn chat_stream(
           &self,
           request: ChatRequest,
       ) -> Result<Box<dyn Stream<Item = Result<StreamChunk>> + Send + Unpin>>;
       
       /// Validate API key
       async fn validate_api_key(&self, api_key: &str) -> Result<bool>;
       
       /// List available models
       async fn list_models(&self) -> Result<Vec<String>>;
   }
   ```

2. **Implement OpenAI Provider**
   
   Create `server/src/ai/providers/openai.rs`:
   ```rust
   use super::*;
   use reqwest::Client;
   use serde_json::json;
   
   pub struct OpenAIProvider {
       api_key: String,
       client: Client,
       base_url: String,
   }
   
   impl OpenAIProvider {
       pub fn new(api_key: String) -> Self {
           Self {
               api_key,
               client: Client::new(),
               base_url: "https://api.openai.com/v1".to_string(),
           }
       }
       
       pub fn with_base_url(mut self, base_url: String) -> Self {
           self.base_url = base_url;
           self
       }
   }
   
   #[async_trait]
   impl AIProvider for OpenAIProvider {
       fn name(&self) -> &str {
           "openai"
       }
       
       async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
           let url = format!("{}/chat/completions", self.base_url);
           
           let body = json!({
               "model": request.parameters.model,
               "messages": request.messages,
               "temperature": request.parameters.temperature,
               "max_tokens": request.parameters.max_tokens,
               "top_p": request.parameters.top_p,
               "presence_penalty": request.parameters.presence_penalty,
               "frequency_penalty": request.parameters.frequency_penalty,
               "stop": request.parameters.stop,
           });
           
           let response = self.client
               .post(&url)
               .header("Authorization", format!("Bearer {}", self.api_key))
               .json(&body)
               .send()
               .await?
               .json::<serde_json::Value>()
               .await?;
           
           // Parse OpenAI response
           let message = response["choices"][0]["message"]["content"]
               .as_str()
               .ok_or_else(|| anyhow::anyhow!("Invalid response"))?
               .to_string();
           
           let finish_reason = response["choices"][0]["finish_reason"]
               .as_str()
               .map(String::from);
           
           let usage = response.get("usage").map(|u| TokenUsage {
               prompt_tokens: u["prompt_tokens"].as_i64().unwrap_or(0) as i32,
               completion_tokens: u["completion_tokens"].as_i64().unwrap_or(0) as i32,
               total_tokens: u["total_tokens"].as_i64().unwrap_or(0) as i32,
           });
           
           Ok(ChatResponse {
               message,
               finish_reason,
               usage,
               model: request.parameters.model,
           })
       }
       
       async fn chat_stream(
           &self,
           request: ChatRequest,
       ) -> Result<Box<dyn Stream<Item = Result<StreamChunk>> + Send + Unpin>> {
           // Implement SSE streaming
           // Use eventsource-stream or similar crate
           todo!("Implement streaming")
       }
       
       async fn validate_api_key(&self, api_key: &str) -> Result<bool> {
           let url = format!("{}/models", self.base_url);
           let response = self.client
               .get(&url)
               .header("Authorization", format!("Bearer {}", api_key))
               .send()
               .await?;
           
           Ok(response.status().is_success())
       }
       
       async fn list_models(&self) -> Result<Vec<String>> {
           let url = format!("{}/models", self.base_url);
           let response = self.client
               .get(&url)
               .header("Authorization", format!("Bearer {}", self.api_key))
               .send()
               .await?
               .json::<serde_json::Value>()
               .await?;
           
           let models = response["data"]
               .as_array()
               .ok_or_else(|| anyhow::anyhow!("Invalid response"))?
               .iter()
               .filter_map(|m| m["id"].as_str().map(String::from))
               .collect();
           
           Ok(models)
       }
   }
   ```

3. **Implement Anthropic Provider**
   
   Create `server/src/ai/providers/anthropic.rs`:
   ```rust
   // Similar structure to OpenAI but with Anthropic's API format
   pub struct AnthropicProvider {
       api_key: String,
       client: Client,
   }
   
   #[async_trait]
   impl AIProvider for AnthropicProvider {
       fn name(&self) -> &str {
           "anthropic"
       }
       
       async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
           // Anthropic uses different format:
           // - system message is separate parameter
           // - messages format is slightly different
           // - uses "max_tokens" instead of "max_tokens"
           // Implementation details...
           todo!()
       }
       
       // ... rest of implementation
   }
   ```

4. **Implement Google Provider**
   
   Create `server/src/ai/providers/google.rs`:
   ```rust
   pub struct GoogleProvider {
       api_key: String,
       client: Client,
   }
   
   #[async_trait]
   impl AIProvider for GoogleProvider {
       fn name(&self) -> &str {
           "google"
       }
       
       // Implement for Gemini API
       // ...
   }
   ```

5. **Create Provider Factory**
   
   Create `server/src/ai/factory.rs`:
   ```rust
   use super::providers::*;
   use super::AIProvider;
   use anyhow::{Result, anyhow};
   
   pub struct ProviderFactory;
   
   impl ProviderFactory {
       pub fn create(
           provider_name: &str,
           api_key: String,
           options: Option<serde_json::Value>,
       ) -> Result<Box<dyn AIProvider>> {
           match provider_name {
               "openai" => {
                   let mut provider = OpenAIProvider::new(api_key);
                   if let Some(opts) = options {
                       if let Some(base_url) = opts["base_url"].as_str() {
                           provider = provider.with_base_url(base_url.to_string());
                       }
                   }
                   Ok(Box::new(provider))
               }
               "anthropic" => Ok(Box::new(AnthropicProvider::new(api_key))),
               "google" => Ok(Box::new(GoogleProvider::new(api_key))),
               _ => Err(anyhow!("Unknown provider: {}", provider_name)),
           }
       }
   }
   ```

6. **Create API Routes**
   
   Create `server/src/routes/chat.rs`:
   ```rust
   use axum::{
       extract::{Path, State},
       http::StatusCode,
       response::{Json, Response, sse::{Event, Sse}},
       Extension,
   };
   use futures::stream::Stream;
   use crate::ai::{ProviderFactory, ChatRequest};
   use crate::middleware::auth::Claims;
   
   pub async fn chat_completion(
       State(pool): State<DbPool>,
       Extension(claims): Extension<Claims>,
       Json(request): Json<ChatCompletionRequest>,
   ) -> Result<Json<ChatCompletionResponse>, AppError> {
       // 1. Get user's API key for the provider
       let api_key = get_user_api_key(&pool, &claims.sub, &request.endpoint).await?;
       
       // 2. Create provider
       let provider = ProviderFactory::create(
           &request.endpoint,
           api_key,
           request.provider_options,
       )?;
       
       // 3. Build chat request
       let chat_request = ChatRequest {
           messages: request.messages,
           parameters: request.parameters,
           system: request.system,
       };
       
       // 4. Call provider
       let response = provider.chat(chat_request).await?;
       
       // 5. Save message to database
       save_chat_message(&pool, &claims.sub, request.conversation_id, &response).await?;
       
       Ok(Json(response.into()))
   }
   
   pub async fn chat_stream(
       State(pool): State<DbPool>,
       Extension(claims): Extension<Claims>,
       Json(request): Json<ChatCompletionRequest>,
   ) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
       // Similar to chat_completion but returns SSE stream
       todo!()
   }
   ```

7. **Create Repositories**
   
   Create `server/src/db/repositories/conversation_repository.rs`:
   ```rust
   use diesel::prelude::*;
   use diesel_async::{AsyncPgConnection, RunQueryDsl};
   use uuid::Uuid;
   use crate::db::{models::*, schema::*};
   
   pub struct ConversationRepository;
   
   impl ConversationRepository {
       pub async fn create(
           conn: &mut AsyncPgConnection,
           user_id: &str,
           conversation: NewConversation,
       ) -> Result<Conversation, diesel::result::Error> {
           diesel::insert_into(conversations::table)
               .values(&conversation)
               .get_result(conn)
               .await
       }
       
       pub async fn list_by_user(
           conn: &mut AsyncPgConnection,
           user_id: &str,
           is_archived: bool,
       ) -> Result<Vec<Conversation>, diesel::result::Error> {
           conversations::table
               .filter(conversations::user_id.eq(user_id))
               .filter(conversations::is_archived.eq(is_archived))
               .order(conversations::updated_at.desc())
               .load(conn)
               .await
       }
       
       pub async fn get_by_id(
           conn: &mut AsyncPgConnection,
           id: Uuid,
           user_id: &str,
       ) -> Result<Conversation, diesel::result::Error> {
           conversations::table
               .filter(conversations::id.eq(id))
               .filter(conversations::user_id.eq(user_id))
               .first(conn)
               .await
       }
       
       pub async fn update(
           conn: &mut AsyncPgConnection,
           id: Uuid,
           user_id: &str,
           updates: ConversationUpdate,
       ) -> Result<Conversation, diesel::result::Error> {
           diesel::update(conversations::table)
               .filter(conversations::id.eq(id))
               .filter(conversations::user_id.eq(user_id))
               .set(&updates)
               .get_result(conn)
               .await
       }
       
       pub async fn delete(
           conn: &mut AsyncPgConnection,
           id: Uuid,
           user_id: &str,
       ) -> Result<usize, diesel::result::Error> {
           diesel::delete(conversations::table)
               .filter(conversations::id.eq(id))
               .filter(conversations::user_id.eq(user_id))
               .execute(conn)
               .await
       }
   }
   ```

   Create similar repositories for:
   - `message_repository.rs`
   - `preset_repository.rs`
   - `agent_repository.rs`
   - `file_repository.rs`

**Deliverables:**
- ✅ AI Provider trait system
- ✅ OpenAI, Anthropic, Google providers implemented
- ✅ Provider factory
- ✅ Chat completion endpoints (sync + streaming)
- ✅ Repository pattern for data access
- ✅ Integration with database

---

### Phase 2B: Frontend - Multi-Provider Chat Interface

**Timeline:** Week 3-4  
**Developer:** Frontend Developer

#### Tasks:

1. **Create Endpoint Selector Component**
   
   Create `ui/src/components/Endpoints/EndpointSelector.tsx`:
   ```tsx
   import { useState } from 'react';
   import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
   import type { Endpoint } from '@/types/librechat';
   
   const ENDPOINTS: { value: Endpoint; label: string; icon: string }[] = [
     { value: 'openai', label: 'OpenAI', icon: '🤖' },
     { value: 'anthropic', label: 'Anthropic (Claude)', icon: '🧠' },
     { value: 'google', label: 'Google (Gemini)', icon: '✨' },
     { value: 'custom', label: 'Custom', icon: '⚙️' },
   ];
   
   export function EndpointSelector({
     value,
     onChange,
   }: {
     value: Endpoint;
     onChange: (endpoint: Endpoint) => void;
   }) {
     return (
       <Select value={value} onValueChange={onChange}>
         <SelectTrigger className="w-[200px]">
           <SelectValue placeholder="Select AI Provider" />
         </SelectTrigger>
         <SelectContent>
           {ENDPOINTS.map((endpoint) => (
             <SelectItem key={endpoint.value} value={endpoint.value}>
               <span className="flex items-center gap-2">
                 <span>{endpoint.icon}</span>
                 <span>{endpoint.label}</span>
               </span>
             </SelectItem>
           ))}
         </SelectContent>
       </Select>
     );
   }
   ```

2. **Create Endpoint Settings Panel**
   
   Create `ui/src/components/Endpoints/EndpointSettings.tsx`:
   ```tsx
   import { useState } from 'react';
   import { Label } from '@/components/ui/label';
   import { Input } from '@/components/ui/input';
   import { Slider } from '@/components/ui/slider';
   import type { EndpointOption } from '@/types/librechat';
   
   export function EndpointSettings({
     options,
     onChange,
   }: {
     options: EndpointOption;
     onChange: (options: EndpointOption) => void;
   }) {
     return (
       <div className="space-y-4 p-4">
         <div>
           <Label>Temperature</Label>
           <Slider
             value={[options.temperature ?? 0.7]}
             onValueChange={([value]) => onChange({ ...options, temperature: value })}
             min={0}
             max={2}
             step={0.1}
           />
           <span className="text-sm text-muted-foreground">
             {options.temperature?.toFixed(1) ?? 0.7}
           </span>
         </div>
         
         <div>
           <Label>Max Tokens</Label>
           <Input
             type="number"
             value={options.maxTokens ?? 1024}
             onChange={(e) => onChange({ ...options, maxTokens: parseInt(e.target.value) })}
           />
         </div>
         
         {/* Add more settings based on endpoint */}
         {options.endpoint === 'openai' && (
           <>
             <div>
               <Label>Top P</Label>
               <Slider
                 value={[options.topP ?? 1]}
                 onValueChange={([value]) => onChange({ ...options, topP: value })}
                 min={0}
                 max={1}
                 step={0.05}
               />
             </div>
             
             <div>
               <Label>Frequency Penalty</Label>
               <Slider
                 value={[options.frequencyPenalty ?? 0]}
                 onValueChange={([value]) => onChange({ ...options, frequencyPenalty: value })}
                 min={-2}
                 max={2}
                 step={0.1}
               />
             </div>
           </>
         )}
       </div>
     );
   }
   ```

3. **Enhance Chat Components**
   
   Update `ui/src/components/chat/ChatView.tsx`:
   ```tsx
   import { useState, useEffect } from 'react';
   import { EndpointSelector } from '@/components/Endpoints/EndpointSelector';
   import { ModelSelector } from '@/components/model/ModelSelector';
   import { MessageList } from './MessageList';
   import { MessageInput } from './MessageInput';
   import { useAppStore } from '@/stores/appStore';
   import { librechatClient } from '@/lib/librechat-client';
   
   export function ChatView() {
     const {
       currentConversation,
       messages,
       endpointOptions,
       setEndpointOptions,
       addMessage,
     } = useAppStore();
     
     const [isLoading, setIsLoading] = useState(false);
     
     const handleSendMessage = async (text: string) => {
       if (!text.trim() || isLoading) return;
       
       setIsLoading(true);
       
       // Add user message
       const userMessage = {
         id: crypto.randomUUID(),
         messageId: crypto.randomUUID(),
         conversationId: currentConversation?.id ?? '',
         sender: 'user',
         text,
         isCreatedByUser: true,
         createdAt: new Date().toISOString(),
       };
       addMessage(userMessage);
       
       try {
         // Stream response
         const eventSource = librechatClient.chat.streamMessage({
           conversationId: currentConversation?.conversationId,
           message: text,
           endpointOptions,
         });
         
         let aiMessage = {
           id: crypto.randomUUID(),
           messageId: crypto.randomUUID(),
           conversationId: currentConversation?.id ?? '',
           sender: 'assistant',
           text: '',
           isCreatedByUser: false,
           createdAt: new Date().toISOString(),
         };
         
         addMessage(aiMessage);
         
         eventSource.onmessage = (event) => {
           const data = JSON.parse(event.data);
           aiMessage.text += data.delta;
           updateMessage(aiMessage.id, { text: aiMessage.text });
         };
         
         eventSource.onerror = () => {
           eventSource.close();
           setIsLoading(false);
         };
         
       } catch (error) {
         console.error('Error sending message:', error);
       } finally {
         setIsLoading(false);
       }
     };
     
     return (
       <div className="flex h-full flex-col">
         {/* Header with endpoint/model selector */}
         <div className="flex items-center gap-2 border-b p-4">
           <EndpointSelector
             value={endpointOptions?.endpoint ?? 'openai'}
             onChange={(endpoint) => setEndpointOptions({ ...endpointOptions, endpoint })}
           />
           <ModelSelector
             endpoint={endpointOptions?.endpoint}
             value={endpointOptions?.model}
             onChange={(model) => setEndpointOptions({ ...endpointOptions, model })}
           />
         </div>
         
         {/* Messages */}
         <MessageList messages={messages} isLoading={isLoading} />
         
         {/* Input */}
         <MessageInput onSend={handleSendMessage} disabled={isLoading} />
       </div>
     );
   }
   ```

4. **Create Conversation List**
   
   Create `ui/src/components/Chat/ConversationList.tsx`:
   ```tsx
   import { useEffect } from 'react';
   import { useAppStore } from '@/stores/appStore';
   import { librechatClient } from '@/lib/librechat-client';
   import { ConversationItem } from './ConversationItem';
   import { Button } from '@/components/ui/button';
   import { PlusIcon } from 'lucide-react';
   
   export function ConversationList() {
     const {
       conversations,
       currentConversation,
       setConversations,
       setCurrentConversation,
     } = useAppStore();
     
     useEffect(() => {
       loadConversations();
     }, []);
     
     const loadConversations = async () => {
       try {
         const convos = await librechatClient.conversations.list();
         setConversations(convos);
       } catch (error) {
         console.error('Error loading conversations:', error);
       }
     };
     
     const handleNewChat = async () => {
       try {
         const newConvo = await librechatClient.conversations.create({
           title: 'New Chat',
           endpoint: 'openai',
           isArchived: false,
         });
         setConversations([newConvo, ...conversations]);
         setCurrentConversation(newConvo);
       } catch (error) {
         console.error('Error creating conversation:', error);
       }
     };
     
     return (
       <div className="flex h-full flex-col">
         <div className="p-2">
           <Button onClick={handleNewChat} className="w-full">
             <PlusIcon className="mr-2 h-4 w-4" />
             New Chat
           </Button>
         </div>
         
         <div className="flex-1 overflow-y-auto">
           {conversations.map((convo) => (
             <ConversationItem
               key={convo.id}
               conversation={convo}
               isActive={currentConversation?.id === convo.id}
               onClick={() => setCurrentConversation(convo)}
             />
           ))}
         </div>
       </div>
     );
   }
   ```

5. **Implement Preset System**
   
   Create `ui/src/components/Presets/PresetSelector.tsx`:
   ```tsx
   // Component to quickly load saved presets (endpoint + model + settings)
   ```

6. **Add File Upload Support**
   
   Create `ui/src/components/Files/FileUpload.tsx`:
   ```tsx
   // Drag-and-drop file upload for multimodal conversations
   ```

**Deliverables:**
- ✅ Endpoint selector component
- ✅ Endpoint settings panel
- ✅ Enhanced chat view with multi-provider support
- ✅ Conversation list with new chat creation
- ✅ Preset system UI
- ✅ File upload component
- ✅ Streaming message support

---

## Development Guidelines

### Backend Best Practices (Rust)

1. **Error Handling**
   - Use `anyhow::Result` for business logic
   - Create custom error types for API responses
   - Implement proper error logging with context

2. **Async Patterns**
   - Use `tokio` for async runtime
   - Use `diesel_async` for database operations
   - Properly handle connection pools

3. **Type Safety**
   - Leverage Diesel's type-safe queries
   - Use `serde` for JSON serialization
   - Create NewType wrappers for IDs

4. **Testing**
   - Unit tests for repositories
   - Integration tests for API endpoints
   - Mock AI providers for testing

5. **Security**
   - Never log API keys
   - Encrypt sensitive data in database
   - Validate all user inputs
   - Use parameterized queries (Diesel does this automatically)

### Frontend Best Practices (React/TypeScript)

1. **Component Design**
   - Keep components small and focused
   - Use composition over inheritance
   - Implement proper prop types

2. **State Management**
   - Use Zustand for global state
   - Keep component state local when possible
   - Implement optimistic updates

3. **Performance**
   - Memoize expensive computations
   - Virtualize long lists
   - Lazy load routes and components

4. **Accessibility**
   - Use semantic HTML
   - Implement keyboard navigation
   - Add proper ARIA labels

5. **Testing**
   - Unit tests for utility functions
   - Component tests with React Testing Library
   - E2E tests for critical flows

### Code Organization

**Backend Structure:**
```
server/src/
├── main.rs
├── config/
├── db/
│   ├── mod.rs
│   ├── models/
│   ├── schema.rs
│   └── repositories/
├── ai/
│   ├── mod.rs
│   ├── providers/
│   │   ├── openai.rs
│   │   ├── anthropic.rs
│   │   └── google.rs
│   └── factory.rs
├── routes/
│   ├── mod.rs
│   ├── conversations.rs
│   ├── messages.rs
│   ├── chat.rs
│   ├── presets.rs
│   └── agents.rs
├── middleware/
└── utils/
```

**Frontend Structure:**
```
ui/src/
├── components/
│   ├── Chat/
│   ├── Endpoints/
│   ├── Presets/
│   ├── Agents/
│   └── ui/ (ShadCN)
├── pages/
├── stores/
├── hooks/
├── lib/
├── types/
└── utils/
```

---

## Testing Strategy

### Phase 1 Testing

**Backend:**
- [ ] Migration runs successfully
- [ ] All tables created with correct schema
- [ ] Indexes created
- [ ] Model structs compile
- [ ] Repository CRUD operations work

**Frontend:**
- [ ] Type definitions compile
- [ ] Store state management works
- [ ] API client methods defined
- [ ] Components render without errors

### Phase 2 Testing

**Backend:**
- [ ] OpenAI provider chat completion works
- [ ] Anthropic provider chat completion works
- [ ] Google provider chat completion works
- [ ] Streaming responses work
- [ ] API key validation works
- [ ] Messages saved to database correctly
- [ ] Token usage tracked

**Frontend:**
- [ ] Endpoint selector works
- [ ] Model selector updates per endpoint
- [ ] Settings panel shows correct fields
- [ ] Messages stream correctly
- [ ] Conversations list and update
- [ ] New chat creation works
- [ ] File upload works

---

## Deployment Considerations

### Database

- Use PostgreSQL 14+ for best JSONB performance
- Set up proper indexes in production
- Configure connection pooling (min: 5, max: 20)
- Enable query logging for debugging
- Set up automated backups

### Backend

- Deploy as standalone binary
- Use environment variables for configuration
- Set up log rotation
- Configure CORS properly
- Use reverse proxy (nginx/Caddy)
- Enable HTTPS

### Frontend

- Build with `pnpm build`
- Serve static assets from CDN
- Configure proper caching headers
- Enable gzip/brotli compression
- Set up error tracking (Sentry)

---

## Timeline Summary

| Phase | Duration | Focus |
|-------|----------|-------|
| Phase 1A | Week 1-2 | Backend: Database schema & migrations |
| Phase 1B | Week 1-2 | Frontend: Project structure & base components |
| Phase 2A | Week 3-4 | Backend: AI provider abstraction & chat API |
| Phase 2B | Week 3-4 | Frontend: Multi-provider chat interface |

**Total Estimated Time:** 4 weeks with 2 developers working in parallel

---

## Success Criteria

### Phase 1 Complete When:
- ✅ All database tables created and migrated
- ✅ Repository pattern implemented
- ✅ Frontend type system in place
- ✅ Component structure established
- ✅ API client scaffolded

### Phase 2 Complete When:
- ✅ At least 3 AI providers working (OpenAI, Anthropic, Google)
- ✅ Streaming chat responses work
- ✅ Conversations can be created and managed
- ✅ Messages are saved and displayed correctly
- ✅ Endpoint/model switching works
- ✅ Basic file upload works

### Project Complete When:
- ✅ All LibreChat core features implemented
- ✅ Tests passing
- ✅ Documentation complete
- ✅ Can be deployed to production
- ✅ Multi-user support working
- ✅ API keys management working

---

## Future Enhancements (Post Phase 2)

1. **Agent System**
   - Agent marketplace
   - Custom agent creation
   - Tool integration

2. **Advanced Features**
   - Message search (full-text)
   - Conversation branching
   - Voice input/output
   - Image generation

3. **Collaboration**
   - Shared conversations
   - Team workspaces
   - Role-based access control

4. **Integrations**
   - Plugin system
   - Webhooks
   - API for third-party integrations

5. **Performance**
   - Response caching
   - Conversation summarization
   - Token usage optimization

---

## Notes

- This plan prioritizes getting a working multi-provider chat system operational
- Each phase has clear deliverables that can be independently tested
- The division allows backend and frontend developers to work in parallel with minimal conflicts
- PostgreSQL with JSONB provides all the flexibility needed without MongoDB
- The trait-based provider system in Rust mirrors LibreChat's BaseClient pattern
- Maintaining Tailwind CSS v4 and ShadCN ensures visual consistency

---

## Questions to Address Before Starting

1. **API Keys Storage**: How should user API keys be encrypted? (AES-256-GCM recommended)
2. **File Storage**: Where to store uploaded files? (Local filesystem vs S3)
3. **Search**: Use PostgreSQL full-text search or add Meilisearch? (Start with PostgreSQL)
4. **Caching**: Add Redis for caching? (Optional, add later if needed)
5. **Real-time**: Use WebSockets or SSE for streaming? (SSE is simpler, recommend this)
6. **Rate Limiting**: Per-user or per-IP? (Both, with different limits)

---

**Document Version:** 1.0  
**Last Updated:** November 30, 2025  
**Status:** Ready for Review

