-- ==========================================
-- LibreChat Normalized PostgreSQL Schema
-- ==========================================
-- This migration creates a fully normalized database schema
-- for a multi-AI chat platform (LibreChat-inspired)
--
-- Design principles:
-- 1. Proper foreign keys and referential integrity
-- 2. JSONB only for truly dynamic/provider-specific data
-- 3. Junction tables for many-to-many relationships
-- 4. Strategic indexes for performance
-- 5. Denormalized fields only where justified (performance/history)
--
-- Version: 1.0
-- Date: December 1, 2025
-- ==========================================

-- ==========================================
-- CORE TABLES
-- ==========================================

-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,  -- Firebase UID
    email TEXT UNIQUE NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE,
    name TEXT,
    username TEXT UNIQUE,
    avatar_url TEXT,
    provider TEXT NOT NULL DEFAULT 'firebase',
    role TEXT DEFAULT 'user' CHECK (role IN ('user', 'admin', 'moderator')),
    
    -- Security
    password_hash TEXT,  -- for local auth, optional
    two_factor_enabled BOOLEAN DEFAULT FALSE,
    totp_secret TEXT,  -- encrypted
    
    -- Preferences (UI settings, default models, etc.)
    preferences JSONB DEFAULT '{}'::jsonb,
    
    -- Terms
    terms_accepted BOOLEAN DEFAULT FALSE,
    terms_accepted_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- AI Models reference table (metadata/configuration)
CREATE TABLE ai_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider TEXT NOT NULL,  -- openai, anthropic, google, custom
    model_id TEXT NOT NULL,  -- gpt-4-turbo, claude-3-opus, gemini-pro, etc.
    display_name TEXT NOT NULL,
    description TEXT,
    
    -- Capabilities
    context_window INTEGER NOT NULL,
    max_output_tokens INTEGER,
    supports_streaming BOOLEAN DEFAULT TRUE,
    supports_images BOOLEAN DEFAULT FALSE,
    supports_functions BOOLEAN DEFAULT FALSE,
    supports_vision BOOLEAN DEFAULT FALSE,
    
    -- Pricing (per 1M tokens)
    cost_per_input_token NUMERIC(10, 6),
    cost_per_output_token NUMERIC(10, 6),
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    deprecated_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(provider, model_id)
);

-- User API keys (encrypted storage)
CREATE TABLE user_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,  -- openai, anthropic, google, custom
    encrypted_key TEXT NOT NULL,  -- AES-256-GCM encrypted
    key_name TEXT,  -- user-friendly name, e.g., "My OpenAI Key"
    is_default BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    
    UNIQUE(user_id, provider, key_name)
);

-- ==========================================
-- CONVERSATION & MESSAGE TABLES
-- ==========================================

-- Agents table (must be created before conversations due to FK)
CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id TEXT UNIQUE NOT NULL,  -- for API compatibility
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Basic info
    name TEXT NOT NULL,
    description TEXT,
    instructions TEXT,
    
    -- Avatar
    avatar_filepath TEXT,
    avatar_source TEXT,  -- url, upload, default
    
    -- Model configuration
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    model_parameters JSONB DEFAULT '{}'::jsonb,
    
    -- Behavior
    access_level INTEGER DEFAULT 0,  -- 0=private, 1=shared, 2=public
    recursion_limit INTEGER DEFAULT 5,
    hide_sequential_outputs BOOLEAN DEFAULT FALSE,
    end_after_tools BOOLEAN DEFAULT FALSE,
    is_collaborative BOOLEAN DEFAULT FALSE,
    
    -- Tool resources (provider-specific configuration)
    tool_resources JSONB DEFAULT '{}'::jsonb,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Assistants table (OpenAI Assistants API compatibility)
