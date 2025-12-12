-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migrations.


-- Sets up a trigger for the given table to automatically set a column called
-- `updated_at` whenever the row is modified (unless `updated_at` was included
-- in the modified columns)
--
-- # Example
--
-- ```sql
-- CREATE TABLE users (id SERIAL PRIMARY KEY, updated_at TIMESTAMP NOT NULL DEFAULT NOW());
--
-- SELECT diesel_manage_updated_at('users');
-- ```
CREATE OR REPLACE FUNCTION diesel_manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION diesel_set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ==========================================
-- T3Chat Complete Initial Schema
-- ==========================================
-- This migration creates the complete database schema
-- for T3Chat (LibreChat-inspired multi-AI chat platform)
--
-- Includes:
-- 1. Diesel setup
-- 2. Core LibreChat schema
-- 3. User features
-- 4. AI providers and models
-- 5. OIDC authentication support
--
-- Version: 1.0
-- Date: December 2, 2025
-- ==========================================

-- ==========================================
-- DIESEL SCHEMA MIGRATIONS TABLE
-- ==========================================

CREATE TABLE IF NOT EXISTS __diesel_schema_migrations (
    version VARCHAR(50) PRIMARY KEY,
    run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ==========================================
-- CORE TABLES
-- ==========================================

-- Users table (ASP.NET Identity-like structure)
CREATE TABLE users (
    id TEXT PRIMARY KEY,  -- OIDC sub
    email TEXT UNIQUE NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE,
    name TEXT,
    username TEXT UNIQUE,
    avatar_url TEXT,
    provider TEXT NOT NULL DEFAULT 'local',
    
    -- Normalized fields for case-insensitive lookups (OIDC)
    normalized_email TEXT NOT NULL,
    normalized_username TEXT,
    
    -- Security
    password_hash TEXT,  -- for local auth, optional
    two_factor_enabled BOOLEAN DEFAULT FALSE,
    totp_secret TEXT,  -- encrypted
    
    -- Account status (OIDC)
    disabled BOOLEAN NOT NULL DEFAULT false,
    locked_out BOOLEAN NOT NULL DEFAULT false,
    lockout_end TIMESTAMPTZ,
    access_failed_count INTEGER NOT NULL DEFAULT 0,
    
    -- System user flag (cannot be deleted or disabled)
    -- Only the default admin user (system-admin-00000000) can be a system user
    is_system BOOLEAN NOT NULL DEFAULT false,
    
    -- Constraint: Only allow system-admin-00000000 to be a system user
    CONSTRAINT chk_only_one_system_user CHECK (
        is_system = false OR id = 'system-admin-00000000'
    ),
    
    -- Password tracking (OIDC)
    password_changed_at TIMESTAMPTZ,
    
    -- Login tracking (OIDC)
    last_login_at TIMESTAMPTZ,
    login_count INTEGER NOT NULL DEFAULT 0,
    
    -- Terms
    terms_accepted BOOLEAN DEFAULT FALSE,
    terms_accepted_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for users
CREATE UNIQUE INDEX idx_users_normalized_email_unique ON users(normalized_email);
CREATE INDEX idx_users_normalized_email ON users(normalized_email);
CREATE INDEX idx_users_normalized_username ON users(normalized_username) WHERE normalized_username IS NOT NULL;
CREATE INDEX idx_users_disabled ON users(disabled) WHERE disabled = true;
CREATE INDEX idx_users_locked_out ON users(locked_out) WHERE locked_out = true;
CREATE INDEX idx_users_provider ON users(provider);
CREATE INDEX idx_users_is_system ON users(is_system) WHERE is_system = true;

-- Trigger to prevent deletion of system users
CREATE OR REPLACE FUNCTION prevent_system_user_deletion()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.is_system = true THEN
        RAISE EXCEPTION 'Cannot delete system user: % (%). System users are protected.', OLD.username, OLD.email;
    END IF;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_system_user_deletion
BEFORE DELETE ON users
FOR EACH ROW
EXECUTE FUNCTION prevent_system_user_deletion();

-- Trigger to prevent disabling system users
CREATE OR REPLACE FUNCTION prevent_system_user_disable()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_system = true AND NEW.disabled = true AND (OLD.disabled = false OR OLD.disabled IS NULL) THEN
        RAISE EXCEPTION 'Cannot disable system user: % (%). System users must remain active.', NEW.username, NEW.email;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_system_user_disable
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION prevent_system_user_disable();

-- ==========================================
-- ROLES AND PERMISSIONS (ASP.NET Identity Model)
-- ==========================================

-- Roles table
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT UNIQUE NOT NULL,
    normalized_name TEXT UNIQUE NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT false,  -- system roles cannot be deleted
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_roles_normalized_name ON roles(normalized_name);

-- Insert default system roles
INSERT INTO roles (id, name, normalized_name, description, is_system) VALUES
    (gen_random_uuid(), 'User', 'USER', 'Standard user role with basic access', true),
    (gen_random_uuid(), 'Administrator', 'ADMINISTRATOR', 'Full system access with all permissions', true),
    (gen_random_uuid(), 'Moderator', 'MODERATOR', 'Content moderation and user management access', true);

-- User roles junction table (Many-to-Many: users ↔ roles)
CREATE TABLE user_roles (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);

-- Role claims table (Permissions/Claims for roles - ASP.NET Identity Model)
CREATE TABLE role_claims (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    claim_type TEXT NOT NULL,  -- e.g., 'permission', 'feature', 'scope'
    claim_value TEXT NOT NULL,  -- e.g., 'conversations.delete', 'admin.panel.access'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(role_id, claim_type, claim_value)
);

CREATE INDEX idx_role_claims_role_id ON role_claims(role_id);
CREATE INDEX idx_role_claims_type_value ON role_claims(claim_type, claim_value);

-- User claims table (Additional permissions/claims for specific users)
CREATE TABLE user_claims (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    claim_type TEXT NOT NULL,
    claim_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, claim_type, claim_value)
);

CREATE INDEX idx_user_claims_user_id ON user_claims(user_id);
CREATE INDEX idx_user_claims_type_value ON user_claims(claim_type, claim_value);

-- User preferences table (Normalized user settings)
CREATE TABLE user_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    preference_key TEXT NOT NULL,  -- e.g., 'theme', 'language', 'default_model', 'notifications_enabled'
    preference_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, preference_key)
);

CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);
CREATE INDEX idx_user_preferences_key ON user_preferences(preference_key);

-- User features table (feature flags)
CREATE TABLE user_features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    feature TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, feature)
);

CREATE INDEX idx_user_features_user_id ON user_features(user_id);
CREATE INDEX idx_user_features_feature ON user_features(feature);

-- ==========================================
-- AI PROVIDERS AND MODELS
-- ==========================================

-- AI Providers table
CREATE TABLE ai_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    base_url TEXT,
    website_url TEXT,
    documentation_url TEXT,
    disabled BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    requires_api_key BOOLEAN NOT NULL DEFAULT true,
    supports_streaming BOOLEAN NOT NULL DEFAULT true,
    supports_images BOOLEAN NOT NULL DEFAULT false,
    supports_functions BOOLEAN NOT NULL DEFAULT false,
    supports_vision BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_providers_provider_id ON ai_providers(provider_id);
CREATE INDEX idx_ai_providers_disabled ON ai_providers(disabled) WHERE disabled = true;
CREATE INDEX idx_ai_providers_is_active ON ai_providers(is_active) WHERE is_active = true;

-- AI Provider metadata table (Additional provider-specific metadata)
CREATE TABLE ai_provider_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES ai_providers(id) ON DELETE CASCADE,
    metadata_key TEXT NOT NULL,
    metadata_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_id, metadata_key)
);

CREATE INDEX idx_ai_provider_metadata_provider_id ON ai_provider_metadata(provider_id);

-- AI Models reference table (metadata/configuration)
CREATE TABLE ai_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID REFERENCES ai_providers(id) ON DELETE CASCADE, -- Made nullable to match schema.rs if needed, but keeping consistent
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
    disabled BOOLEAN NOT NULL DEFAULT false,
    is_paid BOOLEAN NOT NULL DEFAULT true,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(provider_id, model_id)
);

-- AI models indexes
CREATE INDEX idx_ai_models_provider_id ON ai_models(provider_id);
CREATE INDEX idx_ai_models_active ON ai_models(is_active) WHERE is_active = true;
CREATE INDEX idx_ai_models_disabled ON ai_models(disabled) WHERE disabled = true;
CREATE INDEX idx_ai_models_display_name ON ai_models(display_name);

-- User API keys (encrypted storage)
CREATE TABLE user_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL, -- Changed from provider_id UUID to provider TEXT
    encrypted_key TEXT NOT NULL,  -- AES-256-GCM encrypted
    key_name TEXT,  -- user-friendly name, e.g., "My OpenAI Key"
    is_default BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    
    UNIQUE(user_id, provider, key_name)
);

CREATE INDEX idx_user_api_keys_user_provider ON user_api_keys(user_id, provider);
CREATE INDEX idx_user_api_keys_default ON user_api_keys(user_id, is_default) WHERE is_default = true;

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
    avatar_source TEXT CHECK (avatar_source IN ('url', 'upload', 'default')),
    
    -- Model configuration (Changed to string fields)
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    model_parameters JSONB,
    
    -- Behavior
    access_level INTEGER DEFAULT 0 CHECK (access_level IN (0, 1, 2)),  -- 0=private, 1=shared, 2=public
    recursion_limit INTEGER DEFAULT 5,
    hide_sequential_outputs BOOLEAN DEFAULT FALSE,
    end_after_tools BOOLEAN DEFAULT FALSE,
    is_collaborative BOOLEAN DEFAULT FALSE,
    tool_resources JSONB,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_agents_author_id ON agents(author_id);
