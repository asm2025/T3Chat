-- ==========================================
-- LibreChat Schema Rollback
-- ==========================================
-- This migration drops all tables in reverse order
-- to respect foreign key constraints
-- ==========================================

-- Drop full-text search indexes
DROP INDEX IF EXISTS idx_actions_name_fts;
DROP INDEX IF EXISTS idx_tools_name_fts;
DROP INDEX IF EXISTS idx_files_filename_fts;
DROP INDEX IF EXISTS idx_agents_description_fts;
DROP INDEX IF EXISTS idx_agents_name_fts;
DROP INDEX IF EXISTS idx_messages_text_fts;
DROP INDEX IF EXISTS idx_conversations_title_fts;

-- Drop performance indexes
DROP INDEX IF EXISTS idx_shared_links_expires;
DROP INDEX IF EXISTS idx_shared_links_conversation_id;
DROP INDEX IF EXISTS idx_shared_links_user_id;
DROP INDEX IF EXISTS idx_transactions_created_at;
DROP INDEX IF EXISTS idx_transactions_provider_model;
DROP INDEX IF EXISTS idx_transactions_conversation_id;
DROP INDEX IF EXISTS idx_transactions_message_id;
DROP INDEX IF EXISTS idx_transactions_user_id;
DROP INDEX IF EXISTS idx_tool_calls_status;
DROP INDEX IF EXISTS idx_tool_calls_message_id;
DROP INDEX IF EXISTS idx_actions_type;
DROP INDEX IF EXISTS idx_actions_user_id;
DROP INDEX IF EXISTS idx_prompts_group_id;
DROP INDEX IF EXISTS idx_prompt_groups_project_id;
DROP INDEX IF EXISTS idx_prompt_groups_author_id;
DROP INDEX IF EXISTS idx_projects_owner_id;
DROP INDEX IF EXISTS idx_assistants_file_ids;
DROP INDEX IF EXISTS idx_assistants_user_id;
DROP INDEX IF EXISTS idx_assistant_conv_starters;
DROP INDEX IF EXISTS idx_agent_conv_starters_agent;
DROP INDEX IF EXISTS idx_agent_hierarchy_order;
DROP INDEX IF EXISTS idx_agent_hierarchy_sub_agent;
DROP INDEX IF EXISTS idx_project_agents_order;
DROP INDEX IF EXISTS idx_project_agents_agent_id;
DROP INDEX IF EXISTS idx_agent_actions_action_id;
DROP INDEX IF EXISTS idx_assistant_tools_tool_id;
DROP INDEX IF EXISTS idx_agent_tools_tool_id;
DROP INDEX IF EXISTS idx_tools_active;
DROP INDEX IF EXISTS idx_tools_type;
DROP INDEX IF EXISTS idx_agents_access_level;
DROP INDEX IF EXISTS idx_agents_author_id;
DROP INDEX IF EXISTS idx_presets_default;
DROP INDEX IF EXISTS idx_presets_user_id;
DROP INDEX IF EXISTS idx_files_temporary;
DROP INDEX IF EXISTS idx_files_conversation_id;
DROP INDEX IF EXISTS idx_files_user_id;
DROP INDEX IF EXISTS idx_messages_model;
DROP INDEX IF EXISTS idx_messages_file_ids;
DROP INDEX IF EXISTS idx_messages_parent;
DROP INDEX IF EXISTS idx_messages_conversation_id;
DROP INDEX IF EXISTS idx_conversation_tags_map_tag_id;
DROP INDEX IF EXISTS idx_tags_user_id;
DROP INDEX IF EXISTS idx_conversations_assistant_id;
DROP INDEX IF EXISTS idx_conversations_agent_id;
DROP INDEX IF EXISTS idx_conversations_user_archived;
DROP INDEX IF EXISTS idx_conversations_user_updated;
DROP INDEX IF EXISTS idx_conversations_user_id;
DROP INDEX IF EXISTS idx_user_api_keys_default;
DROP INDEX IF EXISTS idx_user_api_keys_user_provider;
DROP INDEX IF EXISTS idx_ai_models_active;
DROP INDEX IF EXISTS idx_ai_models_provider_model;

-- Drop tables in reverse order of dependencies

-- Sharing
DROP TABLE IF EXISTS shared_links;

-- Billing & Tracking
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS balances;

-- Tool calls
DROP TABLE IF EXISTS tool_calls;

-- Conversation starters
DROP TABLE IF EXISTS assistant_conversation_starters;
DROP TABLE IF EXISTS agent_conversation_starters;

-- Agent relationships
DROP TABLE IF EXISTS agent_hierarchy;
DROP TABLE IF EXISTS project_agents;

-- Actions & Tools
DROP TABLE IF EXISTS agent_actions;
DROP TABLE IF EXISTS actions;
DROP TABLE IF EXISTS assistant_tools;
DROP TABLE IF EXISTS agent_tools;
DROP TABLE IF EXISTS tools;

-- Tags
DROP TABLE IF EXISTS conversation_tags_map;
DROP TABLE IF EXISTS tags;

-- Prompts
DROP TABLE IF EXISTS prompts;
DROP TABLE IF EXISTS prompt_groups;
DROP TABLE IF EXISTS projects;

-- Presets
DROP TABLE IF EXISTS presets;

-- Messages & Conversations
DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS files;
DROP TABLE IF EXISTS conversations;

-- Agents & Assistants
DROP TABLE IF EXISTS assistants;
DROP TABLE IF EXISTS agents;

-- Core tables
DROP TABLE IF EXISTS user_api_keys;
DROP TABLE IF EXISTS ai_models;
DROP TABLE IF EXISTS users;


