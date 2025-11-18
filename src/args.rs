//! CLI argument definitions and environment variable handling

use clap::Parser;
use std::path::PathBuf;

/// CLI arguments for the server
#[derive(Parser, Debug)]
#[command(author, version, about = "Mock OpenAI API server for benchmarking")]
pub struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// Number of pre-generated articles
    #[arg(long, default_value_t = 4096)]
    pub pregen_count: usize,

    /// Mean tokens per generated response
    #[arg(long, default_value_t = 256.0)]
    pub token_mean: f64,

    /// Standard deviation for tokens per response
    #[arg(long, default_value_t = 64.0)]
    pub token_stddev: f64,

    /// Delay in milliseconds per SSE event to emulate network latency
    #[arg(long, default_value_t = 0)]
    pub response_delay_ms: u64,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Path to TLS certificate file (PEM format) for HTTPS/HTTP2 support
    #[arg(long)]
    pub tls_cert: Option<PathBuf>,

    /// Path to TLS private key file (PEM format) for HTTPS/HTTP2 support
    #[arg(long)]
    pub tls_key: Option<PathBuf>,
}

impl Args {
    /// Apply overrides using environment variables.
    ///
    /// This helper reads well-known environment variables and applies them to
    /// the provided `Args` instance. Values that don't parse correctly are
    /// ignored to preserve defaults.
    pub fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("MOCK_OPENAI_PORT") {
            if let Ok(v) = val.parse::<u16>() {
                self.port = v;
            }
        }
        if let Ok(val) = std::env::var("MOCK_OPENAI_PREG_COUNT") {
            if let Ok(v) = val.parse::<usize>() {
                self.pregen_count = v;
            }
        }
        if let Ok(val) = std::env::var("MOCK_OPENAI_TOKEN_MEAN") {
            if let Ok(v) = val.parse::<f64>() {
                self.token_mean = v;
            }
        }
        if let Ok(val) = std::env::var("MOCK_OPENAI_TOKEN_STDDEV") {
            if let Ok(v) = val.parse::<f64>() {
                self.token_stddev = v;
            }
        }
        if let Ok(val) = std::env::var("MOCK_OPENAI_RESPONSE_DELAY_MS") {
            if let Ok(v) = val.parse::<u64>() {
                self.response_delay_ms = v;
            }
        }
        if let Ok(val) = std::env::var("MOCK_OPENAI_VERBOSE") {
            // Accept `true`/`false` or `1`/`0` for compatibility
            if let Ok(v) = val.parse::<bool>() {
                self.verbose = v;
            } else if val == "1" {
                self.verbose = true;
            } else if val == "0" {
                self.verbose = false;
            }
        }
        if let Ok(val) = std::env::var("MOCK_OPENAI_TLS_CERT") {
            self.tls_cert = Some(PathBuf::from(val));
        }
        if let Ok(val) = std::env::var("MOCK_OPENAI_TLS_KEY") {
            self.tls_key = Some(PathBuf::from(val));
        }
    }

    /// Validate that both TLS cert and key are provided if either is specified
    pub fn validate_tls_config(&self) -> Result<(), String> {
        let cert_provided = self.tls_cert.is_some();
        let key_provided = self.tls_key.is_some();

        if cert_provided != key_provided {
            return Err(
                "Both --tls-cert and --tls-key must be provided together for HTTPS support"
                    .to_string(),
            );
        }

        Ok(())
    }
}
