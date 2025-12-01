-- Populate AI models with latest models from major providers
-- OpenAI Models
INSERT INTO ai_models (
    id, provider, model_id, display_name, description, 
    context_window, max_output_tokens, supports_streaming, supports_images, 
    supports_functions, supports_vision, cost_per_input_token, cost_per_output_token, 
    is_active, created_at, updated_at
) VALUES
-- OpenAI GPT-4o Series
(gen_random_uuid(), 'openai', 'gpt-4o', 'GPT-4o', 'Latest GPT-4 with optimized performance and multimodal capabilities', 128000, 16384, true, true, true, true, 2.50, 10.00, true, NOW(), NOW()),
(gen_random_uuid(), 'openai', 'gpt-4o-mini', 'GPT-4o Mini', 'Smaller, faster, and more affordable version of GPT-4o', 128000, 16384, true, true, true, true, 0.15, 0.60, true, NOW(), NOW()),
(gen_random_uuid(), 'openai', 'gpt-4-turbo', 'GPT-4 Turbo', 'Most capable GPT-4 model with vision capabilities', 128000, 4096, true, true, true, true, 10.00, 30.00, true, NOW(), NOW()),
(gen_random_uuid(), 'openai', 'gpt-4', 'GPT-4', 'Standard GPT-4 model', 8192, 4096, true, false, true, false, 30.00, 60.00, true, NOW(), NOW()),
(gen_random_uuid(), 'openai', 'gpt-3.5-turbo', 'GPT-3.5 Turbo', 'Fast and affordable GPT-3.5 model', 16385, 4096, true, false, true, false, 0.50, 1.50, true, NOW(), NOW()),
(gen_random_uuid(), 'openai', 'o1', 'O1', 'Advanced reasoning model optimized for complex tasks', 200000, 100000, true, false, true, false, 15.00, 60.00, true, NOW(), NOW()),
(gen_random_uuid(), 'openai', 'o1-mini', 'O1 Mini', 'Compact reasoning model for faster performance', 128000, 65536, true, false, true, false, 3.00, 12.00, true, NOW(), NOW()),

-- Anthropic Claude Models
(gen_random_uuid(), 'anthropic', 'claude-3.5-sonnet', 'Claude 3.5 Sonnet', 'Latest Claude model with enhanced intelligence and speed', 200000, 8192, true, true, true, true, 3.00, 15.00, true, NOW(), NOW()),
(gen_random_uuid(), 'anthropic', 'claude-3-opus', 'Claude 3 Opus', 'Most capable Claude model for complex tasks', 200000, 4096, true, true, true, true, 15.00, 75.00, true, NOW(), NOW()),
(gen_random_uuid(), 'anthropic', 'claude-3-sonnet', 'Claude 3 Sonnet', 'Balanced performance and speed', 200000, 4096, true, true, true, true, 3.00, 15.00, true, NOW(), NOW()),
(gen_random_uuid(), 'anthropic', 'claude-3-haiku', 'Claude 3 Haiku', 'Fastest and most compact Claude model', 200000, 4096, true, true, true, true, 0.25, 1.25, true, NOW(), NOW()),

-- Google Gemini Models
(gen_random_uuid(), 'google', 'gemini-2.0-flash', 'Gemini 2.0 Flash', 'Next-generation multimodal model with breakthrough speed', 1000000, 8192, true, true, true, true, 0.10, 0.40, true, NOW(), NOW()),
(gen_random_uuid(), 'google', 'gemini-1.5-pro', 'Gemini 1.5 Pro', 'Advanced multimodal model with 1M token context', 1000000, 8192, true, true, true, true, 1.25, 5.00, true, NOW(), NOW()),
(gen_random_uuid(), 'google', 'gemini-1.5-flash', 'Gemini 1.5 Flash', 'Optimized for speed and efficiency', 1000000, 8192, true, true, true, true, 0.075, 0.30, true, NOW(), NOW()),
(gen_random_uuid(), 'google', 'gemini-pro', 'Gemini Pro', 'Standard Gemini model for general use', 32760, 8192, true, true, true, true, 0.50, 1.50, true, NOW(), NOW()),

