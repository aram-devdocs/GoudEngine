//! Round-trip and integration tests for WebRTC networking.

use super::*;
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionState, DisconnectReason, HostConfig, NetworkEvent,
};
use crate::core::providers::ProviderLifecycle;

fn host_config() -> HostConfig {
    HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    }
}

// =========================================================================
// Send and receive
// =========================================================================

#[test]
fn test_webrtc_send_receive_reliable_channel() {
    let mut host = WebRtcNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().unwrap();

    let mut client = WebRtcNetProvider::new();
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

    // Send reliable message from client to host.
    let payload = b"hello-webrtc-reliable";
    client.send(client_conn, Channel(0), payload).unwrap();

    let mut received = None;
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        for event in host.drain_events() {
            if let NetworkEvent::Received { channel, data, .. } = event {
                assert_eq!(channel, Channel(0));
                received = Some(data);
            }
        }
        client.drain_events();
        if received.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert_eq!(received.as_deref(), Some(payload.as_slice()));

    host.shutdown();
    client.shutdown();
}

#[test]
fn test_webrtc_send_receive_unreliable_channel() {
    let mut host = WebRtcNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().unwrap();

    let mut client = WebRtcNetProvider::new();
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

    // Send unreliable message (channel 1).
    let payload = b"hello-webrtc-unreliable";
    client.send(client_conn, Channel(1), payload).unwrap();

    let mut received = None;
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        for event in host.drain_events() {
            if let NetworkEvent::Received { channel, data, .. } = event {
                assert_eq!(channel, Channel(1));
                received = Some(data);
            }
        }
        client.drain_events();
        if received.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert_eq!(received.as_deref(), Some(payload.as_slice()));

    host.shutdown();
    client.shutdown();
}

// =========================================================================
// Remote disconnect
// =========================================================================

#[test]
fn test_webrtc_remote_disconnect_detected() {
    let mut host = WebRtcNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().unwrap();

    let mut client = WebRtcNetProvider::new();
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

    // Disconnect from host side.
    let host_conns = host.connections();
    for hc in &host_conns {
        host.disconnect(*hc).unwrap();
    }

    // Wait for client to detect remote close.
    let mut remote_closed = false;
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        for event in client.drain_events() {
            if matches!(
                event,
                NetworkEvent::Disconnected {
                    reason: DisconnectReason::RemoteClose,
                    ..
                }
            ) {
                remote_closed = true;
            }
        }
        host.drain_events();
        if remote_closed {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert!(remote_closed, "client should detect remote close");

    host.shutdown();
    client.shutdown();
}

// =========================================================================
// Stats tracking
// =========================================================================

#[test]
fn test_webrtc_stats_tracking() {
    let mut host = WebRtcNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().unwrap();

    let mut client = WebRtcNetProvider::new();
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

    // Send data.
    client.send(client_conn, Channel(1), b"stats-test").unwrap();

    // Poll to process.
    for _ in 0..50 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        host.drain_events();
        client.drain_events();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let client_stats = client.stats();
    assert!(client_stats.bytes_sent > 0, "client should have sent bytes");
    assert!(
        client_stats.packets_sent > 0,
        "client should have sent packets"
    );

    let conn_stats = client.connection_stats(client_conn);
    assert!(conn_stats.is_some(), "connection stats should be available");

    host.shutdown();
    client.shutdown();
}

// =========================================================================
// Broadcast
// =========================================================================

#[test]
fn test_webrtc_broadcast_sends_to_all_connected() {
    let mut host = WebRtcNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().unwrap();

    let mut client1 = WebRtcNetProvider::new();
    let c1 = client1.connect(&host_addr.to_string()).unwrap();

    let mut client2 = WebRtcNetProvider::new();
    let c2 = client2.connect(&host_addr.to_string()).unwrap();

    // Wait for both connections.
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client1.update(0.0).unwrap();
        client2.update(0.0).unwrap();
        host.drain_events();
        client1.drain_events();
        client2.drain_events();
        if client1.connection_state(c1) == ConnectionState::Connected
            && client2.connection_state(c2) == ConnectionState::Connected
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    // Broadcast from host.
    let payload = b"broadcast-msg";
    host.broadcast(Channel(1), payload).unwrap();

    let mut c1_received = false;
    let mut c2_received = false;
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client1.update(0.0).unwrap();
        client2.update(0.0).unwrap();
        host.drain_events();
        for event in client1.drain_events() {
            if let NetworkEvent::Received { data, .. } = event {
                if data == payload {
                    c1_received = true;
                }
            }
        }
        for event in client2.drain_events() {
            if let NetworkEvent::Received { data, .. } = event {
                if data == payload {
                    c2_received = true;
                }
            }
        }
        if c1_received && c2_received {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert!(c1_received, "client1 should receive broadcast");
    assert!(c2_received, "client2 should receive broadcast");

    host.shutdown();
    client1.shutdown();
    client2.shutdown();
}
