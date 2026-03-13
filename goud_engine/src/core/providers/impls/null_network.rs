//! Null network provider -- no-op for games that do not use networking.

use crate::core::error::GoudResult;
use crate::core::providers::diagnostics::NetworkDiagnosticsV1;
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, HostConfig, NetworkCapabilities,
    NetworkEvent, NetworkStats,
};
use crate::core::providers::{Provider, ProviderLifecycle};

/// A network provider that does nothing. Used for headless testing and as
/// a default when no networking backend is needed.
pub struct NullNetworkProvider {
    capabilities: NetworkCapabilities,
}

impl NullNetworkProvider {
    /// Create a new null network provider.
    pub fn new() -> Self {
        Self {
            capabilities: NetworkCapabilities {
                supports_hosting: false,
                max_connections: 0,
                max_channels: 0,
                max_message_size: 0,
            },
        }
    }
}

impl Default for NullNetworkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for NullNetworkProvider {
    fn name(&self) -> &str {
        "null"
    }

    fn version(&self) -> &str {
        "0.0.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for NullNetworkProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {}
}

impl NetworkProvider for NullNetworkProvider {
    fn host(&mut self, _config: &HostConfig) -> GoudResult<()> {
        Ok(())
    }

    fn connect(&mut self, _addr: &str) -> GoudResult<ConnectionId> {
        Ok(ConnectionId(0))
    }

    fn disconnect(&mut self, _conn: ConnectionId) -> GoudResult<()> {
        Ok(())
    }

    fn disconnect_all(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn send(&mut self, _conn: ConnectionId, _channel: Channel, _data: &[u8]) -> GoudResult<()> {
        Ok(())
    }

    fn broadcast(&mut self, _channel: Channel, _data: &[u8]) -> GoudResult<()> {
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<NetworkEvent> {
        Vec::new()
    }

    fn connections(&self) -> Vec<ConnectionId> {
        Vec::new()
    }

    fn connection_state(&self, _conn: ConnectionId) -> ConnectionState {
        ConnectionState::Disconnected
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_network_construction() {
        let provider = NullNetworkProvider::new();
        assert_eq!(provider.name(), "null");
        assert_eq!(provider.version(), "0.0.0");
    }

    #[test]
    fn test_null_network_default() {
        let provider = NullNetworkProvider::default();
        assert_eq!(provider.name(), "null");
    }

    #[test]
    fn test_null_network_lifecycle() {
        let mut provider = NullNetworkProvider::new();
        assert!(provider.init().is_ok());
        assert!(provider.update(0.016).is_ok());
        provider.shutdown();
    }

    #[test]
    fn test_null_network_capabilities() {
        let provider = NullNetworkProvider::new();
        let caps = provider.network_capabilities();
        assert!(!caps.supports_hosting);
        assert_eq!(caps.max_connections, 0);
        assert_eq!(caps.max_channels, 0);
        assert_eq!(caps.max_message_size, 0);
    }

    #[test]
    fn test_null_network_host_and_connect() {
        let mut provider = NullNetworkProvider::new();
        let config = HostConfig {
            bind_address: "0.0.0.0".to_string(),
            port: 7777,
            max_connections: 16,
            tls_cert_path: None,
            tls_key_path: None,
        };
        assert!(provider.host(&config).is_ok());

        let conn = provider.connect("127.0.0.1:7777").unwrap();
        assert_eq!(conn, ConnectionId(0));

        assert!(provider.disconnect(conn).is_ok());
        assert!(provider.disconnect_all().is_ok());
    }

    #[test]
    fn test_null_network_send_and_broadcast() {
        let mut provider = NullNetworkProvider::new();
        let data = b"hello";
        assert!(provider.send(ConnectionId(0), Channel(0), data).is_ok());
        assert!(provider.broadcast(Channel(0), data).is_ok());
    }

    #[test]
    fn test_null_network_drain_events() {
        let mut provider = NullNetworkProvider::new();
        let events = provider.drain_events();
        assert!(events.is_empty());
    }

    #[test]
    fn test_null_network_connections() {
        let provider = NullNetworkProvider::new();
        assert!(provider.connections().is_empty());
        assert_eq!(
            provider.connection_state(ConnectionId(0)),
            ConnectionState::Disconnected
        );
        assert!(provider.local_id().is_none());
    }

    #[test]
    fn test_null_network_stats() {
        let provider = NullNetworkProvider::new();
        let stats = provider.stats();
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.packets_sent, 0);
        assert_eq!(stats.packets_received, 0);
        assert_eq!(stats.packets_lost, 0);
        assert_eq!(stats.rtt_ms, 0.0);
        assert_eq!(stats.send_bandwidth_bytes_per_sec, 0.0);
        assert_eq!(stats.receive_bandwidth_bytes_per_sec, 0.0);
        assert_eq!(stats.packet_loss_percent, 0.0);
        assert_eq!(stats.jitter_ms, 0.0);
        assert!(provider.connection_stats(ConnectionId(0)).is_none());
    }
}
