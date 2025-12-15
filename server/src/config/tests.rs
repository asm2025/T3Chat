use super::app_config::DerivedAppConfig;
use super::t3chat::{
    CustomEndpoint, EndpointModels, EndpointsConfig, ModelSpec, ModelSpecPreset, OpenAIEndpoint,
    T3ChatConfig,
};
use super::*;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

// --- Config Loading Tests ---

#[tokio::test]
async fn test_load_config_missing_file() {
    // Override env var to point to non-existent file
    let loaded =
        temp_env::with_var("CONFIG_PATH", Some("non_existent.yaml"), || load_config()).await;

    assert!(loaded.config.is_none());
    // When CONFIG_PATH is set, the source is File even if it doesn't exist (yet)
    assert!(matches!(loaded.metadata.source, ConfigSource::File { .. }));
    assert!(!loaded.metadata.notices.is_empty());
    let notice = &loaded.metadata.notices[0];
    assert_eq!(notice.level, NoticeLevel::Warning); // Warning for missing file
    assert!(notice.message.contains("not found"));
}

#[tokio::test]
async fn test_load_config_invalid_yaml() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("librechat.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "invalid: yaml: [ content").unwrap();

    let loaded = temp_env::with_var("CONFIG_PATH", Some(file_path.to_str().unwrap()), || {
        load_config()
    })
    .await;

    assert!(loaded.config.is_none());
    assert!(matches!(loaded.metadata.source, ConfigSource::File { .. }));
    assert!(!loaded.metadata.notices.is_empty());
    let notice = &loaded.metadata.notices[0];
    assert_eq!(notice.level, NoticeLevel::Error); // Error for invalid parse
}

#[tokio::test]
async fn test_load_config_unknown_fields_strict() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("librechat.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "version: 1.0\nunknown_field: true").unwrap();

    let loaded = temp_env::with_var("CONFIG_PATH", Some(file_path.to_str().unwrap()), || {
        load_config()
    })
    .await;

    // Should fail strict parsing
    assert!(loaded.config.is_none());
    assert!(!loaded.metadata.notices.is_empty());
    assert!(
        loaded.metadata.notices[0]
            .message
            .contains("parse configuration")
    );
}

#[tokio::test]
async fn test_load_config_valid() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("librechat.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        r#"
version: 1.0.0
endpoints:
  openAI:
    apiKey: "sk-123"
"#
    )
    .unwrap();

    let loaded = temp_env::with_var("CONFIG_PATH", Some(file_path.to_str().unwrap()), || {
        load_config()
    })
    .await;

    if loaded.config.is_none() {
        for notice in &loaded.metadata.notices {
            println!("Notice: {:?} {}", notice.level, notice.message);
            if let Some(detail) = &notice.detail {
                println!("Detail: {}", detail);
            }
        }
    }

    assert!(loaded.config.is_some());
    let cfg = loaded.config.unwrap();
    assert_eq!(cfg.version, Some("1.0.0".to_string()));
    assert!(cfg.endpoints.is_some());
}

// --- App Config Transformation Tests ---

