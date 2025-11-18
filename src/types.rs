//! Request and response types for the mock OpenAI API

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Shared application state - optimized for zero-copy and pre-computed data
pub struct AppState {
    pub articles: Vec<Arc<String>>,

    /// Pre-computed random token samples for SSE streaming (avoid per-request RNG)
    /// Circular buffer; use atomic counter to cycle through without locks
    pub stream_token_samples: Arc<Vec<usize>>,
    pub stream_samples_idx: std::sync::atomic::AtomicUsize,

    pub token_mean: f64,
    pub token_stddev: f64,
    pub response_delay_ms: u64,
}

/// Helper message types
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Chat Completions request (subset of OpenAI API)
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Option<Vec<Message>>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<usize>,
    pub n: Option<usize>,
    pub stream: Option<bool>,
}

/// Chat completion response with lifetime parameter for borrowed content
#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse<'a> {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub usage: Usage,
    pub choices: Vec<ChatChoice<'a>>,
}

/// Chat choice with lifetime parameter for borrowed message content
#[derive(Debug, Serialize)]
pub struct ChatChoice<'a> {
    pub index: usize,
    pub message: ChatMessage<'a>,
    pub finish_reason: &'a str,
}

/// Chat message with lifetime parameter for borrowed content
#[derive(Debug, Serialize)]
pub struct ChatMessage<'a> {
    pub role: &'a str,
    pub content: &'a str,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Completions (legacy) request & response
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionsRequest {
    pub model: String,
    pub prompt: Option<String>,
    pub max_tokens: Option<usize>,
    pub n: Option<usize>,
    pub stream: Option<bool>,
}

/// Completions response with lifetime parameter for borrowed content
#[derive(Debug, Serialize)]
pub struct CompletionsResponse<'a> {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub usage: Usage,
    pub choices: Vec<CompletionChoice<'a>>,
}

/// Completion choice with lifetime parameter for borrowed text
#[derive(Debug, Serialize)]
pub struct CompletionChoice<'a> {
    pub index: usize,
    pub text: &'a str,
    pub finish_reason: &'a str,
}

/// Embeddings request/response
#[derive(Debug, Deserialize, Serialize)]
pub struct EmbeddingRequest {
    pub input: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponseItem {
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingResponseItem>,
}

/// Models list
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsListResponse {
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub owned_by: String,
}
