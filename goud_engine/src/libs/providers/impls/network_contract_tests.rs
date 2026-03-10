use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionState, HostConfig, NetworkEvent, NetworkSimulationConfig,
};
use crate::core::providers::ProviderLifecycle;
#[cfg(any(debug_assertions, test))]
use crate::libs::providers::impls::NetworkSimProvider;
#[cfg(feature = "net-tcp")]
use crate::libs::providers::impls::TcpNetProvider;
use crate::libs::providers::impls::UdpNetProvider;

#[cfg(feature = "net-tcp")]
#[test]
fn test_tcp_provider_host_connect_send_receive_round_trip() {
    let mut host = TcpNetProvider::new();
    let config = HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    };
    host.host(&config).unwrap();
    let host_addr = host
        .local_addr()
        .expect("tcp host should expose local addr");

    let mut client = TcpNetProvider::new();
    let client_conn = client.connect(&host_addr.to_string()).unwrap();

    let mut host_connected = false;
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();

        if host
            .drain_events()
            .iter()
            .any(|event| matches!(event, NetworkEvent::Connected { .. }))
        {
            host_connected = true;
        }

        if host_connected && client.connection_state(client_conn) == ConnectionState::Connected {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert!(host_connected, "host must observe a connected event");
    assert_eq!(
        client.connection_state(client_conn),
        ConnectionState::Connected,
        "client must reach connected state"
    );

    let payload = b"tcp-frame";
    client.send(client_conn, Channel(0), payload).unwrap();

    let mut host_received = None;
    for _ in 0..100 {
        host.update(0.0).unwrap();
        for event in host.drain_events() {
            if let NetworkEvent::Received { channel, data, .. } = event {
                assert_eq!(channel, Channel(0));
                host_received = Some(data);
            }
        }
        if host_received.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert_eq!(host_received.as_deref(), Some(payload.as_slice()));
}

#[test]
fn test_network_sim_provider_applies_configured_latency_jitter_and_loss() {
    let inner = UdpNetProvider::new();
    let mut simulated = NetworkSimProvider::new(inner);

    let sim = NetworkSimulationConfig {
        one_way_latency_ms: 30,
        jitter_ms: 7,
        packet_loss_percent: 35.0,
    };

    simulated.set_simulation_config(sim).unwrap();
    let applied = simulated.simulation_config().unwrap();

    assert_eq!(applied.one_way_latency_ms, 30);
    assert_eq!(applied.jitter_ms, 7);
    assert!((applied.packet_loss_percent - 35.0).abs() < f32::EPSILON);
}

#[test]
fn test_network_stats_expose_overlay_metrics_fields() {
    let provider = UdpNetProvider::new();
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
}
