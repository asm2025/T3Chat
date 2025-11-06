use anyhow::Result;
use emix::{
    app::AppInfo,
    env::{get_database_url, get_env, get_required_env},
};
use once_cell::sync::Lazy;

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
