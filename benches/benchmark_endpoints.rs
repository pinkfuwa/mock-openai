//! Comprehensive benchmark suite for mock-openai endpoints
//!
//! This benchmark tests all endpoints under various configurations:
//! - Different response delays (0ms, 10ms, 50ms)
//! - Different response sizes (small, medium, large)
//! - Streaming vs non-streaming (where applicable)
//! - Different token distributions
//!
//! **Optimization**: AppState is created once per benchmark group and cloned
//! for use in each thread (not per iteration), avoiding OOM issues.
//!
//! Run with:
//!   cargo bench --bench benchmark_endpoints
//!   cargo bench --bench benchmark_endpoints -- --verbose
//!   cargo bench --bench benchmark_endpoints --release

use actix_web::{test, web, App};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use lipsum::lipsum_words;
use mock_openai::endpoints::*;
use mock_openai::types::*;
use mock_openai::utils::*;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

/// Configuration for a benchmark scenario
#[derive(Clone, Copy, Debug)]
struct BenchConfig {
    response_delay_ms: u64,
    token_mean: f64,
    token_stddev: f64,
    pregen_count: usize,
}

impl BenchConfig {
    #[allow(dead_code)]
    fn low_latency() -> Self {
        BenchConfig {
            response_delay_ms: 0,
            token_mean: 100.0,
            token_stddev: 20.0,
            pregen_count: 256,
        }
    }

    fn medium_latency() -> Self {
        BenchConfig {
            response_delay_ms: 10,
            token_mean: 200.0,
            token_stddev: 40.0,
            pregen_count: 512,
        }
    }

    fn high_latency() -> Self {
        BenchConfig {
            response_delay_ms: 50,
            token_mean: 512.0,
            token_stddev: 100.0,
            pregen_count: 1024,
        }
    }

    fn small_response() -> Self {
        BenchConfig {
            response_delay_ms: 0,
            token_mean: 50.0,
            token_stddev: 10.0,
            pregen_count: 256,
        }
    }

    fn medium_response() -> Self {
        BenchConfig {
            response_delay_ms: 0,
            token_mean: 256.0,
            token_stddev: 50.0,
            pregen_count: 512,
        }
    }

    #[allow(dead_code)]
    fn large_response() -> Self {
        BenchConfig {
            response_delay_ms: 0,
            token_mean: 1000.0,
            token_stddev: 200.0,
            pregen_count: 1024,
        }
    }
}

/// Generate mock articles using lipsum
fn generate_articles(count: usize, token_mean: f64, token_stddev: f64) -> Vec<Arc<String>> {
    use rand::SeedableRng;

    let mut rng = rand::rngs::StdRng::from_entropy();
    let mut articles = Vec::with_capacity(count);

    for _ in 0..count {
        let sampled = sample_normal_f64(&mut rng, token_mean, token_stddev).round() as isize;
        let tokens = sampled.max(1) as usize;
        let chars = tokens_to_chars(tokens);
        let words = std::cmp::max(1, (chars as f64 / 6.0).round() as usize);
        let article_str = lipsum_words(words);
        articles.push(Arc::new(article_str));
    }

    articles
}

/// Generate pre-computed token samples for streaming
fn generate_stream_samples(count: usize, token_mean: f64, token_stddev: f64) -> Vec<usize> {
    use rand::SeedableRng;

    let mut rng = rand::rngs::StdRng::from_entropy();
    let mut samples = Vec::with_capacity(count);

    for _ in 0..count {
        let sampled = sample_normal_f64(&mut rng, token_mean, token_stddev).round() as isize;
        samples.push(sampled.max(0) as usize);
    }

    samples
}

/// Create app state with given configuration.
/// This is created once per benchmark group and cloned for threads.
fn create_app_state(config: BenchConfig) -> Arc<AppState> {
    let articles = generate_articles(config.pregen_count, config.token_mean, config.token_stddev);
    let stream_samples = generate_stream_samples(20_000, config.token_mean, config.token_stddev);

    Arc::new(AppState {
        articles,
        stream_token_samples: Arc::new(stream_samples),
        stream_samples_idx: AtomicUsize::new(0),
        token_mean: config.token_mean,
        token_stddev: config.token_stddev,
        response_delay_ms: config.response_delay_ms,
    })
}

