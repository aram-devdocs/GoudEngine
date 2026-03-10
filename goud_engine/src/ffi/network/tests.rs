use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::*;
use crate::core::providers::impls::NullNetworkProvider;
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, HostConfig, NetworkCapabilities,
    NetworkEvent, NetworkSimulationConfig, NetworkStats,
};
use crate::core::providers::{Provider, ProviderLifecycle};
use crate::ffi::network::provider_factory::create_provider;
use crate::ffi::network::registry::{
    reset_registry_for_tests, with_instance, with_registry, NetInstance, NetRegistryInner,
};

#[path = "tests_live.rs"]
mod live_tests;
#[path = "tests_release.rs"]
mod tests_release;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

struct RegistryResetGuard {
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl RegistryResetGuard {
    fn new() -> Self {
        let guard = TEST_MUTEX.lock().expect("test mutex poisoned");
        reset_registry_for_tests();
        Self { _guard: guard }
    }
}

impl Drop for RegistryResetGuard {
    fn drop(&mut self) {
        reset_registry_for_tests();
    }
}

fn insert_provider(provider: Box<dyn NetworkProvider>) -> i64 {
    with_registry(|reg| {
        Ok(reg.insert(NetInstance {
            provider,
            recv_queue: std::collections::VecDeque::new(),
        }))
    })
    .expect("failed to insert provider")
}

fn insert_null_provider() -> i64 {
    insert_provider(Box::new(NullNetworkProvider::new()))
}

fn first_supported_protocol() -> Option<i32> {
    #[cfg(feature = "net-udp")]
    {
        return Some(super::provider_factory::PROTOCOL_UDP);
    }
    #[cfg(all(not(feature = "net-udp"), feature = "net-ws"))]
    {
        return Some(super::provider_factory::PROTOCOL_WS);
    }
    #[cfg(all(not(feature = "net-udp"), not(feature = "net-ws"), feature = "net-tcp"))]
    {
        return Some(super::provider_factory::PROTOCOL_TCP);
    }
    #[allow(unreachable_code)]
    None
}

#[derive(Debug, Clone)]
struct FixedStatsProvider {
    stats: NetworkStats,
    caps: NetworkCapabilities,
}

impl FixedStatsProvider {
    fn new(stats: NetworkStats) -> Self {
        Self {
            stats,
            caps: NetworkCapabilities {
                supports_hosting: false,
                max_connections: 1,
                max_channels: 1,
                max_message_size: 1024,
            },
        }
    }
}

impl Provider for FixedStatsProvider {
    fn name(&self) -> &str {
        "fixed-stats"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.caps.clone())
    }
}

impl ProviderLifecycle for FixedStatsProvider {
    fn init(&mut self) -> crate::core::error::GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> crate::core::error::GoudResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {}
}

impl NetworkProvider for FixedStatsProvider {
    fn host(&mut self, _config: &HostConfig) -> crate::core::error::GoudResult<()> {
        Ok(())
    }

    fn connect(&mut self, _addr: &str) -> crate::core::error::GoudResult<ConnectionId> {
        Ok(ConnectionId(1))
    }

    fn disconnect(&mut self, _conn: ConnectionId) -> crate::core::error::GoudResult<()> {
        Ok(())
    }

    fn disconnect_all(&mut self) -> crate::core::error::GoudResult<()> {
        Ok(())
    }

    fn send(
        &mut self,
        _conn: ConnectionId,
        _channel: Channel,
        _data: &[u8],
    ) -> crate::core::error::GoudResult<()> {
        Ok(())
    }

    fn broadcast(&mut self, _channel: Channel, _data: &[u8]) -> crate::core::error::GoudResult<()> {
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
        &self.caps
    }

    fn stats(&self) -> NetworkStats {
        self.stats.clone()
    }

    fn connection_stats(&self, _conn: ConnectionId) -> Option<ConnectionStats> {
        None
    }
}

