use anyhow::{Context, Result};
use tiktoken_rs::cl100k_base;

pub fn count_tokens(text: &str) -> Result<usize> {
    let bpe = cl100k_base()?;
    let tokens = bpe.encode_with_special_tokens(text);
    Ok(tokens.len())
}

pub fn estimate_cost(text: &str, price_per_1k_tokens: f64) -> Result<f64> {
    let n = count_tokens(text)?;
    Ok((n as f64 / 1000.0) * price_per_1k_tokens)
}

pub fn truncate_to_token_count(text: &str, max_tokens: usize) -> Result<String> {
    let bpe = cl100k_base()?;
    let tokens = bpe.encode_with_special_tokens(text);

    if tokens.len() <= max_tokens {
        return Ok(text.to_string());
    }

    bpe.decode(tokens[..max_tokens].to_vec())
        .context("failed to decode truncated tokens")
}

#[cfg(test)]
mod tests {
    use super::{count_tokens, truncate_to_token_count};

    #[test]
    fn truncate_returns_original_text_when_under_limit() {
        let text = "short text";

        let truncated = truncate_to_token_count(text, 100).unwrap();

        assert_eq!(truncated, text);
    }

    #[test]
    fn truncate_reduces_token_count_when_over_limit() {
        let text = "This is a longer sample that should be truncated to fewer tokens.";

        let truncated = truncate_to_token_count(text, 3).unwrap();

        assert!(count_tokens(&truncated).unwrap() <= 3);
        assert_ne!(truncated, text);
    }

    #[test]
    fn truncate_handles_empty_text_and_zero_limit() {
        assert_eq!(truncate_to_token_count("", 0).unwrap(), "");
        assert_eq!(count_tokens(&truncate_to_token_count("hello", 0).unwrap()).unwrap(), 0);
    }
}