CREATE INDEX idx_agents_access_level ON agents(access_level);
CREATE INDEX idx_agents_name_fts ON agents USING GIN(to_tsvector('english', COALESCE(name, '')));
CREATE INDEX idx_agents_description_fts ON agents USING GIN(to_tsvector('english', COALESCE(description, '')));

-- Agent model parameters table (Normalized AI parameters)
CREATE TABLE agent_model_parameters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    parameter_key TEXT NOT NULL,  -- temperature, top_p, max_tokens, etc.
    parameter_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(agent_id, parameter_key)
);

CREATE INDEX idx_agent_model_parameters_agent_id ON agent_model_parameters(agent_id);

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
    avatar_source TEXT CHECK (avatar_source IN ('url', 'upload', 'default')),
    
    -- Configuration (Changed)
    model TEXT NOT NULL,
    tools JSONB,
    file_ids UUID[],
    
    -- Behavior
    access_level INTEGER DEFAULT 0 CHECK (access_level IN (0, 1, 2)),
    append_current_datetime BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_assistants_user_id ON assistants(user_id);

-- Conversations table
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id TEXT UNIQUE NOT NULL,  -- for API compatibility
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT DEFAULT 'New Chat',
    
    -- Current AI model (user's last selection, can change per message)
    endpoint TEXT NOT NULL,
    model TEXT NOT NULL,
    model_label TEXT,
    
    -- Parameters
    model_parameters JSONB,
    
    -- System/Instructions
    system_message TEXT,
    instructions TEXT,
    
    -- Features
    feature_flags JSONB,
    
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

CREATE INDEX idx_conversations_user_id ON conversations(user_id);
CREATE INDEX idx_conversations_user_updated ON conversations(user_id, updated_at DESC);
CREATE INDEX idx_conversations_user_archived ON conversations(user_id, is_archived, updated_at DESC);
CREATE INDEX idx_conversations_agent_id ON conversations(agent_id) WHERE agent_id IS NOT NULL;
CREATE INDEX idx_conversations_assistant_id ON conversations(assistant_id) WHERE assistant_id IS NOT NULL;
CREATE INDEX idx_conversations_title_fts ON conversations USING GIN(to_tsvector('english', title));

-- Files table (independent entity) - MOVED AFTER conversations
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id TEXT UNIQUE NOT NULL,  -- for API compatibility
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    conversation_id UUID REFERENCES conversations(id) ON DELETE SET NULL, -- Added FK
    
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
    source TEXT DEFAULT 'upload' CHECK (source IN ('upload', 'url', 'generated')),
    metadata JSONB, -- Added
    
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

CREATE INDEX idx_files_user_id ON files(user_id);
CREATE INDEX idx_files_conversation_id ON files(conversation_id);
CREATE INDEX idx_files_file_type ON files(file_type);
CREATE INDEX idx_files_temporary ON files(expires_at) WHERE is_temporary = true;
CREATE INDEX idx_files_filename_fts ON files USING GIN(to_tsvector('english', filename));

-- Assistant-Files junction table (Many-to-many: assistants ↔ files)
CREATE TABLE assistant_files (
    assistant_id UUID NOT NULL REFERENCES assistants(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    attached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (assistant_id, file_id)
);

CREATE INDEX idx_assistant_files_file_id ON assistant_files(file_id);

-- Conversation model parameters table (Normalized AI parameters)
CREATE TABLE conversation_model_parameters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    parameter_key TEXT NOT NULL,  -- temperature, top_p, top_k, max_tokens, presence_penalty, frequency_penalty, etc.
    parameter_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(conversation_id, parameter_key)
);

CREATE INDEX idx_conversation_model_parameters_conversation_id ON conversation_model_parameters(conversation_id);

-- Conversation feature flags table (Provider-specific feature flags)
CREATE TABLE conversation_feature_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    flag_key TEXT NOT NULL,  -- resend_files, resend_images, image_detail, prompt_cache, thinking, etc.
    flag_value BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(conversation_id, flag_key)
);

CREATE INDEX idx_conversation_feature_flags_conversation_id ON conversation_feature_flags(conversation_id);

-- Conversation-Files junction table (Many-to-many: conversations ↔ files)
CREATE TABLE conversation_files (
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    attached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (conversation_id, file_id)
);

CREATE INDEX idx_conversation_files_file_id ON conversation_files(file_id);

-- Conversation-Tags junction table (Many-to-many) - Moved up as it depends on conversations
-- Need tags table first. Tags depends on users.
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

CREATE INDEX idx_tags_user_id ON tags(user_id, position);

CREATE TABLE conversation_tags_map (
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (conversation_id, tag_id)
);