CREATE TABLE assistants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assistant_id TEXT UNIQUE NOT NULL,  -- OpenAI assistant ID
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Basic info
    name TEXT,
    description TEXT,
    instructions TEXT,
    
    -- Avatar
    avatar_filepath TEXT,
    avatar_source TEXT,
    
    -- Configuration
    model TEXT NOT NULL,
    tools JSONB DEFAULT '[]'::jsonb,  -- OpenAI tools format (keep for API compatibility)
    file_ids UUID[] DEFAULT ARRAY[]::UUID[],  -- references files.id
    
    -- Behavior
    access_level INTEGER DEFAULT 0,
    append_current_datetime BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Conversations table
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id TEXT UNIQUE NOT NULL,  -- for API compatibility
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT DEFAULT 'New Chat',
    
    -- Current provider/model (user's last selection, can change per message)
    endpoint TEXT NOT NULL,  -- openai, anthropic, google, custom, etc.
    model TEXT NOT NULL,  -- current model selection
    model_label TEXT,  -- display name, can be NULL to use model
    
    -- AI Parameters (JSONB - truly dynamic, varies by provider)
    -- Stores: temperature, top_p, top_k, max_tokens, presence_penalty, frequency_penalty, stop_sequences, etc.
    model_parameters JSONB DEFAULT '{}'::jsonb,
    
    -- System/Instructions
    system_message TEXT,
    instructions TEXT,
    
    -- Feature Flags (JSONB - optional boolean flags, varies by provider)
    -- Stores: resend_files, resend_images, image_detail, prompt_cache, thinking, thinking_budget, etc.
    feature_flags JSONB DEFAULT '{}'::jsonb,
    
    -- Agent/Assistant references (optional)
    agent_id UUID REFERENCES agents(id) ON DELETE SET NULL,
    assistant_id UUID REFERENCES assistants(id) ON DELETE SET NULL,
    agent_options JSONB,
    
    -- Metadata
    is_archived BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Files table (must be created before messages due to FK)
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id TEXT UNIQUE NOT NULL,  -- for API compatibility
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    conversation_id UUID REFERENCES conversations(id) ON DELETE SET NULL,
    
    -- File info
    filename TEXT NOT NULL,
    filepath TEXT NOT NULL,  -- relative path in storage
    mime_type TEXT NOT NULL,  -- image/png, application/pdf, etc.
    size_bytes BIGINT NOT NULL,
    
    -- File type categorization
    file_type TEXT NOT NULL CHECK (file_type IN ('image', 'document', 'audio', 'video', 'other')),
    
    -- Content (for text files / OCR)
    text_content TEXT,
    is_embedded BOOLEAN DEFAULT FALSE,  -- vector embeddings created
    
    -- Image-specific
    width INTEGER,
    height INTEGER,
    
    -- Metadata
    source TEXT DEFAULT 'upload',  -- upload, url, generated
    metadata JSONB DEFAULT '{}'::jsonb,
    
    -- Usage tracking
    usage_count INTEGER DEFAULT 0,
    last_used_at TIMESTAMPTZ,
    
    -- Temporary files (e.g., from file uploads pending message send)
    is_temporary BOOLEAN DEFAULT FALSE,
    expires_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Messages table
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id TEXT UNIQUE NOT NULL,  -- for API compatibility
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    parent_message_id UUID REFERENCES messages(id) ON DELETE SET NULL,  -- for branching
    
    -- Message basics
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system', 'tool')),
    text TEXT,
    is_created_by_user BOOLEAN NOT NULL,
    
    -- AI/Model info (stored for historical accuracy)
    -- These are NOT redundant despite being in conversations table
    -- User can switch models mid-conversation
    model TEXT,  -- actual model used for THIS message, e.g., "gpt-4-turbo"
    endpoint TEXT,  -- actual endpoint used for THIS message, e.g., "openai"
    
    -- Content (for multimodal messages - images, files, structured content)
    content JSONB,  -- array of content blocks: text, image_url, etc.
    
    -- Completion info
    token_count INTEGER,  -- tokens used for this message
    finish_reason TEXT,  -- stop, length, content_filter, tool_calls, etc.
    error BOOLEAN DEFAULT FALSE,
    
    -- File attachments (normalized - references files table)
    file_ids UUID[] DEFAULT ARRAY[]::UUID[],  -- references files.id
    
    -- Tool/Plugin data
    tool_call_id TEXT,  -- for tool/function calling
    plugin_data JSONB,  -- plugin-specific metadata
    
    -- Metadata
    thread_id TEXT,  -- for OpenAI Assistants API
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ==========================================
-- PRESETS & CONFIGURATION
-- ==========================================

-- Presets (Saved conversation configurations)
CREATE TABLE presets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    preset_id TEXT UNIQUE NOT NULL,  -- for API compatibility
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    is_default BOOLEAN DEFAULT FALSE,
    order_index INTEGER DEFAULT 0,
    
    -- Provider/Model
    endpoint TEXT NOT NULL,
    model TEXT NOT NULL,
    model_label TEXT,
    
    -- AI Parameters (same structure as conversations)
    model_parameters JSONB DEFAULT '{}'::jsonb,
    
    -- System/Instructions
    system_message TEXT,
    instructions TEXT,
    
    -- Feature Flags
    feature_flags JSONB DEFAULT '{}'::jsonb,
    
    -- Agent reference (optional)
    agent_id UUID REFERENCES agents(id) ON DELETE SET NULL,
    agent_options JSONB,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ==========================================
-- PROJECTS & PROMPTS
-- ==========================================

-- Projects table
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    owner_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(owner_id, name)
);

