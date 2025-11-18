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
//! - HTTP/2 support with TLS certificates
//!
//! This implementation is intentionally minimal and optimized for benchmarking.
//!
//! Usage:
//!   Build:
//!     cargo build --release
//!   Run (HTTP):
//!     ./target/release/mock-openai --port 3000 --response-delay-ms 10 --pregen-count 4096
//!   Run (HTTPS/HTTP2):
//!     ./target/release/mock-openai --port 3000 --tls-cert cert.pem --tls-key key.pem

mod args;
mod endpoints;
mod tls;
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

extern crate jemallocator;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut args = Args::parse();
    // Allow environment variables to override parameters set on the CLI
    args.apply_env_overrides();

    // Validate TLS configuration
    if let Err(e) = args.validate_tls_config() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    let protocol = if args.tls_cert.is_some() {
        "HTTPS/HTTP2"
    } else {
        "HTTP"
    };

    println!("Starting mock-openai on port {} ({})", args.port, protocol);
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

    let bind_addr = format!("0.0.0.0:{}", args.port);

    // Configure and run the server with optional TLS
    if let (Some(cert_path), Some(key_path)) = (&args.tls_cert, &args.tls_key) {
        println!(
            "Loading TLS certificates from {} and {}",
            cert_path.display(),
            key_path.display()
        );

        match tls::load_tls_config(cert_path, key_path) {
            Ok((certs, key)) => {
                // Build server config with no client auth
                let mut server_config = rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(certs, key)
                    .map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
                    })?;

                // Enable HTTP/2 and HTTP/1.1 via ALPN
                server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

                println!("✓ TLS configuration loaded successfully");
                println!("✓ HTTP/2 enabled (ALPN protocols: h2, http/1.1)");

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
                .bind_rustls_0_23(&bind_addr, server_config)?
                .run()
                .await
            }
            Err(e) => {
                eprintln!("Failed to load TLS configuration: {}", e);
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("TLS configuration error: {}", e),
                ))
            }
        }
    } else {
        println!("Running without TLS (HTTP only)");

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
        .bind(&bind_addr)?
        .run()
        .await
    }
}
