# mock-openai

A high-performance mock OpenAI API server compatible with a subset of OpenAI/OpenRouter endpoints. It is designed for benchmarking and local development. The server simulates completions, chat completions (streaming and non-streaming), embeddings, and model listing endpoints with configurable token distributions, streaming behavior, and artificial latency.

## Features

- **High Performance**: Pre-generated mock articles and optimized token sampling
- **Streaming Support**: Server-Sent Events (SSE) for realistic streaming responses
- **HTTP/2 Support**: Full HTTP/2 with TLS certificates for high-performance benchmarking
- **Configurable Behavior**: Token distributions, response delays, and more
- **Environment Variable Overrides**: Easy configuration via env vars or CLI args

## Quick Start

Build (release):
```bash
cargo build --release
```

Run (HTTP mode):
```bash
./target/release/mock-openai --port 3000
```

Run (HTTPS/HTTP2 mode):
```bash
# First, generate self-signed certificates for testing
openssl genrsa -out key.pem 2048
openssl req -new -x509 -key key.pem -out cert.pem -days 365

# Then run with TLS
./target/release/mock-openai --port 3000 --tls-cert cert.pem --tls-key key.pem
```

## Usage

```
Mock OpenAI API server for benchmarking

Usage: mock-openai [OPTIONS]

Options:
  -p, --port <PORT>
          Port to listen on [default: 3000]
      --pregen-count <PREGEN_COUNT>
          Number of pre-generated articles [default: 4096]
      --token-mean <TOKEN_MEAN>
          Mean tokens per generated response [default: 256]
      --token-stddev <TOKEN_STDDEV>
          Standard deviation for tokens per response [default: 64]
      --response-delay-ms <RESPONSE_DELAY_MS>
          Delay in milliseconds per SSE event to emulate network latency [default: 0]
      --tls-cert <TLS_CERT>
          Path to TLS certificate file (PEM format) for HTTPS/HTTP2 support
      --tls-key <TLS_KEY>
          Path to TLS private key file (PEM format) for HTTPS/HTTP2 support
  -v, --verbose
          Verbose output
  -h, --help
          Print help
  -V, --version
          Print version
```

> [!TIP] The server also supports environment variable overrides: `MOCK_OPENAI_PORT`, `MOCK_OPENAI_PREG_COUNT`, `MOCK_OPENAI_TOKEN_MEAN`, `MOCK_OPENAI_TOKEN_STDDEV`, `MOCK_OPENAI_RESPONSE_DELAY_MS`, `MOCK_OPENAI_VERBOSE`, `MOCK_OPENAI_TLS_CERT`, `MOCK_OPENAI_TLS_KEY`.

## API Endpoints

All endpoints are mounted at the server root. By default the server listens on `http://127.0.0.1:3000` (or `https://127.0.0.1:3000` with TLS).

- GET /health
- GET /v1/models
- GET /v1/models/{id}
- POST /v1/completions
- POST /v1/chat/completions
- POST /v1/embeddings

## HTTP/2 Support

The server now supports HTTP/2 over HTTPS/TLS for high-performance benchmarking scenarios.

### Enable HTTP/2

Provide TLS certificate and key files:

```bash
./target/release/mock-openai --port 3000 \
  --tls-cert /path/to/cert.pem \
  --tls-key /path/to/key.pem
```

The server will:
- Configure TLS 1.3 for strong encryption
- Enable HTTP/2 via ALPN protocol negotiation
- Fall back to HTTP/1.1 for clients that don't support HTTP/2

### Testing HTTP/2

```bash
# Using curl with HTTP/2
curl --http2 https://localhost:3000/health --insecure

# Check which protocol is being used
curl --http2 https://localhost:3000/health --insecure -v
```

For more details, see [HTTP2.md](./HTTP2.md).

## Behavior & Configuration

- The server pre-generates a pool of mock articles (default 4096) at startup to avoid runtime allocation overhead and mimic realistic text for responses.
- Token/character mapping:
  - The project approximates 1 token ≈ 4 characters (`AVG_CHARS_PER_TOKEN = 4`), used for slicing pre-generated articles.
- Tokens per response are sampled from a normal distribution using the configured mean and standard deviation. Passing `max_tokens` in the request caps the emitted tokens.
- For SSE streaming, each event will optionally wait `--response-delay-ms` milliseconds between chunks to emulate network latency.

## Testing

Run unit tests:
```bash
cargo test --all
```

## Performance Notes

- **HTTP/1.1**: Good for single-threaded benchmarks and simple load tests
- **HTTP/2**: Better for concurrent request scenarios with multiplexing benefits
- Pre-generated articles and token samples eliminate per-request allocation overhead
- Jemalloc is used as the global allocator for better performance

## Production Considerations

For production deployment with HTTP/2:

1. Use certificates from a trusted Certificate Authority, not self-signed certificates
2. Implement certificate rotation and monitoring
3. Consider rate limiting and authentication
4. Use appropriate TLS configuration for your security requirements

See [HTTP2.md](./HTTP2.md) for detailed production guidance.