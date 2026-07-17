use anyhow::Result;
use tramway_rust::Tramway;

const SYSTEM_PROMPT: &str = "You are an expert in text chunking for RAG (Retrieval-Augmented Generation) \
    pipelines. Analyze the given text sample and recommend an optimal chunking strategy. \
    Consider the content type (prose, code, emails, structured data, etc.) and suggest \
    appropriate chunk sizes (in characters or tokens), overlap settings, and any special \
    handling required. Be concise and specific — output a short structured recommendation.";

const DEFAULT_MAX_SAMPLE_TOKENS: usize = 500;

/// Calls an LLM via tramway to get a recommended chunking strategy for the given text.
/// `model`            — model identifier forwarded to tramway (e.g. "claude-sonnet-4-6")
/// `tramway_url`      — optional base URL for the tramway server; defaults to localhost:8080
/// `max_sample_tokens` — caps how much of `text` gets sent, in tokens (via the estimator's
///                       tokenizer); defaults to DEFAULT_MAX_SAMPLE_TOKENS if not provided
pub async fn recommend_strategy(
    text: &str,
    model: &str,
    tramway_url: Option<&str>,
    max_sample_tokens: Option<usize>,
) -> Result<String> {
    let client = match tramway_url {
        Some(url) => Tramway::with_url(url),
        None => Tramway::new(),
    };

    let cap = max_sample_tokens.unwrap_or(DEFAULT_MAX_SAMPLE_TOKENS);
    let sample = estimator::truncate_to_token_count(text, cap);

    let input = format!("Analyze this text sample and recommend an optimal chunking strategy:\n\n{sample}");

    client.respond(model, SYSTEM_PROMPT, &input).await
}
