//! Session server orchestration.

use std::collections::HashSet;
use std::time::Instant;

use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{ConnectionId, ConnectionState, NetworkEvent};

use super::authority::{AuthorityPolicy, BuiltInAuthorityPolicy};
use super::discovery::{
    register_native_lan_session, unregister_native_lan_session, update_native_lan_population,
};
use super::protocol::{decode_message, encode_message, ProtocolMessage};
use super::types::{ServerConfig, ServerEvent, SessionDescriptor};

/// Session server managing authoritative state for multiple clients.
pub struct SessionServer {
    provider: Box<dyn NetworkProvider>,
    authority: Box<dyn AuthorityPolicy>,
    connected_clients: HashSet<ConnectionId>,
    joined_clients: HashSet<ConnectionId>,
    authoritative_state: Vec<u8>,
    state_sequence: u64,
    auto_broadcast_commands: bool,
    config: Option<ServerConfig>,
    advertised_session_id: Option<String>,
}

impl std::fmt::Debug for SessionServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionServer")
            .field("connected_clients", &self.connected_clients.len())
            .field("joined_clients", &self.joined_clients.len())
            .field("state_sequence", &self.state_sequence)
            .finish()
    }
}

impl SessionServer {
    /// Creates a server with a custom authority policy.
    pub fn new(provider: Box<dyn NetworkProvider>, authority: Box<dyn AuthorityPolicy>) -> Self {
        Self {
            provider,
            authority,
            connected_clients: HashSet::new(),
            joined_clients: HashSet::new(),
            authoritative_state: Vec::new(),
            state_sequence: 0,
            auto_broadcast_commands: true,
            config: None,
            advertised_session_id: None,
        }
    }

    /// Creates a server with one of the built-in authority policies.
    pub fn with_policy(provider: Box<dyn NetworkProvider>, policy: BuiltInAuthorityPolicy) -> Self {
        Self::new(provider, policy.build())
    }

    /// Starts hosting a session.
    pub fn host(&mut self, config: ServerConfig) -> GoudResult<()> {
        self.provider.host(&config.host)?;

        let should_unregister_previous =
            self.advertised_session_id
                .as_deref()
                .is_some_and(|previous_id| {
                    !config.advertise_on_lan || previous_id != config.session.id.as_str()
                });
        if should_unregister_previous {
            if let Some(previous_id) = self.advertised_session_id.as_deref() {
                unregister_native_lan_session(previous_id).map_err(|error| {
                    network_error(format!("Failed to unregister LAN session: {error:?}"))
                })?;
            }
        }

        if config.advertise_on_lan {
            register_native_lan_session(config.session.clone()).map_err(|error| {
                network_error(format!("Failed to register LAN session: {error:?}"))
            })?;
            self.advertised_session_id = Some(config.session.id.clone());
        } else {
            self.advertised_session_id = None;
        }
        self.auto_broadcast_commands = config.auto_broadcast_commands;
        self.config = Some(config);
        Ok(())
    }

    /// Returns the number of clients that completed join handshake.
    pub fn client_count(&self) -> usize {
        self.joined_clients.len()
    }

    /// Returns the latest authoritative state bytes.
    pub fn authoritative_state(&self) -> &[u8] {
        &self.authoritative_state
    }

    /// Returns the latest authoritative state sequence.
    pub fn state_sequence(&self) -> u64 {
        self.state_sequence
    }

    /// Returns whether accepted client commands auto-broadcast as authoritative state.
    pub fn auto_broadcast_commands(&self) -> bool {
        self.auto_broadcast_commands
    }

    /// Enables or disables automatic authoritative-state broadcasts for accepted commands.
    pub fn set_auto_broadcast_commands(&mut self, enabled: bool) {
        self.auto_broadcast_commands = enabled;
    }

    /// Returns the current hosted session descriptor, if the server is hosted.
    pub fn session_descriptor(&self) -> Option<&SessionDescriptor> {
        self.config.as_ref().map(|config| &config.session)
    }

