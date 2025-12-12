-- ==========================================
-- T3Chat Complete Initial Schema Rollback
-- ==========================================
-- This migration drops all tables in reverse order
-- to respect foreign key constraints
-- ==========================================

-- Drop tables in reverse order of creation

-- Sharing
DROP TABLE IF EXISTS shared_links CASCADE;

-- Billing & Tracking
DROP TABLE IF EXISTS transactions CASCADE;
DROP TABLE IF EXISTS balances CASCADE;

-- Tool Calls & Execution
DROP TABLE IF EXISTS tool_call_output_files CASCADE;
DROP TABLE IF EXISTS tool_calls CASCADE;

-- Agent & Assistant Relationships
DROP TABLE IF EXISTS assistant_conversation_starters CASCADE;
DROP TABLE IF EXISTS agent_conversation_starters CASCADE;
DROP TABLE IF EXISTS agent_hierarchy CASCADE;
DROP TABLE IF EXISTS project_agents CASCADE;

-- Tools & Actions System
DROP TABLE IF EXISTS agent_actions CASCADE;
DROP TABLE IF EXISTS action_auth_config CASCADE;
DROP TABLE IF EXISTS action_settings CASCADE;
DROP TABLE IF EXISTS actions CASCADE;
DROP TABLE IF EXISTS assistant_tool_config CASCADE;
DROP TABLE IF EXISTS assistant_tools CASCADE;
DROP TABLE IF EXISTS agent_tool_config CASCADE;
DROP TABLE IF EXISTS agent_tools CASCADE;
DROP TABLE IF EXISTS tool_config_schema CASCADE;
DROP TABLE IF EXISTS tools CASCADE;

-- Projects & Prompts
DROP TABLE IF EXISTS prompts CASCADE;
DROP TABLE IF EXISTS prompt_groups CASCADE;
DROP TABLE IF EXISTS projects CASCADE;

-- Presets
DROP TABLE IF EXISTS preset_feature_flags CASCADE;
DROP TABLE IF EXISTS preset_model_parameters CASCADE;
DROP TABLE IF EXISTS presets CASCADE;

-- Messages & Conversations
DROP TABLE IF EXISTS message_content_blocks CASCADE;
DROP TABLE IF EXISTS message_files CASCADE;
DROP TABLE IF EXISTS messages CASCADE;
DROP TABLE IF EXISTS conversation_tags_map CASCADE;
DROP TABLE IF EXISTS tags CASCADE;
DROP TABLE IF EXISTS conversation_files CASCADE;
DROP TABLE IF EXISTS conversation_feature_flags CASCADE;
DROP TABLE IF EXISTS conversation_model_parameters CASCADE;
DROP TABLE IF EXISTS assistant_files CASCADE;
DROP TABLE IF EXISTS files CASCADE;
DROP TABLE IF EXISTS conversations CASCADE;
DROP TABLE IF EXISTS assistants CASCADE;
DROP TABLE IF EXISTS agent_model_parameters CASCADE;
DROP TABLE IF EXISTS agents CASCADE;

-- API Keys & Models
DROP TABLE IF EXISTS user_api_keys CASCADE;
DROP TABLE IF EXISTS ai_models CASCADE;
DROP TABLE IF EXISTS ai_provider_metadata CASCADE;
DROP TABLE IF EXISTS ai_providers CASCADE;

-- User Features & Roles
DROP TABLE IF EXISTS user_features CASCADE;
DROP TABLE IF EXISTS user_preferences CASCADE;
DROP TABLE IF EXISTS user_claims CASCADE;
DROP TABLE IF EXISTS role_claims CASCADE;
DROP TABLE IF EXISTS user_roles CASCADE;
DROP TABLE IF EXISTS roles CASCADE;

-- Users
DROP TRIGGER IF EXISTS trg_prevent_system_user_disable ON users;
DROP FUNCTION IF EXISTS prevent_system_user_disable();
DROP TRIGGER IF EXISTS trg_prevent_system_user_deletion ON users;
DROP FUNCTION IF EXISTS prevent_system_user_deletion();
DROP TABLE IF EXISTS users CASCADE;

-- Diesel Migrations
DROP TABLE IF EXISTS __diesel_schema_migrations CASCADE;

-- Diesel setup rollback
DROP FUNCTION IF EXISTS diesel_manage_updated_at(_tbl regclass);
DROP FUNCTION IF EXISTS diesel_set_updated_at();