-- Prompt groups table
CREATE TABLE prompt_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    author_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(author_id, name)
);

-- Prompts table
CREATE TABLE prompts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES prompt_groups(id) ON DELETE CASCADE,
    title TEXT,
    prompt_text TEXT NOT NULL,
    prompt_type TEXT NOT NULL CHECK (prompt_type IN ('system', 'user', 'template')),
    variables TEXT[] DEFAULT ARRAY[]::TEXT[],  -- e.g., ['name', 'topic'] for template vars
    order_index INTEGER DEFAULT 0,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ==========================================
-- TAGS SYSTEM
-- ==========================================

-- Tags table (User-defined tags for organization)
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,  -- hex color for UI, e.g., '#FF5733'
    position INTEGER DEFAULT 0,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id, name)
);

-- Conversation-Tags junction table (Many-to-many)
CREATE TABLE conversation_tags_map (
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (conversation_id, tag_id)
);

-- ==========================================
-- TOOLS & ACTIONS SYSTEM
-- ==========================================

-- Tools table (Available tools/capabilities for agents)
CREATE TABLE tools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT UNIQUE NOT NULL,  -- web_search, code_interpreter, dalle, retrieval, etc.
    display_name TEXT NOT NULL,
    description TEXT,
    tool_type TEXT NOT NULL CHECK (tool_type IN ('system', 'plugin', 'function', 'action')),
    icon_url TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    is_system BOOLEAN DEFAULT FALSE,  -- system tools can't be deleted
    configuration_schema JSONB,  -- JSON schema for tool configuration
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Agent-Tools junction table (Many-to-many: agents ↔ tools)
CREATE TABLE agent_tools (
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    tool_id UUID NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
    configuration JSONB DEFAULT '{}'::jsonb,  -- tool-specific config for this agent
    is_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (agent_id, tool_id)
);

-- Assistant-Tools junction table (Many-to-many: assistants ↔ tools)
CREATE TABLE assistant_tools (
    assistant_id UUID NOT NULL REFERENCES assistants(id) ON DELETE CASCADE,
    tool_id UUID NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
    configuration JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (assistant_id, tool_id)
);

-- Actions table (Custom tools/plugins)
CREATE TABLE actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id TEXT UNIQUE NOT NULL,  -- for API compatibility
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Basic info
    name TEXT NOT NULL,
    description TEXT,
    action_type TEXT DEFAULT 'openapi' CHECK (action_type IN ('openapi', 'function', 'webhook')),
    
    -- Configuration
    domain TEXT,
    endpoint_url TEXT,
    settings JSONB DEFAULT '{}'::jsonb,
    
    -- Authentication
    auth_type TEXT CHECK (auth_type IN ('none', 'api_key', 'oauth', 'bearer')),
    auth_config JSONB DEFAULT '{}'::jsonb,  -- stores encrypted credentials
    
    -- OpenAPI spec
    openapi_spec TEXT,  -- raw OpenAPI/Swagger spec
    
    -- Privacy
    privacy_policy_url TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Agent-Actions junction table (Many-to-many: agents ↔ actions)
CREATE TABLE agent_actions (
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    action_id UUID NOT NULL REFERENCES actions(id) ON DELETE CASCADE,
    is_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (agent_id, action_id)
);

-- ==========================================
-- AGENT & ASSISTANT RELATIONSHIPS
-- ==========================================

-- Project-Agents junction table (Many-to-many: projects ↔ agents)
CREATE TABLE project_agents (
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    role TEXT,  -- optional role description
    order_index INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (project_id, agent_id)
);

-- Agent hierarchy junction table (Many-to-many: parent agents ↔ sub-agents)
CREATE TABLE agent_hierarchy (
    parent_agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    sub_agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    order_index INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (parent_agent_id, sub_agent_id),
    CHECK (parent_agent_id != sub_agent_id)  -- Prevent self-reference
);

