//! TLS configuration utilities for HTTPS/HTTP2 support

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::fs;
use std::io::BufReader;
use std::path::Path;

/// Load TLS certificate and private key from PEM files
///
/// # Arguments
/// * `cert_path` - Path to the certificate file (PEM format)
/// * `key_path` - Path to the private key file (PEM format)
///
/// # Returns
/// A tuple of (certificates, private key) or an error
pub fn load_tls_config(
    cert_path: &Path,
    key_path: &Path,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), Box<dyn std::error::Error>> {
    // Load certificate chain
    let cert_file = fs::File::open(cert_path)?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;

    if certs.is_empty() {
        return Err("No certificates found in cert file".into());
    }

    // Load private key
    let key_file = fs::File::open(key_path)?;
    let mut key_reader = BufReader::new(key_file);
    let keys = rustls_pemfile::private_key(&mut key_reader)?;

    let key = keys.ok_or("No private key found in key file")?;

    Ok((certs, key))
}
