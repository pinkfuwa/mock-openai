# Mock OpenAI API Server - Design Document

## Overview

A high-performance OpenAI API mock server designed for benchmarking and testing other servers. This server mimics OpenAI's API endpoints and responses with configurable latency, throughput, and resource constraints to simulate realistic scenarios.

## Goals

1. **High Performance**: Handle thousands of concurrent requests with minimal latency overhead
2. **API Compatibility**: Support OpenAI and OpenRouter API endpoints
3. **Benchmarking**: Provide configurable server behavior for realistic load testing
4. **Production Ready**: Robust error handling, logging, and graceful shutdown
5. **Flexibility**: Easy configuration for different test scenarios

## Architecture

### Core Components

#### 1. HTTP Server
- **Framework**: Actix-web (async, fast, production-grade)
- **Runtime**: Tokio (multi-threaded async runtime)
- **Protocol**: HTTP/1.1 and HTTP/2 ready

#### 2. API Endpoints

##### Chat Completions
- `POST /v1/chat/completions` - Main chat endpoint
- Support for both streaming and non-streaming responses
- Configurable response time and token generation

##### Completions
- `POST /v1/completions` - Legacy completions endpoint
- Text-only mode support

##### Models
- `GET /v1/models` - List available models
- `GET /v1/models/{model_id}` - Get model details

##### Health Check
- `GET /health` - Service availability check

#### 3. Request/Response Models
- Serde for serialization/deserialization
- Full JSON schema compatibility with OpenAI API
- Support for optional fields and extensions

#### 4. Configuration System
- Command-line arguments via Clap
- Environment variable support
- Configuration profiles for different test scenarios

#### 5. Response Generation
- Lorem ipsum text generation (via lipsum crate)
- Configurable token counts
- Realistic token distribution (configurable normal distribution for output tokens)
- Pre-generate 4096 mock articles with token distribution for fast response
- Only one copy per response data (avoid unnecessary allocations, zero-copy where possible)

#### 6. Logging & Observability
- Simple logging (stdout)

## Key Features

### Configurable Behavior

- Configurable output token distribution (normal distribution: mean and stddev)
- Respect `max_tokens` field in request body (if present, limit output tokens accordingly)
- Configurable streaming token count range (for each SSE event, send `<n>` tokens per event)

### Performance Optimizations

1. **Connection Pooling**: Reuse connections efficiently
2. **Zero-Copy Responses**: Stream responses directly to clients
3. **Memory Efficiency**: Pre-allocated buffers, minimal allocations
4. **Async I/O**: Full async/await stack, no blocking operations

### Benchmark Modes

- **Token Distribution**: Output token count per response is sampled from a configurable normal distribution (mean, stddev).
- **Streaming Token Count**: For streaming responses, each SSE event sends a configurable number of tokens (range).
- **Pre-generated Articles**: At startup, 4096 mock articles are generated with token distributions for fast, realistic responses.

#### Throughput Mode
- Fast responses with minimal processing
- Realistic token generation
- Perfect for measuring client throughput limits

#### Latency Mode
- Variable response times following distribution patterns
- Realistic network jitter simulation
- Percentile-based performance tracking

#### Load Mode
- Configurable concurrent request limits
- Resource constraint simulation
- Connection queue management

## Implementation Details

### Request Flow

```
Client Request
    ↓
Load Balancer / Router
    ↓
Request Validation
    ↓
Response Simulation (delay, tokens)
    ↓
Response Generation
    ↓
Streaming or Direct Response
    ↓
Client
```

### Data Structures

#### ChatCompletionRequest
```json
{
  "model": "gpt-4",
  "messages": [
    { "role": "user", "content": "..." }
  ],
  "temperature": 0.7,
  "max_tokens": 2048,
  "stream": false
}
```

#### ChatCompletionResponse
```json
{
  "id": "chatcmpl-...",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "gpt-4",
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 20,
    "total_tokens": 30
  },
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "..."
      },
      "finish_reason": "stop"
    }
  ]
}
```

### Streaming Response (SSE)

```
data: {"choices":[{"delta":{"content":"Hello"},...}]}
data: {"choices":[{"delta":{"content":" world"},...}]}
data: [DONE]
```

## Deployment

### Requirements
- Rust 1.70+
- 2GB RAM minimum
- Single CPU core minimum (scales to multi-core)

### Build
```bash
cargo build --release
```

### Run
```bash
./target/release/mock-openai --port 3000 --response-delay 10
```

### Docker Support (Future)
- Multi-stage build for minimal image size
- Health check configuration
- Signal handling for graceful shutdown

## Testing Strategy

### Unit Tests
- Request/response parsing
- Token generation logic
- Configuration validation

### Integration Tests
- Full request/response cycle
- Streaming functionality
- Error handling

### Load Tests
- Apache Bench / wrk for throughput
- Configurable concurrency levels
- Response time distribution analysis

## Security Considerations

1. **Rate Limiting**: Optional per-IP request limits
2. **Input Validation**: Strict schema validation
3. **Error Messages**: Non-leaking error responses
4. **No Real Data**: All responses are generated, no data storage
5. **CORS**: Configurable CORS headers for browser testing

## Future Enhancements

1. **TLS/SSL Support**: HTTPS endpoints
2. **Authentication**: Mock API key validation
3. **Request Logging**: Detailed request/response logging to files
4. **Custom Models**: User-defined model configurations

## Dependencies Rationale

| Dependency | Version | Purpose |
|------------|---------|---------|
| actix-web | 4.12.0 | HTTP framework - high performance, async-first |
| tokio | 1.48.0 | Async runtime - production-grade, well-tested |
| serde | 1.0.228 | Serialization - fast, with derive macros |
| serde_json | 1.0.145 | JSON handling - Serde's official JSON support |
| clap | 4.5.52 | CLI parsing - ergonomic, well-maintained |
| lipsum | 0.9.1 | Mock text generation - Lorem ipsum generation |

## Performance Targets

- **Throughput**: 10,000+ RPS on a single core
- **Latency (p50)**: <1ms response generation
- **Latency (p99)**: <10ms response generation
- **Memory**: <100MB for idle server
- **Concurrent Connections**: 10,000+
- **Pre-generated Articles**: 4096 articles with token distribution, used for fast response generation.
- **Zero-Copy**: Only one copy per response data, avoid unnecessary allocations.

## Monitoring & Debugging
### Logging

- Simple stdout logging for server lifecycle events and errors.