fn assert_within_5_percent(actual: f32, expected: f32) {
    let delta = (actual - expected).abs();
    let tolerance = (expected.abs() * 0.05).max(0.001);
    assert!(
        delta <= tolerance,
        "expected {actual} within 5% of {expected} (delta={delta}, tolerance={tolerance})"
    );
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

#[test]
fn test_active_handle_prefers_override_then_default() {
    let context_id = GoudContextId::new(200, 1);
    let mut reg = NetRegistryInner::new();
    reg.instances.insert(
        10,
        NetInstance {
            provider: Box::new(NullNetworkProvider::new()),
            recv_queue: std::collections::VecDeque::new(),
        },
    );
    reg.instances.insert(
        11,
        NetInstance {
            provider: Box::new(NullNetworkProvider::new()),
            recv_queue: std::collections::VecDeque::new(),
        },
    );

    reg.set_default_handle_for_context(context_id, 10);
    assert_eq!(reg.active_handle_for_context(context_id), Some(10));

    reg.set_overlay_override_handle_for_context(context_id, Some(11));
    assert_eq!(reg.active_handle_for_context(context_id), Some(11));

    reg.set_overlay_override_handle_for_context(context_id, None);
    assert_eq!(reg.active_handle_for_context(context_id), Some(10));
}

#[test]
fn test_clear_associations_removes_default_and_override() {
    let context_id = GoudContextId::new(201, 1);
    let mut reg = NetRegistryInner::new();
    reg.default_handles_by_context
        .insert(NetRegistryInner::context_key(context_id), 55);
    reg.overlay_override_handles_by_context
        .insert(NetRegistryInner::context_key(context_id), 55);

    reg.clear_associations_for_handle(55);

    assert!(reg.default_handles_by_context.is_empty());
    assert!(reg.overlay_override_handles_by_context.is_empty());
}

#[test]
fn test_goud_network_get_stats_v2_writes_stats() {
    let _registry = RegistryResetGuard::new();
    let context_id = GoudContextId::new(300, 1);
    let handle = insert_null_provider();
    let mut ffi_stats = FfiNetworkStats {
        bytes_sent: 1,
        bytes_received: 1,
        packets_sent: 1,
        packets_received: 1,
        packets_lost: 1,
        rtt_ms: 1.0,
        send_bandwidth_bytes_per_sec: 1.0,
        receive_bandwidth_bytes_per_sec: 1.0,
        packet_loss_percent: 1.0,
        jitter_ms: 1.0,
    };

    // SAFETY: `ffi_stats` is a valid, writable out-parameter for one FfiNetworkStats value.
    let rc = unsafe { goud_network_get_stats_v2(context_id, handle, &mut ffi_stats) };

    assert_eq!(rc, 0);
    assert_eq!(ffi_stats, FfiNetworkStats::default());
}

#[test]
fn test_goud_network_connect_with_peer_rejects_null_out_params() {
    let _registry = RegistryResetGuard::new();
    let context_id = GoudContextId::new(307, 1);
    let address = b"127.0.0.1";
    let mut peer_id = 0u64;

    // SAFETY: `address` is valid input and this test intentionally passes a null out pointer.
    let rc = unsafe {
        goud_network_connect_with_peer(
            context_id,
            0,
            address.as_ptr(),
            address.len() as i32,
            12345,
            std::ptr::null_mut(),
            &mut peer_id,
        )
    };

    assert_eq!(rc, ERR_INVALID_STATE);
}

#[test]
fn test_overlay_handle_public_exports_set_and_clear_override() {
    let _registry = RegistryResetGuard::new();
    let context_id = GoudContextId::new(301, 1);
    let handle = insert_null_provider();

    assert_eq!(goud_network_set_overlay_handle(context_id, handle), 0);
    assert_eq!(network_overlay_handle_for_context(context_id), Some(handle));

    assert_eq!(goud_network_clear_overlay_handle(context_id), 0);
    assert_eq!(network_overlay_handle_for_context(context_id), None);

    assert_eq!(
        goud_network_set_overlay_handle(context_id, handle + 999),
        ERR_INVALID_STATE
    );
}

#[test]
fn test_overlay_snapshot_metrics_are_within_five_percent_of_provider_stats() {
    let _registry = RegistryResetGuard::new();
    let context_id = GoudContextId::new(302, 1);

    let expected_stats = NetworkStats {
        bytes_sent: 20_000,
        bytes_received: 10_000,
        packets_sent: 500,
        packets_received: 450,
        packets_lost: 12,
        rtt_ms: 68.0,
        send_bandwidth_bytes_per_sec: 4096.0,
        receive_bandwidth_bytes_per_sec: 2048.0,
        packet_loss_percent: 2.4,
        jitter_ms: 4.8,
    };

    let handle = insert_provider(Box::new(FixedStatsProvider::new(expected_stats.clone())));
    with_registry(|reg| {
        reg.set_default_handle_for_context(context_id, handle);
        Ok(())
    })
    .expect("failed to bind default handle");

    let snapshot =
        network_overlay_snapshot_for_context(context_id).expect("expected overlay snapshot");
    assert_eq!(snapshot.handle, handle);

    assert_within_5_percent(snapshot.metrics.rtt_ms, expected_stats.rtt_ms);
    assert_within_5_percent(
        snapshot.metrics.send_bandwidth_bytes_per_sec,
        expected_stats.send_bandwidth_bytes_per_sec,
    );
    assert_within_5_percent(
        snapshot.metrics.receive_bandwidth_bytes_per_sec,
        expected_stats.receive_bandwidth_bytes_per_sec,
    );
    assert_within_5_percent(
        snapshot.metrics.packet_loss_percent,
        expected_stats.packet_loss_percent,
    );
    assert_within_5_percent(snapshot.metrics.jitter_ms, expected_stats.jitter_ms);
}

#[cfg(any(feature = "net-udp", feature = "net-ws", feature = "net-tcp"))]
#[test]
fn test_simulation_ffi_exports_succeed_for_normal_provider_handles_in_debug_or_test() {
    let _registry = RegistryResetGuard::new();
    let context_id = GoudContextId::new(303, 1);
    let protocol = first_supported_protocol().expect("at least one transport feature expected");
    let provider = create_provider(protocol).expect("provider creation should succeed");
    let handle = insert_provider(provider);

    let config = NetworkSimulationConfig {
        one_way_latency_ms: 40,
        jitter_ms: 6,
        packet_loss_percent: 7.5,
    };

    assert_eq!(goud_network_set_simulation(context_id, handle, config), 0);

    let applied = with_instance(handle, |inst| Ok(inst.provider.simulation_config()))
        .expect("failed to access provider")
        .expect("simulation config should be set on wrapped provider");

    assert_eq!(applied.one_way_latency_ms, config.one_way_latency_ms);
    assert_eq!(applied.jitter_ms, config.jitter_ms);
    assert!((applied.packet_loss_percent - config.packet_loss_percent).abs() < 0.0001);

    assert_eq!(goud_network_clear_simulation(context_id, handle), 0);
    let cleared = with_instance(handle, |inst| Ok(inst.provider.simulation_config()))
        .expect("failed to access provider after clear");
    assert!(cleared.is_none(), "simulation config should be cleared");
}

#[cfg(any(feature = "net-udp", feature = "net-ws", feature = "net-tcp"))]
#[test]
fn test_simulation_ffi_exports_reject_invalid_config() {
    let _registry = RegistryResetGuard::new();
    let context_id = GoudContextId::new(304, 1);
    let protocol = first_supported_protocol().expect("at least one transport feature expected");
    let provider = create_provider(protocol).expect("provider creation should succeed");
    let handle = insert_provider(provider);

    let invalid = NetworkSimulationConfig {
        one_way_latency_ms: 1,
        jitter_ms: 1,
        packet_loss_percent: 101.0,
    };

    assert_eq!(
        goud_network_set_simulation(context_id, handle, invalid),
        ERR_INVALID_STATE
    );
}

#[cfg(any(feature = "net-udp", feature = "net-ws", feature = "net-tcp"))]
#[test]
fn test_simulation_ffi_exports_reject_unknown_handle() {
    let _registry = RegistryResetGuard::new();
    let context_id = GoudContextId::new(305, 1);
    let unknown_handle = 99_999;

    assert_eq!(
        goud_network_set_simulation(
            context_id,
            unknown_handle,
            NetworkSimulationConfig::default()
        ),
        ERR_INVALID_STATE
    );
    assert_eq!(
        goud_network_clear_simulation(context_id, unknown_handle),
        ERR_INVALID_STATE
    );
}
