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

-   Message `content` (array of content blocks for multimodal messages)
-   Message `attachments` (flexible file attachments)
-   Message `plugin` data
-   Conversation `agent_options` (dynamic configuration)
-   Agent `model_parameters` (provider-specific settings)
-   Agent `tool_resources` (flexible tool configuration)
-   Preset `agent_options`
-   File `metadata`

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

-   Trait-based abstraction for AI providers (similar to BaseClient)
-   Axum routes for different endpoints
-   Diesel ORM with JSONB for flexible fields
-   Repository pattern for data access

**Frontend (React):**

-   Reuse LibreChat's React component patterns
-   Adapt state management to our existing approach
-   Maintain Tailwind CSS v4 styling
-   Use ShadCN UI components

---

## Phase 1: Database Foundation & Core Backend Infrastructure

### Phase 1A: Backend - Database Schema & Migrations

**Timeline:** Week 1-2  
**Developer:** Backend Developer

#### Overview: PostgreSQL-Optimized Schema Design

This schema is **normalized for PostgreSQL** (not a direct port from MongoDB). Key improvements:

**📊 Normalization Benefits:**
| Aspect | Before (MongoDB-style) | After (PostgreSQL-optimized) | Benefit |
|--------|----------------------|----------------------------|---------|
| Model info | Separate fields in multiple places | `ai_models` table + denormalized in messages | Reference data centralized, historical accuracy preserved |
| User in messages | Redundant `user_id` field | Removed (get via conversation JOIN) | Saves 16 bytes/message, maintains integrity |
| File references | JSONB with file data | `file_ids UUID[]` with FK | Referential integrity, easier queries |
| AI parameters | 10+ separate columns | `model_parameters JSONB` | Flexible, provider-agnostic |
| Feature flags | 6+ boolean columns | `feature_flags JSONB` | Easier to extend, cleaner schema |
| Foreign keys | TEXT IDs everywhere | Proper UUID FKs with constraints | Data integrity, cascading deletes |

**⚡ Performance Decisions:**

-   **Kept `model` TEXT in messages**: Avoid JOIN on hottest query (load conversation+messages)
-   **Composite indexes**: `(user_id, updated_at)` for fast conversation lists
-   **Partial indexes**: `WHERE is_archived = false` for common queries
-   **GIN indexes**: Array fields (tags, file_ids) and full-text search
-   **Array types**: Better than junction tables for simple many-to-many

**🔗 Referential Integrity:**

-   All foreign keys properly defined with CASCADE/SET NULL
-   `ai_models` table for reference data (not enforced FK in messages)
-   UUID primary keys for internal entities
-   TEXT IDs only where needed for external compatibility (Firebase UID, API IDs)

#### Tasks:

1. **Delete Old Migrations**

    - Remove all existing migrations in `server/migrations/` (except diesel_initial_setup)
    - Clean slate for new schema

