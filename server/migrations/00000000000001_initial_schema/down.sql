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
DROP TABLE IF EXISTS tool_calls CASCADE;

-- Agent & Assistant Relationships
DROP TABLE IF EXISTS assistant_conversation_starters CASCADE;
DROP TABLE IF EXISTS agent_conversation_starters CASCADE;
DROP TABLE IF EXISTS agent_hierarchy CASCADE;
DROP TABLE IF EXISTS project_agents CASCADE;

-- Tools & Actions System
DROP TABLE IF EXISTS agent_actions CASCADE;
DROP TABLE IF EXISTS actions CASCADE;
DROP TABLE IF EXISTS assistant_tools CASCADE;
DROP TABLE IF EXISTS agent_tools CASCADE;
DROP TABLE IF EXISTS tools CASCADE;

-- Tags System
DROP TABLE IF EXISTS conversation_tags_map CASCADE;
DROP TABLE IF EXISTS tags CASCADE;

-- Projects & Prompts
DROP TABLE IF EXISTS prompts CASCADE;
DROP TABLE IF EXISTS prompt_groups CASCADE;
DROP TABLE IF EXISTS projects CASCADE;

-- Presets
DROP TABLE IF EXISTS presets CASCADE;

-- Messages & Conversations
DROP TABLE IF EXISTS messages CASCADE;
DROP TABLE IF EXISTS files CASCADE;
DROP TABLE IF EXISTS conversations CASCADE;
DROP TABLE IF EXISTS assistants CASCADE;
DROP TABLE IF EXISTS agents CASCADE;

-- API Keys & Models
DROP TABLE IF EXISTS user_api_keys CASCADE;
DROP TABLE IF EXISTS ai_models CASCADE;
DROP TABLE IF EXISTS ai_providers CASCADE;

-- User Features & Roles
DROP TABLE IF EXISTS user_features CASCADE;
DROP TABLE IF EXISTS user_roles CASCADE;
DROP TABLE IF EXISTS roles CASCADE;

-- Users
DROP TABLE IF EXISTS users CASCADE;

-- Diesel Migrations
DROP TABLE IF EXISTS __diesel_schema_migrations CASCADE;

