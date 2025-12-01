-- Create user_features table for feature flags
CREATE TABLE user_features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    feature TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, feature)
);

-- Create index for faster lookups
CREATE INDEX idx_user_features_user_id ON user_features(user_id);
CREATE INDEX idx_user_features_feature ON user_features(feature);
