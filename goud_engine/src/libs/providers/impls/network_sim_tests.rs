use super::*;
use std::time::{Duration, Instant};

use crate::core::providers::impls::NullNetworkProvider;
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

fn host_config() -> HostConfig {
    HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    }
}

fn wait_until(
    timeout: Duration,
    poll_interval: Duration,
    failure_message: &str,
    mut condition: impl FnMut() -> bool,
) {
    let deadline = Instant::now() + timeout;
    loop {
        if condition() {
            return;
        }
        assert!(Instant::now() < deadline, "{failure_message}");
        std::thread::sleep(poll_interval);
    }
}

fn assert_never_receives(
    timeout: Duration,
    poll_interval: Duration,
    failure_message: &str,
    mut poll_for_receive: impl FnMut() -> bool,
) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        assert!(!poll_for_receive(), "{failure_message}");
        std::thread::sleep(poll_interval);
    }
    assert!(!poll_for_receive(), "{failure_message}");
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

    wait_until(
        Duration::from_secs(1),
        Duration::from_millis(5),
        "delayed outbound send should flush after the configured latency window",
        || {
            simulated.update(0.0).unwrap();
            !simulated.inner().sent.is_empty()
        },
    );

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

#[cfg(feature = "net-tcp")]
#[test]
fn test_networking_sim_provider_drops_real_tcp_packets() {
    let mut host = TcpNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().expect("host should bind to an address");

    let mut client = NetworkSimProvider::new(TcpNetProvider::new());
    client
        .set_simulation_config(NetworkSimulationConfig {
            one_way_latency_ms: 0,
            jitter_ms: 0,
            packet_loss_percent: 100.0,
        })
        .unwrap();
    let client_conn = client.connect(&host_addr.to_string()).unwrap();

    let mut host_connected = false;
    wait_until(
        Duration::from_secs(2),
        Duration::from_millis(5),
        "host should accept the TCP client connection",
        || {
            host.update(0.0).unwrap();
            client.update(0.0).unwrap();

            host_connected |= host
                .drain_events()
                .into_iter()
                .any(|event| matches!(event, NetworkEvent::Connected { .. }));
            client.drain_events();

            host_connected && client.connection_state(client_conn) == ConnectionState::Connected
        },
    );

    assert!(
        host_connected,
        "host should accept the TCP client connection"
    );
    assert_eq!(
        client.connection_state(client_conn),
        ConnectionState::Connected
    );

    client
        .send(client_conn, Channel(0), b"simulated tcp loss")
        .unwrap();

    let stats = client.stats();
    assert_eq!(stats.packets_lost, 1);
    assert_eq!(stats.packet_loss_percent, 100.0);

    assert_never_receives(
        Duration::from_millis(400),
        Duration::from_millis(10),
        "host should not receive packets that the simulator drops on TCP",
        || {
            host.update(0.0).unwrap();
            client.update(0.0).unwrap();

            host.drain_events()
                .into_iter()
                .any(|event| matches!(event, NetworkEvent::Received { .. }))
        },
    );

    host.shutdown();
    client.shutdown();
}

#[cfg(feature = "net-ws")]
#[test]
fn test_networking_sim_provider_drops_real_websocket_packets() {
    let mut host = WsNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().expect("host should bind to an address");

    let mut client = NetworkSimProvider::new(WsNetProvider::new());
    client
        .set_simulation_config(NetworkSimulationConfig {
            one_way_latency_ms: 0,
            jitter_ms: 0,
            packet_loss_percent: 100.0,
        })
        .unwrap();
    let client_conn = client.connect(&format!("ws://{}", host_addr)).unwrap();

    let mut host_connected = false;
    wait_until(
        Duration::from_secs(5),
        Duration::from_millis(50),
        "host should accept the WebSocket client connection",
        || {
            host.update(0.0).unwrap();
            client.update(0.0).unwrap();

            host_connected |= host
                .drain_events()
                .into_iter()
                .any(|event| matches!(event, NetworkEvent::Connected { .. }));
            client.drain_events();

            host_connected && client.connection_state(client_conn) == ConnectionState::Connected
        },
    );

    assert!(
        host_connected,
        "host should accept the WebSocket client connection"
    );
    assert_eq!(
        client.connection_state(client_conn),
        ConnectionState::Connected
    );

    client
        .send(client_conn, Channel(0), b"simulated websocket loss")
        .unwrap();

    let stats = client.stats();
    assert_eq!(stats.packets_lost, 1);
    assert_eq!(stats.packet_loss_percent, 100.0);

    assert_never_receives(
        Duration::from_millis(500),
        Duration::from_millis(25),
        "host should not receive packets that the simulator drops on WebSocket",
        || {
            host.update(0.0).unwrap();
            client.update(0.0).unwrap();

            host.drain_events()
                .into_iter()
                .any(|event| matches!(event, NetworkEvent::Received { .. }))
        },
    );

    host.shutdown();
    client.shutdown();
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
    assert!(simulated.inner().sent.is_empty());
}