// ============================================================================
// Health Endpoint Benchmarks
// ============================================================================

fn bench_health(c: &mut Criterion) {
    let mut group = c.benchmark_group("health_endpoint");
    group.sample_size(100);

    let app_state = create_app_state(BenchConfig::low_latency());

    let rt = tokio::runtime::Runtime::new().unwrap(); // One runtime for the whole group
    let app_service = rt.block_on(async {
        test::init_service(
            App::new()
                .app_data(web::Data::from(app_state))
                .route("/health", web::get().to(health_handler)),
        )
        .await
    });

    group.bench_function("health_check", |b| {
        let app_service = &app_service;
        b.to_async(&rt).iter(|| {
            let app_service = app_service;
            async move {
                let req = test::TestRequest::get().uri("/health").to_request();
                let resp = test::call_service(&app_service, req).await;
                black_box(resp) // Black-box the response to ensure it's measured
            }
        });
    });

    group.finish();
}

// ============================================================================
// Models Endpoint Benchmarks
// ============================================================================

fn bench_models(c: &mut Criterion) {
    let mut group = c.benchmark_group("models_endpoint");
    group.sample_size(100);

    let app_state = create_app_state(BenchConfig::low_latency());

    let rt = tokio::runtime::Runtime::new().unwrap(); // One runtime for the whole group

    group.bench_function("models_list", |b| {
        let app_service = rt.block_on(async {
            test::init_service(
                App::new()
                    .app_data(web::Data::from(Arc::clone(&app_state)))
                    .route("/v1/models", web::get().to(models_list_handler)),
            )
            .await
        });

        b.to_async(&rt).iter(|| {
            let app_service = &app_service;
            async move {
                let req = test::TestRequest::get().uri("/v1/models").to_request();
                black_box(test::call_service(app_service, req).await)
            }
        });
    });

    group.bench_function("models_get_by_id", |b| {
        let app_service = rt.block_on(async {
            test::init_service(
                App::new()
                    .app_data(web::Data::from(Arc::clone(&app_state)))
                    .route("/v1/models/{id}", web::get().to(model_get_handler)),
            )
            .await
        });

        b.to_async(&rt).iter(|| {
            let app_service = &app_service;
            async move {
                let req = test::TestRequest::get()
                    .uri("/v1/models/gpt-4-mock")
                    .to_request();
                black_box(test::call_service(app_service, req).await)
            }
        });
    });

    group.finish();
}

// ============================================================================
// Embeddings Endpoint Benchmarks
// ============================================================================

fn bench_embeddings(c: &mut Criterion) {
    let mut group = c.benchmark_group("embeddings_endpoint");
    group.sample_size(50);

    let app_state = create_app_state(BenchConfig::small_response());
    let rt = tokio::runtime::Runtime::new().unwrap(); // One runtime for the whole group

    let app_service_single = rt.block_on(async {
        test::init_service(
            App::new()
                .app_data(web::Data::from(Arc::clone(&app_state)))
                .route("/v1/embeddings", web::post().to(embeddings_handler)),
        )
        .await
    });

    group.bench_function("embeddings_single", |b| {
        let app_service = &app_service_single;
        b.to_async(&rt).iter(|| {
            let app_service = app_service;
            async move {
                let payload = serde_json::json!({
                    "model": "text-embedding-3-small",
                    "input": "test input"
                });

                let req = test::TestRequest::post()
                    .uri("/v1/embeddings")
                    .set_json(payload)
                    .to_request();

                black_box(test::call_service(app_service, req).await)
            }
        });
    });

    let app_service_batch = rt.block_on(async {
        test::init_service(
            App::new()
                .app_data(web::Data::from(Arc::clone(&app_state)))
                .route("/v1/embeddings", web::post().to(embeddings_handler)),
        )
        .await
    });

    group.bench_function("embeddings_batch", |b| {
        let app_service = &app_service_batch;
        b.to_async(&rt).iter(|| {
            let app_service = app_service;
            async move {
                let payload = serde_json::json!({
                    "model": "text-embedding-3-small",
                    "input": vec!["test 1", "test 2", "test 3"]
                });

                let req = test::TestRequest::post()
                    .uri("/v1/embeddings")
                    .set_json(payload)
                    .to_request();

                black_box(test::call_service(app_service, req).await)
            }
        });
    });

    group.finish();
}