CREATE INDEX idx_conversation_tags_map_tag_id ON conversation_tags_map(tag_id);

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
    model TEXT,
    endpoint TEXT,
    
    -- Content (for multimodal messages)
    content JSONB,
    
    -- Completion info
    token_count INTEGER,  -- tokens used for this message
    finish_reason TEXT CHECK (finish_reason IN ('stop', 'length', 'content_filter', 'tool_calls', 'error')),
    error BOOLEAN DEFAULT FALSE,
    -- error_message TEXT, -- Removed to match schema.rs
    
    -- File attachments
    file_ids UUID[],
    
    -- Tool/Plugin data
    tool_call_id TEXT,  -- for tool/function calling
    plugin_data JSONB,
    
    -- Metadata
    thread_id TEXT,  -- for OpenAI Assistants API
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_messages_conversation_id ON messages(conversation_id, created_at);
CREATE INDEX idx_messages_parent ON messages(parent_message_id) WHERE parent_message_id IS NOT NULL;
CREATE INDEX idx_messages_role ON messages(role);
CREATE INDEX idx_messages_text_fts ON messages USING GIN(to_tsvector('english', COALESCE(text, '')));

-- Message-Files junction table (Many-to-many: messages ↔ files)
CREATE TABLE message_files (
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    attached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (message_id, file_id)
);

CREATE INDEX idx_message_files_file_id ON message_files(file_id);

-- Message content blocks table (Normalized multimodal content)
CREATE TABLE message_content_blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    block_type TEXT NOT NULL CHECK (block_type IN ('text', 'image_url', 'image_file', 'audio', 'video')),
    content_text TEXT,  -- for text blocks
    file_id UUID REFERENCES files(id) ON DELETE SET NULL,  -- for file-based content
    url TEXT,  -- for URL-based content
    detail TEXT CHECK (detail IN ('auto', 'low', 'high')),  -- for vision models
    order_index INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_message_content_blocks_message_id ON message_content_blocks(message_id, order_index);

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
    
    -- AI Model
    endpoint TEXT NOT NULL,
    model TEXT NOT NULL,
    model_label TEXT,
    model_parameters JSONB,
    
    -- System/Instructions
    system_message TEXT,
    instructions TEXT,
    
    -- Features
    feature_flags JSONB,
    
    -- Agent reference (optional)
    agent_id UUID REFERENCES agents(id) ON DELETE SET NULL,
    agent_options JSONB,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_presets_user_id ON presets(user_id, order_index);
CREATE INDEX idx_presets_default ON presets(user_id) WHERE is_default = true;

-- Preset model parameters table (Normalized AI parameters)
CREATE TABLE preset_model_parameters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    preset_id UUID NOT NULL REFERENCES presets(id) ON DELETE CASCADE,
    parameter_key TEXT NOT NULL,
    parameter_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(preset_id, parameter_key)
);

CREATE INDEX idx_preset_model_parameters_preset_id ON preset_model_parameters(preset_id);

-- Preset feature flags table
CREATE TABLE preset_feature_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    preset_id UUID NOT NULL REFERENCES presets(id) ON DELETE CASCADE,
    flag_key TEXT NOT NULL,
    flag_value BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(preset_id, flag_key)
);

CREATE INDEX idx_preset_feature_flags_preset_id ON preset_feature_flags(preset_id);

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

CREATE INDEX idx_projects_owner_id ON projects(owner_id);

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

CREATE INDEX idx_prompt_groups_author_id ON prompt_groups(author_id);
CREATE INDEX idx_prompt_groups_project_id ON prompt_groups(project_id) WHERE project_id IS NOT NULL;

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

CREATE INDEX idx_prompts_group_id ON prompts(group_id, order_index);

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
    configuration_schema JSONB,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tools_type ON tools(tool_type);
CREATE INDEX idx_tools_active ON tools(is_active) WHERE is_active = true;
CREATE INDEX idx_tools_system ON tools(is_system) WHERE is_system = true;
CREATE INDEX idx_tools_name_fts ON tools USING GIN(to_tsvector('english', display_name));

-- Tool configuration schema table (JSON Schema properties for tool configuration)
CREATE TABLE tool_config_schema (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tool_id UUID NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
    property_name TEXT NOT NULL,
    property_type TEXT NOT NULL,  -- string, number, boolean, array, object
    required BOOLEAN DEFAULT FALSE,
    default_value TEXT,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tool_id, property_name)
);

CREATE INDEX idx_tool_config_schema_tool_id ON tool_config_schema(tool_id);

-- Agent-Tools junction table (Many-to-many: agents ↔ tools)
CREATE TABLE agent_tools (
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    tool_id UUID NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
    configuration JSONB,
    is_enabled BOOLEAN DEFAULT TRUE,
    order_index INTEGER DEFAULT 0, -- Not in schema.rs but seems useful? schema.rs only has created_at?
    -- schema.rs for agent_tools: configuration -> Nullable<Jsonb>, is_enabled -> Nullable<Bool>
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (agent_id, tool_id)
);

CREATE INDEX idx_agent_tools_tool_id ON agent_tools(tool_id);

