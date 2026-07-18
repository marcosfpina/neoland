//! TLS / mTLS configuration and loading.
//!
//! Provides certificate loading, rustls configuration for both server-side
//! (TLS) and client-side (mTLS) connections.
//!
//! Certificate paths are configured via environment variables:
//!   NEOLAND_TLS_CA_CERT        — CA certificate (PEM)
//!   NEOLAND_TLS_SERVER_CERT    — Server certificate (PEM)
//!   NEOLAND_TLS_SERVER_KEY     — Server private key (PEM)
//!   NEOLAND_TLS_CLIENT_CERT    — Client certificate for mTLS (PEM)
//!   NEOLAND_TLS_CLIENT_KEY     — Client private key for mTLS (PEM)

use anyhow::{Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::sync::Arc;

// ── TLS Configuration ─────────────────────────────────────────────────────

/// Holds loaded TLS certificates and keys for the server.
#[derive(Clone)]
pub struct TlsConfig {
    /// Server certificate chain (DER format).
    pub certs: Vec<CertificateDer<'static>>,
    /// Server certificate chain (raw PEM bytes — tonic Identity requer PEM).
    pub certs_pem: Vec<u8>,
    /// Server private key (DER bytes).
    pub key_der: Vec<u8>,
    /// Optional CA certificate for mutual TLS (client verification).
    pub ca_der: Option<Vec<u8>>,
}

impl TlsConfig {
    /// Load TLS certificates from environment variables or default paths.
    pub fn from_env() -> Result<Option<Self>> {
        let ca_path = std::env::var("NEOLAND_TLS_CA_CERT").ok();
        let cert_path = std::env::var("NEOLAND_TLS_SERVER_CERT").ok();
        let key_path = std::env::var("NEOLAND_TLS_SERVER_KEY").ok();

        match (cert_path, key_path) {
            (Some(cert), Some(key)) => {
                tracing::info!(ca = ?ca_path, cert = %cert, key = %key, "🔐 TLS enabled");
                Self::load(&cert, &key, ca_path.as_deref())
            },
            _ => {
                tracing::info!("🔓 TLS not configured — running without encryption");
                Ok(None)
            },
        }
    }

    /// Load certificates from explicit paths.
    pub fn load(cert_path: &str, key_path: &str, ca_path: Option<&str>) -> Result<Option<Self>> {
        let certs = load_certificates(cert_path)?;
        let certs_pem = std::fs::read(cert_path)
            .with_context(|| format!("Failed to read certificate file: {cert_path}"))?;
        let key_der = std::fs::read(key_path)
            .with_context(|| format!("Failed to read key file: {key_path}"))?;
        let ca_der = ca_path
            .map(|p| std::fs::read(p).with_context(|| format!("Failed to read CA: {p}")))
            .transpose()?;

        Ok(Some(Self { certs, certs_pem, key_der, ca_der }))
    }

    pub fn is_mtls(&self) -> bool {
        self.ca_der.is_some()
    }

    pub fn server_config(&self) -> Result<rustls::ServerConfig> {
        let key = load_private_key_from_der(&self.key_der)?;
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(self.certs.clone(), key)
            .context("Failed to build TLS server config")?;
        Ok(config)
    }

    /// ServerConfig com verificação de certificado de cliente quando um CA
    /// está configurado (mTLS); sem CA degrada para TLS simples.
    pub fn mtls_server_config(&self) -> Result<rustls::ServerConfig> {
        if let Some(ref ca_der) = self.ca_der {
            let ca = load_ca_certificate_from_der(ca_der)?;
            let mut roots = rustls::RootCertStore::empty();
            roots.add(ca).context("Failed to add CA to root store")?;

            let key = load_private_key_from_der(&self.key_der)?;
            let verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(roots))
                .build()
                .context("Failed to build client cert verifier")?;

            rustls::ServerConfig::builder()
                .with_client_cert_verifier(verifier)
                .with_single_cert(self.certs.clone(), key)
                .context("Failed to build gRPC mTLS server config")
        } else {
            self.server_config()
        }
    }

    pub fn client_config(
        ca_path: Option<&str>,
        _client_cert: Option<&str>,
        _client_key: Option<&str>,
    ) -> Result<rustls::ClientConfig> {
        let mut roots = rustls::RootCertStore::empty();

        // Add system root CAs
        let native = rustls_native_certs::load_native_certs();
        for cert in native.certs {
            roots.add(cert).ok();
        }
        // Log any errors loading native certs (non-fatal)
        for err in &native.errors {
            tracing::warn!("Failed to load native CA: {err}");
        }

        // Add our custom CA if provided
        if let Some(ca) = ca_path {
            let ca_cert = load_ca_certificate(ca)?;
            roots.add(ca_cert).context("Failed to add custom CA to root store")?;
        }

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        Ok(config)
    }
}

// ── Certificate loading helpers ───────────────────────────────────────────

/// Load a PrivateKeyDer from raw DER bytes.
fn load_private_key_from_der(der: &[u8]) -> Result<PrivateKeyDer<'static>> {
    let mut slice = der;
    if let Some(key) = rustls_pemfile::pkcs8_private_keys(&mut slice).next() {
        return Ok(PrivateKeyDer::Pkcs8(key.map_err(|e| anyhow::anyhow!("PKCS8: {e}"))?));
    }
    let mut slice = der;
    if let Some(key) = rustls_pemfile::rsa_private_keys(&mut slice).next() {
        return Ok(PrivateKeyDer::Pkcs1(key.map_err(|e| anyhow::anyhow!("RSA: {e}"))?));
    }
    anyhow::bail!("Failed to parse private key from DER bytes")
}

/// Load a CertificateDer from raw DER bytes.
fn load_ca_certificate_from_der(der: &[u8]) -> Result<CertificateDer<'static>> {
    let mut slice = der;
    if let Some(cert) = rustls_pemfile::certs(&mut slice).next() {
        return cert.map_err(|e| anyhow::anyhow!("CA cert: {e}"));
    }
    anyhow::bail!("Failed to parse CA certificate from DER bytes")
}

fn load_ca_certificate(path: &str) -> Result<CertificateDer<'static>> {
    let pem = std::fs::read(path).with_context(|| format!("Failed to read CA: {path}"))?;
    load_ca_certificate_from_der(&pem)
}

fn load_certificates(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let pem =
        std::fs::read(path).with_context(|| format!("Failed to read certificate file: {path}"))?;

    let mut certs = Vec::new();
    let mut slice = pem.as_slice();
    for cert in rustls_pemfile::certs(&mut slice) {
        certs.push(cert.map_err(|e| anyhow::anyhow!("PEM cert: {e}"))?);
    }

    if certs.is_empty() {
        anyhow::bail!("No certificates found in {path}");
    }

    Ok(certs)
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_tls_when_unconfigured() {
        // Clear any TLS env vars
        std::env::remove_var("NEOLAND_TLS_SERVER_CERT");
        std::env::remove_var("NEOLAND_TLS_SERVER_KEY");
        let config = TlsConfig::from_env().expect("from_env");
        assert!(config.is_none(), "Should be None when env vars are not set");
    }
}
