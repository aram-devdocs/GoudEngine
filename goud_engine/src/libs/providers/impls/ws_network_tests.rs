use super::*;
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{Channel, ConnectionState, HostConfig, NetworkEvent};
use crate::core::providers::{Provider, ProviderLifecycle};
use std::fs;
use std::sync::Mutex;
use std::time::Duration;

static TLS_ENV_MUTEX: Mutex<()> = Mutex::new(());

fn host_config() -> HostConfig {
    HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 16,
        tls_cert_path: None,
        tls_key_path: None,
    }
}

#[test]
fn test_ws_construction() {
    let provider = WsNetProvider::new();
    assert_eq!(provider.name(), "websocket");
    assert_eq!(provider.version(), "0.1.0");
    assert!(provider.connections().is_empty());
    assert!(provider.local_addr().is_none());

    let caps = provider.network_capabilities();
    assert!(caps.supports_hosting);
    assert_eq!(caps.max_connections, 64);
    assert_eq!(caps.max_channels, 1);
    assert_eq!(caps.max_message_size, 16_777_216);

    let stats = provider.stats();
    assert_eq!(stats.bytes_sent, 0);
    assert_eq!(stats.packets_sent, 0);

    let debug = format!("{:?}", provider);
    assert!(debug.contains("WsNetProvider"));
}

#[test]
fn test_ws_host_and_connect() {
    let mut host = WsNetProvider::new();
    let config = host_config();
    host.host(&config).unwrap();
    assert!(host.local_addr().is_some());

    // Hosting again should fail.
    assert!(host.host(&config).is_err());

    let addr = host.local_addr().unwrap();

    let mut client = WsNetProvider::new();
    let conn_id = client.connect(&format!("ws://{}", addr)).unwrap();
    assert_eq!(
        client.connection_state(conn_id),
        ConnectionState::Connecting
    );

    // Wait for connection to establish.
    let mut host_connected = false;
    let mut client_connected = false;
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();

        if !host_connected {
            let events = host.drain_events();
            if events
                .iter()
                .any(|e| matches!(e, NetworkEvent::Connected { .. }))
            {
                host_connected = true;
            }
        }
        if client.connection_state(conn_id) == ConnectionState::Connected {
            client_connected = true;
        }
        if host_connected && client_connected {
            break;
        }
    }

    assert!(host_connected, "Host should have received Connected event");
    assert!(client_connected, "Client should be Connected");

    client.drain_events();
    assert_eq!(client.connection_state(conn_id), ConnectionState::Connected);

    host.shutdown();
    client.shutdown();
}

#[test]
fn test_ws_send_receive() {
    let mut host = WsNetProvider::new();
    let config = host_config();
    host.host(&config).unwrap();
    let addr = host.local_addr().unwrap();

    let mut client = WsNetProvider::new();
    let conn_id = client.connect(&format!("ws://{}", addr)).unwrap();

    // Wait for connection.
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        host.drain_events();
        if client.connection_state(conn_id) == ConnectionState::Connected {
            client.drain_events();
            break;
        }
    }
    assert_eq!(
        client.connection_state(conn_id),
        ConnectionState::Connected,
        "Client should be connected before send"
    );

    // Client sends data to host.
    let payload = b"hello host";
    client.send(conn_id, Channel(0), payload).unwrap();

    // Host receives.
    let mut received_data = None;
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        host.update(0.0).unwrap();
        for event in host.drain_events() {
            if let NetworkEvent::Received { data, channel, .. } = event {
                assert_eq!(channel, Channel(0));
                received_data = Some(data);
            }
        }
        if received_data.is_some() {
            break;
        }
    }
    assert_eq!(
        received_data.as_deref(),
        Some(payload.as_slice()),
        "Host should have received the payload"
    );

    // Host sends data back to client.
    let host_conn = host.connections().into_iter().next().unwrap();
    let reply = b"hello client";
    host.send(host_conn, Channel(0), reply).unwrap();

    let mut reply_data = None;
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        client.update(0.0).unwrap();
        for event in client.drain_events() {
            if let NetworkEvent::Received { data, .. } = event {
                reply_data = Some(data);
            }
        }
        if reply_data.is_some() {
            break;
        }
    }
    assert_eq!(
        reply_data.as_deref(),
        Some(reply.as_slice()),
        "Client should have received the reply"
    );

    host.shutdown();
    client.shutdown();
}

