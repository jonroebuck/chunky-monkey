use anyhow::Result;
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
