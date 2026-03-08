use super::*;

#[test]
fn test_udp_construction() {
    let provider = UdpNetProvider::new();
    assert_eq!(provider.name(), "udp");
    assert_eq!(provider.version(), "0.1.0");
    assert!(provider.socket.is_none());
    assert!(provider.connections.is_empty());
}

#[test]
fn test_udp_default() {
    let provider = UdpNetProvider::default();
    assert_eq!(provider.name(), "udp");
}

#[test]
fn test_udp_capabilities() {
    let provider = UdpNetProvider::new();
    let caps = provider.network_capabilities();
    assert!(caps.supports_hosting);
    assert_eq!(caps.max_connections, 32);
    assert_eq!(caps.max_channels, 2);
    assert_eq!(caps.max_message_size, (RECV_BUF_SIZE - HEADER_SIZE) as u32);
}

#[test]
fn test_udp_lifecycle() {
    let mut provider = UdpNetProvider::new();
    assert!(provider.init().is_ok());
    assert!(provider.update(0.016).is_ok());
    provider.shutdown();
}

#[test]
fn test_udp_host_binds_socket() {
    let mut provider = UdpNetProvider::new();
    let config = HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0, // OS-assigned
        max_connections: 16,
        tls_cert_path: None,
        tls_key_path: None,
    };
    assert!(provider.host(&config).is_ok());
    assert!(provider.socket.is_some());
    assert!(provider.is_host);

    // Hosting again should fail.
    assert!(provider.host(&config).is_err());

    provider.shutdown();
}

#[test]
fn test_udp_connect_to_host() {
    // Start a host.
    let mut host = UdpNetProvider::new();
    let config = HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 16,
        tls_cert_path: None,
        tls_key_path: None,
    };
    host.host(&config).unwrap();
    let host_addr = host.socket.as_ref().unwrap().local_addr().unwrap();

    // Client connects.
    let mut client = UdpNetProvider::new();
    let conn_id = client.connect(&host_addr.to_string()).unwrap();
    assert_eq!(
        client.connection_state(conn_id),
        ConnectionState::Connecting
    );

    // Complete handshake with retry for OS UDP delivery timing.
    let mut host_connected = false;
    let mut client_connected = false;
    for _ in 0..20 {
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
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    assert!(host_connected, "Host should have received Connected event");
    assert!(client_connected, "Client should be Connected");

    let client_events = client.drain_events();
    assert!(client_events
        .iter()
        .any(|e| matches!(e, NetworkEvent::Connected { .. })));

    host.shutdown();
    client.shutdown();
}

/// Helper: create a connected host+client pair. Returns (host, client, client_conn_id).
fn setup_connected_pair() -> (UdpNetProvider, UdpNetProvider, ConnectionId) {
    let mut host = UdpNetProvider::new();
    let config = HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 16,
        tls_cert_path: None,
        tls_key_path: None,
    };
    host.host(&config).unwrap();
    let host_addr = host.socket.as_ref().unwrap().local_addr().unwrap();

    let mut client = UdpNetProvider::new();
    let conn_id = client.connect(&host_addr.to_string()).unwrap();

    // Complete handshake with retry to handle OS UDP delivery timing.
    for _ in 0..20 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        if client.connection_state(conn_id) == ConnectionState::Connected {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    assert_eq!(
        client.connection_state(conn_id),
        ConnectionState::Connected,
        "Client should be Connected after handshake"
    );

    // Drain handshake events.
    host.drain_events();
    client.drain_events();

    (host, client, conn_id)
}

