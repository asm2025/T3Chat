pub mod t3chat;

use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tracing::warn;

use t3chat::T3ChatConfig;

static ENV_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{([A-Z0-9_]+)\}").expect("failed to compile env regex"));

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration file not found at {0}")]
    MissingFile(PathBuf),
    #[error("failed to read configuration file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse configuration file: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("environment variable {0} referenced in configuration is not set")]
    MissingEnvVar(String),
    #[error("invalid configuration: {0}")]
    Validation(String),
}

pub fn load_config() -> Result<T3ChatConfig, ConfigError> {
    let path = resolve_config_path();
    let file_content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                return Err(ConfigError::MissingFile(path));
            }
            return Err(ConfigError::Io(err));
        }
    };

    let interpolated = interpolate_env(&file_content)?;
    let config: T3ChatConfig = serde_yaml::from_str(&interpolated)?;

    if let Err(errors) = config.validate() {
        return Err(ConfigError::Validation(errors.join("; ")));
    }

    Ok(config)
}

fn resolve_config_path() -> PathBuf {
    if let Ok(custom) = env::var("T3CHAT_CONFIG") {
        if !custom.trim().is_empty() {
            let path = Path::new(&custom);
            let candidate = if path.is_absolute() {
                path.to_path_buf()
            } else {
                env::current_dir().unwrap_or_default().join(path)
            };
            return fs::canonicalize(&candidate).unwrap_or(candidate);
        }
    }

    Path::new("t3chat.yaml").to_path_buf()
}

fn interpolate_env(input: &str) -> Result<String, ConfigError> {
    let mut output = String::with_capacity(input.len());
    let mut last_index = 0;

    for captures in ENV_PATTERN.captures_iter(input) {
        if let Some(m) = captures.get(0) {
            output.push_str(&input[last_index..m.start()]);
            let var_name = captures.get(1).map(|c| c.as_str()).unwrap_or_default();
            match env::var(var_name) {
                Ok(value) => output.push_str(&value),
                Err(_) => return Err(ConfigError::MissingEnvVar(var_name.to_string())),
            }
            last_index = m.end();
        }
    }

    output.push_str(&input[last_index..]);

    if output.contains("${") {
        warn!("Configuration file still contains unresolved placeholders after interpolation");
    }

    Ok(output)
}
