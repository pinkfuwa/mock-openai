# mock-openai

A high-performance mock OpenAI API server compatible with a subset of OpenAI/OpenRouter endpoints. It is designed for benchmarking and local development. The server simulates completions, chat completions (streaming and non-streaming), embeddings, and model listing endpoints with configurable token distributions, streaming behavior, and artificial latency.

## Quick Start

Build (release):
```
$ ./mock-openai
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
      --stream-tokens-min <STREAM_TOKENS_MIN>
          Min tokens per SSE event when streaming [default: 1]
      --stream-tokens-max <STREAM_TOKENS_MAX>
          Max tokens per SSE event when streaming [default: 64]
      --response-delay-ms <RESPONSE_DELAY_MS>
          Delay in milliseconds per SSE event to emulate network latency [default: 0]
  -v, --verbose
          Verbose output
  -h, --help
          Print help
  -V, --version
          Print version
```

> [!TIP] The server also supports environment variable overrides: `MOCK_OPENAI_PORT`, `MOCK_OPENAI_PREG_COUNT`, `MOCK_OPENAI_TOKEN_MEAN`, `MOCK_OPENAI_TOKEN_STDDEV`, `MOCK_OPENAI_STREAM_TOKENS_MIN`, `MOCK_OPENAI_STREAM_TOKENS_MAX`, `MOCK_OPENAI_RESPONSE_DELAY_MS`, `MOCK_OPENAI_VERBOSE`.

## API Endpoints

All endpoints are mounted at the server root. By default the server listens on `http://127.0.0.1:3000`.

- GET /health
- GET /v1/models
- GET /v1/models/{id}
- POST /v1/completions
- POST /v1/chat/completions
- POST /v1/embeddings

## Behavior & Configuration

- The server pre-generates a pool of mock articles (default 4096) at startup to avoid runtime allocation overhead and mimic realistic text for responses.
- Token/character mapping:
  - The project approximates 1 token ≈ 4 characters (`AVG_CHARS_PER_TOKEN = 4`), used for slicing pre-generated articles.
- Tokens per response are sampled from a normal distribution using the configured mean and standard deviation. Passing `max_tokens` in the request caps the emitted tokens.
- For SSE streaming, each event will emit between `--stream-tokens-min` and `--stream-tokens-max` tokens and optionally wait `--response-delay-ms` milliseconds between chunks.

## Testing

Run unit tests:
```bash
cargo test --all
```
