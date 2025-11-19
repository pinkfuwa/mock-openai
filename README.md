# mock-openai 🤖

### A high-performance mock OpenAI API server for development & benchmarking

A drop-in replacement for OpenAI/OpenRouter endpoints, designed for ultra-fast local development and load testing. Simulates completions, chat, embeddings, and model endpoints with configurable latency and streaming behavior.

## ✨ Features

- ⚡ **Blazing Fast**: 70,000+ requests/sec ([streaming chat completions on i5-1240p](./report.png))
- 🔄 **Realistic Streaming**: Full Server-Sent Events (SSE) support
- 🔒 **HTTP/2 & TLS**: Production-grade benchmarking capabilities
- 🎛️ **Highly Configurable**: Token distributions, artificial latency, pool size
- 📦 **Easy to Deploy**: Single binary, environment variables or commandline arguments

## 📦 Installation

### Prerequisites

- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- OpenSSL (for TLS certificate generation)

### Build from source

```bash
git clone https://github.com/pinkfuwa/mock-openai.git
cd mock-openai
cargo build --release
```

The binary will be at `./target/release/mock-openai`.

---

## 🚀 Quick Start

### HTTP Mode (Development)

```bash
./target/release/mock-openai --port 3000
```

### HTTPS/HTTP2 Mode (Benchmarking)

Generate certificates (one-time setup):
```bash
openssl genrsa -out key.pem 2048
openssl req -new -x509 -key key.pem -out cert.pem -days 365
```

Run with TLS:
```bash
./target/release/mock-openai --port 3000 --tls-cert cert.pem --tls-key key.pem
```

✅ **Done!** Your server is ready at `http://127.0.0.1:3000` (or `https://` with TLS)

---

## 💻 API Usage

All endpoints are mounted at the server root. By default the server listens on `http://127.0.0.1:3000` (or `https://127.0.0.1:3000` with TLS).

- GET /health
- GET /v1/models
- GET /v1/models/{id}
- POST /v1/completions
- POST /v1/chat/completions
- POST /v1/embeddings

---

## ⚙️ Configuration

### CLI Options

| Option | Default | Description |
|--------|---------|-------------|
| `-p, --port` | 3000 | Server port |
| `--pregen-count` | 4096 | Size of pre-generated content pool |
| `--token-mean` | 256 | Average tokens per response |
| `--token-stddev` | 64 | Token count standard deviation |
| `--response-delay-ms` | 0 | Artificial latency between SSE chunks (ms) |
| `--tls-cert` | - | Path to TLS certificate (PEM) |
| `--tls-key` | - | Path to TLS private key (PEM) |
| `-v, --verbose` | false | Enable debug logging |

### Environment Variables

All CLI options can be set via env vars (useful for Docker):

```bash
export MOCK_OPENAI_PORT=8080
export MOCK_OPENAI_TOKEN_MEAN=512
export MOCK_OPENAI_RESPONSE_DELAY_MS=50
export MOCK_OPENAI_VERBOSE=1

./target/release/mock-openai
```

> [!TIP]
> Environment variables take precedence over CLI arguments. Great for containerized deployments!

---

## 📊 Benchmarking

Built-in Criterion benchmarks for comprehensive performance testing:

```bash
cargo bench --bench benchmark_endpoints
```

---

**Happy mocking!** 🎉
