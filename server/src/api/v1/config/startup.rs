use crate::AppState;
use crate::config::t3chat::ModelSpecConfig;
use axum::{extract::State, response::Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StartupConfigResponse {
    pub app_title: Option<String>,
    pub interface: InterfaceConfigResponse,
    pub providers: Vec<ProviderSummaryResponse>,
    pub model_specs: Vec<ModelSpecResponse>,
    #[serde(default)]
    pub notices: Vec<StartupNotice>,
}

#[derive(Debug, Serialize)]
pub struct InterfaceConfigResponse {
    pub model_select_enabled: bool,
    pub default_model_spec: Option<String>,
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProviderSummaryResponse {
    pub key: String,
    pub label: String,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelSpecResponse {
    pub name: String,
    pub label: String,
    pub provider: String,
    pub model: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub default: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct StartupNotice {
    pub kind: StartupNoticeKind,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupNoticeKind {
    Info,
    Warning,
}

pub async fn get_startup_config(state: State<AppState>) -> Json<StartupConfigResponse> {
    let config = state.t3_config.clone();
    let metadata = state.config_metadata.clone();
    let providers = config
        .providers
        .resolved_entries()
        .into_iter()
        .filter(|(_, cfg)| cfg.enabled)
        .map(|(key, cfg)| {
            let label = cfg.label_or_key(&key).to_string();
            ProviderSummaryResponse {
                key,
                label,
                icon: cfg.icon.clone(),
            }
        })
        .collect();

    let specs = config
        .model_specs
        .iter()
        .map(ModelSpecResponse::from)
        .collect();

    let mut notices = Vec::new();

    if metadata.missing_file {
        notices.push(StartupNotice {
            kind: StartupNoticeKind::Warning,
            message: format!(
                "Configuration file t3chat.yaml was not found. Using built-in defaults with no managed providers. Check server logs for the expected config path."
            ),
        });
    }

    for warning in metadata.warnings.iter() {
        notices.push(StartupNotice {
            kind: StartupNoticeKind::Warning,
            message: warning.clone(),
        });
    }

    Json(StartupConfigResponse {
        app_title: config.app_title.clone(),
        interface: InterfaceConfigResponse {
            model_select_enabled: config.interface.model_select_enabled,
            default_model_spec: config.interface.default_model_spec.clone(),
            default_provider: config.interface.default_provider.clone(),
            default_model: config.interface.default_model.clone(),
        },
        providers,
        model_specs: specs,
        notices,
    })
}

impl From<&ModelSpecConfig> for ModelSpecResponse {
    fn from(spec: &ModelSpecConfig) -> Self {
        Self {
            name: spec.name.clone(),
            label: spec.label.clone(),
            provider: spec.provider.clone(),
            model: spec.model.clone(),
            description: spec.description.clone(),
            icon: spec.icon.clone(),
            default: spec.default,
            parameters: spec.parameters.clone(),
        }
    }
}