#[test]
fn test_ws_disconnect() {
    let mut host = WsNetProvider::new();
    let config = host_config();
    host.host(&config).unwrap();
    let addr = host.local_addr().unwrap();

    let mut client = WsNetProvider::new();
    let conn_id = client.connect(&format!("ws://{}", addr)).unwrap();

    // Wait for connection.
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        host.drain_events();
        if client.connection_state(conn_id) == ConnectionState::Connected {
            client.drain_events();
            break;
        }
    }
    assert_eq!(client.connection_state(conn_id), ConnectionState::Connected);

    // Client disconnects.
    client.disconnect(conn_id).unwrap();
    let events = client.drain_events();
    assert!(
        events.iter().any(|e| matches!(
            e,
            NetworkEvent::Disconnected {
                reason: crate::core::providers::network_types::DisconnectReason::LocalClose,
                ..
            }
        )),
        "Client should emit LocalClose disconnect event"
    );

    assert_eq!(
        client.connection_state(conn_id),
        ConnectionState::Disconnected
    );

    host.shutdown();
    client.shutdown();
}

#[test]
fn test_ws_host_rejects_partial_tls_config() {
    let mut host = WsNetProvider::new();
    let mut config = host_config();
    config.tls_cert_path = Some("cert.pem".to_string());
    let err = host
        .host(&config)
        .expect_err("host should reject a TLS cert without key");
    assert!(format!("{}", err).contains("tls_cert_path and tls_key_path"));

    let mut host = WsNetProvider::new();
    let mut config = host_config();
    config.tls_key_path = Some("key.pem".to_string());
    let err = host
        .host(&config)
        .expect_err("host should reject a TLS key without cert");
    assert!(format!("{}", err).contains("tls_cert_path and tls_key_path"));
}

#[test]
fn test_wss_host_and_connect_with_custom_ca() {
    let _guard = TLS_ENV_MUTEX.lock().expect("mutex should lock");
    let cert_dir = tempfile::tempdir().expect("tempdir should be created");
    let cert_path = cert_dir.path().join("localhost-cert.pem");
    let key_path = cert_dir.path().join("localhost-key.pem");

    let key_pair = rcgen::KeyPair::generate().expect("key pair should generate");
    let cert = rcgen::CertificateParams::new(vec!["localhost".to_string()])
        .expect("certificate params should generate")
        .self_signed(&key_pair)
        .expect("self-signed cert should generate");
    fs::write(&cert_path, cert.pem()).expect("cert file should be written");
    fs::write(&key_path, key_pair.serialize_pem()).expect("key file should be written");

    let mut host = WsNetProvider::new();
    let mut config = host_config();
    config.tls_cert_path = Some(cert_path.to_string_lossy().into_owned());
    config.tls_key_path = Some(key_path.to_string_lossy().into_owned());
    host.host(&config).unwrap();
    let addr = host.local_addr().expect("host should have local address");
    let client_url = format!("wss://localhost:{}", addr.port());

    std::env::set_var("GOUD_WS_CA_CERT_PATH", cert_path.to_string_lossy().as_ref());
    let mut client = WsNetProvider::new();
    let conn_id = client.connect(&client_url).unwrap();

    let mut host_connected = false;
    let mut client_connected = false;
    for _ in 0..120 {
        std::thread::sleep(Duration::from_millis(50));
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        host_connected |= host
            .drain_events()
            .into_iter()
            .any(|e| matches!(e, NetworkEvent::Connected { .. }));
        client_connected = client.connection_state(conn_id) == ConnectionState::Connected;
        if host_connected && client_connected {
            break;
        }
    }

    let client_events = client.drain_events();
    assert!(
        host_connected,
        "host should observe wss connection; client_state={:?}; client_events={:?}",
        client.connection_state(conn_id),
        client_events
    );
    assert!(
        client_connected,
        "client should connect over wss; client_state={:?}; client_events={:?}",
        client.connection_state(conn_id),
        client_events
    );

    client.send(conn_id, Channel(0), b"hello over wss").unwrap();
    let mut received = false;
    for _ in 0..120 {
        std::thread::sleep(Duration::from_millis(25));
        host.update(0.0).unwrap();
        received |= host.drain_events().into_iter().any(|event| {
            matches!(
                event,
                NetworkEvent::Received { ref data, .. } if data == b"hello over wss"
            )
        });
        if received {
            break;
        }
    }
    assert!(received, "host should receive payload over wss");

    std::env::remove_var("GOUD_WS_CA_CERT_PATH");
    host.shutdown();
    client.shutdown();
}
