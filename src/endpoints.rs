//! HTTP endpoint handlers for the mock OpenAI API

use crate::types::*;
use crate::utils::*;
use actix_web::{web, Error, HttpResponse, Responder};
use bytes::Bytes;
use futures::stream::{unfold, StreamExt};
use rand::{rngs::StdRng, SeedableRng};
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Static string constants to avoid repeated allocations
const FINISH_REASON_STOP: &str = "stop";
const ROLE_ASSISTANT: &str = "assistant";
const OBJECT_CHAT_COMPLETION: &str = "chat.completion";
const OBJECT_TEXT_COMPLETION: &str = "text.completion";
const OBJECT_MODEL: &str = "model";
const OWNED_BY: &str = "mock-openai";

/// GET /health
pub async fn health_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

/// GET /v1/models
pub async fn models_list_handler() -> impl Responder {
    let models = vec![ModelInfo {
        id: "gpt-4-mock".into(),
        object: OBJECT_MODEL.into(),
        owned_by: OWNED_BY.into(),
    }];
    HttpResponse::Ok().json(ModelsListResponse { data: models })
}

/// GET /v1/models/{id}
pub async fn model_get_handler(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let known = ["gpt-4-mock"];
    if known.contains(&id.as_str()) {
        HttpResponse::Ok().json(ModelInfo {
            id,
            object: OBJECT_MODEL.into(),
            owned_by: OWNED_BY.into(),
        })
    } else {
        HttpResponse::NotFound().json(serde_json::json!({ "error": "model_not_found" }))
    }
}

/// POST /v1/completions
pub async fn completions_handler(
    state: web::Data<AppState>,
    req: web::Json<CompletionsRequest>,
) -> Result<HttpResponse, Error> {
    let req = req.into_inner();

    let mut rng = rand::thread_rng();
    let mut sampled =
        sample_normal_f64(&mut rng, state.token_mean, state.token_stddev).round() as isize;
    if sampled < 1 {
        sampled = 1;
    }
    let mut completion_tokens = sampled as usize;
    if let Some(max_tokens) = req.max_tokens {
        if completion_tokens > max_tokens {
            completion_tokens = max_tokens;
        }
    }

    let article = choose_article(&state.articles, &mut rng);
    let content = slice_text_by_tokens(&article, completion_tokens);

    // Recompute actual tokens based on output produced
    let actual_completion_tokens = chars_to_tokens(content.chars().count());

    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let prompt_tokens = req
        .prompt
        .as_ref()
        .map(|p| chars_to_tokens(p.chars().count()))
        .unwrap_or(0);
    let usage = Usage {
        prompt_tokens,
        completion_tokens: actual_completion_tokens,
        total_tokens: prompt_tokens + actual_completion_tokens,
    };

    let choice = CompletionChoice {
        index: 0,
        text: content,
        finish_reason: FINISH_REASON_STOP,
    };

    let resp = CompletionsResponse {
        id: format!("cmpl-{}", Uuid::new_v4()),
        object: OBJECT_TEXT_COMPLETION.to_string(),
        created,
        model: req.model,
        usage,
        choices: vec![choice],
    };

    Ok(HttpResponse::Ok().json(resp))
}

/// POST /v1/embeddings
pub async fn embeddings_handler(req: web::Json<EmbeddingRequest>) -> impl Responder {
    let _req = req.into_inner();
    let dimension = 128usize;
    HttpResponse::Ok().json(generate_embedding(dimension))
}

