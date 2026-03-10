use super::*;
#[cfg(feature = "net-tcp")]
use crate::libs::providers::impls::TcpNetProvider;

#[cfg(feature = "net-tcp")]
#[test]
fn test_overlay_snapshot_metrics_track_live_tcp_provider_within_five_percent() {
    let _registry = RegistryResetGuard::new();
    let context_id = GoudContextId::new(306, 1);

    let mut host = TcpNetProvider::new();
    host.host(&HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    })
    .unwrap();
    let host_addr = host.local_addr().expect("host should bind to an address");

    let mut client = TcpNetProvider::new();
    let client_conn = client.connect(&host_addr.to_string()).unwrap();

    let mut host_conn = None;
    wait_until(
        Duration::from_secs(5),
        Duration::from_millis(5),
        "host and client should establish a TCP connection for overlay stats",
        || {
            host.update(0.0).unwrap();
            client.update(0.0).unwrap();

            for event in host.drain_events() {
                if let NetworkEvent::Connected { conn } = event {
                    host_conn = Some(conn);
                }
            }
            client.drain_events();

            host_conn.is_some()
                && client.connection_state(client_conn) == ConnectionState::Connected
        },
    );

    let host_conn = host_conn.expect("host should observe a connected TCP client");
    assert_eq!(
        client.connection_state(client_conn),
        ConnectionState::Connected
    );

    client
        .send(client_conn, Channel(0), b"overlay-live-client")
        .unwrap();
    wait_until(
        Duration::from_secs(5),
        Duration::from_millis(5),
        "host should receive the client payload for live overlay stats",
        || {
            host.update(0.0).unwrap();
            for event in host.drain_events() {
                if let NetworkEvent::Received { channel, data, .. } = event {
                    assert_eq!(channel, Channel(0));
                    if data == b"overlay-live-client" {
                        return true;
                    }
                }
            }
            false
        },
    );

    host.send(host_conn, Channel(0), b"overlay-live-host")
        .unwrap();
    wait_until(
        Duration::from_secs(5),
        Duration::from_millis(5),
        "client should receive the host payload for live overlay stats",
        || {
            client.update(0.0).unwrap();
            for event in client.drain_events() {
                if let NetworkEvent::Received { channel, data, .. } = event {
                    assert_eq!(channel, Channel(0));
                    if data == b"overlay-live-host" {
                        return true;
                    }
                }
            }
            false
        },
    );

    let expected_stats = client.stats();
    assert!(expected_stats.send_bandwidth_bytes_per_sec > 0.0);
    assert!(expected_stats.receive_bandwidth_bytes_per_sec > 0.0);

    let handle = insert_provider(Box::new(client));
    with_registry(|reg| {
        reg.set_default_handle_for_context(context_id, handle);
        Ok(())
    })
    .expect("failed to bind default handle");

    let snapshot =
        network_overlay_snapshot_for_context(context_id).expect("expected overlay snapshot");
    assert_eq!(snapshot.handle, handle);

    assert_within_5_percent(snapshot.metrics.rtt_ms, expected_stats.rtt_ms);
    assert_within_5_percent(
        snapshot.metrics.send_bandwidth_bytes_per_sec,
        expected_stats.send_bandwidth_bytes_per_sec,
    );
    assert_within_5_percent(
        snapshot.metrics.receive_bandwidth_bytes_per_sec,
        expected_stats.receive_bandwidth_bytes_per_sec,
    );
    assert_within_5_percent(
        snapshot.metrics.packet_loss_percent,
        expected_stats.packet_loss_percent,
    );
    assert_within_5_percent(snapshot.metrics.jitter_ms, expected_stats.jitter_ms);

    let _ = with_instance(handle, |instance| {
        instance.provider.shutdown();
        Ok(())
    });
    host.shutdown();
}

#[cfg(feature = "net-tcp")]
#[test]
fn test_goud_network_connect_with_peer_preserves_connected_peer_id() {
    let _registry = RegistryResetGuard::new();
    let context_id = GoudContextId::new(308, 1);

    let mut host = TcpNetProvider::new();
    host.host(&HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    })
    .unwrap();
    let host_addr = host.local_addr().expect("host should bind to an address");

    let address = host_addr.ip().to_string();
    let mut handle = 0i64;
    let mut peer_id = 0u64;

    // SAFETY: `address`, `handle`, and `peer_id` all point to valid storage for this call.
    let rc = unsafe {
        goud_network_connect_with_peer(
            context_id,
            super::provider_factory::PROTOCOL_TCP,
            address.as_bytes().as_ptr(),
            address.len() as i32,
            host_addr.port(),
            &mut handle,
            &mut peer_id,
        )
    };

    assert_eq!(rc, 0);
    assert!(handle > 0);
    assert_ne!(peer_id, 0);

    let mut host_conn = None;
    wait_until(
        Duration::from_secs(5),
        Duration::from_millis(5),
        "host should observe the FFI-connected TCP peer",
        || {
            host.update(0.0).unwrap();
            for event in host.drain_events() {
                if let NetworkEvent::Connected { conn } = event {
                    host_conn = Some(conn.0);
                    return true;
                }
            }
            false
        },
    );

    assert_eq!(host_conn, Some(peer_id));

    let _ = with_instance(handle, |instance| {
        instance.provider.shutdown();
        Ok(())
    });
    host.shutdown();
}