#[test]
fn test_udp_send_receive() {
    let (mut host, mut client, conn_id) = setup_connected_pair();

    // Client sends data to host.
    let payload = b"hello host";
    client.send(conn_id, Channel(0), payload).unwrap();

    // Host receives (with retry for delivery timing).
    let mut received_data = None;
    for _ in 0..20 {
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
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(received_data.as_deref(), Some(payload.as_slice()));

    // Host sends data back to client.
    let host_conn = host.connections().into_iter().next().unwrap();
    let reply = b"hello client";
    host.send(host_conn, Channel(1), reply).unwrap();

    let mut reply_data = None;
    for _ in 0..20 {
        client.update(0.0).unwrap();
        for event in client.drain_events() {
            if let NetworkEvent::Received { data, channel, .. } = event {
                assert_eq!(channel, Channel(1));
                reply_data = Some(data);
            }
        }
        if reply_data.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(reply_data.as_deref(), Some(reply.as_slice()));

    host.shutdown();
    client.shutdown();
}

#[test]
fn test_udp_disconnect() {
    let (mut host, mut client, conn_id) = setup_connected_pair();

    // Client disconnects.
    client.disconnect(conn_id).unwrap();
    let events = client.drain_events();
    assert!(events.iter().any(|e| matches!(
        e,
        NetworkEvent::Disconnected {
            reason: crate::core::providers::network_types::DisconnectReason::LocalClose,
            ..
        }
    )));

    // Host processes DISCONNECT (with retry for delivery timing).
    let mut host_got_disconnect = false;
    for _ in 0..20 {
        host.update(0.0).unwrap();
        if host.drain_events().iter().any(|e| {
            matches!(
                e,
                NetworkEvent::Disconnected {
                    reason: crate::core::providers::network_types::DisconnectReason::RemoteClose,
                    ..
                }
            )
        }) {
            host_got_disconnect = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(host_got_disconnect, "Host should receive remote disconnect");

    host.shutdown();
    client.shutdown();
}

#[test]
fn test_udp_broadcast() {
    let mut host = UdpNetProvider::new();
    let config = HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 16,
        tls_cert_path: None,
        tls_key_path: None,
    };
    host.host(&config).unwrap();
    let host_addr = host.socket.as_ref().unwrap().local_addr().unwrap();

    // Two clients connect with full handshake.
    let mut c1 = UdpNetProvider::new();
    let mut c2 = UdpNetProvider::new();
    let c1_id = c1.connect(&host_addr.to_string()).unwrap();
    // Complete c1 handshake.
    for _ in 0..20 {
        host.update(0.0).unwrap();
        c1.update(0.0).unwrap();
        if c1.connection_state(c1_id) == ConnectionState::Connected {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    let c2_id = c2.connect(&host_addr.to_string()).unwrap();
    // Complete c2 handshake.
    for _ in 0..20 {
        host.update(0.0).unwrap();
        c2.update(0.0).unwrap();
        if c2.connection_state(c2_id) == ConnectionState::Connected {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    host.drain_events();
    c1.drain_events();
    c2.drain_events();

    // Host broadcasts.
    host.broadcast(Channel(0), b"broadcast").unwrap();

    // Receive with retry for delivery timing.
    let mut c1_got = false;
    let mut c2_got = false;
    for _ in 0..20 {
        c1.update(0.0).unwrap();
        c2.update(0.0).unwrap();
        if c1
            .drain_events()
            .iter()
            .any(|e| matches!(e, NetworkEvent::Received { .. }))
        {
            c1_got = true;
        }
        if c2
            .drain_events()
            .iter()
            .any(|e| matches!(e, NetworkEvent::Received { .. }))
        {
            c2_got = true;
        }
        if c1_got && c2_got {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(c1_got, "Client 1 should have received broadcast");
    assert!(c2_got, "Client 2 should have received broadcast");

    host.shutdown();
    c1.shutdown();
    c2.shutdown();
}

#[test]
fn test_udp_reliable_retransmission() {
    let (mut host, mut client, conn_id) = setup_connected_pair();

    // Send on reliable channel.
    client.send(conn_id, Channel(0), b"reliable").unwrap();

    // Check that reliability layer has pending packet.
    let conn = client.connections.get(&conn_id.0).unwrap();
    assert!(conn.reliability.pending_count() > 0);

    host.shutdown();
    client.shutdown();
}

#[test]
fn test_udp_unreliable_no_retransmit() {
    let (mut host, mut client, conn_id) = setup_connected_pair();

    // Send on unreliable channel.
    client.send(conn_id, Channel(1), b"unreliable").unwrap();

    // Reliability layer should have NO pending packets.
    let conn = client.connections.get(&conn_id.0).unwrap();
    assert_eq!(conn.reliability.pending_count(), 0);

    host.shutdown();
    client.shutdown();
}

#[test]
fn test_udp_stats() {
    let provider = UdpNetProvider::new();
    let stats = provider.stats();
    assert_eq!(stats.bytes_sent, 0);
    assert_eq!(stats.packets_sent, 0);
    assert!(provider.connection_stats(ConnectionId(999)).is_none());
}

#[test]
fn test_udp_debug_format() {
    let provider = UdpNetProvider::new();
    let debug = format!("{:?}", provider);
    assert!(debug.contains("UdpNetProvider"));
    assert!(debug.contains("is_host"));
}

#[test]
fn test_udp_simulated_packet_loss_reliable_delivery() {
    let (mut host, mut client, conn_id) = setup_connected_pair();

    // Client sends a reliable message on Channel(0).
    let payload = b"reliable after loss";
    client.send(conn_id, Channel(0), payload).unwrap();
    assert!(
        client
            .connections
            .get(&conn_id.0)
            .unwrap()
            .reliability
            .pending_count()
            > 0,
        "Reliability layer must track the sent packet"
    );

    // Simulate packet loss: host consumes the first datagram from the OS buffer
    // by reading from its socket directly, discarding the bytes. This ensures
    // the original packet is truly lost — not just delayed.
    std::thread::sleep(std::time::Duration::from_millis(5));
    {
        let socket = host.socket.as_ref().unwrap();
        let mut discard_buf = [0u8; 65536];
        // Drain all pending datagrams (the original data packet).
        while socket.recv_from(&mut discard_buf).is_ok() {}
    }

    // Verify host has no data events (the original packet was discarded).
    host.update(0.0).unwrap();
    let events = host.drain_events();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, NetworkEvent::Received { .. })),
        "Host should NOT have received data (packet was lost)"
    );

    // Sleep past the 100ms RTO so the client's reliability layer retransmits.
    std::thread::sleep(std::time::Duration::from_millis(150));
    client.update(0.0).unwrap();

    // Now the host receives the retransmitted packet.
    let mut received_data: Option<Vec<u8>> = None;
    for _ in 0..20 {
        host.update(0.0).unwrap();
        for event in host.drain_events() {
            if let NetworkEvent::Received { data, channel, .. } = event {
                assert_eq!(channel, Channel(0), "Data must arrive on reliable channel");
                received_data = Some(data);
            }
        }
        if received_data.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert_eq!(
        received_data.as_deref(),
        Some(payload.as_slice()),
        "Host must receive the retransmitted reliable payload"
    );

    host.shutdown();
    client.shutdown();
}
