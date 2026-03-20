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
    assert_eq!(caps.max_message_size, 16_777_215);

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

    assert_eq!(
        client.connection_state(client_conn),
        ConnectionState::Connected
    );
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

#[test]
fn test_tcp_reconnect_config_defaults() {
    let config = ReconnectConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.max_attempts, MAX_RECONNECT_ATTEMPTS);
    assert_eq!(config.delay, DEFAULT_RECONNECT_DELAY);
}

#[test]
fn test_tcp_set_reconnect_on_client_connection() {
    let mut host = TcpNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().unwrap();

    let mut client = TcpNetProvider::new();
    let client_conn = client.connect(&host_addr.to_string()).unwrap();

    // Wait for connection.
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        host.drain_events();
        client.drain_events();
        if client.connection_state(client_conn) == ConnectionState::Connected {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        client.connection_state(client_conn),
        ConnectionState::Connected
    );

    // Enable reconnect.
    let rc = ReconnectConfig {
        enabled: true,
        max_attempts: 3,
        delay: std::time::Duration::from_millis(100),
    };
    client.set_reconnect(client_conn, rc).unwrap();

    host.shutdown();
    client.shutdown();
}

#[test]
fn test_tcp_set_reconnect_rejects_server_side_connection() {
    let mut host = TcpNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().unwrap();

    let mut client = TcpNetProvider::new();
    let _client_conn = client.connect(&host_addr.to_string()).unwrap();

    // Wait for connection on the host side.
    let mut host_conn = None;
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        for event in host.drain_events() {
            if let NetworkEvent::Connected { conn } = event {
                host_conn = Some(conn);
            }
        }
        client.drain_events();
        if host_conn.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let host_conn = host_conn.expect("host should see a connected event");

    // Server-side connections have no remote_addr, so set_reconnect should fail.
    let rc = ReconnectConfig {
        enabled: true,
        max_attempts: 3,
        delay: std::time::Duration::from_millis(100),
    };
    assert!(host.set_reconnect(host_conn, rc).is_err());

    host.shutdown();
    client.shutdown();
}

#[test]
#[ignore] // Timing-sensitive: depends on OS scheduling for TCP disconnect detection
fn test_tcp_reconnect_after_remote_close() {
    let mut host = TcpNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().unwrap();

    let mut client = TcpNetProvider::new();
    let client_conn = client.connect(&host_addr.to_string()).unwrap();

    // Wait for connection.
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        host.drain_events();
        client.drain_events();
        if client.connection_state(client_conn) == ConnectionState::Connected {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        client.connection_state(client_conn),
        ConnectionState::Connected
    );

    // Enable reconnect with short delay.
    let rc = ReconnectConfig {
        enabled: true,
        max_attempts: 3,
        delay: std::time::Duration::from_millis(50),
    };
    client.set_reconnect(client_conn, rc).unwrap();

    // Disconnect from the host side to trigger remote close on client.
    let host_conns = host.connections();
    for hc in &host_conns {
        host.disconnect(*hc).unwrap();
    }
    // Give IO threads time to detect the stream close and process events.
    // Use a generous sleep to handle system load during large test suites.
    std::thread::sleep(std::time::Duration::from_millis(200));

    // First, ensure the client detects the disconnect.
    let mut disconnected = false;
    for _ in 0..200 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        host.drain_events();
        for event in client.drain_events() {
            if matches!(event, NetworkEvent::Disconnected { .. }) {
                disconnected = true;
            }
        }
        if disconnected {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(disconnected, "client should detect disconnect");

    // Now wait for reconnection (reconnect delay is 50ms + connect time).
    let mut reconnected = false;
    for _ in 0..300 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        host.drain_events();
        for event in client.drain_events() {
            if let NetworkEvent::Connected { conn } = event {
                if conn == client_conn {
                    reconnected = true;
                }
            }
        }
        if reconnected {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    assert!(reconnected, "client should reconnect after remote close");
    assert_eq!(
        client.connection_state(client_conn),
        ConnectionState::Connected
    );

    host.shutdown();
    client.shutdown();
}
