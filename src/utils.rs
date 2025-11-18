//! Utility functions for tokenization, sampling, and text processing

use crate::types::{EmbeddingResponse, EmbeddingResponseItem};
use rand::{Rng, SeedableRng};
use std::sync::Arc;

const AVG_CHARS_PER_TOKEN: usize = 4; // Approx 1 token ≈ 4 chars (approximation)

/// Random sampling using Box-Muller transform to produce approximate normal samples
pub fn sample_normal_f64<R: Rng>(rng: &mut R, mean: f64, stddev: f64) -> f64 {
    // We ensure u1 is > 0 to avoid ln(0)
    let mut u1 = rng.gen::<f64>();
    if u1 <= 0.0 {
        u1 = std::f64::EPSILON;
    }
    let u2 = rng.gen::<f64>();
    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    mean + z0 * stddev
}

/// Convert tokens to approximate character count
pub fn tokens_to_chars(tokens: usize) -> usize {
    tokens * AVG_CHARS_PER_TOKEN
}

/// Convert character count to approximate token count
pub fn chars_to_tokens(chars: usize) -> usize {
    ((chars as f64) / (AVG_CHARS_PER_TOKEN as f64)).ceil() as usize
}

/// Choose a random article from pre-generated pool; fallback to short default string
pub fn choose_article<R: Rng>(articles: &[Arc<String>], rng: &mut R) -> Arc<String> {
    if articles.is_empty() {
        Arc::new("Lorem ipsum dolor sit amet".to_string())
    } else {
        let idx = rng.gen_range(0..articles.len());
        Arc::clone(&articles[idx])
    }
}

/// Convert an index defined as a char count (0-based) into a byte offset
pub fn char_pos_to_byte_idx(s: &str, char_pos: usize) -> usize {
    if char_pos == 0 {
        return 0;
    }
    s.char_indices()
        .nth(char_pos)
        .map(|(i, _)| i)
        .unwrap_or_else(|| s.len())
}

/// Slice text by tokens (approximate tokens->chars mapping), returns borrowed &str
pub fn slice_text_by_tokens(s: &str, tokens: usize) -> &str {
    let chars_needed = tokens_to_chars(tokens);
    let total_chars = s.chars().count();
    if total_chars <= chars_needed {
        return s;
    }

    // Find byte end by char count
    let mut end_char_pos = chars_needed;
    if end_char_pos > total_chars {
        end_char_pos = total_chars;
    }
    let end_byte = char_pos_to_byte_idx(s, end_char_pos);

    // Prefer slicing at whitespace to avoid cutting a word
    let trimmed_end_byte = if let Some(rel) = s[..end_byte].rfind(' ') {
        rel
    } else {
        end_byte
    };

    let slice = s[..trimmed_end_byte].trim();
    if slice.is_empty() {
        // fallback to the untrimmed slice
        &s[..end_byte]
    } else {
        slice
    }
}

/// Build minimal SSE event payload from a chunk of content
pub fn sse_event_from_content(content: &str) -> String {
    // Data format: {"choices":[{"delta":{"content":"..."}}]}
    let data = serde_json::json!({
        "choices": [
            {
                "delta": { "content": content }
            }
        ]
    });
    format!("data: {}\n\n", data.to_string())
}

/// Generate a mock embedding vector
pub fn generate_embedding(dimension: usize) -> EmbeddingResponse {
    let mut rng = rand::thread_rng();
    let embedding: Vec<f32> = (0..dimension).map(|_| rng.gen()).collect();
    let data = vec![EmbeddingResponseItem {
        embedding,
        index: 0,
    }];
    EmbeddingResponse {
        object: "list".into(),
        data,
    }
}

/// Pre-generate token samples for streaming (circular buffer of random values)
/// This allows SSE handlers to pull from pre-computed samples without per-request RNG calls
pub fn generate_stream_token_samples(count: usize, mean: f64, stddev: f64) -> Vec<usize> {
    let mut rng = rand::rngs::StdRng::from_entropy();
    let mut samples = Vec::with_capacity(count);

    for _ in 0..count {
        let sampled = sample_normal_f64(&mut rng, mean, stddev).round() as isize;
        samples.push(sampled.max(0) as usize);
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn seeded_rng() -> rand::rngs::StdRng {
        rand::rngs::StdRng::seed_from_u64(42)
    }

    #[test]
    fn test_sample_normal_distribution() {
        let mut rng = seeded_rng();
        let mut sum = 0.0;
        let n = 10_000usize;
        for _ in 0..n {
            let v = sample_normal_f64(&mut rng, 100.0, 10.0);
            sum += v;
        }
        let mean = sum / (n as f64);
        assert!((mean - 100.0).abs() < 1.0, "mean was {}", mean);
    }

    #[test]
    fn test_slicing_text_by_tokens() {
        let s = "hello world this is a test of the slicing function. it should cut off at a token boundary.";
        let cut = slice_text_by_tokens(s, 3);
        assert!(cut.len() > 0 && cut.len() < s.len());
    }

    #[test]
    fn test_generate_stream_token_samples() {
        let samples = generate_stream_token_samples(100, 50.0, 10.0);
        assert_eq!(samples.len(), 100);
        // All samples should be valid (usize is always non-negative)
        assert!(!samples.is_empty());
    }
}