-- Agent tool configuration table (Tool-specific config for agents)
CREATE TABLE agent_tool_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    tool_id UUID NOT NULL,
    config_key TEXT NOT NULL,
    config_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (agent_id, tool_id) REFERENCES agent_tools(agent_id, tool_id) ON DELETE CASCADE,
    UNIQUE(agent_id, tool_id, config_key)
);

CREATE INDEX idx_agent_tool_config_agent_tool ON agent_tool_config(agent_id, tool_id);

-- Assistant-Tools junction table (Many-to-many: assistants ↔ tools)
CREATE TABLE assistant_tools (
    assistant_id UUID NOT NULL REFERENCES assistants(id) ON DELETE CASCADE,
    tool_id UUID NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
    configuration JSONB,
    is_enabled BOOLEAN DEFAULT TRUE, -- Not in schema.rs? schema.rs: configuration -> Nullable<Jsonb>
    order_index INTEGER DEFAULT 0, -- schema.rs doesn't show order_index
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (assistant_id, tool_id)
);

CREATE INDEX idx_assistant_tools_tool_id ON assistant_tools(tool_id);

-- Assistant tool configuration table
CREATE TABLE assistant_tool_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assistant_id UUID NOT NULL,
    tool_id UUID NOT NULL,
    config_key TEXT NOT NULL,
    config_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (assistant_id, tool_id) REFERENCES assistant_tools(assistant_id, tool_id) ON DELETE CASCADE,
    UNIQUE(assistant_id, tool_id, config_key)
);

CREATE INDEX idx_assistant_tool_config_assistant_tool ON assistant_tool_config(assistant_id, tool_id);

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
    settings JSONB,
    
    -- Authentication
    auth_type TEXT DEFAULT 'none' CHECK (auth_type IN ('none', 'api_key', 'oauth', 'bearer')),
    auth_config JSONB,
    
    -- OpenAPI spec
    openapi_spec TEXT,  -- raw OpenAPI/Swagger spec
    
    -- Privacy
    privacy_policy_url TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_actions_user_id ON actions(user_id);
CREATE INDEX idx_actions_type ON actions(action_type);
CREATE INDEX idx_actions_name_fts ON actions USING GIN(to_tsvector('english', name));

-- Action settings table (Normalized configuration)
CREATE TABLE action_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES actions(id) ON DELETE CASCADE,
    setting_key TEXT NOT NULL,
    setting_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(action_id, setting_key)
);

CREATE INDEX idx_action_settings_action_id ON action_settings(action_id);

-- Action authentication config table (Encrypted credentials storage)
CREATE TABLE action_auth_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES actions(id) ON DELETE CASCADE,
    config_key TEXT NOT NULL,  -- api_key, client_id, client_secret, token, etc.
    encrypted_value TEXT NOT NULL,  -- AES-256-GCM encrypted
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(action_id, config_key)
);

CREATE INDEX idx_action_auth_config_action_id ON action_auth_config(action_id);

-- Agent-Actions junction table (Many-to-many: agents ↔ actions)
CREATE TABLE agent_actions (
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    action_id UUID NOT NULL REFERENCES actions(id) ON DELETE CASCADE,
    is_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (agent_id, action_id)
);

CREATE INDEX idx_agent_actions_action_id ON agent_actions(action_id);

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

CREATE INDEX idx_project_agents_agent_id ON project_agents(agent_id);
CREATE INDEX idx_project_agents_order ON project_agents(project_id, order_index);

-- Agent hierarchy junction table (Many-to-many: parent agents ↔ sub-agents)
CREATE TABLE agent_hierarchy (
    parent_agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    sub_agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    order_index INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (parent_agent_id, sub_agent_id),
    CHECK (parent_agent_id != sub_agent_id)  -- Prevent self-reference
);

CREATE INDEX idx_agent_hierarchy_sub_agent ON agent_hierarchy(sub_agent_id);
CREATE INDEX idx_agent_hierarchy_order ON agent_hierarchy(parent_agent_id, order_index);

-- Agent conversation starters table
CREATE TABLE agent_conversation_starters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    order_index INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_agent_conv_starters_agent ON agent_conversation_starters(agent_id, order_index);

