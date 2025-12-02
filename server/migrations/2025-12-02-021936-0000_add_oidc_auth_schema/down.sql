-- ==========================================
-- OIDC Authentication Schema Rollback
-- ==========================================
-- This migration rolls back the OIDC authentication schema changes
-- ==========================================

-- Drop indexes for ai_models new fields
DROP INDEX IF EXISTS idx_ai_models_disabled;
DROP INDEX IF EXISTS idx_ai_models_provider_id;

-- Remove columns from ai_models table
ALTER TABLE ai_models DROP COLUMN IF EXISTS is_paid;
ALTER TABLE ai_models DROP COLUMN IF EXISTS disabled;
ALTER TABLE ai_models DROP COLUMN IF EXISTS provider_id;

-- Drop ai_providers table
DROP INDEX IF EXISTS idx_ai_providers_is_active;
DROP INDEX IF EXISTS idx_ai_providers_disabled;
DROP INDEX IF EXISTS idx_ai_providers_provider_id;
DROP TABLE IF EXISTS ai_providers;

-- Drop user_roles table
DROP INDEX IF EXISTS idx_user_roles_role_name;
DROP INDEX IF EXISTS idx_user_roles_user_id;
DROP TABLE IF EXISTS user_roles;

-- Drop roles table
DROP TABLE IF EXISTS roles;

-- Drop indexes for users new fields
DROP INDEX IF EXISTS idx_users_provider;
DROP INDEX IF EXISTS idx_users_locked_out;
DROP INDEX IF EXISTS idx_users_disabled;
DROP INDEX IF EXISTS idx_users_normalized_email_unique;
DROP INDEX IF EXISTS idx_users_normalized_username;
DROP INDEX IF EXISTS idx_users_normalized_email;

-- Remove columns from users table
ALTER TABLE users DROP COLUMN IF EXISTS login_count;
ALTER TABLE users DROP COLUMN IF EXISTS last_login_at;
ALTER TABLE users DROP COLUMN IF EXISTS password_changed_at;
ALTER TABLE users DROP COLUMN IF EXISTS access_failed_count;
ALTER TABLE users DROP COLUMN IF EXISTS lockout_end;
ALTER TABLE users DROP COLUMN IF EXISTS locked_out;
ALTER TABLE users DROP COLUMN IF EXISTS disabled;
ALTER TABLE users DROP COLUMN IF EXISTS normalized_username;
ALTER TABLE users DROP COLUMN IF EXISTS normalized_email;

