//! Peer-to-peer types for the P2P mesh networking layer.

// =============================================================================
// Peer-to-Peer Types
// =============================================================================

/// Unique identifier for a peer in a P2P mesh network.
///
/// Peer IDs are assigned when a peer joins the mesh. The lowest peer ID
/// in the mesh is elected as the host during host migration.
pub type PeerId = u64;

/// Events emitted by the P2P mesh layer.
///
/// These are higher-level events than `NetworkEvent` and represent mesh
/// topology changes rather than raw transport events.
#[derive(Debug, Clone)]
pub enum P2pEvent {
    /// A new peer joined the mesh.
    PeerJoined(PeerId),
    /// A peer left the mesh (graceful or timeout).
    PeerLeft(PeerId),
    /// The mesh host changed due to the previous host disconnecting.
    HostMigrated {
        /// The peer ID of the previous host.
        old_host: PeerId,
        /// The peer ID of the new host.
        new_host: PeerId,
    },
    /// A message was received from a peer.
    MessageReceived {
        /// The peer that sent the message.
        from: PeerId,
        /// The raw payload bytes.
        data: Vec<u8>,
    },
    /// This peer's direct connection to another peer failed and is now
    /// being relayed through the relay server.
    RelayFallback {
        /// The peer whose connection is now relayed.
        peer: PeerId,
    },
}

/// Mesh topology variant.
///
/// Currently only `FullMesh` is supported. `Star` is reserved for future use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum P2pTopology {
    /// Every peer is connected to every other peer.
    FullMesh = 0,
    /// All peers connect through a central hub (future).
    Star = 1,
}

/// Configuration for creating or joining a P2P mesh.
#[derive(Debug, Clone)]
pub struct P2pMeshConfig {
    /// Maximum number of peers allowed in the mesh (including self).
    pub max_peers: usize,
    /// Optional relay server address for NAT traversal fallback.
    /// Format: `"host:port"`.
    pub relay_server: Option<String>,
    /// Whether host migration is enabled when the current host disconnects.
    pub host_migration: bool,
    /// The mesh topology to use.
    pub topology: P2pTopology,
}

impl Default for P2pMeshConfig {
    fn default() -> Self {
        Self {
            max_peers: 8,
            relay_server: None,
            host_migration: true,
            topology: P2pTopology::FullMesh,
        }
    }
}
