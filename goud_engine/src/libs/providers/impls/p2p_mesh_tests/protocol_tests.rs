//! Protocol-specific tests for the P2P mesh layer.

use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, HostConfig, P2pEvent, P2pMeshConfig,
};
use crate::core::providers::ProviderLifecycle;

use super::super::{P2pMesh, PeerConnectionMode};
use super::MockTransport;

#[test]
fn test_p2p_mesh_user_data_message() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    mesh.host(&HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    })
    .unwrap();

    // Register a peer.
    mesh.register_peer(2, ConnectionId(100), PeerConnectionMode::Direct);

    // Simulate receiving a user data message from peer 2.
    let mut msg = vec![0xF4]; // MESH_MSG_USER_DATA
    msg.extend_from_slice(b"hello mesh");
    mesh.handle_mesh_message(ConnectionId(100), Channel(0), &msg);

    let p2p_events = mesh.drain_p2p_events();
    // Find the MessageReceived event (skip the PeerJoined from register_peer).
    let msg_event = p2p_events
        .iter()
        .find(|e| matches!(e, P2pEvent::MessageReceived { .. }));
    assert!(msg_event.is_some());
    if let Some(P2pEvent::MessageReceived { from, data }) = msg_event {
        assert_eq!(*from, 2);
        assert_eq!(data, b"hello mesh");
    }
}

#[test]
fn test_p2p_mesh_leave_message() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    mesh.host(&HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    })
    .unwrap();

    mesh.register_peer(2, ConnectionId(100), PeerConnectionMode::Direct);
    assert_eq!(mesh.peer_count(), 1);

    // Simulate receiving a leave message.
    mesh.handle_mesh_message(ConnectionId(100), Channel(0), &[0xF3]); // MESH_MSG_LEAVE

    assert_eq!(mesh.peer_count(), 0);
    let p2p_events = mesh.drain_p2p_events();
    assert!(p2p_events
        .iter()
        .any(|e| matches!(e, P2pEvent::PeerLeft(2))));
}

#[test]
fn test_p2p_mesh_join_accept_sets_local_id() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    // Simulate receiving a join-accept with assigned ID 5 and host ID 1.
    let mut payload = Vec::new();
    payload.extend_from_slice(&5u64.to_le_bytes());
    payload.extend_from_slice(&1u64.to_le_bytes());

    mesh.handle_join_accept(&payload);

    assert_eq!(mesh.local_peer_id(), 5);
    assert_eq!(mesh.host_peer_id(), 1);
}

#[test]
fn test_p2p_mesh_lifecycle() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    assert!(mesh.init().is_ok());
    assert!(mesh.update(0.016).is_ok());
    mesh.shutdown();
}

#[test]
fn test_p2p_mesh_drain_events_clears_buffer() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    mesh.host(&HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    })
    .unwrap();

    // Trigger some events.
    mesh.handle_mesh_message(ConnectionId(100), Channel(0), &[0xF1]);

    let events = mesh.drain_events();
    assert!(!events.is_empty());

    // Second drain should be empty.
    let events2 = mesh.drain_events();
    assert!(events2.is_empty());
}

#[test]
fn test_p2p_mesh_connection_state_for_peer() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    // Unknown peer.
    assert_eq!(
        mesh.connection_state(ConnectionId(99)),
        ConnectionState::Disconnected
    );

    mesh.host(&HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    })
    .unwrap();

    mesh.register_peer(2, ConnectionId(100), PeerConnectionMode::Direct);
    assert_eq!(
        mesh.connection_state(ConnectionId(2)),
        ConnectionState::Connected
    );
}

#[test]
fn test_p2p_mesh_local_id() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    // Before hosting/connecting, local_id is None (peer_id is 0).
    assert!(mesh.local_id().is_none());

    mesh.host(&HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    })
    .unwrap();

    assert_eq!(mesh.local_id(), Some(ConnectionId(1)));
}

#[test]
fn test_p2p_mesh_diagnostics() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mesh = P2pMesh::new(Box::new(transport), config);

    let diag = mesh.network_diagnostics();
    assert_eq!(diag.active_connections, 0);
    assert_eq!(diag.bytes_sent, 0);
}

#[test]
fn test_p2p_mesh_debug_format() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mesh = P2pMesh::new(Box::new(transport), config);

    let debug_str = format!("{:?}", mesh);
    assert!(debug_str.contains("P2pMesh"));
    assert!(debug_str.contains("local_peer_id"));
}

#[test]
fn test_p2p_mesh_default_config() {
    let config = P2pMeshConfig::default();
    assert_eq!(config.max_peers, 8);
    assert!(config.host_migration);
    assert!(config.relay_server.is_none());
    assert_eq!(
        config.topology,
        crate::core::providers::network_types::P2pTopology::FullMesh
    );
}

#[test]
fn test_p2p_mesh_elect_new_host_picks_lowest_id() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig {
        host_migration: true,
        ..P2pMeshConfig::default()
    };
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    mesh.local_peer_id = 5;
    mesh.host_id = 1;

    mesh.register_peer(3, ConnectionId(30), PeerConnectionMode::Direct);
    mesh.register_peer(7, ConnectionId(70), PeerConnectionMode::Direct);

    // Simulate host leaving (peer 1 not in peers map).
    mesh.elect_new_host();

    // Lowest among [3, 7, 5(self)] = 3.
    assert_eq!(mesh.host_peer_id(), 3);
}

#[test]
fn test_p2p_mesh_send_to_unknown_peer_fails() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    let result = mesh.send_to_peer(999, Channel(0), b"test");
    assert!(result.is_err());
}

#[test]
fn test_p2p_mesh_empty_message_ignored() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    // Empty data should be silently ignored.
    mesh.handle_mesh_message(ConnectionId(100), Channel(0), &[]);

    let events = mesh.drain_events();
    assert!(events.is_empty());
}
