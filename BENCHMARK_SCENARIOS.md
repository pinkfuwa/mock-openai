# Benchmark Scenarios Reference

This document provides a comprehensive overview of all benchmark scenarios included in the benchmark suite.

## Quick Reference Table

| # | Benchmark Group | Test Name | Endpoint | Configuration | Expected Time | Purpose |
|---|---|---|---|---|---|---|
| 1 | `health_endpoint` | `health_check` | `/health` | Minimal | ~2ms | Health check responsiveness |
| 2 | `models_endpoint` | `models_list` | `/v1/models` | Minimal | ~1ms | List models API |
| 3 | `models_endpoint` | `models_get` | `/v1/models/{id}` | Minimal | ~1ms | Get single model API |
| 4 | `embeddings_endpoint` | `low_latency` | `/v1/embeddings` | 0ms delay, 256 articles | ~1ms | Embeddings with no latency |
| 5 | `embeddings_endpoint` | `medium_latency` | `/v1/embeddings` | 10ms delay, 512 articles | ~10ms | Embeddings with medium latency |
| 6 | `embeddings_endpoint` | `high_latency` | `/v1/embeddings` | 50ms delay, 1024 articles | ~50ms | Embeddings with high latency |
| 7 | `completions_endpoint` | `small_response` | `/v1/completions` | 50 tokens, no delay | ~2ms | Small completion generation |
| 8 | `completions_endpoint` | `medium_response` | `/v1/completions` | 256 tokens, no delay | ~5ms | Medium completion generation |
| 9 | `completions_endpoint` | `large_response` | `/v1/completions` | 1000 tokens, no delay | ~50ms | Large completion generation |
| 10 | `completions_endpoint` | `low_latency` | `/v1/completions` | 0ms delay, 256 tokens | ~3ms | Completions with no latency |
| 11 | `completions_endpoint` | `medium_latency` | `/v1/completions` | 10ms delay, 200 tokens | ~13ms | Completions with 10ms latency |
| 12 | `completions_endpoint` | `high_latency` | `/v1/completions` | 50ms delay, 512 tokens | ~63ms | Completions with 50ms latency |
| 13 | `chat_completions_non_streaming` | `small_response` | `/v1/chat/completions` | 50 tokens, stream=false | ~2ms | Small chat response |
| 14 | `chat_completions_non_streaming` | `medium_response` | `/v1/chat/completions` | 256 tokens, stream=false | ~5ms | Medium chat response |
| 15 | `chat_completions_non_streaming` | `large_response` | `/v1/chat/completions` | 1000 tokens, stream=false | ~50ms | Large chat response |
| 16 | `chat_completions_streaming` | `small_response` | `/v1/chat/completions` | 50 tokens, stream=true | ~5ms | Small streamed chat |
| 17 | `chat_completions_streaming` | `medium_response` | `/v1/chat/completions` | 256 tokens, stream=true | ~40ms | Medium streamed chat |
| 18 | `chat_completions_streaming` | `large_response` | `/v1/chat/completions` | 1000 tokens, stream=true | ~150ms | Large streamed chat |
| 19 | `response_delay_impact` | `0ms` | `/v1/chat/completions` | 0ms delay, stream=true | ~20ms | No simulated latency |
| 20 | `response_delay_impact` | `10ms` | `/v1/chat/completions` | 10ms delay, stream=true | ~40ms | 10ms delay per event |
| 21 | `response_delay_impact` | `50ms` | `/v1/chat/completions` | 50ms delay, stream=true | ~150ms | 50ms delay per event |
| 22 | `article_pool_sizes` | `128_articles` | `/v1/chat/completions` | 128 pre-gen articles | ~3ms | Small pool performance |
| 23 | `article_pool_sizes` | `512_articles` | `/v1/chat/completions` | 512 pre-gen articles | ~3ms | Medium pool performance |
| 24 | `article_pool_sizes` | `2048_articles` | `/v1/chat/completions` | 2048 pre-gen articles | ~3ms | Large pool performance |
| 25 | `combined_configurations` | `high_stress_config` | `/v1/chat/completions` | High tokens, 5ms delay, large pool | ~100ms | High-stress scenario |
| 26 | `combined_configurations` | `low_latency_config` | `/v1/chat/completions` | Low tokens, 0ms delay, small pool | ~1ms | Low-latency scenario |

---

## Detailed Scenario Descriptions

### 1. Health Endpoint (`health_endpoint`)

**Group:** Monitoring & Health

**Tests:**
- `health_check`: Basic health check endpoint

**Configuration:**
- Articles: 256
- Token mean: 100
- Token stddev: 20
- Response delay: 0ms

**What it measures:**
- Raw endpoint overhead
- Minimal processing time
- Should be nearly instant

**Why it matters:**
- Load balancer health checks
- Identifies baseline framework overhead