-- Assistant conversation starters table
CREATE TABLE assistant_conversation_starters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assistant_id UUID NOT NULL REFERENCES assistants(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    order_index INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_assistant_conv_starters ON assistant_conversation_starters(assistant_id, order_index);

-- ==========================================
-- TOOL CALLS & EXECUTION
-- ==========================================

-- Tool calls table (Function/Tool execution logs)
CREATE TABLE tool_calls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    tool_id UUID REFERENCES tools(id) ON DELETE SET NULL,  -- references tools table
    
    -- Tool info
    tool_call_id TEXT NOT NULL,  -- provider's tool call ID
    tool_name TEXT NOT NULL,  -- function name / tool name
    tool_type TEXT DEFAULT 'function' CHECK (tool_type IN ('function', 'code_interpreter', 'retrieval', 'web_search')),
    
    -- Execution
    arguments JSONB,  -- serialized function arguments (JSON)
    result JSONB,  -- serialized function result (JSON)
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    error_message TEXT,
    
    -- Output files
    output_file_ids UUID[],
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_tool_calls_message_id ON tool_calls(message_id);
CREATE INDEX idx_tool_calls_tool_id ON tool_calls(tool_id) WHERE tool_id IS NOT NULL;
CREATE INDEX idx_tool_calls_status ON tool_calls(status, created_at) WHERE status != 'completed';

-- Tool call output files junction table
CREATE TABLE tool_call_output_files (
    tool_call_id UUID NOT NULL REFERENCES tool_calls(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tool_call_id, file_id)
);

CREATE INDEX idx_tool_call_output_files_file_id ON tool_call_output_files(file_id);

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
    
    -- Model info
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
    currency TEXT DEFAULT 'USD' CHECK (currency IN ('USD', 'EUR', 'GBP', 'JPY', 'CNY')),
    
    -- Context
    transaction_type TEXT DEFAULT 'completion' CHECK (transaction_type IN ('completion', 'embedding', 'image', 'tts', 'stt')),
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_transactions_user_id ON transactions(user_id, created_at DESC);
CREATE INDEX idx_transactions_message_id ON transactions(message_id) WHERE message_id IS NOT NULL;
CREATE INDEX idx_transactions_conversation_id ON transactions(conversation_id) WHERE conversation_id IS NOT NULL;
CREATE INDEX idx_transactions_created_at ON transactions(created_at DESC);
CREATE INDEX idx_transactions_type ON transactions(transaction_type);

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

CREATE INDEX idx_shared_links_user_id ON shared_links(user_id);
CREATE INDEX idx_shared_links_conversation_id ON shared_links(conversation_id);
CREATE INDEX idx_shared_links_expires ON shared_links(expires_at) WHERE expires_at IS NOT NULL;

-- ==========================================
-- SEED DATA: AI PROVIDERS AND MODELS
-- ==========================================

-- First, insert AI providers
INSERT INTO ai_providers (
    id, provider_id, display_name, description,
    base_url, website_url, documentation_url,
    disabled, is_active, requires_api_key,
    supports_streaming, supports_images, supports_functions, supports_vision
) VALUES
-- OpenAI
('a0000000-0000-0000-0000-000000000001'::UUID, 'openai', 'OpenAI', 'Leading AI research company with GPT models',
    'https://api.openai.com/v1', 'https://openai.com', 'https://platform.openai.com/docs',
    false, true, true, true, true, true, true),
-- Anthropic
('a0000000-0000-0000-0000-000000000002'::UUID, 'anthropic', 'Anthropic', 'AI safety company behind Claude models',
    'https://api.anthropic.com', 'https://anthropic.com', 'https://docs.anthropic.com',
    false, true, true, true, true, true, true),
-- Google
('a0000000-0000-0000-0000-000000000003'::UUID, 'google', 'Google AI', 'Google''s AI platform with Gemini models',
    'https://generativelanguage.googleapis.com/v1', 'https://ai.google.dev', 'https://ai.google.dev/docs',
    false, true, true, true, true, true, true),
-- Meta
('a0000000-0000-0000-0000-000000000004'::UUID, 'meta', 'Meta AI', 'Meta''s open-source Llama models',
    NULL, 'https://ai.meta.com', 'https://llama.meta.com/docs',
    false, true, false, true, false, true, false),
-- DeepSeek
('a0000000-0000-0000-0000-000000000005'::UUID, 'deepseek', 'DeepSeek', 'AI research company focused on reasoning and coding',
    'https://api.deepseek.com', 'https://deepseek.com', 'https://platform.deepseek.com/docs',
    false, true, true, true, false, true, false),
-- Mistral AI
('a0000000-0000-0000-0000-000000000006'::UUID, 'mistral', 'Mistral AI', 'European AI company with efficient open models',
    'https://api.mistral.ai/v1', 'https://mistral.ai', 'https://docs.mistral.ai',
    false, true, true, true, false, true, false),
-- xAI
('a0000000-0000-0000-0000-000000000007'::UUID, 'xai', 'xAI', 'AI company by Elon Musk with Grok models',
    'https://api.x.ai/v1', 'https://x.ai', 'https://docs.x.ai',
    false, true, true, true, true, true, true),
-- Cohere
('a0000000-0000-0000-0000-000000000008'::UUID, 'cohere', 'Cohere', 'Enterprise AI platform with Command models',
    'https://api.cohere.ai/v1', 'https://cohere.com', 'https://docs.cohere.com',
    false, true, true, true, false, true, false),
-- Alibaba
('a0000000-0000-0000-0000-000000000009'::UUID, 'alibaba', 'Alibaba Cloud', 'Alibaba''s AI platform with Qwen models',
    'https://dashscope.aliyuncs.com/api/v1', 'https://www.alibabacloud.com', 'https://help.aliyun.com/zh/dashscope',
    false, true, true, true, false, true, false),
-- ChatLLM
('a0000000-0000-0000-0000-000000000010'::UUID, 'chatllm', 'ChatLLM', 'ChatLLM by Abacus.AI',
    'https://pa002.abacus.ai/api', 'https://abacus.ai/chatllm', 'https://abacus.ai/help/howTo/chatllm/',
    false, true, true, true, false, true, false);

-- Now insert AI models linked to providers
INSERT INTO ai_models (
    id, provider_id, model_id, display_name, description, 
    context_window, max_output_tokens, supports_streaming, supports_images, 
    supports_functions, supports_vision, cost_per_input_token, cost_per_output_token, 
    is_active, created_at, updated_at
) VALUES
-- OpenAI GPT-4o Series
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000001'::UUID, 'gpt-4o', 'GPT-4o', 'Latest GPT-4 with optimized performance and multimodal capabilities', 128000, 16384, true, true, true, true, 2.50, 10.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000001'::UUID, 'gpt-4o-mini', 'GPT-4o Mini', 'Smaller, faster, and more affordable version of GPT-4o', 128000, 16384, true, true, true, true, 0.15, 0.60, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000001'::UUID, 'gpt-4-turbo', 'GPT-4 Turbo', 'Most capable GPT-4 model with vision capabilities', 128000, 4096, true, true, true, true, 10.00, 30.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000001'::UUID, 'gpt-4', 'GPT-4', 'Standard GPT-4 model', 8192, 4096, true, false, true, false, 30.00, 60.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000001'::UUID, 'gpt-3.5-turbo', 'GPT-3.5 Turbo', 'Fast and affordable GPT-3.5 model', 16385, 4096, true, false, true, false, 0.50, 1.50, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000001'::UUID, 'o1', 'O1', 'Advanced reasoning model optimized for complex tasks', 200000, 100000, true, false, true, false, 15.00, 60.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000001'::UUID, 'o1-mini', 'O1 Mini', 'Compact reasoning model for faster performance', 128000, 65536, true, false, true, false, 3.00, 12.00, true, NOW(), NOW()),

-- Anthropic Claude Models
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000002'::UUID, 'claude-3.5-sonnet', 'Claude 3.5 Sonnet', 'Latest Claude model with enhanced intelligence and speed', 200000, 8192, true, true, true, true, 3.00, 15.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000002'::UUID, 'claude-3-opus', 'Claude 3 Opus', 'Most capable Claude model for complex tasks', 200000, 4096, true, true, true, true, 15.00, 75.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000002'::UUID, 'claude-3-sonnet', 'Claude 3 Sonnet', 'Balanced performance and speed', 200000, 4096, true, true, true, true, 3.00, 15.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000002'::UUID, 'claude-3-haiku', 'Claude 3 Haiku', 'Fastest and most compact Claude model', 200000, 4096, true, true, true, true, 0.25, 1.25, true, NOW(), NOW()),

