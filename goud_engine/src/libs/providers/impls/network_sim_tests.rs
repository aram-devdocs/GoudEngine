use super::*;
use crate::core::providers::impls::NullNetworkProvider;
use crate::core::providers::network_types::DisconnectReason;
#[cfg(feature = "net-tcp")]
use crate::libs::providers::impls::TcpNetProvider;
use crate::libs::providers::impls::UdpNetProvider;
#[cfg(feature = "net-ws")]
use crate::libs::providers::impls::WsNetProvider;

#[derive(Debug, Default)]
struct RecordingProvider {
    sent: Vec<(ConnectionId, Channel, Vec<u8>)>,
    caps: NetworkCapabilities,
}

impl RecordingProvider {
    fn new() -> Self {
        Self {
            sent: Vec::new(),
            caps: NetworkCapabilities {
                supports_hosting: false,
                max_connections: 1,
                max_channels: 4,
                max_message_size: 1024,
            },
        }
    }
}

impl Provider for RecordingProvider {
    fn name(&self) -> &str {
        "recording"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.caps.clone())
    }
}

impl ProviderLifecycle for RecordingProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {}
}

impl NetworkProvider for RecordingProvider {
    fn host(&mut self, _config: &HostConfig) -> GoudResult<()> {
        Ok(())
    }

    fn connect(&mut self, _addr: &str) -> GoudResult<ConnectionId> {
        Ok(ConnectionId(1))
    }

    fn disconnect(&mut self, _conn: ConnectionId) -> GoudResult<()> {
        Ok(())
    }

    fn disconnect_all(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn send(&mut self, conn: ConnectionId, channel: Channel, data: &[u8]) -> GoudResult<()> {
        self.sent.push((conn, channel, data.to_vec()));
        Ok(())
    }

    fn broadcast(&mut self, channel: Channel, data: &[u8]) -> GoudResult<()> {
        self.send(ConnectionId(1), channel, data)
    }

    fn drain_events(&mut self) -> Vec<NetworkEvent> {
        Vec::new()
    }

    fn connections(&self) -> Vec<ConnectionId> {
        vec![ConnectionId(1)]
    }

    fn connection_state(&self, _conn: ConnectionId) -> ConnectionState {
        ConnectionState::Connected
    }

    fn local_id(&self) -> Option<ConnectionId> {
        Some(ConnectionId(99))
    }

    fn network_capabilities(&self) -> &NetworkCapabilities {
        &self.caps
    }

    fn stats(&self) -> NetworkStats {
        NetworkStats::default()
    }

    fn connection_stats(&self, _conn: ConnectionId) -> Option<ConnectionStats> {
        Some(ConnectionStats::default())
    }
}

#[test]
fn test_networking_sim_provider_delays_outbound_sends_until_update() {
    let inner = RecordingProvider::new();
    let mut simulated = NetworkSimProvider::new(inner);
    simulated
        .set_simulation_config(NetworkSimulationConfig {
            one_way_latency_ms: 25,
            jitter_ms: 0,
            packet_loss_percent: 0.0,
        })
        .unwrap();

    simulated
        .send(ConnectionId(1), Channel(0), b"delayed")
        .unwrap();
    assert!(simulated.inner().sent.is_empty());

    simulated.update(0.0).unwrap();
    assert!(simulated.inner().sent.is_empty());

    std::thread::sleep(Duration::from_millis(30));
    simulated.update(0.0).unwrap();

    assert_eq!(simulated.inner().sent.len(), 1);
    assert_eq!(simulated.inner().sent[0].2, b"delayed");
}

#[test]
fn test_networking_sim_provider_reports_dropped_packets_in_stats() {
    let inner = RecordingProvider::new();
    let mut simulated = NetworkSimProvider::new(inner);
    simulated
        .set_simulation_config(NetworkSimulationConfig {
            one_way_latency_ms: 0,
            jitter_ms: 0,
            packet_loss_percent: 100.0,
        })
        .unwrap();

    simulated
        .send(ConnectionId(1), Channel(0), b"lost")
        .unwrap();

    let stats = simulated.stats();
    let conn_stats = simulated.connection_stats(ConnectionId(1)).unwrap();
    assert!(simulated.inner().sent.is_empty());
    assert_eq!(stats.packets_lost, 1);
    assert_eq!(conn_stats.packets_lost, 1);
    assert_eq!(stats.packet_loss_percent, 100.0);
    assert_eq!(conn_stats.packet_loss_percent, 100.0);
}

#[test]
fn test_networking_sim_provider_wraps_supported_native_transports() {
    let udp = NetworkSimProvider::new(UdpNetProvider::new());
    let null = NetworkSimProvider::new(NullNetworkProvider::new());
    assert!(udp.simulation_config().is_none());
    assert!(null.simulation_config().is_none());

    #[cfg(feature = "net-tcp")]
    {
        let tcp = NetworkSimProvider::new(TcpNetProvider::new());
        assert!(tcp.simulation_config().is_none());
    }

    #[cfg(feature = "net-ws")]
    {
        let ws = NetworkSimProvider::new(WsNetProvider::new());
        assert!(ws.simulation_config().is_none());
    }
}

#[test]
fn test_networking_sim_provider_rejects_invalid_loss_range() {
    let inner = RecordingProvider::new();
    let mut simulated = NetworkSimProvider::new(inner);
    let err = simulated
        .set_simulation_config(NetworkSimulationConfig {
            one_way_latency_ms: 0,
            jitter_ms: 0,
            packet_loss_percent: 120.0,
        })
        .unwrap_err();

    assert!(matches!(
        err,
        GoudError::ProviderError {
            subsystem: "network",
            ..
        }
    ));
}

#[test]
fn test_networking_sim_provider_clear_resets_config_and_pending_packets() {
    let inner = RecordingProvider::new();
    let mut simulated = NetworkSimProvider::new(inner);
    simulated
        .set_simulation_config(NetworkSimulationConfig {
            one_way_latency_ms: 50,
            jitter_ms: 0,
            packet_loss_percent: 0.0,
        })
        .unwrap();
    simulated
        .send(ConnectionId(1), Channel(0), b"pending")
        .unwrap();

    simulated.clear_simulation_config().unwrap();
    assert!(simulated.simulation_config().is_none());
    assert!(simulated.pending.is_empty());
    assert_eq!(DisconnectReason::LocalClose, DisconnectReason::LocalClose);
}
