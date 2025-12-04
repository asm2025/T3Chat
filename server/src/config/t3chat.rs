use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize)]
pub struct T3ChatConfig {
    #[serde(default)]
    pub app_title: Option<String>,
    #[serde(default)]
    pub interface: InterfaceConfig,
    #[serde(default)]
    pub providers: ProvidersConfig,
    #[serde(default)]
    pub model_specs: Vec<ModelSpecConfig>,
}

impl T3ChatConfig {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let provider_keys = self.providers.provider_keys();

        if provider_keys.is_empty() {
            errors.push("At least one provider must be configured".to_string());
        }

        if let Some(default_spec) = &self.interface.default_model_spec {
            if !self
                .model_specs
                .iter()
                .any(|spec| &spec.name == default_spec)
            {
                errors.push(format!(
                    "interface.default_model_spec references unknown spec '{default_spec}'"
                ));
            }
        }

        if let Some(default_provider) = &self.interface.default_provider {
            if !provider_keys.contains(default_provider) {
                errors.push(format!(
                    "interface.default_provider '{default_provider}' is not defined"
                ));
            }
        }

        if let Some(default_model) = &self.interface.default_model {
            if let Some(provider) = &self.interface.default_provider {
                if !self
                    .providers
                    .resolved_entries()
                    .into_iter()
                    .any(|(key, cfg)| key == *provider && cfg.models.has_model(default_model))
                {
                    errors.push(format!(
                        "interface.default_model references '{default_model}' \
                         which is not available for provider '{provider}'"
                    ));
                }
            }
        }

        for spec in &self.model_specs {
            if spec.provider.trim().is_empty() {
                errors.push(format!(
                    "model spec '{}' must define a provider key",
                    spec.name
                ));
                continue;
            }

            if !provider_keys.contains(&spec.provider) {
                errors.push(format!(
                    "model spec '{}' references unknown provider '{}'",
                    spec.name, spec.provider
                ));
            }
        }

        for (key, cfg) in self.providers.resolved_entries() {
            if !cfg.enabled {
                continue;
            }

            if !cfg.models.fetch && cfg.models.default.is_empty() {
                errors.push(format!(
                    "provider '{key}' has models.fetch = false but no default models configured"
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct InterfaceConfig {
    #[serde(default = "default_true")]
    pub model_select_enabled: bool,
    pub default_model_spec: Option<String>,
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
}

impl Default for InterfaceConfig {
    fn default() -> Self {
        Self {
            model_select_enabled: true,
            default_model_spec: None,
            default_provider: None,
            default_model: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProvidersConfig {
    pub openai: Option<ProviderConfig>,
    pub anthropic: Option<ProviderConfig>,
    pub google: Option<ProviderConfig>,
    pub bedrock: Option<ProviderConfig>,
    pub azure: Option<ProviderConfig>,
    pub openrouter: Option<ProviderConfig>,
    pub routellm: Option<RouteLLMProviderConfig>,
    #[serde(default)]
    pub custom: Vec<CustomProviderConfig>,
}

impl ProvidersConfig {
    pub fn provider_keys(&self) -> HashSet<String> {
        self.resolved_entries()
            .into_iter()
            .filter_map(|(key, cfg)| cfg.enabled.then_some(key))
            .collect()
    }

    pub fn resolved_entries(&self) -> Vec<(String, ProviderConfig)> {
        let mut entries = Vec::new();

        if let Some(cfg) = &self.openai {
            entries.push(("openai".to_string(), cfg.clone()));
        }
        if let Some(cfg) = &self.anthropic {
            entries.push(("anthropic".to_string(), cfg.clone()));
        }
        if let Some(cfg) = &self.google {
            entries.push(("google".to_string(), cfg.clone()));
        }
        if let Some(cfg) = &self.bedrock {
            entries.push(("bedrock".to_string(), cfg.clone()));
        }
        if let Some(cfg) = &self.azure {
            entries.push(("azure".to_string(), cfg.clone()));
        }
        if let Some(cfg) = &self.openrouter {
            entries.push(("openrouter".to_string(), cfg.clone()));
        }
        if let Some(cfg) = &self.routellm {
            entries.push(("routellm".to_string(), cfg.effective_provider()));
        }
        for custom in &self.custom {
            entries.push((custom.key.clone(), custom.config.clone()));
        }

        entries
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub display_name: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    #[serde(default)]
    pub models: ProviderModelsConfig,
    pub icon: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub user_id_query: Option<bool>,
}

impl ProviderConfig {
    pub fn label_or_key<'a>(&'a self, fallback: &'a str) -> &'a str {
        self.display_name
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(fallback)
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProviderModelsConfig {
    #[serde(default)]
    pub default: Vec<ProviderModelEntry>,
    #[serde(default)]
    pub fetch: bool,
    #[serde(default)]
    pub user_id_query: Option<bool>,
}

impl ProviderModelsConfig {
    pub fn has_model(&self, model_id: &str) -> bool {
        self.default.iter().any(|entry| entry.id() == model_id)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ProviderModelEntry {
    Simple(String),
    Detailed(DetailedModelConfig),
}

impl ProviderModelEntry {
    pub fn as_detailed(&self) -> DetailedModelConfig {
        match self {
            ProviderModelEntry::Simple(id) => DetailedModelConfig::from_id(id),
            ProviderModelEntry::Detailed(detail) => detail.clone(),
        }
    }

    pub fn id(&self) -> String {
        match self {
            ProviderModelEntry::Simple(id) => id.clone(),
            ProviderModelEntry::Detailed(detail) => detail.id.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DetailedModelConfig {
    pub id: String,
    pub label: Option<String>,
    pub context_window: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub supports_streaming: Option<bool>,
    pub supports_images: Option<bool>,
    pub supports_functions: Option<bool>,
    pub supports_vision: Option<bool>,
    pub cost_per_input_token: Option<f64>,
    pub cost_per_output_token: Option<f64>,
}

impl DetailedModelConfig {
    pub fn from_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomProviderConfig {
    pub key: String,
    #[serde(flatten)]
    pub config: ProviderConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RouteLLMProviderConfig {
    #[serde(flatten)]
    pub provider: ProviderConfig,
    #[serde(rename = "routes", default)]
    pub routes: ProviderModelsConfig,
}

impl RouteLLMProviderConfig {
    pub fn effective_provider(&self) -> ProviderConfig {
        let mut config = self.provider.clone();
        if self.routes.fetch || !self.routes.default.is_empty() {
            config.models = self.routes.clone();
        }
        config
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelSpecConfig {
    pub name: String,
    pub label: String,
    #[serde(alias = "endpoint")]
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub parameters: Option<Value>,
    pub description: Option<String>,
    pub icon: Option<String>,
    #[serde(default)]
    pub default: bool,
}

const fn default_true() -> bool {
    true
}