-- Google Gemini Models
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000003'::UUID, 'gemini-2.0-flash', 'Gemini 2.0 Flash', 'Next-generation multimodal model with breakthrough speed', 1000000, 8192, true, true, true, true, 0.10, 0.40, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000003'::UUID, 'gemini-1.5-pro', 'Gemini 1.5 Pro', 'Advanced multimodal model with 1M token context', 1000000, 8192, true, true, true, true, 1.25, 5.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000003'::UUID, 'gemini-1.5-flash', 'Gemini 1.5 Flash', 'Optimized for speed and efficiency', 1000000, 8192, true, true, true, true, 0.075, 0.30, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000003'::UUID, 'gemini-pro', 'Gemini Pro', 'Standard Gemini model for general use', 32760, 8192, true, true, true, true, 0.50, 1.50, true, NOW(), NOW()),

-- Meta Llama Models
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000004'::UUID, 'llama-3.3-70b', 'Llama 3.3 70B', 'Latest Llama model with 70B parameters', 128000, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000004'::UUID, 'llama-3.1-405b', 'Llama 3.1 405B', 'Largest Llama model with 405B parameters', 128000, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000004'::UUID, 'llama-3.1-70b', 'Llama 3.1 70B', 'Llama 3.1 with 70B parameters', 128000, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000004'::UUID, 'llama-3.1-8b', 'Llama 3.1 8B', 'Compact Llama model with 8B parameters', 128000, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),

-- DeepSeek Models
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000005'::UUID, 'deepseek-v3', 'DeepSeek V3', 'Advanced reasoning and coding model', 64000, 8192, true, false, true, false, 0.14, 0.28, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000005'::UUID, 'deepseek-chat', 'DeepSeek Chat', 'General purpose chat model', 32000, 4096, true, false, true, false, 0.14, 0.28, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000005'::UUID, 'deepseek-coder', 'DeepSeek Coder', 'Specialized coding model', 16000, 4096, true, false, true, false, 0.14, 0.28, true, NOW(), NOW()),