---

### 2. Models Endpoints (`models_endpoint`)

**Group:** API Information

**Tests:**
- `models_list`: GET `/v1/models` - List all available models
- `models_get`: GET `/v1/models/{id}` - Get specific model information

**Configuration:**
- Articles: 256
- Token mean: 100
- Token stddev: 20
- Response delay: 0ms

**What it measures:**
- Static data serving performance
- JSON serialization overhead
- Path parameter extraction

**Why it matters:**
- Client discovery of available models
- Model validation requests

---

### 3. Embeddings Endpoint (`embeddings_endpoint`)

**Group:** Vector Operations

**Tests:**
- `low_latency`: No simulated delay
- `medium_latency`: 10ms delay per event
- `high_latency`: 50ms delay per event

**Configuration per test:**

| Test | Articles | Token Mean | Delay |
|------|----------|-----------|-------|
| `low_latency` | 256 | 100 | 0ms |
| `medium_latency` | 512 | 200 | 10ms |
| `high_latency` | 1024 | 512 | 50ms |

**What it measures:**
- Embedding generation under various latencies
- API response consistency
- Impact of network simulation

**Why it matters:**
- Vector database indexing operations
- Real-time embedding requests

---

### 4. Completions Endpoint (`completions_endpoint`)

**Group:** Text Generation

**Tests (by response size):**
- `small_response`: 50 tokens mean
- `medium_response`: 256 tokens mean
- `large_response`: 1000 tokens mean

**Tests (by latency):**
- `low_latency`: 0ms delay
- `medium_latency`: 10ms delay
- `high_latency`: 50ms delay

**Configuration per test:**

| Test | Token Mean | Token Stddev | Delay | Articles |
|------|-----------|------------|-------|----------|
| `small_response` | 50 | 10 | 0ms | 256 |
| `medium_response` | 256 | 50 | 0ms | 512 |
| `large_response` | 1000 | 200 | 0ms | 1024 |
| `low_latency` | 100 | 20 | 0ms | 256 |
| `medium_latency` | 200 | 40 | 10ms | 512 |
| `high_latency` | 512 | 100 | 50ms | 1024 |

**What it measures:**
- Text generation scalability
- Token distribution impact
- Latency effects on completion API

**Why it matters:**
- Legacy API performance
- Impact of response size on throughput
- Latency budget estimation

---

### 5. Chat Completions - Non-Streaming (`chat_completions_non_streaming`)

**Group:** Chat Operations

**Tests:**
- `small_response`: 50 tokens
- `medium_response`: 256 tokens
- `large_response`: 1000 tokens

**Configuration per test:**

| Test | Token Mean | Token Stddev | Delay | Articles |
|------|-----------|------------|-------|----------|
| `small_response` | 50 | 10 | 0ms | 256 |
| `medium_response` | 256 | 50 | 0ms | 512 |
| `large_response` | 1000 | 200 | 0ms | 1024 |

**What it measures:**
- Chat API performance without streaming
- Full response generation time
- Token distribution scaling

**Why it matters:**
- Client applications expecting full responses
- Worst-case latency scenarios
- Throughput under variable response sizes

---

### 6. Chat Completions - Streaming (`chat_completions_streaming`)

**Group:** Chat Operations

**Tests:**
- `small_response`: 50 tokens
- `medium_response`: 256 tokens
- `large_response`: 1000 tokens

**Configuration per test:**

| Test | Token Mean | Token Stddev | Delay | Articles |
|------|-----------|------------|-------|----------|
| `small_response` | 50 | 10 | 0ms | 512 |
| `medium_response` | 256 | 50 | 0ms | 512 |
| `large_response` | 1000 | 200 | 0ms | 512 |

**Sample size:** 30 (lower than other tests due to variance)

**What it measures:**
- SSE streaming overhead
- Per-event processing cost
- Stream generation time

**Why it matters:**
- Real-time client applications
- Streaming UI updates
- Event-driven architecture support
- Higher variance due to async nature

**Note:** Higher variance than non-streaming tests is expected and normal.

---

### 7. Response Delay Impact (`response_delay_impact`)

**Group:** Network Simulation

**Tests:**
- `0ms`: No simulated latency
- `10ms`: 10ms latency per SSE event
- `50ms`: 50ms latency per SSE event

**Configuration per test:**

| Test | Delay | Token Mean | Token Stddev | Articles |
|------|-------|-----------|------------|----------|
| `0ms` | 0ms | 200 | 40 | 512 |
| `10ms` | 10ms | 200 | 40 | 512 |
| `50ms` | 50ms | 200 | 40 | 512 |

**Sample size:** 20 (lower due to cumulative delay)

**What it measures:**
- Linear scaling of latency
- Total stream time impact
- Network degradation effects

