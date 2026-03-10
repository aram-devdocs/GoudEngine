//! Lobby client wrapper.

use crate::core::error::{GoudError, GoudResult};
use crate::core::networking::discovery::{
    DiscoveredSession, DiscoveryError, DiscoveryMode, DiscoveryService,
};
use crate::core::networking::{ClientEvent, SessionClient};
use crate::core::providers::network_types::ConnectionId;
use crate::core::serialization::binary;

use super::types::{LobbyCommand, LobbyEvent, LobbyMember, LobbySnapshot, LobbyState};

/// Client-side lobby wrapper layered on top of a session client.
pub struct LobbyClient {
    session: SessionClient,
    latest_snapshot: Option<LobbySnapshot>,
}

impl LobbyClient {
    /// Creates a new lobby client wrapper.
    pub fn new(session: SessionClient) -> Self {
        Self {
            session,
            latest_snapshot: None,
        }
    }

    /// Returns the wrapped session client.
    pub fn session(&self) -> &SessionClient {
        &self.session
    }

    /// Returns the wrapped session client mutably.
    pub fn session_mut(&mut self) -> &mut SessionClient {
        &mut self.session
    }

    /// Returns the latest server-authoritative lobby state, if any.
    pub fn state(&self) -> Option<LobbyState> {
        self.latest_snapshot.as_ref().map(|snapshot| snapshot.state)
    }

    /// Returns the latest server-authoritative member list, if any.
    pub fn members(&self) -> Option<Vec<LobbyMember>> {
        self.latest_snapshot
            .as_ref()
            .map(|snapshot| snapshot.members.clone())
    }

    /// Lists only public lobbies from the requested discovery mode.
    pub fn discover_public_lobbies(
        &self,
        discovery: &DiscoveryService,
        mode: DiscoveryMode,
    ) -> Result<Vec<DiscoveredSession>, DiscoveryError> {
        let mut sessions = self.session.discover_sessions(discovery, mode)?;
        sessions.retain(|entry| {
            entry
                .session
                .metadata
                .get("lobby.visibility")
                .is_some_and(|value| value == "public")
        });
        Ok(sessions)
    }

    /// Joins a listed public lobby.
    pub fn join_listed_lobby(&mut self, lobby: &DiscoveredSession) -> GoudResult<()> {
        self.session.join_discovered(lobby).map(|_| ())
    }

    /// Joins a public lobby by room ID from the requested discovery mode.
    pub fn join_lobby_by_id(
        &mut self,
        discovery: &DiscoveryService,
        mode: DiscoveryMode,
        lobby_id: &str,
    ) -> GoudResult<()> {
        let lobbies = self
            .discover_public_lobbies(discovery, mode)
            .map_err(map_discovery_error)?;
        let lobby = lobbies
            .into_iter()
            .find(|entry| {
                entry
                    .session
                    .metadata
                    .get("lobby.id")
                    .is_some_and(|value| value == lobby_id)
            })
            .ok_or_else(|| {
                GoudError::ResourceNotFound(format!("Lobby '{lobby_id}' was not found"))
            })?;
        self.join_listed_lobby(&lobby)
    }

    /// Joins a private lobby directly by address.
    pub fn join_private_lobby(&mut self, address: &str) -> GoudResult<()> {
        self.session.join_direct(address).map(|_| ())
    }

    /// Sends a ready-state update.
    pub fn set_ready(&mut self, ready: bool) -> GoudResult<()> {
        self.send_command(&LobbyCommand::SetReady { ready })
    }

    /// Requests a normal start.
    pub fn start_game(&mut self) -> GoudResult<()> {
        self.send_command(&LobbyCommand::StartGame)
    }

    /// Requests a host-only early start.
    pub fn start_early(&mut self) -> GoudResult<()> {
        self.send_command(&LobbyCommand::StartEarly)
    }

    /// Requests a host-only kick.
    pub fn kick(&mut self, connection: ConnectionId) -> GoudResult<()> {
        self.send_command(&LobbyCommand::Kick { connection })
    }

    /// Advances lobby/session state and emits high-level lobby events.
    pub fn tick(&mut self) -> GoudResult<Vec<LobbyEvent>> {
        let mut events = Vec::new();

        for event in self.session.tick()? {
            match event {
                ClientEvent::Joined { snapshot, .. } => {
                    if snapshot.is_empty() {
                        continue;
                    }
                    let snapshot: LobbySnapshot = binary::decode(&snapshot)?;
                    self.latest_snapshot = Some(snapshot.clone());
                    events.push(LobbyEvent::SnapshotUpdated {
                        state: snapshot.state,
                        members: snapshot.members,
                    });
                }
                ClientEvent::StateUpdated { payload, .. } => {
                    let snapshot: LobbySnapshot = binary::decode(&payload)?;
                    self.latest_snapshot = Some(snapshot.clone());
                    events.push(LobbyEvent::SnapshotUpdated {
                        state: snapshot.state,
                        members: snapshot.members,
                    });
                }
                ClientEvent::ValidationRejected { payload, reason } => {
                    let command = binary::decode(&payload).ok();
                    if command.is_some() {
                        events.push(LobbyEvent::CommandRejected { command, reason });
                    } else {
                        events.push(LobbyEvent::JoinRejected { reason });
                    }
                }
                ClientEvent::Left { reason, .. } => {
                    events.push(LobbyEvent::Left { reason });
                }
                ClientEvent::Connected { .. } | ClientEvent::ProtocolError { .. } => {}
            }
        }

        Ok(events)
    }

    fn send_command(&mut self, command: &LobbyCommand) -> GoudResult<()> {
        self.session.send_state_command(binary::encode(command)?)
    }
}

fn map_discovery_error(error: DiscoveryError) -> GoudError {
    GoudError::ProviderError {
        subsystem: "network",
        message: format!("Lobby discovery failed: {error:?}"),
    }
}
