//! Shared lobby and matchmaking types.

use crate::core::providers::network_types::{ConnectionId, DisconnectReason};

/// Lobby visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LobbyVisibility {
    /// Discoverable through lobby list APIs.
    Public,
    /// Hidden from lobby list APIs; joinable only via known address.
    Private,
}

impl LobbyVisibility {
    /// Stable metadata value used for session discovery.
    pub fn as_metadata_value(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }
}

/// Server-authoritative lobby state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LobbyState {
    /// Waiting for players to join or ready up.
    Waiting,
    /// Match has started.
    InGame,
}

impl LobbyState {
    /// Stable metadata value used for session discovery.
    pub fn as_metadata_value(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::InGame => "in_game",
        }
    }
}

/// Host-side lobby configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LobbyConfig {
    /// Stable room identifier.
    pub id: String,
    /// Human-readable room name.
    pub name: String,
    /// Maximum number of players allowed in the room.
    pub max_players: u32,
    /// Discovery visibility for the room.
    pub visibility: LobbyVisibility,
}

impl LobbyConfig {
    /// Creates a new lobby config.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        max_players: u32,
        visibility: LobbyVisibility,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            max_players,
            visibility,
        }
    }
}

/// One server-authoritative lobby member.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LobbyMember {
    /// Transport connection identifier for the member.
    pub connection: ConnectionId,
    /// Whether this member currently owns the host role.
    pub is_host: bool,
    /// Whether this member is ready for a normal start.
    pub ready: bool,
}

/// Client-to-server lobby commands.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LobbyCommand {
    /// Updates the sender's ready state.
    SetReady {
        /// New ready state.
        ready: bool,
    },
    /// Host requests a normal start.
    StartGame,
    /// Host requests an early start that bypasses ready checks.
    StartEarly,
    /// Host requests that one member be removed.
    Kick {
        /// Member connection to remove.
        connection: ConnectionId,
    },
}

/// High-level lobby events emitted by the wrappers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LobbyEvent {
    /// A member joined the lobby.
    MemberJoined {
        /// Joined member snapshot.
        member: LobbyMember,
    },
    /// A member left the lobby.
    MemberLeft {
        /// Connection that left.
        connection: ConnectionId,
    },
    /// The authoritative lobby snapshot changed.
    SnapshotUpdated {
        /// Latest server-authoritative state.
        state: LobbyState,
        /// Latest member list.
        members: Vec<LobbyMember>,
    },
    /// A join attempt was rejected.
    JoinRejected {
        /// Human-readable rejection reason.
        reason: String,
    },
    /// A lobby command was rejected.
    CommandRejected {
        /// Decoded command, when available.
        command: Option<LobbyCommand>,
        /// Human-readable rejection reason.
        reason: String,
    },
    /// The lobby started.
    Started {
        /// Whether the ready gate was bypassed.
        early: bool,
    },
    /// The client left the lobby/session.
    Left {
        /// Transport-level leave reason.
        reason: DisconnectReason,
    },
}

/// Authoritative lobby snapshot stored in the session authoritative-state bytes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct LobbySnapshot {
    pub id: String,
    pub name: String,
    pub max_players: u32,
    pub visibility: LobbyVisibility,
    pub state: LobbyState,
    pub members: Vec<LobbyMember>,
}
