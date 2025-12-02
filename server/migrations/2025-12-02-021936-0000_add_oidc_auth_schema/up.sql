-- ==========================================
-- OIDC Authentication Schema Migration
-- ==========================================
-- This migration adds ASP.NET Identity style fields to users table,
-- creates roles and user_roles tables, and adds ai_providers table
-- as specified in plan.dev.md
-- ==========================================

-- ==========================================
-- ENHANCE USERS TABLE
-- ==========================================

-- Add normalized email field (for case-insensitive lookups)
-- First add as nullable, populate data, then make NOT NULL
ALTER TABLE users ADD COLUMN IF NOT EXISTS normalized_email TEXT;

-- Add normalized username field (for case-insensitive lookups)
ALTER TABLE users ADD COLUMN IF NOT EXISTS normalized_username TEXT;

-- Add account status fields
ALTER TABLE users ADD COLUMN IF NOT EXISTS disabled BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS locked_out BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS lockout_end TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS access_failed_count INTEGER NOT NULL DEFAULT 0;

-- Add password change tracking
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_changed_at TIMESTAMPTZ;

-- Add login tracking
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS login_count INTEGER NOT NULL DEFAULT 0;

-- Create indexes for account status fields
CREATE INDEX IF NOT EXISTS idx_users_disabled ON users(disabled) WHERE disabled = true;
CREATE INDEX IF NOT EXISTS idx_users_locked_out ON users(locked_out) WHERE locked_out = true;
CREATE INDEX IF NOT EXISTS idx_users_provider ON users(provider);

-- Populate normalized_email for existing users
UPDATE users SET normalized_email = LOWER(email) WHERE normalized_email IS NULL;

-- Populate normalized_username for existing users
UPDATE users SET normalized_username = LOWER(username) WHERE normalized_username IS NULL AND username IS NOT NULL;

-- Make normalized_email NOT NULL and add unique constraint
-- Since email is already NOT NULL and UNIQUE, normalized_email should be too
ALTER TABLE users ALTER COLUMN normalized_email SET NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_normalized_email_unique ON users(normalized_email);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_users_normalized_email ON users(normalized_email);
CREATE INDEX IF NOT EXISTS idx_users_normalized_username ON users(normalized_username) WHERE normalized_username IS NOT NULL;

-- ==========================================
-- ROLES TABLE
-- ==========================================

CREATE TABLE IF NOT EXISTS roles (
    name TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default roles
INSERT INTO roles (name, display_name, description) VALUES
    ('user', 'User', 'Standard user role'),
    ('admin', 'Administrator', 'Full system access'),
    ('moderator', 'Moderator', 'Content moderation access')
ON CONFLICT (name) DO NOTHING;

-- ==========================================
-- USER ROLES TABLE
-- ==========================================

CREATE TABLE IF NOT EXISTS user_roles (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_name TEXT NOT NULL REFERENCES roles(name) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by TEXT REFERENCES users(id),
    PRIMARY KEY (user_id, role_name)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_name ON user_roles(role_name);

-- Migrate existing role data from users.role to user_roles table
INSERT INTO user_roles (user_id, role_name)
SELECT id, role
FROM users
WHERE role IS NOT NULL
ON CONFLICT (user_id, role_name) DO NOTHING;

-- ==========================================
-- AI PROVIDERS TABLE
-- ==========================================

CREATE TABLE IF NOT EXISTS ai_providers (
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
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_providers_provider_id ON ai_providers(provider_id);
CREATE INDEX IF NOT EXISTS idx_ai_providers_disabled ON ai_providers(disabled) WHERE disabled = true;
CREATE INDEX IF NOT EXISTS idx_ai_providers_is_active ON ai_providers(is_active) WHERE is_active = true;

-- ==========================================
-- UPDATE AI MODELS TABLE
-- ==========================================

-- Add provider_id foreign key column
ALTER TABLE ai_models ADD COLUMN IF NOT EXISTS provider_id UUID REFERENCES ai_providers(id) ON DELETE CASCADE;

-- Add disabled and is_paid fields
ALTER TABLE ai_models ADD COLUMN IF NOT EXISTS disabled BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE ai_models ADD COLUMN IF NOT EXISTS is_paid BOOLEAN NOT NULL DEFAULT true;

-- Create indexes for new fields
CREATE INDEX IF NOT EXISTS idx_ai_models_provider_id ON ai_models(provider_id) WHERE provider_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_models_disabled ON ai_models(disabled) WHERE disabled = true;

-- Note: Existing ai_models will have provider_id = NULL
-- This will need to be populated by creating ai_providers and updating ai_models
-- This is handled separately as it requires data migration logic

