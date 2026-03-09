//! Session client orchestration.

use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{ConnectionId, DisconnectReason, NetworkEvent};

use super::discovery::{DiscoveredSession, DiscoveryMode, DiscoveryService};
use super::protocol::{decode_message, encode_message, ProtocolMessage};
use super::types::{ClientEvent, SessionChannels};

/// Session client that joins and tracks authoritative updates from a server.
pub struct SessionClient {
    provider: Box<dyn NetworkProvider>,
    channels: SessionChannels,
    server_connection: Option<ConnectionId>,
    joined: bool,
}

impl std::fmt::Debug for SessionClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionClient")
            .field("server_connection", &self.server_connection)
            .field("joined", &self.joined)
            .finish()
    }
}

impl SessionClient {
    /// Creates a session client with default protocol channels.
    pub fn new(provider: Box<dyn NetworkProvider>) -> Self {
        Self::with_channels(provider, SessionChannels::default())
    }

    /// Creates a session client with custom protocol channels.
    pub fn with_channels(provider: Box<dyn NetworkProvider>, channels: SessionChannels) -> Self {
        Self {
            provider,
            channels,
            server_connection: None,
            joined: false,
        }
    }

    /// Discovers sessions in the requested mode.
    pub fn discover_sessions(
        &self,
        discovery: &DiscoveryService,
        mode: DiscoveryMode,
    ) -> Result<Vec<DiscoveredSession>, super::discovery::DiscoveryError> {
        discovery.discover(mode)
    }

    /// Joins a server directly by address.
    pub fn join_direct(&mut self, address: &str) -> GoudResult<ConnectionId> {
        if self.server_connection.is_some() {
            return Err(GoudError::InvalidState(
                "Client already has an active server connection".to_string(),
            ));
        }

        let connection = self.provider.connect(address)?;
        self.server_connection = Some(connection);
        Ok(connection)
    }

    /// Joins a previously discovered session.
    pub fn join_discovered(&mut self, session: &DiscoveredSession) -> GoudResult<ConnectionId> {
        self.join_direct(&session.session.address)
    }

    /// Sends a state-change command to the server.
    pub fn send_state_command(&mut self, payload: Vec<u8>) -> GoudResult<()> {
        let connection = self.server_connection.ok_or_else(|| {
            GoudError::InvalidState("Client has no active server connection".to_string())
        })?;
        let message = ProtocolMessage::StateCommand { payload };
        let encoded = encode_message(&message)?;
        self.provider
            .send(connection, self.channels.command, &encoded)?;
        Ok(())
    }

    /// Leaves the current session gracefully.
    pub fn leave(&mut self, reason: impl Into<String>) -> GoudResult<()> {
        let connection = self.server_connection.ok_or_else(|| {
            GoudError::InvalidState("Client has no active server connection".to_string())
        })?;

        let leave_notice = ProtocolMessage::LeaveNotice {
            reason: reason.into(),
        };
        let encoded = encode_message(&leave_notice)?;
        self.provider
            .send(connection, self.channels.control, &encoded)?;
        self.provider.disconnect(connection)?;
        self.joined = false;
        self.server_connection = None;
        Ok(())
    }

    /// Advances session state and drains client events.
    pub fn tick(&mut self) -> GoudResult<Vec<ClientEvent>> {
        self.provider.update(0.0)?;
        let mut events = Vec::new();

        for network_event in self.provider.drain_events() {
            match network_event {
                NetworkEvent::Connected { conn } => {
                    if self.server_connection.is_none() {
                        self.server_connection = Some(conn);
                    }

                    if self.server_connection == Some(conn) {
                        let join = encode_message(&ProtocolMessage::JoinRequest)?;
                        self.provider.send(conn, self.channels.control, &join)?;
                        events.push(ClientEvent::Connected { connection: conn });
                    }
                }
                NetworkEvent::Disconnected { conn, reason } => {
                    if self.server_connection == Some(conn) {
                        self.joined = false;
                        self.server_connection = None;
                    }
                    events.push(ClientEvent::Left {
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
                    Err(error) => events.push(ClientEvent::ProtocolError {
                        reason: format!("Failed to decode protocol payload: {error}"),
                    }),
                },
                NetworkEvent::Error { conn, message } => {
                    if self.server_connection == Some(conn) {
                        self.joined = false;
                        self.server_connection = None;
                    }
                    events.push(ClientEvent::Left {
                        connection: conn,
                        reason: DisconnectReason::Error(message),
                    });
                }
            }
        }

        Ok(events)
    }

    /// Returns whether the client completed join handshake.
    pub fn is_joined(&self) -> bool {
        self.joined
    }

    /// Returns current server connection, if any.
    pub fn server_connection(&self) -> Option<ConnectionId> {
        self.server_connection
    }

    /// Returns configured session channels.
    pub fn channels(&self) -> SessionChannels {
        self.channels
    }

    fn handle_protocol_message(
        &mut self,
        connection: ConnectionId,
        message: ProtocolMessage,
        events: &mut Vec<ClientEvent>,
    ) -> GoudResult<()> {
        match message {
            ProtocolMessage::JoinAccepted { snapshot } => {
                if self.server_connection == Some(connection) {
                    self.joined = true;
                    events.push(ClientEvent::Joined {
                        connection,
                        snapshot,
                    });
                } else {
                    events.push(ClientEvent::ProtocolError {
                        reason: "Received JoinAccepted on unknown connection".to_string(),
                    });
                }
            }
            ProtocolMessage::StateUpdate { sequence, payload } => {
                events.push(ClientEvent::StateUpdated { sequence, payload });
            }
            ProtocolMessage::ValidationRejected { reason, payload } => {
                events.push(ClientEvent::ValidationRejected { payload, reason });
            }
            ProtocolMessage::LeaveNotice { reason } => {
                if self.server_connection == Some(connection) {
                    self.joined = false;
                    self.server_connection = None;
                }
                events.push(ClientEvent::Left {
                    connection,
                    reason: DisconnectReason::Error(reason),
                });
            }
            ProtocolMessage::JoinRequest | ProtocolMessage::StateCommand { .. } => {
                events.push(ClientEvent::ProtocolError {
                    reason: "Unexpected client-bound protocol message".to_string(),
                });
            }
        }

        Ok(())
    }
}
