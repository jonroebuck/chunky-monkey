pub trait Chunker {
    fn chunk(&self, text: &str) -> Vec<String>;
}

pub struct FixedSizeChunker {
    pub chunk_size: usize,
    pub overlap: usize,
}

impl FixedSizeChunker {
    pub fn new(chunk_size: usize, overlap: usize) -> Self {
        assert!(overlap < chunk_size, "overlap must be less than chunk_size");
        Self { chunk_size, overlap }
    }
}

impl Chunker for FixedSizeChunker {
    fn chunk(&self, text: &str) -> Vec<String> {
        if text.is_empty() {
            return vec![];
        }
        let chars: Vec<char> = text.chars().collect();
        let mut chunks = Vec::new();
        let step = self.chunk_size - self.overlap;
        let mut start = 0;
        while start < chars.len() {
            let end = (start + self.chunk_size).min(chars.len());
            chunks.push(chars[start..end].iter().collect());
            if end == chars.len() {
                break;
            }
            start += step;
        }
        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_without_overlap() {
        let c = FixedSizeChunker::new(4, 0);
        assert_eq!(c.chunk("abcdefgh"), vec!["abcd", "efgh"]);
    }

    #[test]
    fn chunks_with_overlap() {
        let c = FixedSizeChunker::new(4, 1);
        let result = c.chunk("abcdefg");
        assert_eq!(result[0], "abcd");
        assert_eq!(result[1], "defg");
    }

    #[test]
    fn empty_text() {
        let c = FixedSizeChunker::new(4, 0);
        assert!(c.chunk("").is_empty());
    }
}
