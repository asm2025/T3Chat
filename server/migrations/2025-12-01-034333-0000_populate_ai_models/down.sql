-- Remove all AI models that were added by this migration
-- This will delete all models from the specified providers
DELETE FROM ai_models WHERE provider IN (
    'openai',
    'anthropic',
    'google',
    'meta',
    'deepseek',
    'mistral',
    'xai',
    'cohere',
    'alibaba'
);
