pub use chunker_core::{count_tokens, estimate_cost, recommend_strategy, Chunker, FixedSizeChunker};

pub fn chunk(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    FixedSizeChunker::new(chunk_size, overlap).chunk(text)
}
