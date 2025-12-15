use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LibreChatConfig {
    pub version: Option<String>,
    #[serde(default)]
    pub interface: Option<InterfaceConfig>,
    #[serde(default)]
    pub endpoints: Option<EndpointsConfig>,
    #[serde(rename = "modelSpecs", default)]
    pub model_specs: Option<Vec<ModelSpec>>,
    
    // Parse but ignore for now to allow full LibreChat configs
    #[serde(default)]
    pub registration: Option<Value>,
    #[serde(default)]
    pub actions: Option<Value>,
    #[serde(default)]
    pub cache: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceConfig {
    pub privacy_policy: Option<Value>,
    pub terms_of_service: Option<Value>,
    pub endpoints_menu: Option<bool>,
    pub model_select: Option<bool>,
    pub parameters_menu: Option<bool>,
    pub side_panel: Option<bool>,
    pub presets: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EndpointsConfig {
    #[serde(rename = "openAI", default)]
    pub openai: Option<OpenAIEndpoint>,
    #[serde(default)]
    pub anthropic: Option<AnthropicEndpoint>,
    #[serde(default)]
    pub google: Option<GoogleEndpoint>,
    
    // Parse but ignore specific ones we don't fully support yet but want to allow in config
    #[serde(rename = "azureOpenAI", default)]
    pub azure_openai: Option<Value>,
    #[serde(default)]
    pub bedrock: Option<Value>,
    #[serde(default)]
    pub assistants: Option<Value>,
    #[serde(default)]
    pub agents: Option<Value>,

    #[serde(default)]
    pub custom: Option<Vec<CustomEndpoint>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIEndpoint {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Option<EndpointModels>,
    pub title_model: Option<String>,
    pub summarize: Option<bool>,
    pub summary_model: Option<String>,
    pub force_prompt: Option<bool>,
    pub model_display_label: Option<String>,
    pub icon_u_r_l: Option<String>,
    
    // Pass-through for advanced config
    #[serde(default)]
    pub headers: Option<Value>,
    #[serde(default)]
    pub add_params: Option<Value>,
    #[serde(default)]
    pub drop_params: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnthropicEndpoint {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Option<EndpointModels>,
    pub model_display_label: Option<String>,
    pub icon_u_r_l: Option<String>,
    
    #[serde(default)]
    pub headers: Option<Value>,
    #[serde(default)]
    pub add_params: Option<Value>,
    #[serde(default)]
    pub drop_params: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleEndpoint {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Option<EndpointModels>,
    pub model_display_label: Option<String>,
    pub icon_u_r_l: Option<String>,
    
    #[serde(default)]
    pub headers: Option<Value>,
    #[serde(default)]
    pub add_params: Option<Value>,
    #[serde(default)]
    pub drop_params: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomEndpoint {
    pub name: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Option<EndpointModels>,
    pub model_display_label: Option<String>,
    pub icon_u_r_l: Option<String>,
    
    #[serde(default)]
    pub headers: Option<Value>,
    #[serde(default)]
    pub add_params: Option<Value>,
    #[serde(default)]
    pub drop_params: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EndpointModels {
    #[serde(default)]
    pub default: Option<Vec<String>>,
    #[serde(default)]
    pub fetch: Option<bool>,
    #[serde(default)]
    pub not_allowed: Option<Vec<String>>,
    #[serde(default)]
    pub all: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSpec {
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub preset: ModelSpecPreset,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSpecPreset {
    pub endpoint: String,
    pub model: String,
    pub model_label: Option<String>,
    pub greeting: Option<String>,
    pub prompt_prefix: Option<String>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub resend_files: Option<bool>,
    pub image_detail: Option<String>,
    pub tools: Option<bool>,
}
