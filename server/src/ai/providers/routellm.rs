#![allow(dead_code)]

/// Placeholder RouteLLM provider module.
/// The provider API is still under design; this module exists so the config
/// loader and model catalog can reference a routellm provider key without
/// failing compilation. Future work will implement the full client once the
/// routing API contract is finalized.
pub struct RouteLLMProvider;

impl RouteLLMProvider {
    pub fn new() -> Self {
        Self
    }
}
