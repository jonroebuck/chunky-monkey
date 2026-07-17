use anyhow::Result;
use tramway_rust::Tramway;

use crate::config::Config;
use crate::estimator;

const SYSTEM_PROMPT: &str = "You are an expert in text chunking for RAG (Retrieval-Augmented Generation) \
    pipelines. Analyze the given text sample and recommend an optimal chunking strategy. \
    Consider the content type (prose, code, emails, structured data, etc.) and suggest \
    appropriate chunk sizes (in characters or tokens), overlap settings, and any special \
    handling required. Be concise and specific — output a short structured recommendation.";

/// Calls an LLM via tramway to get a recommended chunking strategy for the given text.
/// `model`            — model identifier forwarded to tramway (e.g. "claude-sonnet-4-6")
/// `tramway_url`      — optional base URL for the tramway server; defaults to localhost:8080
/// `max_sample_tokens` — caps how much of `text` gets sent, in tokens (via the estimator's
///                       tokenizer); defaults to the provided config or hardcoded defaults
pub async fn recommend_strategy(
    text: &str,
    model: Option<&str>,
    tramway_url: Option<&str>,
    max_sample_tokens: Option<usize>,
    config: Option<&Config>,
) -> Result<String> {
    let (model, tramway_url, cap) =
        resolve_optimizer_settings(model, tramway_url, max_sample_tokens, config);
    let client = Tramway::with_url(tramway_url);
    let input = build_recommendation_input(text, cap, estimator::truncate_to_token_count)?;

    client.respond(model, SYSTEM_PROMPT, &input).await
}

fn resolve_optimizer_settings<'a>(
    model: Option<&'a str>,
    tramway_url: Option<&'a str>,
    max_sample_tokens: Option<usize>,
    config: Option<&'a Config>,
) -> (&'a str, &'a str, usize) {
    let model =
        model.unwrap_or_else(|| config.map_or(Config::default_model(), |cfg| cfg.optimizer.model.as_str()));
    let tramway_url = tramway_url
        .unwrap_or_else(|| config.map_or(Config::default_tramway_url(), |cfg| cfg.optimizer.tramway_url.as_str()));
    let cap = max_sample_tokens
        .unwrap_or_else(|| config.map_or(Config::default_max_sample_tokens(), |cfg| cfg.optimizer.max_sample_tokens));

    (model, tramway_url, cap)
}

fn build_recommendation_input<F>(text: &str, cap: usize, truncate: F) -> Result<String>
where
    F: FnOnce(&str, usize) -> Result<String>,
{
    let sample = truncate(text, cap)?;
    Ok(format!(
        "Analyze this text sample and recommend an optimal chunking strategy:\n\n{sample}"
    ))
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::{build_recommendation_input, resolve_optimizer_settings};
    use crate::config::Config;

    #[test]
    fn uses_config_when_overrides_are_missing() {
        let config = Config::default();

        let (model, tramway_url, max_sample_tokens) =
            resolve_optimizer_settings(None, None, None, Some(&config));

        assert_eq!(model, Config::default_model());
        assert_eq!(tramway_url, Config::default_tramway_url());
        assert_eq!(max_sample_tokens, Config::default_max_sample_tokens());
    }

    #[test]
    fn explicit_overrides_take_precedence() {
        let config = Config::default();

        let (model, tramway_url, max_sample_tokens) = resolve_optimizer_settings(
            Some("claude-override"),
            Some("http://override"),
            Some(123),
            Some(&config),
        );

        assert_eq!(model, "claude-override");
        assert_eq!(tramway_url, "http://override");
        assert_eq!(max_sample_tokens, 123);
    }

    #[test]
    fn propagates_truncation_errors() {
        let error = build_recommendation_input("text", 1, |_, _| Err(anyhow!("truncate failed")))
            .unwrap_err();

        assert!(error.to_string().contains("truncate failed"));
    }
}
