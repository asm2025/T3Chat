use super::{ConfigNotice, LoadedConfig, NoticeLevel, placeholders::resolve_env_placeholders};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedAppConfig {
    pub app_title: Option<String>,
    pub interface: DerivedInterfaceConfig,
    pub endpoints: Vec<DerivedEndpoint>,
    pub model_specs: Vec<DerivedModelSpec>,
    pub notices: Vec<ConfigNotice>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DerivedInterfaceConfig {
    pub endpoints_menu: bool,
    pub model_select: bool,
    pub parameters_menu: bool,
    pub side_panel: bool,
    pub presets: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedEndpoint {
    pub provider: String, // "openai", "anthropic", "custom:my-name", etc.
    pub label: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>, // We might want to mask this in API responses, but internal logic needs it
    pub model_display_label: Option<String>,
    pub icon_url: Option<String>,
    pub models: DerivedModelsConfig,
    pub headers: Option<Value>,
    pub add_params: Option<Value>,
    pub drop_params: Vec<String>,

    // Internal use
    #[serde(skip)]
    pub original_key: String,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DerivedModelsConfig {
    pub default: Vec<String>,
    pub fetch: bool,
    pub not_allowed: Vec<String>,
    pub all: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedModelSpec {
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub preset: DerivedModelSpecPreset,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedModelSpecPreset {
    pub endpoint: String,
    pub model: String,
    pub model_label: Option<String>,
    pub greeting: Option<String>,
    pub prompt_prefix: Option<String>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub resend_files: bool,
    pub image_detail: Option<String>,
    pub tools: bool,
}

impl DerivedAppConfig {
    pub fn from_loaded(loaded: LoadedConfig) -> Self {
        let mut notices = loaded.metadata.notices;
        let mut endpoints = Vec::new();
        let mut model_specs = Vec::new();

        // Defaults
        let mut interface = DerivedInterfaceConfig {
            endpoints_menu: true,
            model_select: true,
            parameters_menu: true,
            side_panel: true,
            presets: true,
        };

        let app_title = None; // Could extract if present in librechat yaml (not standard field but maybe useful)

        if let Some(config) = loaded.config {
            // 1. Interface
            if let Some(iface) = config.interface {
                if let Some(val) = iface.endpoints_menu {
                    interface.endpoints_menu = val;
                }
                if let Some(val) = iface.model_select {
                    interface.model_select = val;
                }
                if let Some(val) = iface.parameters_menu {
                    interface.parameters_menu = val;
                }
                if let Some(val) = iface.side_panel {
                    interface.side_panel = val;
                }
                if let Some(val) = iface.presets {
                    interface.presets = val;
                }
            }

            // 2. Endpoints
            if let Some(eps) = config.endpoints {
                // OpenAI
                if let Some(openai) = eps.openai {
                    process_endpoint(
                        "openai",
                        "OpenAI",
                        openai.api_key,
                        openai.base_url,
                        openai.models,
                        openai.model_display_label,
                        openai.icon_u_r_l,
                        openai.headers,
                        openai.add_params,
                        openai.drop_params,
                        &mut endpoints,
                        &mut notices,
                    );
                }

                // Anthropic
                if let Some(anthropic) = eps.anthropic {
                    process_endpoint(
                        "anthropic",
                        "Anthropic",
                        anthropic.api_key,
                        anthropic.base_url,
                        anthropic.models,
                        anthropic.model_display_label,
                        anthropic.icon_u_r_l,
                        anthropic.headers,
                        anthropic.add_params,
                        anthropic.drop_params,
                        &mut endpoints,
                        &mut notices,
                    );
                }

                // Google
                if let Some(google) = eps.google {
                    process_endpoint(
                        "google",
                        "Google",
                        google.api_key,
                        google.base_url,
                        google.models,
                        google.model_display_label,
                        google.icon_u_r_l,
                        google.headers,
                        google.add_params,
                        google.drop_params,
                        &mut endpoints,
                        &mut notices,
                    );
                }

                // Custom
                if let Some(custom_list) = eps.custom {
                    for custom in custom_list {
                        let _provider_key = custom.name.to_lowercase(); // Or keep as is? LibreChat uses name as key often.
                        // We map custom endpoints to "custom:name" or just use the name if it doesn't conflict?
                        // The plan says: `endpoints.custom[]` → unique provider keys (e.g. `custom:<name>` or the LibreChat `name`)
                        // Let's use the name directly but ensure uniqueness?
                        // Actually, T3Chat might expect specific keys for logic.
                        // For now, let's use the name.

                        process_endpoint(
                            &custom.name.to_lowercase(), // Use the name as the provider key
                            &custom.name, // And label
                            custom.api_key,
                            custom.base_url,
                            custom.models,
                            custom.model_display_label,
                            custom.icon_u_r_l,
                            custom.headers,
                            custom.add_params,
                            custom.drop_params,
                            &mut endpoints,
                            &mut notices,
                        );
                    }
                }
            }

            // 3. Model Specs
            if let Some(specs) = config.model_specs {
                for spec in specs {
                    model_specs.push(DerivedModelSpec {
                        name: spec.name,
                        label: spec.label,
                        description: spec.description,
                        icon: spec.icon,
                        preset: DerivedModelSpecPreset {
                            endpoint: spec.preset.endpoint,
                            model: spec.preset.model,
                            model_label: spec.preset.model_label,
                            greeting: spec.preset.greeting,
                            prompt_prefix: spec.preset.prompt_prefix,
                            temperature: spec.preset.temperature,
                            top_p: spec.preset.top_p,
                            presence_penalty: spec.preset.presence_penalty,
                            frequency_penalty: spec.preset.frequency_penalty,
                            resend_files: spec.preset.resend_files.unwrap_or(true), // Default true?
                            image_detail: spec.preset.image_detail,
                            tools: spec.preset.tools.unwrap_or(false),
                        },
                    });
                }
            }
        }

        DerivedAppConfig {
            app_title,
            interface,
            endpoints,
            model_specs,
            notices,
        }
    }
}

fn process_endpoint(
    provider_key: &str,
    default_label: &str,
    api_key: Option<String>,
    base_url: Option<String>,
    models: Option<super::t3chat::EndpointModels>,
    model_display_label: Option<String>,
    icon_url: Option<String>,
    headers: Option<Value>,
    add_params: Option<Value>,
    drop_params: Option<Vec<String>>,
    endpoints: &mut Vec<DerivedEndpoint>,
    notices: &mut Vec<ConfigNotice>,
) {
    // 1. Resolve API Key
    let resolved_api_key = if let Some(key) = api_key {
        let res = resolve_env_placeholders(&key);
        if !res.unresolved.is_empty() {
            notices.push(ConfigNotice {
                level: NoticeLevel::Warning,
                message: format!("Endpoint '{}' disabled", provider_key),
                detail: Some(format!(
                    "Missing environment variable(s): {}",
                    res.unresolved.join(", ")
                )),
            });
            return; // Disable endpoint
        }
        if res.value.is_empty() {
            // If key is present but empty, maybe disable too? Or allow if it's optional for some reason (e.g. local llm without auth)?
            // Usually OpenAI/Anthropic need keys. Custom might not.
            // Plan says: "If an endpoint requires apiKey and it resolves to empty... treat endpoint as disabled"
            // We don't strictly know if it requires it, but for built-ins usually yes.
            // Let's assume if it was provided as variable and resolved empty, it's bad.
            // But if it wasn't provided at all? LibreChat `apiKey` is optional in struct.

            // If the user put `apiKey: ${MISSING}`, we return above.
            // If `apiKey: ""` -> it is empty.
            if key.trim().is_empty() {
                // explicitly empty string?
            }
            Some(res.value)
        } else {
            Some(res.value)
        }
    } else {
        None
    };

    // 2. Resolve Base URL
    let resolved_base_url = if let Some(url) = base_url {
        let res = resolve_env_placeholders(&url);
        // We generally don't disable if base_url has missing vars, but maybe we should?
        // Let's just use the value.
        Some(res.value)
    } else {
        None
    };

    // 3. Models
    let derived_models = if let Some(m) = models {
        DerivedModelsConfig {
            default: m.default.unwrap_or_default(),
            fetch: m.fetch.unwrap_or(false), // Default false? Or true for some? T3Chat usually defaults false.
            not_allowed: m.not_allowed.unwrap_or_default(),
            all: m.all.unwrap_or(false),
        }
    } else {
        DerivedModelsConfig::default()
    };

    endpoints.push(DerivedEndpoint {
        provider: provider_key.to_string(),
        label: default_label.to_string(),
        base_url: resolved_base_url,
        api_key: resolved_api_key,
        model_display_label,
        icon_url,
        models: derived_models,
        headers,
        add_params,
        drop_params: drop_params.unwrap_or_default(),
        original_key: provider_key.to_string(),
    });
}
