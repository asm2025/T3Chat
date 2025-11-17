CREATE TABLE ai_models (
    id UUID PRIMARY KEY NOT NULL,
    provider TEXT NOT NULL,
    model_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT,
    context_window INTEGER NOT NULL,
    supports_streaming BOOLEAN NOT NULL DEFAULT false,
    supports_images BOOLEAN NOT NULL DEFAULT false,
    supports_functions BOOLEAN NOT NULL DEFAULT false,
    cost_per_token NUMERIC,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX idx_ai_models_provider_model_id ON ai_models(provider, model_id);
CREATE INDEX idx_ai_models_is_active ON ai_models(is_active);






























