use super::*;

fn host_config() -> HostConfig {
    HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    }
}

#[test]
fn test_tcp_construction() {
    let provider = TcpNetProvider::new();
    assert_eq!(provider.name(), "tcp");
    assert_eq!(provider.version(), "0.1.0");
    assert!(provider.connections().is_empty());
    assert!(provider.local_addr().is_none());

    let caps = provider.network_capabilities();
    assert!(caps.supports_hosting);
    assert_eq!(caps.max_connections, 64);
    assert_eq!(caps.max_message_size, 16_777_216);

    let stats = provider.stats();
    assert_eq!(stats.bytes_sent, 0);
    assert_eq!(stats.bytes_received, 0);
    assert_eq!(stats.packet_loss_percent, 0.0);
    assert_eq!(stats.jitter_ms, 0.0);
}

#[test]
fn test_tcp_lifecycle() {
    let mut provider = TcpNetProvider::new();
    assert!(provider.init().is_ok());
    assert!(provider.update(0.0).is_ok());
    provider.shutdown();
}

#[test]
fn test_tcp_disconnect_emits_local_close() {
    let mut host = TcpNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().unwrap();

    let mut client = TcpNetProvider::new();
    let client_conn = client.connect(&host_addr.to_string()).unwrap();

    for _ in 0..100 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        host.drain_events();
        if client.connection_state(client_conn) == ConnectionState::Connected {
            client.drain_events();
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert_eq!(client.connection_state(client_conn), ConnectionState::Connected);
    client.disconnect(client_conn).unwrap();

    let events = client.drain_events();
    assert!(events.iter().any(|event| matches!(
        event,
        NetworkEvent::Disconnected {
            reason: crate::core::providers::network_types::DisconnectReason::LocalClose,
            ..
        }
    )));

    host.shutdown();
    client.shutdown();
}
