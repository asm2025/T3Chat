use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;

/// Validate and normalize a locale string to ISO-639-1 language code.
///
/// Examples:
/// - "en-US" -> Some("en")
/// - "en" -> Some("en")
/// - "ZH-cn" -> Some("zh")
/// - "english" -> None
pub fn get_validated_language_code(language: Option<&str>) -> Option<String> {
    static LOCALE_RE: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| Regex::new(r"^[a-z]{2}(-[a-z]{2})?$").expect("valid regex"));

    let language = language?.trim();
    if language.is_empty() {
        return None;
    }

    let normalized = language.to_lowercase();
    if LOCALE_RE.is_match(&normalized) {
        return normalized.split('-').next().map(|s| s.to_string());
    }

    tracing::warn!(
        "[STT] Invalid language format {:?}. Expected ISO-639-1 locale like \"en-US\" or \"en\". Skipping language parameter.",
        language
    );
    None
}

#[derive(Clone)]
pub struct OpenAISttClient {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl OpenAISttClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub async fn transcribe(
        &self,
        audio_bytes: Vec<u8>,
        filename: String,
        _mime_type: String,
        language: Option<String>,
        model: String,
    ) -> Result<String> {
        #[derive(Deserialize)]
        struct OpenAITranscriptionResponse {
            text: String,
        }

        let url = format!("{}/audio/transcriptions", self.base_url);

        let mut form = reqwest::multipart::Form::new().text("model", model);

        // Note: OpenAI accepts the file without an explicit per-part Content-Type,
        // so we avoid failing the request if MIME parsing is strict.
        let part = reqwest::multipart::Part::bytes(audio_bytes).file_name(filename);

        form = form.part("file", part);

        if let Some(language) = language.as_deref() {
            if let Some(valid) = get_validated_language_code(Some(language)) {
                form = form.text("language", valid);
            }
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .with_context(|| format!("OpenAI STT request failed: POST {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!(
                status = %status,
                body = %truncate_for_log(&body, 4_096),
                "[STT] OpenAI API error"
            );
            anyhow::bail!("OpenAI STT error ({}): {}", status, body);
        }

        let parsed: OpenAITranscriptionResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI STT response JSON")?;

        Ok(parsed.text.trim().to_string())
    }
}

fn truncate_for_log(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    format!("{}...[truncated]", &s[..max])
}


