use regex::Regex;
use std::env;
use once_cell::sync::Lazy;

static ENV_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{([A-Z0-9_]+)\}").expect("failed to compile env regex"));

#[derive(Debug, Clone)]
pub struct ResolvedString {
    pub value: String,
    pub unresolved: Vec<String>,
}

impl ResolvedString {
    pub fn is_fully_resolved(&self) -> bool {
        self.unresolved.is_empty()
    }
}

pub fn resolve_env_placeholders(input: &str) -> ResolvedString {
    let mut unresolved = Vec::new();
    let result = ENV_PATTERN.replace_all(input, |caps: &regex::Captures| {
        let var_name = &caps[1];
        match env::var(var_name) {
            Ok(val) => val,
            Err(_) => {
                unresolved.push(var_name.to_string());
                // Keep the placeholder if not found, or maybe return empty?
                // The plan says "placeholders resolved where possible".
                // If we return empty string for missing vars, we might break some configs.
                // If we keep the placeholder, it's clear it's missing.
                // However, "missing env vars should disable the affected provider".
                // Let's keep the placeholder text so we can detect it if needed,
                // but simpler might be to replace with empty string if that's the desired behavior.
                // Re-reading: "Preserve `${ENV_VAR}` placeholders at load time."
                // "Only resolve when the value is actually used."
                // "If an endpoint requires apiKey and it resolves to empty or still contains ${...}"
                // So if we keep it as `${...}` it fits "still contains ${...}".
                format!("${{{}}}", var_name)
            }
        }
    });

    ResolvedString {
        value: result.to_string(),
        unresolved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_env_placeholders() {
        temp_env::with_var("TEST_VAR", Some("resolved_value"), || {
            let input = "prefix-${TEST_VAR}-suffix";
            let resolved = resolve_env_placeholders(input);
            assert_eq!(resolved.value, "prefix-resolved_value-suffix");
            assert!(resolved.unresolved.is_empty());
        });

        temp_env::with_var("TEST_VAR", None::<&str>, || {
            let input = "prefix-${TEST_VAR}-suffix";
            let resolved = resolve_env_placeholders(input);
            assert_eq!(resolved.value, "prefix-${TEST_VAR}-suffix");
            assert_eq!(resolved.unresolved, vec!["TEST_VAR"]);
        });
    }
}
