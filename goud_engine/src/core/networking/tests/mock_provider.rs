use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use crate::core::error::{GoudError, GoudResult};
use crate::core::networking::types::{ClientEvent, ServerConfig, ServerEvent, SessionDescriptor};
use crate::core::networking::{SessionClient, SessionServer};
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, DisconnectReason, HostConfig,
    NetworkCapabilities, NetworkEvent, NetworkStats,
};
use crate::core::providers::{Provider, ProviderLifecycle};

fn network_error(message: impl Into<String>) -> GoudError {
    GoudError::ProviderError {
        subsystem: "network",
        message: message.into(),
    }
}

#[derive(Default, Clone)]
pub(super) struct MockNetworkHub {
    state: Arc<Mutex<HubState>>,
}

#[derive(Default)]
struct HubState {
    next_endpoint: u64,
    endpoints: HashMap<u64, EndpointState>,
    addresses: HashMap<String, u64>,
}

#[derive(Default)]
struct EndpointState {
    hosted_address: Option<String>,
    next_connection: u64,
    connections: HashMap<u64, LocalConnection>,
    events: VecDeque<NetworkEvent>,
    stats: NetworkStats,
}

#[derive(Clone)]
struct LocalConnection {
    remote_endpoint: u64,
    remote_connection: ConnectionId,
    state: ConnectionState,
    stats: ConnectionStats,
}

impl MockNetworkHub {
    pub(super) fn provider(&self) -> MockNetworkProvider {
        let mut state = self.state.lock().expect("hub lock should work");
        state.next_endpoint = state.next_endpoint.wrapping_add(1);
        let endpoint_id = state.next_endpoint;
        state
            .endpoints
            .insert(endpoint_id, EndpointState::default());
        MockNetworkProvider::new(self.state.clone(), endpoint_id)
    }
}

pub(super) struct MockNetworkProvider {
    state: Arc<Mutex<HubState>>,
    endpoint_id: u64,
    capabilities: NetworkCapabilities,
}

impl MockNetworkProvider {
    fn new(state: Arc<Mutex<HubState>>, endpoint_id: u64) -> Self {
        Self {
            state,
            endpoint_id,
            capabilities: NetworkCapabilities {
                supports_hosting: true,
                max_connections: 64,
                max_channels: 8,
                max_message_size: 16_384,
            },
        }
    }

    fn allocate_connection(endpoint: &mut EndpointState) -> ConnectionId {
        endpoint.next_connection = endpoint.next_connection.wrapping_add(1);
        ConnectionId(endpoint.next_connection)
    }

    fn endpoint_mut(state: &mut HubState, id: u64) -> Result<&mut EndpointState, GoudError> {
        state
            .endpoints
            .get_mut(&id)
            .ok_or_else(|| network_error(format!("Unknown endpoint {id}")))
    }

    fn disconnect_internal(&mut self, connection: ConnectionId) -> GoudResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| network_error(format!("Hub lock poisoned: {e}")))?;

        let endpoint = Self::endpoint_mut(&mut state, self.endpoint_id)?;
        let local = endpoint
            .connections
            .remove(&connection.0)
            .ok_or_else(|| network_error(format!("Unknown connection {:?}", connection)))?;

        endpoint.events.push_back(NetworkEvent::Disconnected {
            conn: connection,
            reason: DisconnectReason::LocalClose,
        });

        if let Some(remote_endpoint) = state.endpoints.get_mut(&local.remote_endpoint) {
            remote_endpoint
                .connections
                .remove(&local.remote_connection.0);
            remote_endpoint
                .events
                .push_back(NetworkEvent::Disconnected {
                    conn: local.remote_connection,
                    reason: DisconnectReason::RemoteClose,
                });
        }

        Ok(())
    }

    fn send_internal(
        &mut self,
        connection: ConnectionId,
        channel: Channel,
        data: &[u8],
    ) -> GoudResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| network_error(format!("Hub lock poisoned: {e}")))?;

        let local = {
            let endpoint = Self::endpoint_mut(&mut state, self.endpoint_id)?;
            endpoint.stats.bytes_sent += data.len() as u64;
            endpoint.stats.packets_sent += 1;

            let stats_connection = endpoint
                .connections
                .get_mut(&connection.0)
                .ok_or_else(|| network_error(format!("Unknown connection {:?}", connection)))?;
            stats_connection.stats.bytes_sent += data.len() as u64;
            stats_connection.clone()
        };

        let Some(remote_endpoint) = state.endpoints.get_mut(&local.remote_endpoint) else {
            return Err(network_error("Remote endpoint does not exist"));
        };

        remote_endpoint.events.push_back(NetworkEvent::Received {
            conn: local.remote_connection,
            channel,
            data: data.to_vec(),
        });
        remote_endpoint.stats.bytes_received += data.len() as u64;
        remote_endpoint.stats.packets_received += 1;

        if let Some(remote_connection) = remote_endpoint
            .connections
            .get_mut(&local.remote_connection.0)
        {
            remote_connection.stats.bytes_received += data.len() as u64;
        }

        Ok(())
    }
}

