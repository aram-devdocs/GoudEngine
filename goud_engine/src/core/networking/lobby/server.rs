//! Lobby server wrapper.

use std::collections::BTreeMap;

use crate::core::error::{GoudError, GoudResult};
use crate::core::networking::{ServerConfig, ServerEvent, SessionServer};
use crate::core::providers::network_types::ConnectionId;
use crate::core::serialization::binary;

use super::types::{
    LobbyCommand, LobbyConfig, LobbyEvent, LobbyMember, LobbySnapshot, LobbyState,
    LobbyValidationEnvelope, LobbyVisibility,
};

const LOBBY_FULL_REASON: &str = "lobby is full";
const HOST_ONLY_KICK_REASON: &str = "only the host can kick members";
const HOST_SELF_KICK_REASON: &str = "host cannot kick themselves";
const HOST_ONLY_START_REASON: &str = "only the host can start the lobby";
const HOST_ONLY_EARLY_START_REASON: &str = "only the host can start early";
const READY_GATE_REASON: &str = "all players must be ready before starting";

/// Server-authoritative lobby manager layered on top of a session server.
pub struct LobbyServer {
    session: SessionServer,
    config: LobbyConfig,
    state: LobbyState,
    members: BTreeMap<ConnectionId, LobbyMember>,
}

impl LobbyServer {
    /// Creates a new lobby server wrapper.
    pub fn new(mut session: SessionServer, config: LobbyConfig) -> Self {
        session.set_auto_broadcast_commands(false);
        Self {
            session,
            config,
            state: LobbyState::Waiting,
            members: BTreeMap::new(),
        }
    }

    /// Returns the wrapped session server.
    pub fn session(&self) -> &SessionServer {
        &self.session
    }

    /// Returns the wrapped session server mutably.
    pub fn session_mut(&mut self) -> &mut SessionServer {
        &mut self.session
    }

    /// Returns the configured lobby settings.
    pub fn config(&self) -> &LobbyConfig {
        &self.config
    }

    /// Returns the current server-authoritative lobby state.
    pub fn state(&self) -> LobbyState {
        self.state
    }

    /// Returns the current member list.
    pub fn members(&self) -> Vec<LobbyMember> {
        self.members.values().cloned().collect()
    }

    /// Hosts the lobby on top of the wrapped session server.
    pub fn host(&mut self, mut server_config: ServerConfig) -> GoudResult<()> {
        if self.config.max_players == 0 {
            return Err(GoudError::InvalidState(
                "Lobby max_players must be greater than zero".to_string(),
            ));
        }

        server_config.session.id = self.config.id.clone();
        server_config.session.name = self.config.name.clone();
        server_config.session.max_clients = self.config.max_players;
        server_config.session.current_clients = 0;
        server_config.session.metadata = self.metadata();
        server_config.auto_broadcast_commands = false;
        if self.config.visibility == LobbyVisibility::Private {
            server_config.advertise_on_lan = false;
        }

        self.session.host(server_config)?;
        self.broadcast_snapshot()?;
        Ok(())
    }

    /// Advances lobby/session state and emits authoritative lobby events.
    pub fn tick(&mut self) -> GoudResult<Vec<LobbyEvent>> {
        let mut lobby_events = Vec::new();
        let mut snapshot_dirty = false;

        for event in self.session.tick()? {
            match event {
                ServerEvent::ClientJoined { connection } => {
                    if self.members.len() as u32 >= self.config.max_players {
                        self.send_rejection(
                            connection,
                            LobbyValidationEnvelope::JoinRequest,
                            LOBBY_FULL_REASON,
                        )?;
                        self.session
                            .disconnect_client(connection, LOBBY_FULL_REASON)?;
                        lobby_events.push(LobbyEvent::JoinDenied {
                            connection,
                            reason: LOBBY_FULL_REASON.to_string(),
                        });
                        continue;
                    }

                    let member = LobbyMember {
                        connection,
                        is_host: self.members.is_empty(),
                        ready: false,
                    };
                    self.members.insert(connection, member.clone());
                    lobby_events.push(LobbyEvent::MemberJoined { member });
                    snapshot_dirty = true;
                }
                ServerEvent::ClientLeft { connection, .. } => {
                    if self.members.remove(&connection).is_some() {
                        self.reassign_host();
                        lobby_events.push(LobbyEvent::MemberLeft { connection });
                        snapshot_dirty = true;
                    }
                }
                ServerEvent::CommandAccepted {
                    connection,
                    payload,
                } => {
                    snapshot_dirty |=
                        self.handle_command(connection, payload, &mut lobby_events)?;
                }
                ServerEvent::CommandRejected {
                    payload, reason, ..
                } => {
                    lobby_events.push(LobbyEvent::CommandRejected {
                        command: binary::decode(&payload).ok(),
                        reason,
                    });
                }
                ServerEvent::StateBroadcast { .. } | ServerEvent::ProtocolError { .. } => {}
            }
        }

        if snapshot_dirty {
            self.sync_descriptor()?;
            self.broadcast_snapshot()?;
            lobby_events.push(LobbyEvent::SnapshotUpdated {
                state: self.state,
                members: self.members(),
            });
        }

        Ok(lobby_events)
    }

