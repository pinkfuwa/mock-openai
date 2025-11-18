# HTTP/2 Support

This document describes the HTTP/2 and HTTPS support added to mock-openai.

## Overview

mock-openai now supports HTTP/2 over HTTPS (TLS 1.3) for high-performance benchmarking scenarios. HTTP/2 enables:

- **Multiplexing**: Multiple concurrent requests over a single connection
- **Header Compression**: Reduced overhead for headers
- **Server Push**: Potential for proactive resource delivery
- **Binary Framing**: More efficient protocol parsing
- **Flow Control**: Better resource management

## Quick Start

### Generate Self-Signed Certificates (for testing)

```bash
# Generate a private key
openssl genrsa -out key.pem 2048

# Generate a self-signed certificate (valid for 365 days)
openssl req -new -x509 -key key.pem -out cert.pem -days 365
```

### Run with HTTP/2

```bash
# Using command-line arguments
./target/release/mock-openai --port 3000 --tls-cert cert.pem --tls-key key.pem

# Using environment variables
export MOCK_OPENAI_TLS_CERT=cert.pem
export MOCK_OPENAI_TLS_KEY=key.pem
./target/release/mock-openai --port 3000
```

### Test with HTTP/2

```bash
# Using curl with HTTP/2 support
curl --http2 https://localhost:3000/health --insecure

# Using h2 (curl alternative with better HTTP/2 support)
h2 https://localhost:3000/health --insecure
```

## Configuration

### CLI Arguments

```
--tls-cert <PATH>    Path to TLS certificate file (PEM format)
--tls-key <PATH>     Path to TLS private key file (PEM format)
```

Both arguments must be provided together to enable HTTPS/HTTP/2. If only one is provided, the server will exit with an error.

### Environment Variables

```
MOCK_OPENAI_TLS_CERT    Path to certificate file
MOCK_OPENAI_TLS_KEY     Path to private key file
```

Environment variables can be used instead of or in addition to CLI arguments.

## Technical Details

### Protocol Configuration

When TLS certificates are provided, the server configures:

- **TLS Version**: TLS 1.3 (required for optimal HTTP/2 support)
- **ALPN Protocols**: `h2` (HTTP/2) and `http/1.1` (fallback)
- **Client Authentication**: None (anonymous clients accepted)

The ALPN (Application Layer Protocol Negotiation) extension allows clients to negotiate HTTP/2 support during the TLS handshake.

### Certificate Requirements

The certificate file must be in **PEM format** and should contain:
- The server certificate
- Optionally, the certificate chain (intermediate CA certificates)

The private key file must be in **PEM format** and contain:
- The RSA private key corresponding to the certificate

### Port Configuration

The same `--port` argument is used for both HTTP and HTTPS:

```bash
# HTTP on port 3000
./target/release/mock-openai --port 3000

# HTTPS/HTTP/2 on port 3000
./target/release/mock-openai --port 3000 --tls-cert cert.pem --tls-key key.pem
```

## Production Considerations

### Certificate Management

For production use:

1. **Use Certificates from a Trusted CA**: Obtain certificates from a certificate authority (e.g., Let's Encrypt, DigiCert)
2. **Implement Certificate Rotation**: Periodically refresh certificates before expiration
3. **Monitor Certificate Expiration**: Set up alerts for expiring certificates
4. **Use Strong Key Sizes**: Minimum 2048-bit RSA (4096-bit recommended)

### Performance

HTTP/2 provides performance benefits for:

- **Multiple concurrent requests**: Multiplexing reduces connection overhead
- **Small requests**: Header compression becomes more effective
- **High-latency networks**: Better utilization of available bandwidth

For single large request scenarios, HTTP/2 overhead may not provide significant benefits over HTTP/1.1.

### Security

- **TLS 1.3**: Provides strong encryption and authentication
- **Perfect Forward Secrecy**: Supported by default with TLS 1.3
- **Certificate Pinning**: Can be implemented by clients to prevent MITM attacks
- **HSTS**: Consider implementing HSTS headers for production (requires reverse proxy)

## Troubleshooting

### Certificate Loading Errors

```
Failed to load TLS configuration: No certificates found in cert file
```

**Solution**: Verify the certificate file is in PEM format and contains valid certificate data.

```
Failed to load TLS configuration: No private key found in key file
```

**Solution**: Verify the private key file is in PEM format and contains the private key.

### Configuration Errors

```
Configuration error: Both --tls-cert and --tls-key must be provided together
```

**Solution**: Provide both certificate and key files, or provide neither for HTTP-only mode.

### Client Connection Issues

If clients can't connect with HTTP/2:

1. **Verify TLS Support**: Check that the client supports TLS 1.3
2. **Check ALPN Negotiation**: Use `openssl s_client` to debug:
   ```bash
   openssl s_client -connect localhost:3000 -alpn h2,http/1.1
   ```
3. **Certificate Validation**: For self-signed certs, clients must bypass validation (use `--insecure` with curl)

## Examples

### Complete Setup for Development

```bash
# Generate certificates
openssl genrsa -out dev-key.pem 2048
openssl req -new -x509 -key dev-key.pem -out dev-cert.pem -days 365 \
  -subj "/CN=localhost"

# Build release binary
cargo build --release

# Run with HTTP/2
./target/release/mock-openai \
  --port 3000 \
  --tls-cert dev-cert.pem \
  --tls-key dev-key.pem \
  --pregen-count 4096 \
  --response-delay-ms 10 \
  --verbose

# In another terminal, test
curl --http2 https://localhost:3000/health --insecure -v
```

### Load Testing with HTTP/2

```bash
# Using k6 with HTTP/2 support
k6 run - <<EOF
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  vus: 10,
  duration: '30s',
  insecureSkipTLSVerify: true,
};

export default function () {
  let response = http.get('https://localhost:3000/v1/models', {
    headers: { 'User-Agent': 'k6' },
  });
  check(response, {
    'status is 200': (r) => r.status === 200,
  });
}
EOF
```

### Environment Variable Setup

```bash
#!/bin/bash
# Setup script for HTTPS/HTTP/2

export MOCK_OPENAI_PORT=3000
export MOCK_OPENAI_TLS_CERT=/path/to/cert.pem
export MOCK_OPENAI_TLS_KEY=/path/to/key.pem
export MOCK_OPENAI_PREG_COUNT=4096
export MOCK_OPENAI_RESPONSE_DELAY_MS=10
export MOCK_OPENAI_VERBOSE=1

./target/release/mock-openai
```

## Comparison: HTTP vs HTTPS/HTTP/2

| Feature | HTTP | HTTPS/HTTP/2 |
|---------|------|--------------|
| Encryption | None | TLS 1.3 |
| Multiplexing | No (HTTP/1.1) | Yes |
| Header Compression | None | HPACK |
| Connection Setup | TCP 3-way | TCP + TLS handshake |
| Protocol Overhead | Low | Medium |
| Security | None | Strong |
| Suitable for | Local benchmarking | Production, remote testing |

## See Also

- [HTTP/2 Specification](https://http2.github.io/)
- [RFC 7540 - Hypertext Transfer Protocol Version 2](https://tools.ietf.org/html/rfc7540)
- [TLS 1.3 Specification](https://tools.ietf.org/html/rfc8446)
- [ALPN Specification](https://tools.ietf.org/html/rfc7301)