-- Meta Llama Models
(gen_random_uuid(), 'meta', 'llama-3.3-70b', 'Llama 3.3 70B', 'Latest Llama model with 70B parameters', 128000, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'meta', 'llama-3.1-405b', 'Llama 3.1 405B', 'Largest Llama model with 405B parameters', 128000, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'meta', 'llama-3.1-70b', 'Llama 3.1 70B', 'Llama 3.1 with 70B parameters', 128000, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'meta', 'llama-3.1-8b', 'Llama 3.1 8B', 'Compact Llama model with 8B parameters', 128000, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),

-- DeepSeek Models
(gen_random_uuid(), 'deepseek', 'deepseek-v3', 'DeepSeek V3', 'Advanced reasoning and coding model', 64000, 8192, true, false, true, false, 0.14, 0.28, true, NOW(), NOW()),
(gen_random_uuid(), 'deepseek', 'deepseek-chat', 'DeepSeek Chat', 'General purpose chat model', 32000, 4096, true, false, true, false, 0.14, 0.28, true, NOW(), NOW()),
(gen_random_uuid(), 'deepseek', 'deepseek-coder', 'DeepSeek Coder', 'Specialized coding model', 16000, 4096, true, false, true, false, 0.14, 0.28, true, NOW(), NOW()),

-- Mistral Models
(gen_random_uuid(), 'mistral', 'mistral-large', 'Mistral Large', 'Most capable Mistral model', 128000, 8192, true, false, true, false, 2.00, 6.00, true, NOW(), NOW()),
(gen_random_uuid(), 'mistral', 'mistral-medium', 'Mistral Medium', 'Balanced performance model', 32000, 8192, true, false, true, false, 2.70, 8.10, true, NOW(), NOW()),
(gen_random_uuid(), 'mistral', 'mistral-small', 'Mistral Small', 'Fast and efficient model', 32000, 8192, true, false, true, false, 0.20, 0.60, true, NOW(), NOW()),
(gen_random_uuid(), 'mistral', 'codestral', 'Codestral', 'Specialized code generation model', 32000, 8192, true, false, true, false, 0.20, 0.60, true, NOW(), NOW()),

-- xAI Grok Models
(gen_random_uuid(), 'xai', 'grok-2', 'Grok 2', 'Latest Grok model with enhanced capabilities', 131072, 8192, true, true, true, true, 2.00, 10.00, true, NOW(), NOW()),
(gen_random_uuid(), 'xai', 'grok-2-mini', 'Grok 2 Mini', 'Compact version of Grok 2', 131072, 8192, true, false, true, false, 0.50, 2.50, true, NOW(), NOW()),

-- Cohere Models
(gen_random_uuid(), 'cohere', 'command-r-plus', 'Command R+', 'Enhanced retrieval and generation model', 128000, 4096, true, false, true, false, 2.50, 10.00, true, NOW(), NOW()),
(gen_random_uuid(), 'cohere', 'command-r', 'Command R', 'Retrieval-augmented generation model', 128000, 4096, true, false, true, false, 0.15, 0.60, true, NOW(), NOW()),
(gen_random_uuid(), 'cohere', 'command', 'Command', 'General purpose instruction following model', 4096, 4096, true, false, true, false, 1.00, 2.00, true, NOW(), NOW()),

-- Alibaba Qwen Models
(gen_random_uuid(), 'alibaba', 'qwen-2.5-72b', 'Qwen 2.5 72B', 'Advanced language model with 72B parameters', 32768, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'alibaba', 'qwen-2.5-32b', 'Qwen 2.5 32B', 'Efficient model with 32B parameters', 32768, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW()),
(gen_random_uuid(), 'alibaba', 'qwen-2.5-14b', 'Qwen 2.5 14B', 'Compact model with 14B parameters', 32768, 8192, true, false, true, false, 0.00, 0.00, true, NOW(), NOW());