    /// Replaces the hosted session descriptor and refreshes discovery advertisement state.
    pub fn update_session_descriptor(&mut self, descriptor: SessionDescriptor) -> GoudResult<()> {
        let Some(config) = self.config.as_mut() else {
            return Err(GoudError::InvalidState(
                "Session server must be hosted before updating its descriptor".to_string(),
            ));
        };

        let previous_id = config.session.id.clone();
        config.session = descriptor.clone();

        if config.advertise_on_lan {
            if previous_id != descriptor.id {
                unregister_native_lan_session(&previous_id).map_err(|error| {
                    network_error(format!("Failed to unregister LAN session: {error:?}"))
                })?;
            }
            register_native_lan_session(descriptor.clone()).map_err(|error| {
                network_error(format!("Failed to register LAN session: {error:?}"))
            })?;
            self.advertised_session_id = Some(descriptor.id);
        }

        Ok(())
    }

    /// Sends a rejection response to one client without changing authoritative state.
    pub fn send_validation_rejection(
        &mut self,
        connection: ConnectionId,
        payload: Vec<u8>,
        reason: impl Into<String>,
    ) -> GoudResult<()> {
        self.send_protocol(
            connection,
            &ProtocolMessage::ValidationRejected {
                reason: reason.into(),
                payload,
            },
        )
    }

    /// Gracefully disconnects one client and updates membership tracking immediately.
    pub fn disconnect_client(
        &mut self,
        connection: ConnectionId,
        reason: impl Into<String>,
    ) -> GoudResult<()> {
        let reason = reason.into();
        if self.provider.connection_state(connection) != ConnectionState::Disconnected {
            let _ = self.send_protocol(
                connection,
                &ProtocolMessage::LeaveNotice {
                    reason: reason.clone(),
                },
            );
            let _ = self.provider.disconnect(connection);
        }
        self.cleanup_connection_state(connection)
    }

    /// Advances session state and drains server events.
    pub fn tick(&mut self) -> GoudResult<Vec<ServerEvent>> {
        self.provider.update(0.0)?;
        let mut events = Vec::new();

        for network_event in self.provider.drain_events() {
            match network_event {
                NetworkEvent::Connected { conn } => {
                    self.connected_clients.insert(conn);
                }
                NetworkEvent::Disconnected { conn, reason } => {
                    self.cleanup_connection_state(conn)?;
                    events.push(ServerEvent::ClientLeft {
                        connection: conn,
                        reason,
                    });
                }
                NetworkEvent::Received {
                    conn,
                    channel: _,
                    data,
                } => match decode_message(&data) {
                    Ok(message) => self.handle_protocol_message(conn, message, &mut events)?,
                    Err(error) => events.push(ServerEvent::ProtocolError {
                        connection: conn,
                        reason: format!("Failed to decode protocol payload: {error}"),
                    }),
                },
                NetworkEvent::Error { conn, message } => {
                    self.cleanup_connection_state(conn)?;
                    events.push(ServerEvent::ProtocolError {
                        connection: conn,
                        reason: message,
                    });
                }
            }
        }

        Ok(events)
    }

    /// Applies and broadcasts a new authoritative state payload.
    pub fn broadcast_authoritative_state(&mut self, payload: Vec<u8>) -> GoudResult<ServerEvent> {
        self.state_sequence = self.state_sequence.wrapping_add(1);
        self.authoritative_state = payload.clone();

        let message = ProtocolMessage::StateUpdate {
            sequence: self.state_sequence,
            payload: payload.clone(),
        };
        let bytes = encode_message(&message)?;
        let targets: Vec<ConnectionId> = self.joined_clients.iter().copied().collect();

        for connection in &targets {
            self.provider
                .send(*connection, self.channels().state, &bytes)?;
        }

        Ok(ServerEvent::StateBroadcast {
            sequence: self.state_sequence,
            recipients: targets.len(),
            payload,
        })
    }