// ============================================================================
// Completions Endpoint Benchmarks
// ============================================================================

fn bench_completions(c: &mut Criterion) {
    let mut group = c.benchmark_group("completions_endpoint");
    group.sample_size(50);

    let app_state = create_app_state(BenchConfig::medium_response());
    let rt = tokio::runtime::Runtime::new().unwrap(); // One runtime for the whole group

    let app_service = rt.block_on(async {
        test::init_service(
            App::new()
                .app_data(web::Data::from(Arc::clone(&app_state)))
                .route("/v1/completions", web::post().to(completions_handler)),
        )
        .await
    });

    group.bench_function("completions_small", |b| {
        let app_service = &app_service;
        b.to_async(&rt).iter(|| {
            let app_service = app_service;
            async move {
                let payload = serde_json::json!({
                    "model": "gpt-4-mock",
                    "prompt": "Once upon a time",
                    "max_tokens": 100
                });

                let req = test::TestRequest::post()
                    .uri("/v1/completions")
                    .set_json(payload)
                    .to_request();

                black_box(test::call_service(app_service, req).await)
            }
        });
    });

    group.finish();
}

// ============================================================================
// Chat Completions Endpoint Benchmarks
// ============================================================================

fn bench_chat_completions_non_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("chat_completions_non_streaming");
    group.sample_size(50);

    let app_state = create_app_state(BenchConfig::medium_response());
    let rt = tokio::runtime::Runtime::new().unwrap(); // One runtime for the whole group

    let app_service = rt.block_on(async {
        test::init_service(
            App::new()
                .app_data(web::Data::from(Arc::clone(&app_state)))
                .route(
                    "/v1/chat/completions",
                    web::post().to(chat_completions_handler),
                ),
        )
        .await
    });

    group.bench_function("chat_non_streaming", |b| {
        let app_service = &app_service;
        b.to_async(&rt).iter(|| {
            let app_service = app_service;
            async move {
                let payload = serde_json::json!({
                    "model": "gpt-4-mock",
                    "messages": [
                        {"role": "user", "content": "Hello!"}
                    ],
                    "stream": false
                });

                let req = test::TestRequest::post()
                    .uri("/v1/chat/completions")
                    .set_json(payload)
                    .to_request();

                black_box(test::call_service(app_service, req).await)
            }
        });
    });

    group.finish();
}

fn bench_chat_completions_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("chat_completions_streaming");
    group.sample_size(50);

    let app_state = create_app_state(BenchConfig::medium_response());
    let rt = tokio::runtime::Runtime::new().unwrap(); // One runtime for the whole group

    let app_service = rt.block_on(async {
        test::init_service(
            App::new()
                .app_data(web::Data::from(Arc::clone(&app_state)))
                .route(
                    "/v1/chat/completions",
                    web::post().to(chat_completions_handler),
                ),
        )
        .await
    });

    group.bench_function("chat_streaming", |b| {
        let app_service = &app_service;
        b.to_async(&rt).iter(|| {
            let app_service = app_service;
            async move {
                let payload = serde_json::json!({
                    "model": "gpt-4-mock",
                    "messages": [
                        {"role": "user", "content": "Hello!"}
                    ],
                    "stream": true
                });

                let req = test::TestRequest::post()
                    .uri("/v1/chat/completions")
                    .set_json(payload)
                    .to_request();

                black_box(test::call_service(app_service, req).await)
            }
        });
    });

    group.finish();
}

// ============================================================================
// Response Delay Impact Benchmarks
// ============================================================================