-- Mistral Models
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000006'::UUID, 'mistral-large', 'Mistral Large', 'Most capable Mistral model', 128000, 8192, true, false, true, false, 2.00, 6.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000006'::UUID, 'mistral-medium', 'Mistral Medium', 'Balanced performance model', 32000, 8192, true, false, true, false, 2.70, 8.10, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000006'::UUID, 'mistral-small', 'Mistral Small', 'Fast and efficient model', 32000, 8192, true, false, true, false, 0.20, 0.60, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000006'::UUID, 'codestral', 'Codestral', 'Specialized code generation model', 32000, 8192, true, false, true, false, 0.20, 0.60, true, NOW(), NOW()),

-- xAI Grok Models
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000007'::UUID, 'grok-2', 'Grok 2', 'Latest Grok model with enhanced capabilities', 131072, 8192, true, true, true, true, 2.00, 10.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000007'::UUID, 'grok-2-mini', 'Grok 2 Mini', 'Compact version of Grok 2', 131072, 8192, true, false, true, false, 0.50, 2.50, true, NOW(), NOW()),

-- Cohere Models
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000008'::UUID, 'command-r-plus', 'Command R+', 'Enhanced retrieval and generation model', 128000, 4096, true, false, true, false, 2.50, 10.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000008'::UUID, 'command-r', 'Command R', 'Retrieval-augmented generation model', 128000, 4096, true, false, true, false, 0.15, 0.60, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000008'::UUID, 'command', 'Command', 'General purpose instruction following model', 4096, 4096, true, false, true, false, 1.00, 2.00, true, NOW(), NOW()),

-- Alibaba Qwen Models
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000009'::UUID, 'qwen-2.5-72b', 'Qwen 2.5 72B', 'Advanced language model with 72B parameters', 32768, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000009'::UUID, 'qwen-2.5-32b', 'Qwen 2.5 32B', 'Efficient model with 32B parameters', 32768, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000009'::UUID, 'qwen-2.5-14b', 'Qwen 2.5 14B', 'Compact model with 14B parameters', 32768, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),

-- ChatLLM Models
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000010'::UUID, 'gpt-4o', 'ChatLLM GPT-4o', 'GPT-4o via ChatLLM', 128000, 4096, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000010'::UUID, 'claude-3-5-sonnet-20241022', 'ChatLLM Claude 3.5 Sonnet', 'Claude 3.5 Sonnet via ChatLLM', 200000, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000010'::UUID, 'gemini-1-5-pro', 'ChatLLM Gemini 1.5 Pro', 'Gemini 1.5 Pro via ChatLLM', 1000000, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000010'::UUID, 'lib-chatgpt-4o', 'ChatLLM Liberty GPT-4o', 'Liberty GPT-4o via ChatLLM', 128000, 4096, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000010'::UUID, 'x-ai-grok-2', 'ChatLLM Grok 2', 'Grok 2 via ChatLLM', 131072, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000010'::UUID, 'deepseek-v3', 'ChatLLM DeepSeek V3', 'DeepSeek V3 via ChatLLM', 64000, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'a0000000-0000-0000-0000-000000000010'::UUID, 'o1', 'ChatLLM O1', 'O1 via ChatLLM', 200000, 100000, true, false, true, false, 0.00, 0.00, true, NOW(), NOW());

-- ==========================================
-- SEED DATA: SYSTEM ADMIN USER
-- ==========================================

-- Insert default system administrator user
-- This user cannot be deleted or disabled (enforced by database triggers)
-- Password: P@$$w0rd (bcrypt hash below)
-- Can be updated: email, username, password_hash, name, avatar_url, and other non-security-critical fields
INSERT INTO users (
    id,
    email,
    email_verified,
    name,
    username,
    provider,
    normalized_email,
    normalized_username,
    password_hash,
    is_system,
    terms_accepted,
    terms_accepted_at,
    created_at,
    updated_at
) VALUES (
    'system-admin-00000000',
    'admin@localhost',
    true,
    'System Administrator',
    'admin',
    'local',
    'ADMIN@LOCALHOST',
    'ADMIN',
    -- Bcrypt hash for: P@$$w0rd
    -- Generated with cost factor 12
    -- To change password, generate new bcrypt hash and update this field
    '$2b$12$DsM/lf1mhgWuvmZvBVNJz.C/Kg9FpizEJ7hzi3mYN81YJfOYH9rEC',
    true,  -- is_system = true (cannot be deleted or disabled)
    true,
    NOW(),
    NOW(),
    NOW()
);

-- Assign Administrator role to system admin user
INSERT INTO user_roles (user_id, role_id, assigned_at, assigned_by)
SELECT 
    'system-admin-00000000',
    id,
    NOW(),
    'system-admin-00000000'  -- self-assigned during initialization
FROM roles
WHERE normalized_name = 'ADMINISTRATOR';