**Why it matters:**
- Simulates real network conditions
- Predicts performance on high-latency networks
- Validates latency budget calculations
- Total time ≈ (token_count / tokens_per_event) × delay_ms

---

### 8. Article Pool Sizes (`article_pool_sizes`)

**Group:** Memory & Caching

**Tests:**
- `128_articles`: 128 pre-generated articles
- `512_articles`: 512 pre-generated articles
- `2048_articles`: 2048 pre-generated articles

**Configuration per test:**

| Test | Articles | Token Mean | Token Stddev | Delay |
|------|----------|-----------|------------|-------|
| `128_articles` | 128 | 256 | 64 | 0ms |
| `512_articles` | 512 | 256 | 64 | 0ms |
| `2048_articles` | 2048 | 256 | 64 | 0ms |

**What it measures:**
- Cache behavior
- Memory pool impact on RNG
- Random selection performance

**Why it matters:**
- Memory footprint vs performance tradeoff
- Typically minimal impact (articles pre-computed)
- Validates predictable performance

**Expected result:** Minimal variance across pool sizes (< 5%)

---

### 9. Combined Configurations (`combined_configurations`)

**Group:** Realistic Scenarios

**Tests:**

#### 9a. High Stress Configuration
- Articles: 1024
- Token mean: 512
- Token stddev: 128
- Response delay: 5ms per event
- Streaming: Enabled
- Max tokens: 256

**Represents:** High-load, high-latency production scenario

#### 9b. Low Latency Configuration
- Articles: 128
- Token mean: 64
- Token stddev: 16
- Response delay: 0ms
- Streaming: Disabled
- Max tokens: 50

**Represents:** Low-latency, minimal-response edge-case

**Sample size:** 20 (reflects realistic measurement time)

**What it measures:**
- Real-world performance scenarios
- Worst-case vs best-case comparison
- Combined effects of multiple parameters

**Why it matters:**
- Validates server behavior under realistic conditions
- Provides performance baselines for capacity planning
- Helps identify performance bottlenecks in combined scenarios

---

## Performance Classification

### Fast (< 5ms)
- Health checks
- Model list/get
- Embeddings (0ms delay)
- Small completions
- Small chat responses (non-streaming)

### Medium (5-50ms)
- Medium completions
- Medium chat responses
- Streaming with small-medium responses
- Latency-impacted responses (10ms)

### Slow (50-200ms)
- Large completions
- Large streaming responses
- High-latency responses (50ms)
- Stress test scenarios

### Very Slow (200ms+)
- Large responses with high latency
- Pathological edge cases

---

## Running Specific Scenarios

### Run all health-related tests
```bash
cargo bench --bench benchmark_endpoints -- health_endpoint
```

### Run all embedding tests
```bash
cargo bench --bench benchmark_endpoints -- embeddings_endpoint
```

### Run streaming comparison
```bash
cargo bench --bench benchmark_endpoints -- chat_completions
```

### Run latency impact analysis
```bash
cargo bench --bench benchmark_endpoints -- response_delay_impact
```

### Run only stress tests
```bash
cargo bench --bench benchmark_endpoints -- combined_configurations
```

### Run specific scenario
```bash
cargo bench --bench benchmark_endpoints -- "chat_completions_streaming/medium_response"
```

---

## Comparing Scenarios

### Small vs Medium vs Large Responses
```bash
# See response scaling
cargo bench --bench benchmark_endpoints -- completions_endpoint
```

**Expected pattern:** Linear or super-linear scaling with token count

### Streaming Impact
```bash
# Compare streaming vs non-streaming
cargo bench --bench benchmark_endpoints -- chat_completions
```

**Expected pattern:** Streaming slightly slower due to SSE overhead

### Latency Scaling
```bash
# See latency multiplication effect
cargo bench --bench benchmark_endpoints -- response_delay_impact
```

**Expected pattern:** Linear scaling with delay parameter

---

## Statistical Significance

- **Variance < 5%**: Very consistent (deterministic operations)
- **Variance 5-15%**: Normal (RNG-based operations)
- **Variance 15-30%**: Higher variance (streaming/async operations)
- **Variance > 30%**: Investigate system noise or configuration

---

## Memory Profile

Approximate memory usage per configuration:

- **Minimal setup** (128 articles): ~5-10 MB
- **Medium setup** (512 articles): ~20-30 MB
- **Large setup** (2048 articles): ~80-120 MB

Pre-generated articles use Arc<String>, so memory is shared across requests.

---

## See Also

- [BENCHMARKS.md](./BENCHMARKS.md) - Detailed guide and best practices
- [BENCHMARK_QUICKSTART.md](./BENCHMARK_QUICKSTART.md) - Quick reference
- [DESIGN.md](./DESIGN.md) - Architecture and design decisions