fn bench_response_delay_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_delay_impact");
    group.sample_size(30);
    let rt = tokio::runtime::Runtime::new().unwrap(); // One runtime for the whole group

    for (name, config) in &[
        ("no_delay", BenchConfig::low_latency()),
        ("medium_delay", BenchConfig::medium_latency()),
        ("high_delay", BenchConfig::high_latency()),
    ] {
        let app_state = create_app_state(*config);

        let app_service = rt.block_on(async {
            test::init_service(
                App::new()
                    .app_data(web::Data::from(Arc::clone(&app_state)))
                    .route(
                        "/v1/chat/completions",
                        web::post().to(chat_completions_handler),
                    ),
            )
            .await
        });

        group.bench_with_input(BenchmarkId::from_parameter(name), name, |b, _| {
            let app_service = &app_service;
            b.to_async(&rt).iter(|| {
                let app_service = app_service;
                async move {
                    let payload = serde_json::json!({
                        "model": "gpt-4-mock",
                        "messages": [
                            {"role": "user", "content": "Hello!"}
                        ],
                        "stream": false
                    });

                    let req = test::TestRequest::post()
                        .uri("/v1/chat/completions")
                        .set_json(payload)
                        .to_request();

                    black_box(test::call_service(app_service, req).await)
                }
            });
        });
    }

    group.finish();
}

// ============================================================================
// Article Pool Size Benchmarks
// ============================================================================

fn bench_article_pool_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("article_pool_sizes");
    group.sample_size(30);
    let rt = tokio::runtime::Runtime::new().unwrap(); // One runtime for the whole group

    for pool_size in &[128, 512, 2048] {
        let config = BenchConfig {
            response_delay_ms: 0,
            token_mean: 256.0,
            token_stddev: 50.0,
            pregen_count: *pool_size,
        };
        let app_state = create_app_state(config);

        let app_service = rt.block_on(async {
            test::init_service(
                App::new()
                    .app_data(web::Data::from(Arc::clone(&app_state)))
                    .route(
                        "/v1/chat/completions",
                        web::post().to(chat_completions_handler),
                    ),
            )
            .await
        });

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("pool_{}", pool_size)),
            pool_size,
            |b, _| {
                let app_service = &app_service;
                b.to_async(&rt).iter(|| {
                    let app_service = app_service;
                    async move {
                        let payload = serde_json::json!({
                            "model": "gpt-4-mock",
                            "messages": [
                                {"role": "user", "content": "Test message"}
                            ],
                            "stream": false
                        });

                        let req = test::TestRequest::post()
                            .uri("/v1/chat/completions")
                            .set_json(payload)
                            .to_request();

                        black_box(test::call_service(app_service, req).await)
                    }
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Combined Configuration Stress Tests
// ============================================================================

fn bench_combined_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_stress");
    group.sample_size(20);
    let rt = tokio::runtime::Runtime::new().unwrap(); // One runtime for the whole group

    let configs = vec![
        ("low_latency_small", BenchConfig::low_latency()),
        ("medium_latency_medium", BenchConfig::medium_latency()),
        ("high_latency_large", BenchConfig::high_latency()),
    ];

    for (name, config) in configs {
        let app_state = create_app_state(config);

        let app_service = rt.block_on(async {
            test::init_service(
                App::new()
                    .app_data(web::Data::from(Arc::clone(&app_state)))
                    .route("/health", web::get().to(health_handler))
                    .route("/v1/models", web::get().to(models_list_handler))
                    .route(
                        "/v1/chat/completions",
                        web::post().to(chat_completions_handler),
                    ),
            )
            .await
        });

        group.bench_with_input(BenchmarkId::from_parameter(name), &config, |b, _| {
            let app_service = &app_service;
            b.to_async(&rt).iter(|| {
                let app_service = app_service;
                async move {
                    // Simulate a mixed workload
                    let health_req = test::TestRequest::get().uri("/health").to_request();
                    let _ = test::call_service(app_service, health_req).await;

                    let models_req = test::TestRequest::get().uri("/v1/models").to_request();
                    let _ = test::call_service(app_service, models_req).await;

                    let payload = serde_json::json!({
                        "model": "gpt-4-mock",
                        "messages": [
                            {"role": "user", "content": "Hello!"}
                        ],
                        "stream": false
                    });

                    let chat_req = test::TestRequest::post()
                        .uri("/v1/chat/completions")
                        .set_json(payload)
                        .to_request();

                    black_box(test::call_service(app_service, chat_req).await)
                }
            });
        });
    }

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    bench_health,
    bench_models,
    bench_embeddings,
    bench_completions,
    bench_chat_completions_non_streaming,
    bench_chat_completions_streaming,
    bench_response_delay_impact,
    bench_article_pool_sizes,
    bench_combined_configurations
);

criterion_main!(benches);
