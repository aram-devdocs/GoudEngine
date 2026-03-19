//! Tests for the P2P mesh layer.

use crate::core::providers::diagnostics::NetworkDiagnosticsV1;
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, HostConfig, NetworkCapabilities,
    NetworkEvent, NetworkStats, P2pEvent, P2pMeshConfig,
};
use crate::core::providers::{Provider, ProviderLifecycle};
use crate::libs::error::GoudResult;

use super::P2pMesh;

// ---------------------------------------------------------------------------
// Mock transport for unit testing
// ---------------------------------------------------------------------------

/// A simple in-memory mock network provider for testing the mesh layer.
struct MockTransport {
    capabilities: NetworkCapabilities,
    connections: Vec<ConnectionId>,
    next_id: u64,
    events: Vec<NetworkEvent>,
    sent_messages: Vec<(ConnectionId, Channel, Vec<u8>)>,
    is_hosting: bool,
}

impl MockTransport {
    fn new() -> Self {
        Self {
            capabilities: NetworkCapabilities {
                supports_hosting: true,
                max_connections: 32,
                max_channels: 2,
                max_message_size: 65535,
            },
            connections: Vec::new(),
            next_id: 1,
            events: Vec::new(),
            sent_messages: Vec::new(),
            is_hosting: false,
        }
    }
}

impl Provider for MockTransport {
    fn name(&self) -> &str {
        "mock"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for MockTransport {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }
    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        Ok(())
    }
    fn shutdown(&mut self) {
        self.connections.clear();
    }
}

impl NetworkProvider for MockTransport {
    fn host(&mut self, _config: &HostConfig) -> GoudResult<()> {
        self.is_hosting = true;
        Ok(())
    }

    fn connect(&mut self, _addr: &str) -> GoudResult<ConnectionId> {
        let id = ConnectionId(self.next_id);
        self.next_id += 1;
        self.connections.push(id);
        Ok(id)
    }

    fn disconnect(&mut self, conn: ConnectionId) -> GoudResult<()> {
        self.connections.retain(|c| *c != conn);
        Ok(())
    }

    fn disconnect_all(&mut self) -> GoudResult<()> {
        self.connections.clear();
        Ok(())
    }

    fn send(&mut self, conn: ConnectionId, channel: Channel, data: &[u8]) -> GoudResult<()> {
        self.sent_messages.push((conn, channel, data.to_vec()));
        Ok(())
    }

    fn broadcast(&mut self, channel: Channel, data: &[u8]) -> GoudResult<()> {
        for conn in &self.connections {
            self.sent_messages.push((*conn, channel, data.to_vec()));
        }
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<NetworkEvent> {
        std::mem::take(&mut self.events)
    }

    fn connections(&self) -> Vec<ConnectionId> {
        self.connections.clone()
    }

    fn connection_state(&self, conn: ConnectionId) -> ConnectionState {
        if self.connections.contains(&conn) {
            ConnectionState::Connected
        } else {
            ConnectionState::Disconnected
        }
    }

    fn local_id(&self) -> Option<ConnectionId> {
        None
    }

    fn network_capabilities(&self) -> &NetworkCapabilities {
        &self.capabilities
    }

    fn stats(&self) -> NetworkStats {
        NetworkStats::default()
    }

    fn connection_stats(&self, _conn: ConnectionId) -> Option<ConnectionStats> {
        None
    }

    fn network_diagnostics(&self) -> NetworkDiagnosticsV1 {
        NetworkDiagnosticsV1::default()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_p2p_mesh_host_assigns_peer_id_1() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    let host_config = HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    };
    mesh.host(&host_config).unwrap();

    assert_eq!(mesh.local_peer_id(), 1);
    assert_eq!(mesh.host_peer_id(), 1);
    assert!(mesh.is_host());
}

#[test]
fn test_p2p_mesh_provider_identity() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mesh = P2pMesh::new(Box::new(transport), config);

    assert_eq!(mesh.name(), "p2p_mesh");
    assert_eq!(mesh.version(), "0.1.0");
}

#[test]
fn test_p2p_mesh_capabilities_reflect_config() {
    let config = P2pMeshConfig {
        max_peers: 16,
        ..P2pMeshConfig::default()
    };
    let transport = MockTransport::new();
    let mesh = P2pMesh::new(Box::new(transport), config);

    let caps = mesh.network_capabilities();
    assert_eq!(caps.max_connections, 16);
    assert!(caps.supports_hosting);
}