impl Provider for MockNetworkProvider {
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

impl ProviderLifecycle for MockNetworkProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {
        let _ = self.disconnect_all();

        if let Ok(mut state) = self.state.lock() {
            let hosted_address = state
                .endpoints
                .get_mut(&self.endpoint_id)
                .and_then(|endpoint| endpoint.hosted_address.take());
            if let Some(address) = hosted_address {
                state.addresses.remove(&address);
            }
        }
    }
}

impl NetworkProvider for MockNetworkProvider {
    fn host(&mut self, config: &HostConfig) -> GoudResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| network_error(format!("Hub lock poisoned: {e}")))?;

        let address = format!("{}:{}", config.bind_address, config.port);
        if state.addresses.contains_key(&address) {
            return Err(network_error(format!("Address {address} already hosted")));
        }

        {
            let endpoint = Self::endpoint_mut(&mut state, self.endpoint_id)?;
            endpoint.hosted_address = Some(address.clone());
        }
        state.addresses.insert(address, self.endpoint_id);

        Ok(())
    }

    fn connect(&mut self, address: &str) -> GoudResult<ConnectionId> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| network_error(format!("Hub lock poisoned: {e}")))?;

        let remote_id = *state
            .addresses
            .get(address)
            .ok_or_else(|| network_error(format!("No host listening at {address}")))?;

        if remote_id == self.endpoint_id {
            return Err(network_error("Loopback self-connect is not supported"));
        }

        let local_connection = {
            let local_endpoint = Self::endpoint_mut(&mut state, self.endpoint_id)?;
            Self::allocate_connection(local_endpoint)
        };

        let remote_connection = {
            let remote_endpoint = Self::endpoint_mut(&mut state, remote_id)?;
            Self::allocate_connection(remote_endpoint)
        };

        {
            let local_endpoint = Self::endpoint_mut(&mut state, self.endpoint_id)?;
            local_endpoint.connections.insert(
                local_connection.0,
                LocalConnection {
                    remote_endpoint: remote_id,
                    remote_connection,
                    state: ConnectionState::Connected,
                    stats: ConnectionStats::default(),
                },
            );
            local_endpoint.events.push_back(NetworkEvent::Connected {
                conn: local_connection,
            });
        }

        {
            let remote_endpoint = Self::endpoint_mut(&mut state, remote_id)?;
            remote_endpoint.connections.insert(
                remote_connection.0,
                LocalConnection {
                    remote_endpoint: self.endpoint_id,
                    remote_connection: local_connection,
                    state: ConnectionState::Connected,
                    stats: ConnectionStats::default(),
                },
            );
            remote_endpoint.events.push_back(NetworkEvent::Connected {
                conn: remote_connection,
            });
        }

        Ok(local_connection)
    }

    fn disconnect(&mut self, connection: ConnectionId) -> GoudResult<()> {
        self.disconnect_internal(connection)
    }

    fn disconnect_all(&mut self) -> GoudResult<()> {
        let ids: Vec<ConnectionId> = {
            let state = self
                .state
                .lock()
                .map_err(|e| network_error(format!("Hub lock poisoned: {e}")))?;
            let endpoint = state
                .endpoints
                .get(&self.endpoint_id)
                .ok_or_else(|| network_error("Endpoint not found"))?;
            endpoint
                .connections
                .keys()
                .copied()
                .map(ConnectionId)
                .collect()
        };

        for id in ids {
            let _ = self.disconnect_internal(id);
        }

        Ok(())
    }

    fn send(&mut self, connection: ConnectionId, channel: Channel, data: &[u8]) -> GoudResult<()> {
        self.send_internal(connection, channel, data)
    }

    fn broadcast(&mut self, channel: Channel, data: &[u8]) -> GoudResult<()> {
        let ids = self.connections();
        for connection in ids {
            self.send_internal(connection, channel, data)?;
        }
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<NetworkEvent> {
        if let Ok(mut state) = self.state.lock() {
            if let Some(endpoint) = state.endpoints.get_mut(&self.endpoint_id) {
                return endpoint.events.drain(..).collect();
            }
        }

        Vec::new()
    }

    fn connections(&self) -> Vec<ConnectionId> {
        if let Ok(state) = self.state.lock() {
            if let Some(endpoint) = state.endpoints.get(&self.endpoint_id) {
                return endpoint
                    .connections
                    .keys()
                    .copied()
                    .map(ConnectionId)
                    .collect();
            }
        }

        Vec::new()
    }

    fn connection_state(&self, connection: ConnectionId) -> ConnectionState {
        if let Ok(state) = self.state.lock() {
            if let Some(endpoint) = state.endpoints.get(&self.endpoint_id) {
                return endpoint
                    .connections
                    .get(&connection.0)
                    .map(|conn| conn.state)
                    .unwrap_or(ConnectionState::Disconnected);
            }
        }

        ConnectionState::Disconnected
    }

    fn local_id(&self) -> Option<ConnectionId> {
        None
    }

    fn network_capabilities(&self) -> &NetworkCapabilities {
        &self.capabilities
    }

    fn stats(&self) -> NetworkStats {
        if let Ok(state) = self.state.lock() {
            if let Some(endpoint) = state.endpoints.get(&self.endpoint_id) {
                return endpoint.stats.clone();
            }
        }

        NetworkStats::default()
    }

    fn connection_stats(&self, connection: ConnectionId) -> Option<ConnectionStats> {
        if let Ok(state) = self.state.lock() {
            if let Some(endpoint) = state.endpoints.get(&self.endpoint_id) {
                return endpoint
                    .connections
                    .get(&connection.0)
                    .map(|conn| conn.stats.clone());
            }
        }

        None
    }
}

pub(super) fn pump(
    server: &mut SessionServer,
    clients: &mut [&mut SessionClient],
    ticks: usize,
) -> (Vec<ServerEvent>, Vec<Vec<ClientEvent>>) {
    let mut server_events = Vec::new();
    let mut client_events: Vec<Vec<ClientEvent>> = (0..clients.len()).map(|_| Vec::new()).collect();

    for _ in 0..ticks {
        server_events.extend(server.tick().expect("server tick should succeed"));
        for (index, client) in clients.iter_mut().enumerate() {
            client_events[index].extend(client.tick().expect("client tick should succeed"));
        }
    }

    (server_events, client_events)
}

pub(super) fn create_server_config(port: u16, id: &str, advertise_on_lan: bool) -> ServerConfig {
    let host = HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port,
        max_connections: 16,
        tls_cert_path: None,
        tls_key_path: None,
    };

    let mut session = SessionDescriptor::new(id, "Test Session", format!("127.0.0.1:{port}"));
    session.max_clients = 16;

    let mut config = ServerConfig::new(host, session);
    config.advertise_on_lan = advertise_on_lan;
    config
}
