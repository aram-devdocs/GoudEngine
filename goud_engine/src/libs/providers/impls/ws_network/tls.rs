use std::sync::Arc;

use crate::libs::error::{GoudError, GoudResult};

fn net_err(msg: String) -> GoudError {
    GoudError::ProviderError {
        subsystem: "network",
        message: msg,
    }
}

pub fn load_server_tls_config(
    cert_path: &str,
    key_path: &str,
) -> GoudResult<Arc<rustls::ServerConfig>> {
    use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};

    let certs = CertificateDer::pem_file_iter(cert_path)
        .map_err(|e| net_err(format!("open TLS cert {}: {}", cert_path, e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| net_err(format!("parse TLS cert {}: {}", cert_path, e)))?;
    if certs.is_empty() {
        return Err(net_err(format!(
            "no certificates found in TLS cert file {}",
            cert_path
        )));
    }

    let key = PrivateKeyDer::from_pem_file(key_path)
        .map_err(|e| net_err(format!("parse TLS key {}: {}", key_path, e)))?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| net_err(format!("build TLS server config: {}", e)))?;
    Ok(Arc::new(config))
}

fn build_rustls_client_config_with_custom_ca(
    ca_cert_path: &str,
) -> GoudResult<Arc<rustls::ClientConfig>> {
    use rustls::pki_types::{pem::PemObject, CertificateDer};

    let ca_certs = CertificateDer::pem_file_iter(ca_cert_path)
        .map_err(|e| net_err(format!("open custom CA cert {}: {}", ca_cert_path, e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| net_err(format!("parse custom CA cert {}: {}", ca_cert_path, e)))?;
    if ca_certs.is_empty() {
        return Err(net_err(format!(
            "no certificates found in custom CA file {}",
            ca_cert_path
        )));
    }

    let mut roots = rustls::RootCertStore::empty();
    let (added, _ignored) = roots.add_parsable_certificates(ca_certs);
    if added == 0 {
        return Err(net_err(format!(
            "failed to add custom CA certs from {}",
            ca_cert_path
        )));
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

pub fn connect_with_optional_custom_ca(
    url: &str,
) -> Result<
    (
        tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>,
        tungstenite::handshake::client::Response,
    ),
    Box<tungstenite::Error>,
> {
    let custom_ca_path = match std::env::var("GOUD_WS_CA_CERT_PATH") {
        Ok(v) if !v.trim().is_empty() => Some(v),
        _ => None,
    };

    if url.starts_with("wss://") {
        if let Some(ca_path) = custom_ca_path {
            use std::net::ToSocketAddrs;
            use tungstenite::client::IntoClientRequest;

            let request = url.into_client_request()?;
            let uri = request.uri();
            let host = uri.host().ok_or(tungstenite::Error::Url(
                tungstenite::error::UrlError::NoHostName,
            ))?;
            let port = uri.port_u16().unwrap_or(443);
            let addrs = (host, port)
                .to_socket_addrs()
                .map_err(|e| Box::new(tungstenite::Error::Io(e)))?;
            let stream = addrs
                .into_iter()
                .find_map(|addr| std::net::TcpStream::connect(addr).ok())
                .ok_or_else(|| {
                    tungstenite::Error::Url(tungstenite::error::UrlError::UnableToConnect(
                        uri.to_string(),
                    ))
                })?;
            let tls_config = build_rustls_client_config_with_custom_ca(&ca_path)
                .map_err(|e| tungstenite::Error::Io(std::io::Error::other(format!("{}", e))))?;
            return tungstenite::client_tls_with_config(
                request,
                stream,
                None,
                Some(tungstenite::Connector::Rustls(tls_config)),
            )
            .map_err(|e| match e {
                tungstenite::HandshakeError::Failure(f) => f,
                tungstenite::HandshakeError::Interrupted(_) => tungstenite::Error::Io(
                    std::io::Error::other("TLS handshake interrupted unexpectedly"),
                ),
            })
            .map_err(Box::new);
        }
    }

    tungstenite::connect(url).map_err(Box::new)
}
