CREATE TABLE user_features (
    id UUID PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    feature TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_user_features_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT unique_user_feature UNIQUE (user_id, feature)
);

CREATE INDEX idx_user_features_user_id ON user_features(user_id);
CREATE INDEX idx_user_features_feature ON user_features(feature);
CREATE INDEX idx_user_features_enabled ON user_features(enabled);

