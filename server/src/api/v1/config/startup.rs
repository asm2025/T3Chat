use crate::AppState;
use crate::config::{ConfigNotice, app_config::{DerivedModelSpec, DerivedModelSpecPreset}};
use axum::{extract::State, response::Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupConfigResponse {
    pub app_title: Option<String>,
    pub interface: InterfaceConfigResponse,
    pub providers: Vec<ProviderSummaryResponse>,
    pub model_specs: Vec<ModelSpecResponse>,
    #[serde(default)]
    pub notices: Vec<ConfigNotice>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceConfigResponse {
    pub endpoints_menu: bool,
    pub model_select: bool,
    pub parameters_menu: bool,
    pub side_panel: bool,
    pub presets: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSummaryResponse {
    pub key: String,
    pub label: String,
    pub icon: Option<String>,
    pub model_display_label: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSpecResponse {
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub preset: DerivedModelSpecPreset,
}

pub async fn get_startup_config(state: State<AppState>) -> Json<StartupConfigResponse> {
    let config = state.app_config.clone();
    
    // We already have notices in app_config
    let notices = config.notices.clone();

    let providers = config
        .endpoints
        .iter()
        .map(|cfg| {
            ProviderSummaryResponse {
                key: cfg.provider.clone(),
                label: cfg.label.clone(),
                icon: cfg.icon_url.clone(),
                model_display_label: cfg.model_display_label.clone(),
            }
        })
        .collect();

    let specs = config
        .model_specs
        .iter()
        .map(ModelSpecResponse::from)
        .collect();

    Json(StartupConfigResponse {
        app_title: config.app_title.clone(),
        interface: InterfaceConfigResponse {
            endpoints_menu: config.interface.endpoints_menu,
            model_select: config.interface.model_select,
            parameters_menu: config.interface.parameters_menu,
            side_panel: config.interface.side_panel,
            presets: config.interface.presets,
        },
        providers,
        model_specs: specs,
        notices,
    })
}

impl From<&DerivedModelSpec> for ModelSpecResponse {
    fn from(spec: &DerivedModelSpec) -> Self {
        Self {
            name: spec.name.clone(),
            label: spec.label.clone(),
            description: spec.description.clone(),
            icon: spec.icon.clone(),
            preset: spec.preset.clone(),
        }
    }
}