2. **Create New Migration Structure**

    Create a single comprehensive migration: `2025-01-01-000001_librechat_schema`

    **Tables to create:**

    a. **users**

    ```sql
    - id: TEXT PRIMARY KEY (Firebase UID)
    - email: TEXT UNIQUE NOT NULL
    - email_verified: BOOLEAN DEFAULT FALSE
    - name: TEXT
    - username: TEXT UNIQUE
    - avatar_url: TEXT
    - provider: TEXT NOT NULL DEFAULT 'firebase'
    - role: TEXT DEFAULT 'user' CHECK (role IN ('user', 'admin', 'moderator'))
    -
    - # Security
    - password_hash: TEXT (for local auth, optional)
    - two_factor_enabled: BOOLEAN DEFAULT FALSE
    - totp_secret: TEXT (encrypted)
    -
    - # Preferences
    - preferences: JSONB DEFAULT '{}'::jsonb (UI settings, default models, etc.)
    -
    - # Terms
    - terms_accepted: BOOLEAN DEFAULT FALSE
    - terms_accepted_at: TIMESTAMPTZ
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    a2. **ai_models** (Reference/Configuration Table)

    ```sql
    - id: UUID PRIMARY KEY
    - provider: TEXT NOT NULL (openai, anthropic, google, custom)
    - model_id: TEXT NOT NULL (gpt-4-turbo, claude-3-opus, gemini-pro, etc.)
    - display_name: TEXT NOT NULL
    - description: TEXT
    -
    - # Capabilities
    - context_window: INTEGER NOT NULL
    - max_output_tokens: INTEGER
    - supports_streaming: BOOLEAN DEFAULT TRUE
    - supports_images: BOOLEAN DEFAULT FALSE
    - supports_functions: BOOLEAN DEFAULT FALSE
    - supports_vision: BOOLEAN DEFAULT FALSE
    -
    - # Pricing (per 1M tokens)
    - cost_per_input_token: NUMERIC(10, 6)
    - cost_per_output_token: NUMERIC(10, 6)
    -
    - # Status
    - is_active: BOOLEAN DEFAULT TRUE
    - deprecated_at: TIMESTAMPTZ
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    -
    - UNIQUE(provider, model_id)
    ```

    **Note**: This table is for metadata/configuration only. Messages store `model` as TEXT
    for historical accuracy (not a foreign key). This allows:

    - Historical accuracy even if model is deprecated
    - Faster queries (no JOIN needed to display messages)
    - Flexibility (model names from custom providers)

    a3. **user_api_keys** (Encrypted API key storage)

    ```sql
    - id: UUID PRIMARY KEY
    - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    - provider: TEXT NOT NULL (openai, anthropic, google, custom)
    - encrypted_key: TEXT NOT NULL (AES-256-GCM encrypted)
    - key_name: TEXT (user-friendly name, e.g., "My OpenAI Key")
    - is_default: BOOLEAN DEFAULT FALSE
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - last_used_at: TIMESTAMPTZ
    -
    - UNIQUE(user_id, provider, key_name)
    ```

    b. **conversations**

    ```sql
    - id: UUID PRIMARY KEY
    - conversation_id: TEXT UNIQUE NOT NULL (for API compatibility)
    - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    - title: TEXT DEFAULT 'New Chat'
    -
    - # Current provider/model (user's last selection, can change per message)
    - endpoint: TEXT NOT NULL (openai, anthropic, google, custom, etc.)
    - model: TEXT NOT NULL (current model selection)
    - model_label: TEXT (display name, can be NULL to use model)
    -
    - # AI Parameters (JSONB - truly dynamic, varies by provider)
    - # Stores: temperature, top_p, top_k, max_tokens, presence_penalty, frequency_penalty, stop_sequences, etc.
    - model_parameters: JSONB DEFAULT '{}'::jsonb
    -
    - # System/Instructions
    - system_message: TEXT
    - instructions: TEXT
    -
    - # Feature Flags (JSONB - optional boolean flags, varies by provider)
    - # Stores: resend_files, resend_images, image_detail, prompt_cache, thinking, thinking_budget, etc.
    - feature_flags: JSONB DEFAULT '{}'::jsonb
    -
    - # Agent/Assistant references (optional)
    - agent_id: UUID REFERENCES agents(id) ON DELETE SET NULL
    - assistant_id: UUID REFERENCES assistants(id) ON DELETE SET NULL
    - agent_options: JSONB
    -
    - # Metadata
    - is_archived: BOOLEAN DEFAULT FALSE
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    **Note**: Tags are now in `conversation_tags_map` junction table for proper normalization.

    c. **messages**

    ```sql
    - id: UUID PRIMARY KEY
    - message_id: TEXT UNIQUE NOT NULL (for API compatibility)
    - conversation_id: UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE
    - parent_message_id: UUID REFERENCES messages(id) ON DELETE SET NULL (for branching)
    -
    - # Message basics
    - role: TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system', 'tool'))
    - text: TEXT
    - is_created_by_user: BOOLEAN NOT NULL
    -
    - # AI/Model info (stored for historical accuracy - user can switch models mid-conversation)
    - # These are NOT redundant despite being in conversations table
    - model: TEXT (actual model used for THIS message, e.g., "gpt-4-turbo")
    - endpoint: TEXT (actual endpoint used for THIS message, e.g., "openai")
    -
    - # Content (for multimodal messages - images, files, structured content)
    - content: JSONB (array of content blocks: text, image_url, etc.)
    -
    - # Completion info
    - token_count: INTEGER (tokens used for this message)
    - finish_reason: TEXT (stop, length, content_filter, tool_calls, etc.)
    - error: BOOLEAN DEFAULT FALSE
    -
    - # File attachments (normalized - references files table)
    - file_ids: UUID[] DEFAULT ARRAY[]::UUID[] (references files.id)
    -
    - # Tool/Plugin data
    - tool_call_id: TEXT (for tool/function calling)
    - plugin_data: JSONB (plugin-specific metadata)
    -
    - # Metadata
    - thread_id: TEXT (for OpenAI Assistants API)
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    **Normalization decisions:**

    - ✅ **Removed `user_id`**: Get from `JOIN conversations` (saves 16 bytes per message)
    - ✅ **Removed `summary_token_count`**: Calculate on demand if needed
    - ✅ **Removed `unfinished`**: Derivable from `finish_reason IS NULL`
    - ✅ **Removed `icon_url`**: Get from agent/assistant/model
    - ✅ **Changed `files` JSONB to `file_ids` UUID[]**: Proper FK relationship
    - ✅ **Consolidated `sender` and `is_created_by_user` to `role`**: Single source of truth
    - ❌ **Kept `model` TEXT**: Historical accuracy (not FK to ai_models)
    - ❌ **Kept `endpoint` TEXT**: Can change per message

    ````

    d. **presets** (Saved conversation configurations)
    ```sql
    - id: UUID PRIMARY KEY
    - preset_id: TEXT UNIQUE NOT NULL (for API compatibility)
    - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    - title: TEXT NOT NULL
    - is_default: BOOLEAN DEFAULT FALSE
    - order_index: INTEGER DEFAULT 0
    -
    - # Provider/Model
    - endpoint: TEXT NOT NULL
    - model: TEXT NOT NULL
    - model_label: TEXT
    -
    - # AI Parameters (same structure as conversations)
    - model_parameters: JSONB DEFAULT '{}'::jsonb
    -
    - # System/Instructions
    - system_message: TEXT
    - instructions: TEXT
    -
    - # Feature Flags
    - feature_flags: JSONB DEFAULT '{}'::jsonb
    -
    - # Agent reference (optional)
    - agent_id: UUID REFERENCES agents(id) ON DELETE SET NULL
    - agent_options: JSONB
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ````

    e. **agents**

    ```sql
    - id: UUID PRIMARY KEY
    - agent_id: TEXT UNIQUE NOT NULL (for API compatibility)
    - author_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    -
    - # Basic info
    - name: TEXT NOT NULL
    - description: TEXT
    - instructions: TEXT
    -
    - # Avatar
    - avatar_filepath: TEXT
    - avatar_source: TEXT (url, upload, default)
    -
    - # Model configuration
    - provider: TEXT NOT NULL
    - model: TEXT NOT NULL
    - model_parameters: JSONB DEFAULT '{}'::jsonb
    -
    - # Behavior
    - access_level: INTEGER DEFAULT 0 (0=private, 1=shared, 2=public)
    - recursion_limit: INTEGER DEFAULT 5
    - hide_sequential_outputs: BOOLEAN DEFAULT FALSE
    - end_after_tools: BOOLEAN DEFAULT FALSE
    - is_collaborative: BOOLEAN DEFAULT FALSE
    -
    - # Tool resources (provider-specific configuration like vector store IDs, etc.)
    - tool_resources: JSONB DEFAULT '{}'::jsonb
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    **Note**: Tools, actions, sub-agents, and projects are now in junction tables:

    - `agent_tools` for agent ↔ tools relationship
    - `agent_actions` for agent ↔ actions relationship
    - `agent_hierarchy` for agent ↔ sub-agents relationship
    - `project_agents` for agent ↔ projects relationship
    - Conversation starters moved to `agent_conversation_starters` table (see below)

    f. **assistants** (OpenAI Assistants API compatibility)

    ```sql
    - id: UUID PRIMARY KEY
    - assistant_id: TEXT UNIQUE NOT NULL (OpenAI assistant ID)
    - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    -
    - # Basic info
    - name: TEXT
    - description: TEXT
    - instructions: TEXT
    -
    - # Avatar
    - avatar_filepath: TEXT
    - avatar_source: TEXT
    -
    - # Configuration
    - model: TEXT NOT NULL
    - tools: JSONB DEFAULT '[]'::jsonb (OpenAI tools format - keep as JSONB for API compatibility)
    - file_ids: UUID[] DEFAULT ARRAY[]::UUID[] (references files.id - OK as array, ordered list)
    -
    - # Behavior
    - access_level: INTEGER DEFAULT 0
    - append_current_datetime: BOOLEAN DEFAULT FALSE
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    **Note**:

    - `tools` stays as JSONB for OpenAI API compatibility (they use their own format)
    - `file_ids` stays as array (it's an ordered list of UUIDs with FK validation)
    - Conversation starters moved to `assistant_conversation_starters` table

    f2. **agent_conversation_starters**

    ```sql
    - id: UUID PRIMARY KEY
    - agent_id: UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE
    - text: TEXT NOT NULL
    - order_index: INTEGER DEFAULT 0
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    f3. **assistant_conversation_starters**

    ```sql
    - id: UUID PRIMARY KEY
    - assistant_id: UUID NOT NULL REFERENCES assistants(id) ON DELETE CASCADE
    - text: TEXT NOT NULL
    - order_index: INTEGER DEFAULT 0
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    g. **files**

    ```sql
    - id: UUID PRIMARY KEY
    - file_id: TEXT UNIQUE NOT NULL (for API compatibility)
    - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    - conversation_id: UUID REFERENCES conversations(id) ON DELETE SET NULL
    -
    - # File info
    - filename: TEXT NOT NULL
    - filepath: TEXT NOT NULL (relative path in storage)
    - mime_type: TEXT NOT NULL (image/png, application/pdf, etc.)
    - size_bytes: BIGINT NOT NULL
    -
    - # File type categorization
    - file_type: TEXT NOT NULL CHECK (file_type IN ('image', 'document', 'audio', 'video', 'other'))
    -
    - # Content (for text files / OCR)
    - text_content: TEXT
    - is_embedded: BOOLEAN DEFAULT FALSE (vector embeddings created)
    -
    - # Image-specific
    - width: INTEGER
    - height: INTEGER
    -
    - # Metadata
    - source: TEXT DEFAULT 'upload' (upload, url, generated)
    - metadata: JSONB DEFAULT '{}'::jsonb (additional provider-specific data)
    -
    - # Usage tracking
    - usage_count: INTEGER DEFAULT 0
    - last_used_at: TIMESTAMPTZ
    -
    - # Temporary files (e.g., from file uploads pending message send)
    - is_temporary: BOOLEAN DEFAULT FALSE
    - expires_at: TIMESTAMPTZ
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    h. **projects**

    ```sql
    - id: UUID PRIMARY KEY
    - name: TEXT NOT NULL
    - description: TEXT
    - owner_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    -
    - UNIQUE(owner_id, name)
    ```

    i. **prompt_groups**

    ```sql
    - id: UUID PRIMARY KEY
    - name: TEXT NOT NULL
    - description: TEXT
    - author_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    - project_id: UUID REFERENCES projects(id) ON DELETE SET NULL
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    -
    - UNIQUE(author_id, name)
    ```

    j. **prompts**

    ```sql
    - id: UUID PRIMARY KEY
    - group_id: UUID NOT NULL REFERENCES prompt_groups(id) ON DELETE CASCADE
    - title: TEXT
    - prompt_text: TEXT NOT NULL
    - prompt_type: TEXT NOT NULL CHECK (prompt_type IN ('system', 'user', 'template'))
    - variables: TEXT[] DEFAULT ARRAY[]::TEXT[] (e.g., ['name', 'topic'] for template vars)
    - order_index: INTEGER DEFAULT 0
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    k. **tags** (User-defined tags for organization)

    ```sql
    - id: UUID PRIMARY KEY
    - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    - name: TEXT NOT NULL
    - description: TEXT
    - color: TEXT (hex color for UI, e.g., '#FF5733')
    - position: INTEGER DEFAULT 0
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    -
    - UNIQUE(user_id, name)
    ```

    k2. **conversation_tags_map** (Junction table: conversations ↔ tags)

    ```sql
    - conversation_id: UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE
    - tag_id: UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    -
    - PRIMARY KEY (conversation_id, tag_id)
    ```

    k3. **tools** (Available tools/capabilities for agents)

    ```sql
    - id: UUID PRIMARY KEY
    - name: TEXT UNIQUE NOT NULL (web_search, code_interpreter, dalle, retrieval, etc.)
    - display_name: TEXT NOT NULL
    - description: TEXT
    - tool_type: TEXT NOT NULL CHECK (tool_type IN ('system', 'plugin', 'function', 'action'))
    - icon_url: TEXT
    - is_active: BOOLEAN DEFAULT TRUE
    - is_system: BOOLEAN DEFAULT FALSE (system tools can't be deleted)
    - configuration_schema: JSONB (JSON schema for tool configuration)
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    k4. **agent_tools** (Junction table: agents ↔ tools)

    ```sql
    - agent_id: UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE
    - tool_id: UUID NOT NULL REFERENCES tools(id) ON DELETE CASCADE
    - configuration: JSONB DEFAULT '{}'::jsonb (tool-specific config for this agent)
    - is_enabled: BOOLEAN DEFAULT TRUE
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    -
    - PRIMARY KEY (agent_id, tool_id)
    ```

    k5. **assistant_tools** (Junction table: assistants ↔ tools)

    ```sql
    - assistant_id: UUID NOT NULL REFERENCES assistants(id) ON DELETE CASCADE
    - tool_id: UUID NOT NULL REFERENCES tools(id) ON DELETE CASCADE
    - configuration: JSONB DEFAULT '{}'::jsonb
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    -
    - PRIMARY KEY (assistant_id, tool_id)
    ```

    k6. **agent_actions** (Junction table: agents ↔ actions)

    ```sql
    - agent_id: UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE
    - action_id: UUID NOT NULL REFERENCES actions(id) ON DELETE CASCADE
    - is_enabled: BOOLEAN DEFAULT TRUE
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    -
    - PRIMARY KEY (agent_id, action_id)
    ```

    k7. **project_agents** (Junction table: projects ↔ agents)

    ```sql
    - project_id: UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE
    - agent_id: UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE
    - role: TEXT (optional role description)
    - order_index: INTEGER DEFAULT 0
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    -
    - PRIMARY KEY (project_id, agent_id)
    ```

    k8. **agent_hierarchy** (Junction table: parent agents ↔ sub-agents)

    ```sql
    - parent_agent_id: UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE
    - sub_agent_id: UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE
    - order_index: INTEGER DEFAULT 0
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    -
    - PRIMARY KEY (parent_agent_id, sub_agent_id)
    - CHECK (parent_agent_id != sub_agent_id) -- Prevent self-reference
    ```

    l. **actions** (Custom tools/plugins)

    ```sql
    - id: UUID PRIMARY KEY
    - action_id: TEXT UNIQUE NOT NULL (for API compatibility)
    - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    -
    - # Basic info
    - name: TEXT NOT NULL
    - description: TEXT
    - action_type: TEXT DEFAULT 'openapi' CHECK (action_type IN ('openapi', 'function', 'webhook'))
    -
    - # Configuration
    - domain: TEXT
    - endpoint_url: TEXT
    - settings: JSONB DEFAULT '{}'::jsonb
    -
    - # Authentication
    - auth_type: TEXT CHECK (auth_type IN ('none', 'api_key', 'oauth', 'bearer'))
    - auth_config: JSONB DEFAULT '{}'::jsonb (stores encrypted credentials)
    -
    - # OpenAPI spec
    - openapi_spec: TEXT (raw OpenAPI/Swagger spec)
    -
    - # Privacy
    - privacy_policy_url: TEXT
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    m. **tool_calls** (Function/Tool execution logs)

    ```sql
    - id: UUID PRIMARY KEY
    - message_id: UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE
    -
    - # Tool info
    - tool_call_id: TEXT NOT NULL (provider's tool call ID)
    - tool_name: TEXT NOT NULL (function name / tool name)
    - tool_type: TEXT DEFAULT 'function' CHECK (tool_type IN ('function', 'code_interpreter', 'retrieval', 'web_search'))
    -
    - # Execution
    - arguments: JSONB (function arguments)
    - result: JSONB (function result)
    - status: TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed'))
    - error_message: TEXT
    -
    - # Output files (e.g., from code_interpreter)
    - output_file_ids: UUID[] DEFAULT ARRAY[]::UUID[]
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - completed_at: TIMESTAMPTZ
    ```

    n. **balances** (User token/credit balance)

    ```sql
    - id: UUID PRIMARY KEY
    - user_id: TEXT UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE
    -
    - # Token credits (if using credit system)
    - token_credit_balance: BIGINT DEFAULT 0
    - token_credit_consumed: BIGINT DEFAULT 0
    -
    - # Monetary credits (if using prepaid system)
    - monetary_balance: NUMERIC(10, 2) DEFAULT 0.00
    - monetary_consumed: NUMERIC(10, 2) DEFAULT 0.00
    - currency: TEXT DEFAULT 'USD'
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    o. **transactions** (Token usage tracking for billing/analytics)

    ```sql
    - id: UUID PRIMARY KEY
    - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    - message_id: UUID REFERENCES messages(id) ON DELETE SET NULL
    - conversation_id: UUID REFERENCES conversations(id) ON DELETE SET NULL
    -
    - # Model info (denormalized for historical accuracy)
    - provider: TEXT NOT NULL
    - model: TEXT NOT NULL
    -
    - # Token usage
    - input_tokens: INTEGER DEFAULT 0
    - output_tokens: INTEGER DEFAULT 0
    - total_tokens: INTEGER NOT NULL
    -
    - # Cost (calculated at time of transaction)
    - cost_per_input_token: NUMERIC(10, 8)
    - cost_per_output_token: NUMERIC(10, 8)
    - total_cost: NUMERIC(10, 6)
    - currency: TEXT DEFAULT 'USD'
    -
    - # Context
    - transaction_type: TEXT DEFAULT 'completion' CHECK (transaction_type IN ('completion', 'embedding', 'image', 'tts', 'stt'))
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    ```

    p. **shared_links** (Conversation sharing)

    ```sql
    - id: UUID PRIMARY KEY
    - share_id: TEXT UNIQUE NOT NULL (public share ID, e.g., 'abc123xyz')
    - conversation_id: UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE
    - user_id: TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE
    -
    - # Sharing settings
    - is_public: BOOLEAN DEFAULT FALSE
    - is_anonymous: BOOLEAN DEFAULT FALSE (hide user info)
    - title: TEXT (custom title for shared link)
    -
    - # Access control
    - password_hash: TEXT (optional password protection)
    - max_views: INTEGER (NULL = unlimited)
    - view_count: INTEGER DEFAULT 0
    - expires_at: TIMESTAMPTZ
    -
    - # Timestamps
    - created_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - updated_at: TIMESTAMPTZ NOT NULL DEFAULT NOW()
    - last_viewed_at: TIMESTAMPTZ
    ```

3. **Create Indexes**

    ```sql
    -- ===== PERFORMANCE INDEXES =====

    -- ai_models: lookup by provider and model_id
    CREATE INDEX idx_ai_models_provider_model ON ai_models(provider, model_id);
    CREATE INDEX idx_ai_models_active ON ai_models(is_active) WHERE is_active = true;

    -- user_api_keys: lookup by user and provider
    CREATE INDEX idx_user_api_keys_user_provider ON user_api_keys(user_id, provider);
    CREATE INDEX idx_user_api_keys_default ON user_api_keys(user_id, is_default) WHERE is_default = true;

     -- conversations: most frequent queries
     CREATE INDEX idx_conversations_user_id ON conversations(user_id);
     CREATE INDEX idx_conversations_user_updated ON conversations(user_id, updated_at DESC);
     CREATE INDEX idx_conversations_user_archived ON conversations(user_id, is_archived, updated_at DESC);
     CREATE INDEX idx_conversations_agent_id ON conversations(agent_id) WHERE agent_id IS NOT NULL;
     CREATE INDEX idx_conversations_assistant_id ON conversations(assistant_id) WHERE assistant_id IS NOT NULL;

     -- tags: lookup by user
     CREATE INDEX idx_tags_user_id ON tags(user_id, position);

     -- conversation_tags_map: junction table indexes
     CREATE INDEX idx_conversation_tags_map_tag_id ON conversation_tags_map(tag_id);
     -- Note: conversation_id already indexed via PRIMARY KEY

    -- messages: critical for performance (most queried table)
    CREATE INDEX idx_messages_conversation_id ON messages(conversation_id, created_at);
    CREATE INDEX idx_messages_parent ON messages(parent_message_id) WHERE parent_message_id IS NOT NULL;
    CREATE INDEX idx_messages_file_ids ON messages USING GIN(file_ids);
    CREATE INDEX idx_messages_model ON messages(model) WHERE model IS NOT NULL;

    -- files: lookup by user and conversation
    CREATE INDEX idx_files_user_id ON files(user_id);
    CREATE INDEX idx_files_conversation_id ON files(conversation_id) WHERE conversation_id IS NOT NULL;
    CREATE INDEX idx_files_temporary ON files(expires_at) WHERE is_temporary = true;

    -- presets: lookup by user
    CREATE INDEX idx_presets_user_id ON presets(user_id, order_index);
    CREATE INDEX idx_presets_default ON presets(user_id) WHERE is_default = true;

     -- agents: lookup by author and access level
     CREATE INDEX idx_agents_author_id ON agents(author_id);
     CREATE INDEX idx_agents_access_level ON agents(access_level);

     -- tools: lookup by name and type
     CREATE INDEX idx_tools_type ON tools(tool_type);
     CREATE INDEX idx_tools_active ON tools(is_active) WHERE is_active = true;

     -- agent_tools: junction table indexes
     CREATE INDEX idx_agent_tools_tool_id ON agent_tools(tool_id);
     -- Note: agent_id already indexed via PRIMARY KEY

     -- assistant_tools: junction table indexes
     CREATE INDEX idx_assistant_tools_tool_id ON assistant_tools(tool_id);

     -- agent_actions: junction table indexes
     CREATE INDEX idx_agent_actions_action_id ON agent_actions(action_id);

     -- project_agents: junction table indexes
     CREATE INDEX idx_project_agents_agent_id ON project_agents(agent_id);
     CREATE INDEX idx_project_agents_order ON project_agents(project_id, order_index);

     -- agent_hierarchy: junction table indexes
     CREATE INDEX idx_agent_hierarchy_sub_agent ON agent_hierarchy(sub_agent_id);
     CREATE INDEX idx_agent_hierarchy_order ON agent_hierarchy(parent_agent_id, order_index);

     -- agent_conversation_starters: lookup by agent
     CREATE INDEX idx_agent_conv_starters_agent ON agent_conversation_starters(agent_id, order_index);

     -- assistant_conversation_starters: lookup by assistant
     CREATE INDEX idx_assistant_conv_starters ON assistant_conversation_starters(assistant_id, order_index);

    -- assistants: lookup by user
    CREATE INDEX idx_assistants_user_id ON assistants(user_id);
    CREATE INDEX idx_assistants_file_ids ON assistants USING GIN(file_ids);

    -- projects: lookup by owner
    CREATE INDEX idx_projects_owner_id ON projects(owner_id);

    -- prompt_groups: lookup by author and project
    CREATE INDEX idx_prompt_groups_author_id ON prompt_groups(author_id);
    CREATE INDEX idx_prompt_groups_project_id ON prompt_groups(project_id) WHERE project_id IS NOT NULL;

     -- prompts: lookup by group
     CREATE INDEX idx_prompts_group_id ON prompts(group_id, order_index);

     -- actions: lookup by user
     CREATE INDEX idx_actions_user_id ON actions(user_id);
     CREATE INDEX idx_actions_type ON actions(action_type);

    -- tool_calls: lookup by message
    CREATE INDEX idx_tool_calls_message_id ON tool_calls(message_id);
    CREATE INDEX idx_tool_calls_status ON tool_calls(status, created_at) WHERE status != 'completed';

    -- transactions: analytics and billing queries
    CREATE INDEX idx_transactions_user_id ON transactions(user_id, created_at DESC);
    CREATE INDEX idx_transactions_message_id ON transactions(message_id) WHERE message_id IS NOT NULL;
    CREATE INDEX idx_transactions_conversation_id ON transactions(conversation_id) WHERE conversation_id IS NOT NULL;
    CREATE INDEX idx_transactions_provider_model ON transactions(provider, model, created_at);
    CREATE INDEX idx_transactions_created_at ON transactions(created_at DESC);

    -- shared_links: lookup by share_id (most common)
    -- Note: UNIQUE constraint on share_id already creates an index
    CREATE INDEX idx_shared_links_user_id ON shared_links(user_id);
    CREATE INDEX idx_shared_links_conversation_id ON shared_links(conversation_id);
    CREATE INDEX idx_shared_links_expires ON shared_links(expires_at) WHERE expires_at IS NOT NULL;

     -- ===== FULL-TEXT SEARCH INDEXES =====
     -- For conversation/message search functionality
     CREATE INDEX idx_conversations_title_fts ON conversations USING GIN(to_tsvector('english', title));
     CREATE INDEX idx_messages_text_fts ON messages USING GIN(to_tsvector('english', COALESCE(text, '')));
     CREATE INDEX idx_agents_name_fts ON agents USING GIN(to_tsvector('english', COALESCE(name, '')));
     CREATE INDEX idx_agents_description_fts ON agents USING GIN(to_tsvector('english', COALESCE(description, '')));
     CREATE INDEX idx_files_filename_fts ON files USING GIN(to_tsvector('english', filename));
     CREATE INDEX idx_tools_name_fts ON tools USING GIN(to_tsvector('english', display_name));
     CREATE INDEX idx_actions_name_fts ON actions USING GIN(to_tsvector('english', name));
    ```

4. **Update Diesel Schema**

    - Run `diesel migration run`
    - Update `src/db/schema.rs` with new tables

5. **Create Rust Models**
    - Update `src/db/models/` with structs for all new tables
    - Use `#[derive(Queryable, Insertable, AsChangeset)]` appropriately
    - Use `serde_json::Value` for JSONB fields

**Schema Design Principles:**

✅ **Normalization Decisions:**

1.  **Removed truly redundant fields:**

    -   `user_id` from messages (get via JOIN with conversations)
    -   Consolidated multiple JSONB fields into single structured fields
    -   Changed file references from JSONB to UUID[] for referential integrity

2.  **Kept "denormalized" fields for performance:**

    -   `model` and `endpoint` in messages (TEXT, not FK) because:
        -   Historical accuracy: Preserve exact model/endpoint used
        -   Query performance: Most common query is "load conversation + messages" (avoid extra JOIN)
        -   Flexibility: Support custom/deprecated models
    -   Model/provider info in transactions (for billing accuracy)

3.  **Proper foreign keys and junction tables:**

    -   All ID references use proper FKs (UUID → UUID, TEXT → TEXT)
    -   Cascade deletes where appropriate (user deletes cascade)
    -   SET NULL for optional relationships
    -   Junction tables for many-to-many relationships:
        -   `conversation_tags_map` for conversation ↔ tags
        -   `agent_tools` for agent ↔ tools
        -   `assistant_tools` for assistant ↔ tools
        -   `agent_actions` for agent ↔ actions
        -   `project_agents` for project ↔ agents
        -   `agent_hierarchy` for agent ↔ sub-agents (multi-agent systems)

4.  **JSONB ONLY for truly dynamic/flexible data:**

    -   `model_parameters`: Provider-specific AI settings (OpenAI uses different params than Anthropic)
    -   `feature_flags`: Optional features that vary by provider
    -   `tool_resources`: Provider-specific tool configuration (e.g., vector store IDs)
    -   `metadata`: Extension points for future features
    -   `assistants.tools`: OpenAI API compatibility (their specific format)

5.  **UUID arrays ONLY for ordered lists with FK validation:**
    -   `file_ids UUID[]`: Ordered list of files (order matters for context)
    -   Simple, performant, maintains referential integrity via FK constraint
    -   NOT used for lookup data (use junction tables instead)

**Performance Optimizations:**

-   Composite indexes on (user_id, updated_at) for conversation lists
-   Partial indexes for common WHERE clauses (is_archived, is_active)
-   GIN indexes for array and full-text search columns
-   Denormalized model/endpoint in messages to avoid JOINs on hot path

**Deliverables:**

-   ✅ Clean migration structure (single comprehensive migration)
-   ✅ All tables created with proper constraints
-   ✅ Optimized indexes for performance
-   ✅ Proper foreign key relationships
-   ✅ Balanced normalization (performance + data integrity)
-   ✅ Updated schema.rs
-   ✅ Rust model structs

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
    export type Endpoint = "openai" | "anthropic" | "google" | "custom" | "bedrock";

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
    import { apiClient } from "./api-client";
    import type { Conversation, Message, Preset, Agent } from "@/types/librechat";

    export const librechatClient = {
        // Conversations
        conversations: {
            list: () => apiClient.get<Conversation[]>("/api/v1/conversations"),
            get: (id: string) => apiClient.get<Conversation>(`/api/v1/conversations/${id}`),
            create: (data: Partial<Conversation>) => apiClient.post<Conversation>("/api/v1/conversations", data),
            update: (id: string, data: Partial<Conversation>) => apiClient.put<Conversation>(`/api/v1/conversations/${id}`, data),
            delete: (id: string) => apiClient.delete(`/api/v1/conversations/${id}`),
        },

        // Messages
        messages: {
            list: (conversationId: string) => apiClient.get<Message[]>(`/api/v1/conversations/${conversationId}/messages`),
            create: (conversationId: string, data: Partial<Message>) => apiClient.post<Message>(`/api/v1/conversations/${conversationId}/messages`, data),
        },

        // Presets
        presets: {
            list: () => apiClient.get<Preset[]>("/api/v1/presets"),
            create: (data: Partial<Preset>) => apiClient.post<Preset>("/api/v1/presets", data),
            update: (id: string, data: Partial<Preset>) => apiClient.put<Preset>(`/api/v1/presets/${id}`, data),
            delete: (id: string) => apiClient.delete(`/api/v1/presets/${id}`),
        },

        // Agents
        agents: {
            list: () => apiClient.get<Agent[]>("/api/v1/agents"),
            get: (id: string) => apiClient.get<Agent>(`/api/v1/agents/${id}`),
            create: (data: Partial<Agent>) => apiClient.post<Agent>("/api/v1/agents", data),
            update: (id: string, data: Partial<Agent>) => apiClient.put<Agent>(`/api/v1/agents/${id}`, data),
            delete: (id: string) => apiClient.delete(`/api/v1/agents/${id}`),
        },

        // Chat
        chat: {
            sendMessage: (data: { conversationId?: string; message: string; endpointOptions: any }) => apiClient.post("/api/v1/chat", data),

            streamMessage: (data: { conversationId?: string; message: string; endpointOptions: any }) => {
                // EventSource implementation for SSE
                const params = new URLSearchParams();
                params.append("data", JSON.stringify(data));
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

-   ✅ Type definitions for all entities
-   ✅ Extended API client
-   ✅ Component structure
-   ✅ Store with conversation/message state
-   ✅ Styling foundation

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

-   ✅ AI Provider trait system
-   ✅ OpenAI, Anthropic, Google providers implemented
-   ✅ Provider factory
-   ✅ Chat completion endpoints (sync + streaming)
-   ✅ Repository pattern for data access
-   ✅ Integration with database

---

### Phase 2B: Frontend - Multi-Provider Chat Interface

**Timeline:** Week 3-4  
**Developer:** Frontend Developer

#### Tasks:

1. **Create Endpoint Selector Component**

    Create `ui/src/components/Endpoints/EndpointSelector.tsx`:

    ```tsx
    import { useState } from "react";
    import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
    import type { Endpoint } from "@/types/librechat";

    const ENDPOINTS: { value: Endpoint; label: string; icon: string }[] = [
        { value: "openai", label: "OpenAI", icon: "🤖" },
        { value: "anthropic", label: "Anthropic (Claude)", icon: "🧠" },
        { value: "google", label: "Google (Gemini)", icon: "✨" },
        { value: "custom", label: "Custom", icon: "⚙️" },
    ];

    export function EndpointSelector({ value, onChange }: { value: Endpoint; onChange: (endpoint: Endpoint) => void }) {
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
    import { useState } from "react";
    import { Label } from "@/components/ui/label";
    import { Input } from "@/components/ui/input";
    import { Slider } from "@/components/ui/slider";
    import type { EndpointOption } from "@/types/librechat";

    export function EndpointSettings({ options, onChange }: { options: EndpointOption; onChange: (options: EndpointOption) => void }) {
        return (
            <div className="space-y-4 p-4">
                <div>
                    <Label>Temperature</Label>
                    <Slider value={[options.temperature ?? 0.7]} onValueChange={([value]) => onChange({ ...options, temperature: value })} min={0} max={2} step={0.1} />
                    <span className="text-sm text-muted-foreground">{options.temperature?.toFixed(1) ?? 0.7}</span>
                </div>

                <div>
                    <Label>Max Tokens</Label>
                    <Input type="number" value={options.maxTokens ?? 1024} onChange={(e) => onChange({ ...options, maxTokens: parseInt(e.target.value) })} />
                </div>

                {/* Add more settings based on endpoint */}
                {options.endpoint === "openai" && (
                    <>
                        <div>
                            <Label>Top P</Label>
                            <Slider value={[options.topP ?? 1]} onValueChange={([value]) => onChange({ ...options, topP: value })} min={0} max={1} step={0.05} />
                        </div>

                        <div>
                            <Label>Frequency Penalty</Label>
                            <Slider value={[options.frequencyPenalty ?? 0]} onValueChange={([value]) => onChange({ ...options, frequencyPenalty: value })} min={-2} max={2} step={0.1} />
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
    import { useState, useEffect } from "react";
    import { EndpointSelector } from "@/components/Endpoints/EndpointSelector";
    import { ModelSelector } from "@/components/model/ModelSelector";
    import { MessageList } from "./MessageList";
    import { MessageInput } from "./MessageInput";
    import { useAppStore } from "@/stores/appStore";
    import { librechatClient } from "@/lib/librechat-client";

    export function ChatView() {
        const { currentConversation, messages, endpointOptions, setEndpointOptions, addMessage } = useAppStore();

        const [isLoading, setIsLoading] = useState(false);

        const handleSendMessage = async (text: string) => {
            if (!text.trim() || isLoading) return;

            setIsLoading(true);

            // Add user message
            const userMessage = {
                id: crypto.randomUUID(),
                messageId: crypto.randomUUID(),
                conversationId: currentConversation?.id ?? "",
                sender: "user",
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
                    conversationId: currentConversation?.id ?? "",
                    sender: "assistant",
                    text: "",
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
                console.error("Error sending message:", error);
            } finally {
                setIsLoading(false);
            }
        };

        return (
            <div className="flex h-full flex-col">
                {/* Header with endpoint/model selector */}
                <div className="flex items-center gap-2 border-b p-4">
                    <EndpointSelector value={endpointOptions?.endpoint ?? "openai"} onChange={(endpoint) => setEndpointOptions({ ...endpointOptions, endpoint })} />
                    <ModelSelector endpoint={endpointOptions?.endpoint} value={endpointOptions?.model} onChange={(model) => setEndpointOptions({ ...endpointOptions, model })} />
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
    import { useEffect } from "react";
    import { useAppStore } from "@/stores/appStore";
    import { librechatClient } from "@/lib/librechat-client";
    import { ConversationItem } from "./ConversationItem";
    import { Button } from "@/components/ui/button";
    import { PlusIcon } from "lucide-react";

    export function ConversationList() {
        const { conversations, currentConversation, setConversations, setCurrentConversation } = useAppStore();

        useEffect(() => {
            loadConversations();
        }, []);

        const loadConversations = async () => {
            try {
                const convos = await librechatClient.conversations.list();
                setConversations(convos);
            } catch (error) {
                console.error("Error loading conversations:", error);
            }
        };

        const handleNewChat = async () => {
            try {
                const newConvo = await librechatClient.conversations.create({
                    title: "New Chat",
                    endpoint: "openai",
                    isArchived: false,
                });
                setConversations([newConvo, ...conversations]);
                setCurrentConversation(newConvo);
            } catch (error) {
                console.error("Error creating conversation:", error);
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
                        <ConversationItem key={convo.id} conversation={convo} isActive={currentConversation?.id === convo.id} onClick={() => setCurrentConversation(convo)} />
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

-   ✅ Endpoint selector component
-   ✅ Endpoint settings panel
-   ✅ Enhanced chat view with multi-provider support
-   ✅ Conversation list with new chat creation
-   ✅ Preset system UI
-   ✅ File upload component
-   ✅ Streaming message support

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

-   [ ] Migration runs successfully
-   [ ] All tables created with correct schema
-   [ ] Indexes created
-   [ ] Model structs compile
-   [ ] Repository CRUD operations work

**Frontend:**

-   [ ] Type definitions compile
-   [ ] Store state management works
-   [ ] API client methods defined
-   [ ] Components render without errors

### Phase 2 Testing

**Backend:**

-   [ ] OpenAI provider chat completion works
-   [ ] Anthropic provider chat completion works
-   [ ] Google provider chat completion works
-   [ ] Streaming responses work
-   [ ] API key validation works
-   [ ] Messages saved to database correctly
-   [ ] Token usage tracked

**Frontend:**

-   [ ] Endpoint selector works
-   [ ] Model selector updates per endpoint
-   [ ] Settings panel shows correct fields
-   [ ] Messages stream correctly
-   [ ] Conversations list and update
-   [ ] New chat creation works
-   [ ] File upload works

---

## Deployment Considerations

### Database

-   Use PostgreSQL 14+ for best JSONB performance
-   Set up proper indexes in production
-   Configure connection pooling (min: 5, max: 20)
-   Enable query logging for debugging
-   Set up automated backups

### Backend

-   Deploy as standalone binary
-   Use environment variables for configuration
-   Set up log rotation
-   Configure CORS properly
-   Use reverse proxy (nginx/Caddy)
-   Enable HTTPS

### Frontend

-   Build with `pnpm build`
-   Serve static assets from CDN
-   Configure proper caching headers
-   Enable gzip/brotli compression
-   Set up error tracking (Sentry)

---

## Timeline Summary

| Phase    | Duration | Focus                                         |
| -------- | -------- | --------------------------------------------- |
| Phase 1A | Week 1-2 | Backend: Database schema & migrations         |
| Phase 1B | Week 1-2 | Frontend: Project structure & base components |
| Phase 2A | Week 3-4 | Backend: AI provider abstraction & chat API   |
| Phase 2B | Week 3-4 | Frontend: Multi-provider chat interface       |

**Total Estimated Time:** 4 weeks with 2 developers working in parallel

---

## Success Criteria

### Phase 1 Complete When:

-   ✅ All database tables created and migrated
-   ✅ Repository pattern implemented
-   ✅ Frontend type system in place
-   ✅ Component structure established
-   ✅ API client scaffolded

### Phase 2 Complete When:

-   ✅ At least 3 AI providers working (OpenAI, Anthropic, Google)
-   ✅ Streaming chat responses work
-   ✅ Conversations can be created and managed
-   ✅ Messages are saved and displayed correctly
-   ✅ Endpoint/model switching works
-   ✅ Basic file upload works

### Project Complete When:

-   ✅ All LibreChat core features implemented
-   ✅ Tests passing
-   ✅ Documentation complete
-   ✅ Can be deployed to production
-   ✅ Multi-user support working
-   ✅ API keys management working

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

-   This plan prioritizes getting a working multi-provider chat system operational
-   Each phase has clear deliverables that can be independently tested
-   The division allows backend and frontend developers to work in parallel with minimal conflicts
-   PostgreSQL with JSONB provides all the flexibility needed without MongoDB
-   The trait-based provider system in Rust mirrors LibreChat's BaseClient pattern
-   Maintaining Tailwind CSS v4 and ShadCN ensures visual consistency

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
