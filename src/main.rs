//! Mock OpenAI API - High-performance mock server for benchmarking
//!
//! This server implements a lightweight, high-performance mock server compatible
//! with a subset of the OpenAI APIs. Features:
//! - Pre-generated mock text articles for zero-copy-like behavior (Arc<String>)
//! - Pre-computed token samples for streaming to eliminate per-request RNG
//! - POST /v1/chat/completions (streaming SSE & non-streaming)
//! - POST /v1/completions
//! - POST /v1/embeddings
//! - GET /v1/models
//! - GET /v1/models/{id}
//! - GET /health
//!
//! This implementation is intentionally minimal and optimized for benchmarking.
//!
//! Usage:
//!   Build:
//!     cargo build --release
//!   Run:
//!     ./target/release/mock-openai --port 3000 --response-delay-ms 10 --pregen-count 4096

mod args;
mod endpoints;
mod types;
mod utils;

use actix_web::{web, App, HttpServer};
use args::Args;
use clap::Parser;
use endpoints::{
    chat_completions_handler, completions_handler, embeddings_handler, health_handler,
    model_get_handler, models_list_handler,
};
use lipsum::lipsum_words;
use rand::{rngs::StdRng, SeedableRng};
use std::sync::Arc;
use types::AppState;
use utils::{generate_stream_token_samples, sample_normal_f64, tokens_to_chars};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut args = Args::parse();
    // Allow environment variables to override parameters set on the CLI
    args.apply_env_overrides();
    println!("Starting mock-openai on port {}", args.port);
    if args.verbose {
        println!("Configuration: {:?}", args);
    }

    // Pre-generate mock articles
    println!("Pre-generating {} mock articles...", args.pregen_count);
    let mut rng = StdRng::from_entropy();
    let mut articles: Vec<Arc<String>> = Vec::with_capacity(args.pregen_count);
    for _ in 0..args.pregen_count {
        let mut sampled =
            sample_normal_f64(&mut rng, args.token_mean, args.token_stddev).round() as isize;
        if sampled < 1 {
            sampled = 1;
        }
        let tokens = sampled as usize;
        let chars = tokens_to_chars(tokens);
        // approximate words needed: chars / (avg word size + space ~ 6)
        let words = std::cmp::max(1, (chars as f64 / 6.0).round() as usize);
        let article_str = lipsum_words(words);
        articles.push(Arc::new(article_str));
    }

    println!("Pre-generated {} articles", articles.len());

    // Pre-generate token samples for SSE streaming
    println!("Pre-generating token sample stream...");
    let stream_sample_count = 20_000;
    let stream_token_samples =
        generate_stream_token_samples(stream_sample_count, args.token_mean, args.token_stddev);

    let app_state = web::Data::new(AppState {
        articles,
        stream_token_samples: Arc::new(stream_token_samples),
        stream_samples_idx: std::sync::atomic::AtomicUsize::new(0),
        token_mean: args.token_mean,
        token_stddev: args.token_stddev,
        response_delay_ms: args.response_delay_ms,
    });

    let bind = format!("0.0.0.0:{}", args.port);
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_handler))
            .route("/v1/models", web::get().to(models_list_handler))
            .route("/v1/models/{id}", web::get().to(model_get_handler))
            .route(
                "/v1/chat/completions",
                web::post().to(chat_completions_handler),
            )
            .route("/v1/completions", web::post().to(completions_handler))
            .route("/v1/embeddings", web::post().to(embeddings_handler))
    })
    .bind(bind)?
    .run()
    .await
}