    fn handle_protocol_message(
        &mut self,
        connection: ConnectionId,
        message: ProtocolMessage,
        events: &mut Vec<ServerEvent>,
    ) -> GoudResult<()> {
        match message {
            ProtocolMessage::JoinRequest => {
                if !self.connected_clients.contains(&connection) {
                    events.push(ServerEvent::ProtocolError {
                        connection,
                        reason: "JoinRequest received from unknown connection".to_string(),
                    });
                    return Ok(());
                }

                let newly_joined = self.joined_clients.insert(connection);
                self.send_protocol(
                    connection,
                    &ProtocolMessage::JoinAccepted {
                        snapshot: self.authoritative_state.clone(),
                    },
                )?;
                if newly_joined {
                    self.sync_lan_population()?;
                    events.push(ServerEvent::ClientJoined { connection });
                }
            }
            ProtocolMessage::StateCommand { payload } => {
                if !self.joined_clients.contains(&connection) {
                    let reason = "Client must join before sending state commands".to_string();
                    self.send_protocol(
                        connection,
                        &ProtocolMessage::ValidationRejected {
                            reason: reason.clone(),
                            payload: payload.clone(),
                        },
                    )?;
                    events.push(ServerEvent::CommandRejected {
                        connection,
                        payload,
                        reason,
                    });
                    return Ok(());
                }

                let decision = self
                    .authority
                    .validate(&super::authority::ValidationContext {
                        connection,
                        payload: &payload,
                        received_at: Instant::now(),
                    });

                match decision {
                    super::authority::AuthorityDecision::Accept => {
                        events.push(ServerEvent::CommandAccepted {
                            connection,
                            payload: payload.clone(),
                        });
                        if self.auto_broadcast_commands {
                            let broadcast_event = self.broadcast_authoritative_state(payload)?;
                            events.push(broadcast_event);
                        }
                    }
                    super::authority::AuthorityDecision::Reject { reason } => {
                        self.send_protocol(
                            connection,
                            &ProtocolMessage::ValidationRejected {
                                reason: reason.clone(),
                                payload: payload.clone(),
                            },
                        )?;
                        events.push(ServerEvent::CommandRejected {
                            connection,
                            payload,
                            reason,
                        });
                    }
                }
            }
            ProtocolMessage::LeaveNotice { reason } => {
                if self.provider.connection_state(connection)
                    != crate::core::providers::network_types::ConnectionState::Disconnected
                {
                    let _ =
                        self.send_protocol(connection, &ProtocolMessage::LeaveNotice { reason });
                    let _ = self.provider.disconnect(connection);
                }
            }
            ProtocolMessage::JoinAccepted { .. }
            | ProtocolMessage::StateUpdate { .. }
            | ProtocolMessage::ValidationRejected { .. } => {
                events.push(ServerEvent::ProtocolError {
                    connection,
                    reason: "Unexpected server-bound protocol message".to_string(),
                });
            }
        }

        Ok(())
    }

    fn send_protocol(
        &mut self,
        connection: ConnectionId,
        message: &ProtocolMessage,
    ) -> GoudResult<()> {
        let channel = protocol_channel(self.channels(), message);
        let encoded = encode_message(message)?;
        self.provider.send(connection, channel, &encoded)
    }

    fn channels(&self) -> super::types::SessionChannels {
        self.config
            .as_ref()
            .map(|config| config.channels)
            .unwrap_or_default()
    }

    fn cleanup_connection_state(&mut self, connection: ConnectionId) -> GoudResult<()> {
        self.connected_clients.remove(&connection);
        let was_joined = self.joined_clients.remove(&connection);
        self.authority.on_client_disconnected(connection);
        if was_joined {
            self.sync_lan_population()?;
        }
        Ok(())
    }

    fn sync_lan_population(&mut self) -> GoudResult<()> {
        let Some(config) = self.config.as_mut() else {
            return Ok(());
        };

        config.session.current_clients = self.joined_clients.len() as u32;
        if config.advertise_on_lan {
            update_native_lan_population(&config.session.id, config.session.current_clients)
                .map_err(|error| {
                    network_error(format!("Failed to update LAN population: {error:?}"))
                })?;
        }
        Ok(())
    }
}

impl Drop for SessionServer {
    fn drop(&mut self) {
        if let Some(session_id) = &self.advertised_session_id {
            let _ = unregister_native_lan_session(session_id);
        }
    }
}

fn protocol_channel(
    channels: super::types::SessionChannels,
    message: &ProtocolMessage,
) -> crate::core::providers::network_types::Channel {
    match message {
        ProtocolMessage::StateCommand { .. } => channels.command,
        ProtocolMessage::StateUpdate { .. } => channels.state,
        ProtocolMessage::JoinRequest
        | ProtocolMessage::JoinAccepted { .. }
        | ProtocolMessage::ValidationRejected { .. }
        | ProtocolMessage::LeaveNotice { .. } => channels.control,
    }
}

fn network_error(message: impl Into<String>) -> GoudError {
    GoudError::ProviderError {
        subsystem: "network",
        message: message.into(),
    }
}