    fn handle_command(
        &mut self,
        connection: ConnectionId,
        payload: Vec<u8>,
        events: &mut Vec<LobbyEvent>,
    ) -> GoudResult<bool> {
        let command: LobbyCommand = match binary::decode(&payload) {
            Ok(command) => command,
            Err(error) => {
                let reason = format!("invalid lobby command: {error}");
                self.send_rejection(
                    connection,
                    LobbyValidationEnvelope::Command { command: None },
                    reason.clone(),
                )?;
                events.push(LobbyEvent::CommandRejected {
                    command: None,
                    reason,
                });
                return Ok(false);
            }
        };

        match command.clone() {
            LobbyCommand::SetReady { ready } => {
                let Some(member) = self.members.get_mut(&connection) else {
                    return self.reject_command(
                        connection,
                        command,
                        "unknown lobby member",
                        events,
                    );
                };
                member.ready = ready;
                Ok(true)
            }
            LobbyCommand::StartGame => {
                if !self.is_host(connection) {
                    return self.reject_command(
                        connection,
                        command,
                        HOST_ONLY_START_REASON,
                        events,
                    );
                }
                if !self.all_players_ready() {
                    return self.reject_command(connection, command, READY_GATE_REASON, events);
                }
                self.state = LobbyState::InGame;
                events.push(LobbyEvent::Started { early: false });
                Ok(true)
            }
            LobbyCommand::StartEarly => {
                if !self.is_host(connection) {
                    return self.reject_command(
                        connection,
                        command,
                        HOST_ONLY_EARLY_START_REASON,
                        events,
                    );
                }
                self.state = LobbyState::InGame;
                events.push(LobbyEvent::Started { early: true });
                Ok(true)
            }
            LobbyCommand::Kick { connection: target } => {
                if !self.is_host(connection) {
                    return self.reject_command(connection, command, HOST_ONLY_KICK_REASON, events);
                }
                if target == connection {
                    return self.reject_command(connection, command, HOST_SELF_KICK_REASON, events);
                }
                if self.members.remove(&target).is_none() {
                    return self.reject_command(
                        connection,
                        command,
                        "target member is not in the lobby",
                        events,
                    );
                }
                self.session.disconnect_client(target, "kicked by host")?;
                self.reassign_host();
                events.push(LobbyEvent::MemberLeft { connection: target });
                Ok(true)
            }
        }
    }

    fn reject_command(
        &mut self,
        connection: ConnectionId,
        command: LobbyCommand,
        reason: &str,
        events: &mut Vec<LobbyEvent>,
    ) -> GoudResult<bool> {
        self.send_rejection(
            connection,
            LobbyValidationEnvelope::Command {
                command: Some(command.clone()),
            },
            reason,
        )?;
        events.push(LobbyEvent::CommandRejected {
            command: Some(command),
            reason: reason.to_string(),
        });
        Ok(false)
    }

    fn send_rejection(
        &mut self,
        connection: ConnectionId,
        envelope: LobbyValidationEnvelope,
        reason: impl Into<String>,
    ) -> GoudResult<()> {
        self.session.send_validation_rejection(
            connection,
            binary::encode(&envelope)?,
            reason.into(),
        )
    }

    fn is_host(&self, connection: ConnectionId) -> bool {
        self.members
            .get(&connection)
            .is_some_and(|member| member.is_host)
    }

    fn all_players_ready(&self) -> bool {
        !self.members.is_empty() && self.members.values().all(|member| member.ready)
    }

    fn reassign_host(&mut self) {
        let next_host = self.members.keys().next().copied();
        for member in self.members.values_mut() {
            member.is_host = Some(member.connection) == next_host;
        }
    }

    fn broadcast_snapshot(&mut self) -> GoudResult<()> {
        self.session
            .broadcast_authoritative_state(binary::encode(&self.snapshot())?)?;
        Ok(())
    }

    fn sync_descriptor(&mut self) -> GoudResult<()> {
        let Some(mut descriptor) = self.session.session_descriptor().cloned() else {
            return Ok(());
        };
        descriptor.metadata = self.metadata();
        descriptor.current_clients = self.members.len() as u32;
        descriptor.max_clients = self.config.max_players;
        descriptor.name = self.config.name.clone();
        self.session.update_session_descriptor(descriptor)
    }

    fn snapshot(&self) -> LobbySnapshot {
        LobbySnapshot {
            id: self.config.id.clone(),
            name: self.config.name.clone(),
            max_players: self.config.max_players,
            visibility: self.config.visibility,
            state: self.state,
            members: self.members(),
        }
    }

    fn metadata(&self) -> BTreeMap<String, String> {
        let mut metadata = BTreeMap::new();
        metadata.insert("lobby.id".to_string(), self.config.id.clone());
        metadata.insert("lobby.name".to_string(), self.config.name.clone());
        metadata.insert(
            "lobby.visibility".to_string(),
            self.config.visibility.as_metadata_value().to_string(),
        );
        metadata.insert(
            "lobby.state".to_string(),
            self.state.as_metadata_value().to_string(),
        );
        metadata.insert(
            "lobby.current_players".to_string(),
            self.members.len().to_string(),
        );
        metadata.insert(
            "lobby.max_players".to_string(),
            self.config.max_players.to_string(),
        );
        metadata
    }
}
