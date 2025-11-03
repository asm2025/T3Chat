use std::env;

pub fn get_env(key: &str) -> Option<String> {
    env::var(key).ok()
}

pub fn get_env_or(key: &str, default: &str) -> String {
    get_env(key).unwrap_or_else(|| default.to_string())
}

pub fn get_required_env(key: &str) -> Result<String, String> {
    get_env(key).ok_or_else(|| format!("Required environment variable {} is not set", key))
}

pub fn get_port() -> u16 {
    get_env_or("PORT", "8787").parse().unwrap_or(8787)
}

pub fn get_database_url() -> Option<String> {
    get_env("DATABASE_URL")
}

pub fn is_local_embedded_postgres() -> bool {
    if let Some(url) = get_database_url() {
        url.contains("localhost:") && url.contains("postgres:password")
    } else {
        false
    }
}

pub fn get_firebase_project_id() -> Result<String, String> {
    get_required_env("FIREBASE_PROJECT_ID")
}

pub fn get_firebase_auth_emulator_host() -> Option<String> {
    get_env("FIREBASE_AUTH_EMULATOR_HOST")
}

pub fn is_development() -> bool {
    get_env("NODE_ENV").as_deref() == Some("development")
        || get_firebase_auth_emulator_host().is_some()
}

pub fn get_allow_anonymous_users() -> bool {
    get_env("ALLOW_ANONYMOUS_USERS") != Some("false".to_string())
}