#[test]
fn test_derive_app_config_openai() {
    let libre_config = T3ChatConfig {
        version: Some("1.0".into()),
        endpoints: Some(EndpointsConfig {
            openai: Some(OpenAIEndpoint {
                api_key: Some("sk-test".into()),
                base_url: None,
                models: Some(EndpointModels {
                    default: Some(vec!["gpt-4".into()]),
                    fetch: Some(true),
                    ..Default::default()
                }),
                title_model: None,
                summarize: None,
                summary_model: None,
                force_prompt: None,
                model_display_label: Some("OpenAI API".into()),
                icon_u_r_l: None,
                headers: None,
                add_params: None,
                drop_params: None,
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let loaded = LoadedConfig {
        config: Some(libre_config),
        metadata: ConfigMetadata {
            source: ConfigSource::Missing {
                expected: std::path::PathBuf::from("test"),
            },
            notices: vec![],
        },
    };

    let app_config = DerivedAppConfig::from_loaded(loaded);

    assert_eq!(app_config.endpoints.len(), 1);
    let endpoint = &app_config.endpoints[0];
    assert_eq!(endpoint.provider, "openai");
    assert_eq!(endpoint.label, "OpenAI");
    assert_eq!(endpoint.api_key, Some("sk-test".into()));
    assert_eq!(endpoint.models.default, vec!["gpt-4".to_string()]);
    assert_eq!(endpoint.models.fetch, true);
    assert_eq!(endpoint.model_display_label, Some("OpenAI API".into()));
}

#[test]
fn test_derive_app_config_custom_endpoint() {
    let libre_config = T3ChatConfig {
        version: Some("1.0".into()),
        endpoints: Some(EndpointsConfig {
            custom: Some(vec![CustomEndpoint {
                name: "mistral".into(),
                api_key: Some("sk-mistral".into()),
                base_url: Some("https://api.mistral.ai".into()),
                models: Some(EndpointModels {
                    default: Some(vec!["mistral-large".into()]),
                    ..Default::default()
                }),
                model_display_label: None,
                icon_u_r_l: None,
                headers: None,
                add_params: None,
                drop_params: None,
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let loaded = LoadedConfig {
        config: Some(libre_config),
        metadata: ConfigMetadata {
            source: ConfigSource::Missing {
                expected: std::path::PathBuf::from("test"),
            },
            notices: vec![],
        },
    };

    let app_config = DerivedAppConfig::from_loaded(loaded);

    assert_eq!(app_config.endpoints.len(), 1);
    let endpoint = &app_config.endpoints[0];
    assert_eq!(endpoint.provider, "mistral");
    assert_eq!(endpoint.label, "mistral");
    assert_eq!(endpoint.base_url, Some("https://api.mistral.ai".into()));
}

#[test]
fn test_derive_app_config_env_vars() {
    temp_env::with_var("OPENAI_API_KEY", Some("sk-env-key"), || {
        let libre_config = T3ChatConfig {
            version: Some("1.0".into()),
            endpoints: Some(EndpointsConfig {
                openai: Some(OpenAIEndpoint {
                    api_key: Some("${OPENAI_API_KEY}".into()),
                    ..OpenAIEndpoint {
                        api_key: None,
                        base_url: None,
                        models: None,
                        title_model: None,
                        summarize: None,
                        summary_model: None,
                        force_prompt: None,
                        model_display_label: None,
                        icon_u_r_l: None,
                        headers: None,
                        add_params: None,
                        drop_params: None,
                    }
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let loaded = LoadedConfig {
            config: Some(libre_config),
            metadata: ConfigMetadata {
                source: ConfigSource::Missing {
                    expected: std::path::PathBuf::from("test"),
                },
                notices: vec![],
            },
        };

        let app_config = DerivedAppConfig::from_loaded(loaded);
        let endpoint = &app_config.endpoints[0];
        assert_eq!(endpoint.api_key, Some("sk-env-key".into()));
        assert!(app_config.notices.is_empty());
    });
}

#[test]
fn test_derive_app_config_missing_env_var() {
    temp_env::with_var("OPENAI_API_KEY", None::<&str>, || {
        let libre_config = T3ChatConfig {
            version: Some("1.0".into()),
            endpoints: Some(EndpointsConfig {
                openai: Some(OpenAIEndpoint {
                    api_key: Some("${OPENAI_API_KEY}".into()),
                    ..OpenAIEndpoint {
                        api_key: None,
                        base_url: None,
                        models: None,
                        title_model: None,
                        summarize: None,
                        summary_model: None,
                        force_prompt: None,
                        model_display_label: None,
                        icon_u_r_l: None,
                        headers: None,
                        add_params: None,
                        drop_params: None,
                    }
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let loaded = LoadedConfig {
            config: Some(libre_config),
            metadata: ConfigMetadata {
                source: ConfigSource::Missing {
                    expected: std::path::PathBuf::from("test"),
                },
                notices: vec![],
            },
        };

        let app_config = DerivedAppConfig::from_loaded(loaded);
        // Endpoint should be disabled (not present in the list) because of missing env var
        assert!(app_config.endpoints.is_empty());

        // Should have a warning notice
        assert!(!app_config.notices.is_empty());
        let notice = &app_config.notices[0];
        assert_eq!(notice.level, NoticeLevel::Warning);
        assert!(notice.message.contains("disabled"));
        assert!(notice.detail.as_ref().unwrap().contains("OPENAI_API_KEY"));
    });
}

#[test]
fn test_derive_app_config_model_specs() {
    let libre_config = T3ChatConfig {
        version: Some("1.0".into()),
        model_specs: Some(vec![ModelSpec {
            name: "creative".into(),
            label: "Creative Helper".into(),
            description: Some("Helps with writing".into()),
            icon: Some("writer".into()),
            preset: ModelSpecPreset {
                endpoint: "openai".into(),
                model: "gpt-4".into(),
                temperature: Some(0.9),
                ..ModelSpecPreset {
                    endpoint: "".into(),
                    model: "".into(),
                    model_label: None,
                    greeting: None,
                    prompt_prefix: None,
                    temperature: None,
                    top_p: None,
                    presence_penalty: None,
                    frequency_penalty: None,
                    resend_files: None,
                    image_detail: None,
                    tools: None,
                }
            },
        }]),
        ..Default::default()
    };

    let loaded = LoadedConfig {
        config: Some(libre_config),
        metadata: ConfigMetadata {
            source: ConfigSource::Missing {
                expected: std::path::PathBuf::from("test"),
            },
            notices: vec![],
        },
    };

    let app_config = DerivedAppConfig::from_loaded(loaded);
    assert_eq!(app_config.model_specs.len(), 1);
    let spec = &app_config.model_specs[0];
    assert_eq!(spec.name, "creative");
    assert_eq!(spec.preset.temperature, Some(0.9));
}