/// POST /v1/chat/completions - supports streaming SSE & non-streaming JSON
pub async fn chat_completions_handler(
    state: web::Data<AppState>,
    req: web::Json<ChatCompletionRequest>,
) -> Result<HttpResponse, Error> {
    let req = req.into_inner();
    if req.model.is_empty() {
        return Ok(
            HttpResponse::BadRequest().json(serde_json::json!({ "error": "model_required" }))
        );
    }

    let stream_flag = req.stream.unwrap_or(false);
    if !stream_flag {
        // Non-streaming response
        let mut rng = rand::thread_rng();
        let mut sampled =
            sample_normal_f64(&mut rng, state.token_mean, state.token_stddev).round() as isize;
        if sampled < 1 {
            sampled = 1;
        }
        let mut completion_tokens = sampled as usize;
        if let Some(max_tokens) = req.max_tokens {
            if completion_tokens > max_tokens {
                completion_tokens = max_tokens;
            }
        }

        let article = choose_article(&state.articles, &mut rng);
        let content = slice_text_by_tokens(&article, completion_tokens);

        let actual_completion_tokens = chars_to_tokens(content.chars().count());
        let created = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let prompt_tokens = req
            .messages
            .as_ref()
            .map(|msgs| {
                let chars: usize = msgs.iter().map(|m| m.content.chars().count()).sum();
                chars_to_tokens(chars)
            })
            .unwrap_or(0);

        let usage = Usage {
            prompt_tokens,
            completion_tokens: actual_completion_tokens,
            total_tokens: prompt_tokens + actual_completion_tokens,
        };

        let choice = ChatChoice {
            index: 0,
            message: ChatMessage {
                role: ROLE_ASSISTANT,
                content,
            },
            finish_reason: FINISH_REASON_STOP,
        };

        let resp = ChatCompletionResponse {
            id: format!("chatcmpl-{}", Uuid::new_v4()),
            object: OBJECT_CHAT_COMPLETION.to_string(),
            created,
            model: req.model,
            usage,
            choices: vec![choice],
        };

        return Ok(HttpResponse::Ok().json(resp));
    }

    // Streaming mode (SSE)
    // Sample total tokens to emit
    let mut rng = StdRng::from_entropy();
    let mut sampled =
        sample_normal_f64(&mut rng, state.token_mean, state.token_stddev).round() as isize;
    if sampled < 1 {
        sampled = 1;
    }
    let mut total_tokens = sampled as usize;
    if let Some(max_tokens) = req.max_tokens {
        if total_tokens > max_tokens {
            total_tokens = max_tokens;
        }
    }

    let article_arc = choose_article(&state.articles, &mut rng);
    let article_len_chars = article_arc.chars().count();
    let chars_remaining = tokens_to_chars(total_tokens);

    // We'll track position in chars (not bytes), because char boundaries matter
    let initial_char_pos = 0usize;
    let response_delay_ms = state.response_delay_ms;

    // Get the sample stream and sample count (pre-computed at startup)
    let stream_samples = state.stream_token_samples.clone();
    let samples_len = stream_samples.len();

    // Get current index and increment for next request (lock-free)
    let sample_start_idx = state.stream_samples_idx.fetch_add(1, Ordering::Relaxed);

    // A pinned, boxed stream of chunks (SSE events) which the HTTP response will stream
    let s = unfold(
        (
            article_arc.clone(),
            chars_remaining,
            initial_char_pos,
            sample_start_idx,
            response_delay_ms,
            false, // done_sent
            stream_samples,
            samples_len,
        ),
        move |(
            article,
            chars_remaining,
            char_pos,
            mut sample_idx,
            response_delay_ms,
            done_sent,
            stream_samples,
            samples_len,
        )| async move {
            // If all characters have been emitted already
            if chars_remaining == 0 {
                if done_sent {
                    return None;
                } else {
                    let done_event = "data: [DONE]\n\n".to_string();
                    return Some((
                        Ok::<Bytes, actix_web::Error>(Bytes::from(done_event)),
                        (
                            article,
                            0usize,
                            char_pos,
                            sample_idx,
                            response_delay_ms,
                            true,
                            stream_samples,
                            samples_len,
                        ),
                    ));
                }
            }

            // This eliminates the RNG call for every SSE event
            let chunk_tokens = stream_samples[sample_idx % samples_len];
            sample_idx += 1;

            let mut chunk_chars = tokens_to_chars(chunk_tokens);
            if chunk_chars > chars_remaining {
                chunk_chars = chars_remaining;
            }

            // Determine byte indices
            let start_byte = char_pos_to_byte_idx(&article, char_pos);
            let end_char_pos = std::cmp::min(article_len_chars, char_pos + chunk_chars);
            let mut end_byte = char_pos_to_byte_idx(&article, end_char_pos);

            // Avoid splitting words - try to find whitespace before end_byte
            if end_byte < article.len() {
                if let Some(rel) = article[..end_byte].rfind(' ') {
                    // Only use the whitespace split if it advances the position
                    if rel > start_byte {
                        end_byte = rel;
                    }
                }
            }

            let slice = if end_byte > start_byte {
                &article[start_byte..end_byte]
            } else {
                // Fallback in case we couldn't find a whitespace; pick a single char
                let next_byte = char_pos_to_byte_idx(&article, char_pos + 1).min(article.len());
                &article[start_byte..next_byte]
            };

            let actual_chars_sent = slice.chars().count();

            let chars_remaining_next = chars_remaining.saturating_sub(actual_chars_sent);
            let char_pos_next = char_pos + actual_chars_sent;

            // Delay if requested
            if response_delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(response_delay_ms)).await;
            }

            let sse = sse_event_from_content(slice);

            Some((
                Ok::<Bytes, actix_web::Error>(Bytes::from(sse)),
                (
                    article,
                    chars_remaining_next,
                    char_pos_next,
                    sample_idx,
                    response_delay_ms,
                    false,
                    stream_samples,
                    samples_len,
                ),
            ))
        },
    );

    // Map the stream output to a boxed stream of results consumed by actix-web
    let boxed_stream: Pin<Box<dyn futures::Stream<Item = Result<Bytes, Error>> + Send>> =
        Box::pin(s.map(|item| match item {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
        }));

    Ok(HttpResponse::Ok()
        .append_header((actix_web::http::header::CONTENT_TYPE, "text/event-stream"))
        .append_header((actix_web::http::header::CACHE_CONTROL, "no-cache"))
        .append_header((actix_web::http::header::CONNECTION, "keep-alive"))
        .streaming(boxed_stream))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_completions_capped_by_max_tokens() {
        let articles = vec![std::sync::Arc::new("hello world test".to_string())];
        let stream_samples = vec![10, 20, 30];
        let app_state = web::Data::new(AppState {
            articles,
            stream_token_samples: std::sync::Arc::new(stream_samples),
            stream_samples_idx: std::sync::atomic::AtomicUsize::new(0),
            token_mean: 100.0,
            token_stddev: 20.0,
            response_delay_ms: 0,
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .route("/v1/completions", web::post().to(completions_handler)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/v1/completions")
            .set_json(serde_json::json!({
                "model": "text-davinci-003",
                "prompt": "hello",
                "max_tokens": 5
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_chat_streaming_sse() {
        let articles = vec![std::sync::Arc::new(
            "Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor"
                .to_string(),
        )];
        let stream_samples = vec![5, 10, 15, 20, 10, 5];
        let app_state = web::Data::new(AppState {
            articles,
            stream_token_samples: std::sync::Arc::new(stream_samples),
            stream_samples_idx: std::sync::atomic::AtomicUsize::new(0),
            token_mean: 50.0,
            token_stddev: 10.0,
            response_delay_ms: 0,
        });

        let app = test::init_service(App::new().app_data(app_state).route(
            "/v1/chat/completions",
            web::post().to(chat_completions_handler),
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/v1/chat/completions")
            .set_json(serde_json::json!({
                "model": "gpt-4",
                "messages": [{"role": "user", "content": "hello"}],
                "stream": true
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_embeddings_endpoint() {
        let app_state = web::Data::new(AppState {
            articles: Vec::new(),
            stream_token_samples: std::sync::Arc::new(vec![]),
            stream_samples_idx: std::sync::atomic::AtomicUsize::new(0),
            token_mean: 100.0,
            token_stddev: 20.0,
            response_delay_ms: 0,
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .route("/v1/embeddings", web::post().to(embeddings_handler)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/v1/embeddings")
            .set_json(serde_json::json!({
                "model": "text-embedding-3-small",
                "input": "hello world"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app_state = web::Data::new(AppState {
            articles: Vec::new(),
            stream_token_samples: std::sync::Arc::new(vec![]),
            stream_samples_idx: std::sync::atomic::AtomicUsize::new(0),
            token_mean: 100.0,
            token_stddev: 20.0,
            response_delay_ms: 0,
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .route("/health", web::get().to(health_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/health").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_models_endpoint() {
        let app_state = web::Data::new(AppState {
            articles: Vec::new(),
            stream_token_samples: std::sync::Arc::new(vec![]),
            stream_samples_idx: std::sync::atomic::AtomicUsize::new(0),
            token_mean: 100.0,
            token_stddev: 20.0,
            response_delay_ms: 0,
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .route("/v1/models", web::get().to(models_list_handler))
                .route("/v1/models/{id}", web::get().to(model_get_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/v1/models").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