#[test]
fn test_p2p_mesh_connect_sends_join_request() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig::default();
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    let conn = mesh.connect("127.0.0.1:8080").unwrap();

    // The mesh should have registered the host as peer 1.
    assert_eq!(mesh.host_peer_id(), 1);
    assert_eq!(conn, ConnectionId(1));
}

#[test]
fn test_p2p_mesh_host_handles_join_request() {
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

    // Simulate a join request message: [0xF1]
    let join_request = vec![0xF1]; // MESH_MSG_JOIN_REQUEST
    mesh.handle_mesh_message(ConnectionId(100), Channel(0), &join_request);

    // The mesh should have assigned peer ID 2 and emitted a PeerJoined event.
    let p2p_events = mesh.drain_p2p_events();
    assert_eq!(p2p_events.len(), 1);
    assert!(matches!(p2p_events[0], P2pEvent::PeerJoined(2)));

    assert_eq!(mesh.peer_count(), 1);
    assert!(mesh.peer_ids().contains(&2));
}

#[test]
fn test_p2p_mesh_host_rejects_when_full() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig {
        max_peers: 2, // Only room for host + 1 peer.
        ..P2pMeshConfig::default()
    };
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    mesh.host(&HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    })
    .unwrap();

    // Add first peer (fills the mesh).
    mesh.handle_mesh_message(ConnectionId(100), Channel(0), &[0xF1]);
    assert_eq!(mesh.peer_count(), 1);

    // Try to add another peer -- should be rejected.
    mesh.handle_mesh_message(ConnectionId(101), Channel(0), &[0xF1]);
    assert_eq!(mesh.peer_count(), 1); // Still 1.
}

#[test]
fn test_p2p_mesh_host_migration_on_host_disconnect() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig {
        host_migration: true,
        ..P2pMeshConfig::default()
    };
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    // Set up as a non-host peer with peer ID 3, host is peer 1.
    mesh.local_peer_id = 3;
    mesh.host_id = 1;

    // Register host as a direct peer.
    mesh.register_peer(
        1,
        ConnectionId(10),
        crate::libs::providers::impls::p2p_mesh::PeerConnectionMode::Direct,
    );
    // Register another peer.
    mesh.register_peer(
        2,
        ConnectionId(20),
        crate::libs::providers::impls::p2p_mesh::PeerConnectionMode::Direct,
    );

    // Simulate the host disconnecting.
    mesh.unregister_peer(1);
    mesh.elect_new_host();

    // The new host should be peer 2 (lowest remaining).
    assert_eq!(mesh.host_peer_id(), 2);

    let p2p_events = mesh.drain_p2p_events();
    assert!(p2p_events.iter().any(|e| matches!(
        e,
        P2pEvent::HostMigrated {
            old_host: 1,
            new_host: 2
        }
    )));
}

#[test]
fn test_p2p_mesh_host_migration_disabled() {
    let transport = MockTransport::new();
    let config = P2pMeshConfig {
        host_migration: false,
        ..P2pMeshConfig::default()
    };
    let mut mesh = P2pMesh::new(Box::new(transport), config);

    mesh.local_peer_id = 3;
    mesh.host_id = 1;
    mesh.register_peer(
        1,
        ConnectionId(10),
        crate::libs::providers::impls::p2p_mesh::PeerConnectionMode::Direct,
    );
    mesh.register_peer(
        2,
        ConnectionId(20),
        crate::libs::providers::impls::p2p_mesh::PeerConnectionMode::Direct,
    );

    // Simulate host disconnect -- but migration is disabled.
    mesh.unregister_peer(1);
    // The host ID should remain 1 (no migration).
    assert_eq!(mesh.host_peer_id(), 1);
}

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
    mesh.register_peer(
        2,
        ConnectionId(100),
        crate::libs::providers::impls::p2p_mesh::PeerConnectionMode::Direct,
    );

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

    mesh.register_peer(
        2,
        ConnectionId(100),
        crate::libs::providers::impls::p2p_mesh::PeerConnectionMode::Direct,
    );
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

    mesh.register_peer(
        2,
        ConnectionId(100),
        crate::libs::providers::impls::p2p_mesh::PeerConnectionMode::Direct,
    );
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

    mesh.register_peer(
        3,
        ConnectionId(30),
        crate::libs::providers::impls::p2p_mesh::PeerConnectionMode::Direct,
    );
    mesh.register_peer(
        7,
        ConnectionId(70),
        crate::libs::providers::impls::p2p_mesh::PeerConnectionMode::Direct,
    );

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
