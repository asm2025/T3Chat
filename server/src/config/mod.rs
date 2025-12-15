pub mod app_config;
pub mod placeholders;
pub mod t3chat;

#[cfg(test)]
mod tests;

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};
use t3chat::T3ChatConfig;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub config: Option<T3ChatConfig>,
    pub metadata: ConfigMetadata,
}

#[derive(Debug, Clone)]
pub struct ConfigMetadata {
    pub source: ConfigSource,
    pub notices: Vec<ConfigNotice>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigSource {
    Missing { expected: PathBuf },
    File { path: PathBuf },
    Url { url: String },
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigNotice {
    pub level: NoticeLevel,
    pub message: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum NoticeLevel {
    Info,
    Warning,
    Error,
}

use std::future::Future;

pub fn load_config() -> impl Future<Output = LoadedConfig> {
    let source_path = resolve_config_source();

    async move {
        let mut notices = Vec::new();

        let raw_yaml = match &source_path {
            ConfigSource::File { path } => match fs::read_to_string(path) {
                Ok(content) => {
                    info!("Loading configuration from file: {}", path.display());
                    Some(content)
                }
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        warn!(
                            "Config file not found at {}; using defaults",
                            path.display()
                        );
                        notices.push(ConfigNotice {
                            level: NoticeLevel::Warning,
                            message: "Configuration file not found".to_string(),
                            detail: Some(format!("Expected at: {}", path.display())),
                        });
                    } else {
                        error!("Failed to read config file: {}", err);
                        notices.push(ConfigNotice {
                            level: NoticeLevel::Error,
                            message: "Failed to read configuration file".to_string(),
                            detail: Some(err.to_string()),
                        });
                    }
                    None
                }
            },
            ConfigSource::Url { url } => match fetch_url_config(url).await {
                Ok(content) => {
                    info!("Loaded configuration from URL: {}", url);
                    Some(content)
                }
                Err(err) => {
                    error!("Failed to fetch config from URL {}: {}", url, err);
                    notices.push(ConfigNotice {
                        level: NoticeLevel::Error,
                        message: "Failed to fetch configuration from URL".to_string(),
                        detail: Some(format!("URL: {}, Error: {}", url, err)),
                    });
                    None
                }
            },
            ConfigSource::Missing { expected } => {
                warn!(
                    "No config path specified and default not found at {}",
                    expected.display()
                );
                notices.push(ConfigNotice {
                    level: NoticeLevel::Warning,
                    message: "Configuration file not found".to_string(),
                    detail: Some(format!("Expected at: {}", expected.display())),
                });
                None
            }
        };

        let config = if let Some(yaml_content) = raw_yaml {
            match serde_yaml::from_str::<T3ChatConfig>(&yaml_content) {
                Ok(cfg) => Some(cfg),
                Err(err) => {
                    error!("Failed to parse configuration: {}", err);
                    notices.push(ConfigNotice {
                        level: NoticeLevel::Error,
                        message: "Failed to parse configuration file".to_string(),
                        detail: Some(err.to_string()),
                    });
                    None
                }
            }
        } else {
            None
        };

        LoadedConfig {
            config,
            metadata: ConfigMetadata {
                source: source_path,
                notices,
            },
        }
    }
}

fn resolve_config_source() -> ConfigSource {
    if let Ok(custom) = env::var("CONFIG_PATH") {
        if !custom.trim().is_empty() {
            if custom.starts_with("http://") || custom.starts_with("https://") {
                return ConfigSource::Url { url: custom };
            }

            let path = Path::new(&custom);
            let candidate = if path.is_absolute() {
                path.to_path_buf()
            } else {
                env::current_dir().unwrap_or_default().join(path)
            };
            return ConfigSource::File {
                path: fs::canonicalize(&candidate).unwrap_or(candidate),
            };
        }
    }

    // Also check T3CHAT_CONFIG for backward compatibility or alias
    if let Ok(custom) = env::var("T3CHAT_CONFIG") {
        if !custom.trim().is_empty() {
            // Treat as file path mostly
            let path = Path::new(&custom);
            let candidate = if path.is_absolute() {
                path.to_path_buf()
            } else {
                env::current_dir().unwrap_or_default().join(path)
            };
            return ConfigSource::File {
                path: fs::canonicalize(&candidate).unwrap_or(candidate),
            };
        }
    }

    // Default: t3chat.yaml or librechat.yaml in current dir
    let cwd = env::current_dir().unwrap_or_default();
    let t3chat_candidate = cwd.join("t3chat.yaml");
    let librechat_candidate = cwd.join("librechat.yaml");

    if t3chat_candidate.exists() {
        ConfigSource::File {
            path: fs::canonicalize(&t3chat_candidate).unwrap_or(t3chat_candidate),
        }
    } else if librechat_candidate.exists() {
        ConfigSource::File {
            path: fs::canonicalize(&librechat_candidate).unwrap_or(librechat_candidate),
        }
    } else {
        ConfigSource::Missing {
            expected: t3chat_candidate,
        }
    }
}

async fn fetch_url_config(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client.get(url).send().await?;
    let content = response.error_for_status()?.text().await?;
    Ok(content)
}
