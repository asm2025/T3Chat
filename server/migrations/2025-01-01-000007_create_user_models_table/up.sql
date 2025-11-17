CREATE TABLE user_models (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    model_id UUID NOT NULL REFERENCES ai_models(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, model_id)
);

CREATE INDEX idx_user_models_user_id ON user_models(user_id);
CREATE INDEX idx_user_models_model_id ON user_models(model_id);
CREATE INDEX idx_user_models_enabled ON user_models(enabled);

