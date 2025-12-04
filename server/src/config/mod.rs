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

#[derive(Debug, Clone, Default)]
pub struct ConfigMetadata {
    pub missing_file: bool,
    pub missing_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub config: T3ChatConfig,
    pub metadata: ConfigMetadata,
}

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

pub fn load_config() -> Result<LoadedConfig, ConfigError> {
    let path = resolve_config_path();
    let file_content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                warn!(
                    path = %path.display(),
                    "t3chat.yaml not found; continuing with default configuration"
                );
                return Ok(LoadedConfig {
                    config: T3ChatConfig::default(),
                    metadata: ConfigMetadata {
                        missing_file: true,
                        missing_path: Some(path),
                        warnings: Vec::new(),
                    },
                });
            }
            return Err(ConfigError::Io(err));
        }
    };

    let interpolated = interpolate_env(&file_content)?;
    let config: T3ChatConfig = serde_yaml::from_str(&interpolated)?;

    if let Err(errors) = config.validate() {
        return Err(ConfigError::Validation(errors.join("; ")));
    }

    Ok(LoadedConfig {
        config,
        metadata: ConfigMetadata::default(),
    })
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

    // Fallback: t3chat.yaml in the current working directory.
    // Use an absolute path so logs and UI notices can show the full expected location.
    let cwd = env::current_dir().unwrap_or_default();
    let candidate = cwd.join("t3chat.yaml");
    fs::canonicalize(&candidate).unwrap_or(candidate)
}

fn interpolate_env(input: &str) -> Result<String, ConfigError> {
    let mut output = String::with_capacity(input.len());
    let mut last_index = 0;

    for captures in ENV_PATTERN.captures_iter(input) {
        if let Some(m) = captures.get(0) {
            // Push everything from the previous match up to the start of this one
            output.push_str(&input[last_index..m.start()]);

            // Determine if this placeholder is inside a YAML comment.
            //
            // YAML comments start with `#` and run to the end of the line. If there is a `#`
            // between the start of the current line and the start of the placeholder, we treat
            // the placeholder as part of the comment and leave it untouched.
            let line_start = input[..m.start()]
                .rfind('\n')
                .map(|idx| idx + 1)
                .unwrap_or(0);
            let is_commented = input[line_start..m.start()].contains('#');

            if is_commented {
                // Inside a comment: keep the literal text as‑is.
                output.push_str(m.as_str());
            } else {
                // Real placeholder: perform environment interpolation.
                let var_name = captures.get(1).map(|c| c.as_str()).unwrap_or_default();
                match env::var(var_name) {
                    Ok(value) => output.push_str(&value),
                    Err(_) => return Err(ConfigError::MissingEnvVar(var_name.to_string())),
                }
            }

            last_index = m.end();
        }
    }

    output.push_str(&input[last_index..]);

    for (i, line) in output.lines().enumerate() {
        if let Some(idx) = line.find("${") {
            let is_commented = line[..idx].trim_start().starts_with('#');
            if !is_commented {
                warn!(
                    "Configuration file line {} contains unresolved placeholder: {}",
                    i + 1,
                    line.trim()
                );
            }
        }
    }

    Ok(output)
}