-- Agent conversation starters table
CREATE TABLE agent_conversation_starters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    order_index INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Assistant conversation starters table
CREATE TABLE assistant_conversation_starters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assistant_id UUID NOT NULL REFERENCES assistants(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    order_index INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ==========================================
-- TOOL CALLS & EXECUTION
-- ==========================================

-- Tool calls table (Function/Tool execution logs)
CREATE TABLE tool_calls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    
    -- Tool info
    tool_call_id TEXT NOT NULL,  -- provider's tool call ID
    tool_name TEXT NOT NULL,  -- function name / tool name
    tool_type TEXT DEFAULT 'function' CHECK (tool_type IN ('function', 'code_interpreter', 'retrieval', 'web_search')),
    
    -- Execution
    arguments JSONB,  -- function arguments
    result JSONB,  -- function result
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    error_message TEXT,
    
    -- Output files (e.g., from code_interpreter)
    output_file_ids UUID[] DEFAULT ARRAY[]::UUID[],
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- ==========================================
-- BILLING & TRACKING
-- ==========================================

-- Balances table (User token/credit balance)
CREATE TABLE balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Token credits (if using credit system)
    token_credit_balance BIGINT DEFAULT 0,
    token_credit_consumed BIGINT DEFAULT 0,
    
    -- Monetary credits (if using prepaid system)
    monetary_balance NUMERIC(10, 2) DEFAULT 0.00,
    monetary_consumed NUMERIC(10, 2) DEFAULT 0.00,
    currency TEXT DEFAULT 'USD',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Transactions table (Token usage tracking for billing/analytics)
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    conversation_id UUID REFERENCES conversations(id) ON DELETE SET NULL,
    
    -- Model info (denormalized for historical accuracy)
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    
    -- Token usage
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    total_tokens INTEGER NOT NULL,
    
    -- Cost (calculated at time of transaction)
    cost_per_input_token NUMERIC(10, 8),
    cost_per_output_token NUMERIC(10, 8),
    total_cost NUMERIC(10, 6),
    currency TEXT DEFAULT 'USD',
    
    -- Context
    transaction_type TEXT DEFAULT 'completion' CHECK (transaction_type IN ('completion', 'embedding', 'image', 'tts', 'stt')),
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ==========================================
-- SHARING
-- ==========================================

-- Shared links table (Conversation sharing)
CREATE TABLE shared_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    share_id TEXT UNIQUE NOT NULL,  -- public share ID, e.g., 'abc123xyz'
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Sharing settings
    is_public BOOLEAN DEFAULT FALSE,
    is_anonymous BOOLEAN DEFAULT FALSE,  -- hide user info
    title TEXT,  -- custom title for shared link
    
    -- Access control
    password_hash TEXT,  -- optional password protection
    max_views INTEGER,  -- NULL = unlimited
    view_count INTEGER DEFAULT 0,
    expires_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_viewed_at TIMESTAMPTZ
);

-- ==========================================
-- PERFORMANCE INDEXES
-- ==========================================

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
CREATE INDEX idx_shared_links_user_id ON shared_links(user_id);
CREATE INDEX idx_shared_links_conversation_id ON shared_links(conversation_id);
CREATE INDEX idx_shared_links_expires ON shared_links(expires_at) WHERE expires_at IS NOT NULL;

-- ==========================================
-- FULL-TEXT SEARCH INDEXES
-- ==========================================

-- For conversation/message search functionality
CREATE INDEX idx_conversations_title_fts ON conversations USING GIN(to_tsvector('english', title));
CREATE INDEX idx_messages_text_fts ON messages USING GIN(to_tsvector('english', COALESCE(text, '')));
CREATE INDEX idx_agents_name_fts ON agents USING GIN(to_tsvector('english', COALESCE(name, '')));
CREATE INDEX idx_agents_description_fts ON agents USING GIN(to_tsvector('english', COALESCE(description, '')));
CREATE INDEX idx_files_filename_fts ON files USING GIN(to_tsvector('english', filename));
CREATE INDEX idx_tools_name_fts ON tools USING GIN(to_tsvector('english', display_name));
CREATE INDEX idx_actions_name_fts ON actions USING GIN(to_tsvector('english', name));


