use anyhow::Result;
use dotenvy::from_filename;
use emix::{
    app::AppInfo,
    env::{get_database_url, get_env, get_required_env},
};
use once_cell::sync::Lazy;
use std::{path::Path, sync::LazyLock};
use tracing::warn;

const DEFAULT_APP_ENV: &str = "development";

static ENV_FILES_LOADED: LazyLock<()> = LazyLock::new(|| {
    load_env_files();
});

pub static APP_INFO: Lazy<AppInfo> = Lazy::new(|| {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");
    let description = env!("CARGO_PKG_DESCRIPTION");
    let license = env!("CARGO_PKG_LICENSE");
    AppInfo::new(
        name,
        version,
        authors,
        if !description.is_empty() {
            Some(description)
        } else {
            None
        },
        if !license.is_empty() {
            Some(license)
        } else {
            None
        },
    )
});

pub fn is_local_embedded_postgres() -> bool {
    if let Some(url) = get_database_url() {
        url.contains("localhost:") && url.contains("postgres:password")
    } else {
        false
    }
}

pub fn get_firebase_project_id() -> Result<String> {
    get_required_env("FIREBASE_PROJECT_ID").map_err(|e| e.into())
}

pub fn get_firebase_auth_emulator_host() -> Option<String> {
    get_env("FIREBASE_AUTH_EMULATOR_HOST")
}

pub fn app_env() -> String {
    get_required_env("APP_ENV")
        .and_then(|e| Ok(e.trim().to_lowercase()))
        .unwrap_or_else(|_| DEFAULT_APP_ENV.to_string())
}

pub fn is_development() -> bool {
    app_env() == "development"
}

pub fn is_staging() -> bool {
    app_env() == "staging"
}

pub fn is_swagger_enabled() -> bool {
    matches!(app_env().as_str(), "development" | "staging")
}

pub fn ensure_env_loaded() {
    LazyLock::force(&ENV_FILES_LOADED);
}

fn load_env_files() {
    let env = app_env();
    let candidates = [
        format!(".env.{env}.local"),
        format!(".env.{env}"),
        ".env.local".to_string(),
        ".env".to_string(),
    ];

    for candidate in candidates {
        if Path::new(&candidate).exists() {
            if let Err(err) = from_filename(&candidate) {
                warn!("Failed to load environment file {candidate}: {err}");
            }
        }
    }
}
