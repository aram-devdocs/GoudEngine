//! Debug-only network simulation wrapper for native transports.
//!
//! The wrapper injects one-way outbound latency, jitter, and packet loss
//! without adding simulation branches to release builds.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, HostConfig, NetworkCapabilities,
    NetworkEvent, NetworkSimulationConfig, NetworkStats,
};
use crate::core::providers::{Provider, ProviderLifecycle};

#[derive(Debug, Clone, Copy)]
struct TimedCount {
    at: Instant,
    value: u64,
}

#[derive(Debug, Default, Clone)]
struct SimMetrics {
    total_packets_lost: u64,
    attempted_window: VecDeque<TimedCount>,
    lost_window: VecDeque<TimedCount>,
    delay_samples_ms: VecDeque<f32>,
}

impl SimMetrics {
    const WINDOW: Duration = Duration::from_secs(1);
    const SAMPLE_CAPACITY: usize = 32;

    fn record_attempt(&mut self) {
        let now = Instant::now();
        self.attempted_window
            .push_back(TimedCount { at: now, value: 1 });
        self.prune(now);
    }

    fn record_loss(&mut self) {
        let now = Instant::now();
        self.total_packets_lost += 1;
        self.lost_window.push_back(TimedCount { at: now, value: 1 });
        self.prune(now);
    }

    fn record_delay_sample(&mut self, delay_ms: f32) {
        if self.delay_samples_ms.len() >= Self::SAMPLE_CAPACITY {
            self.delay_samples_ms.pop_front();
        }
        self.delay_samples_ms.push_back(delay_ms.max(0.0));
    }

    fn overlay_network_stats(&self, stats: &mut NetworkStats) {
        stats.packets_lost += self.total_packets_lost;

        let now = Instant::now();
        let packet_loss_percent = self.packet_loss_percent(now);
        if packet_loss_percent > 0.0 {
            stats.packet_loss_percent = packet_loss_percent;
        }

        let jitter_ms = self.jitter_ms();
        if jitter_ms > 0.0 {
            stats.jitter_ms = jitter_ms;
        }
    }

    fn overlay_connection_stats(&self, stats: &mut ConnectionStats) {
        stats.packets_lost += self.total_packets_lost;

        let now = Instant::now();
        let packet_loss_percent = self.packet_loss_percent(now);
        if packet_loss_percent > 0.0 {
            stats.packet_loss_percent = packet_loss_percent;
        }

        let jitter_ms = self.jitter_ms();
        if jitter_ms > 0.0 {
            stats.jitter_ms = jitter_ms;
        }
    }

    fn packet_loss_percent(&self, now: Instant) -> f32 {
        let attempted = Self::window_sum(&self.attempted_window, now);
        let lost = Self::window_sum(&self.lost_window, now);
        if attempted == 0 {
            return 0.0;
        }
        (lost as f32 / attempted as f32) * 100.0
    }

    fn jitter_ms(&self) -> f32 {
        if self.delay_samples_ms.len() < 2 {
            return 0.0;
        }

        let mean = self.delay_samples_ms.iter().sum::<f32>() / self.delay_samples_ms.len() as f32;
        let variance = self
            .delay_samples_ms
            .iter()
            .map(|sample| {
                let delta = sample - mean;
                delta * delta
            })
            .sum::<f32>()
            / self.delay_samples_ms.len() as f32;

        variance.sqrt()
    }

    fn prune(&mut self, now: Instant) {
        while let Some(front) = self.attempted_window.front() {
            if now.duration_since(front.at) <= Self::WINDOW {
                break;
            }
            self.attempted_window.pop_front();
        }
        while let Some(front) = self.lost_window.front() {
            if now.duration_since(front.at) <= Self::WINDOW {
                break;
            }
            self.lost_window.pop_front();
        }
    }

    fn window_sum(window: &VecDeque<TimedCount>, now: Instant) -> u64 {
        window
            .iter()
            .filter(|sample| now.duration_since(sample.at) <= Self::WINDOW)
            .map(|sample| sample.value)
            .sum()
    }
}

#[derive(Debug)]
struct PendingSend {
    due_at: Instant,
    conn: ConnectionId,
    channel: Channel,
    data: Vec<u8>,
    delay_ms: f32,
}

/// Debug-only wrapper that injects latency, jitter, and packet loss.
pub struct NetworkSimProvider<P> {
    inner: P,
    config: Option<NetworkSimulationConfig>,
    rng_state: u64,
    pending: Vec<PendingSend>,
    metrics: SimMetrics,
    per_connection_metrics: HashMap<u64, SimMetrics>,
}

impl<P> NetworkSimProvider<P> {
    /// Create a new simulator wrapper around an existing provider.
    pub fn new(inner: P) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            inner,
            config: None,
            rng_state: seed | 1,
            pending: Vec::new(),
            metrics: SimMetrics::default(),
            per_connection_metrics: HashMap::new(),
        }
    }

    /// Return a shared reference to the wrapped provider.
    pub fn inner(&self) -> &P {
        &self.inner
    }

    /// Return a mutable reference to the wrapped provider.
    pub fn inner_mut(&mut self) -> &mut P {
        &mut self.inner
    }

    /// Consume the wrapper and return the wrapped provider.
    pub fn into_inner(self) -> P {
        self.inner
    }
}

impl<P: NetworkProvider> NetworkSimProvider<P> {
    fn metric_for_connection(&mut self, conn: ConnectionId) -> &mut SimMetrics {
        self.per_connection_metrics.entry(conn.0).or_default()
    }

    fn next_u64(&mut self) -> u64 {
        let mut state = self.rng_state;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        self.rng_state = state;
        state
    }

    fn should_drop(&mut self, config: NetworkSimulationConfig) -> bool {
        if config.packet_loss_percent <= 0.0 {
            return false;
        }
        let sample = (self.next_u64() % 10_000) as f32 / 100.0;
        sample < config.packet_loss_percent
    }

    fn compute_delay_ms(&mut self, config: NetworkSimulationConfig) -> f32 {
        let base = config.one_way_latency_ms as i64;
        let jitter = config.jitter_ms as i64;
        if jitter == 0 {
            return base.max(0) as f32;
        }

        let spread = (self.next_u64() % ((jitter * 2 + 1) as u64)) as i64 - jitter;
        (base + spread).max(0) as f32
    }

    fn enqueue_or_send(
        &mut self,
        conn: ConnectionId,
        channel: Channel,
        data: &[u8],
    ) -> GoudResult<()> {
        let config = match self.config {
            Some(config) if config.is_enabled() => config,
            _ => return self.inner.send(conn, channel, data),
        };

        self.metrics.record_attempt();
        self.metric_for_connection(conn).record_attempt();

        if self.should_drop(config) {
            self.metrics.record_loss();
            self.metric_for_connection(conn).record_loss();
            return Ok(());
        }

        let delay_ms = self.compute_delay_ms(config);
        if delay_ms <= 0.0 {
            self.metrics.record_delay_sample(0.0);
            self.metric_for_connection(conn).record_delay_sample(0.0);
            return self.inner.send(conn, channel, data);
        }

        self.pending.push(PendingSend {
            due_at: Instant::now() + Duration::from_secs_f32(delay_ms / 1000.0),
            conn,
            channel,
            data: data.to_vec(),
            delay_ms,
        });
        Ok(())
    }

    fn flush_due_sends(&mut self) -> GoudResult<()> {
        let now = Instant::now();
        let mut remaining = Vec::with_capacity(self.pending.len());
        let mut due = Vec::new();

        for pending in self.pending.drain(..) {
            if pending.due_at <= now {
                due.push(pending);
            } else {
                remaining.push(pending);
            }
        }
        self.pending = remaining;

        for pending in due {
            self.metrics.record_delay_sample(pending.delay_ms);
            self.metric_for_connection(pending.conn)
                .record_delay_sample(pending.delay_ms);
            self.inner.send(pending.conn, pending.channel, &pending.data)?;
        }

        Ok(())
    }
}

impl<P: Provider> Provider for NetworkSimProvider<P> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn version(&self) -> &str {
        self.inner.version()
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        self.inner.capabilities()
    }
}

impl<P: ProviderLifecycle + NetworkProvider> ProviderLifecycle for NetworkSimProvider<P> {
    fn init(&mut self) -> GoudResult<()> {
        self.inner.init()
    }

    fn update(&mut self, delta: f32) -> GoudResult<()> {
        self.flush_due_sends()?;
        self.inner.update(delta)
    }

    fn shutdown(&mut self) {
        self.pending.clear();
        self.inner.shutdown();
    }
}

impl<P: NetworkProvider> NetworkProvider for NetworkSimProvider<P> {
    fn host(&mut self, config: &HostConfig) -> GoudResult<()> {
        self.inner.host(config)
    }

    fn connect(&mut self, addr: &str) -> GoudResult<ConnectionId> {
        self.inner.connect(addr)
    }

    fn disconnect(&mut self, conn: ConnectionId) -> GoudResult<()> {
        self.per_connection_metrics.remove(&conn.0);
        self.inner.disconnect(conn)
    }

    fn disconnect_all(&mut self) -> GoudResult<()> {
        self.pending.clear();
        self.per_connection_metrics.clear();
        self.inner.disconnect_all()
    }

    fn send(&mut self, conn: ConnectionId, channel: Channel, data: &[u8]) -> GoudResult<()> {
        self.enqueue_or_send(conn, channel, data)
    }

    fn broadcast(&mut self, channel: Channel, data: &[u8]) -> GoudResult<()> {
        let ids = self.inner.connections();
        for conn in ids {
            self.enqueue_or_send(conn, channel, data)?;
        }
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<NetworkEvent> {
        self.inner.drain_events()
    }

    fn connections(&self) -> Vec<ConnectionId> {
        self.inner.connections()
    }

    fn connection_state(&self, conn: ConnectionId) -> ConnectionState {
        self.inner.connection_state(conn)
    }

    fn local_id(&self) -> Option<ConnectionId> {
        self.inner.local_id()
    }

    fn network_capabilities(&self) -> &NetworkCapabilities {
        self.inner.network_capabilities()
    }

    fn stats(&self) -> NetworkStats {
        let mut stats = self.inner.stats();
        self.metrics.overlay_network_stats(&mut stats);
        stats
    }

    fn connection_stats(&self, conn: ConnectionId) -> Option<ConnectionStats> {
        let mut stats = self.inner.connection_stats(conn)?;
        if let Some(metrics) = self.per_connection_metrics.get(&conn.0) {
            metrics.overlay_connection_stats(&mut stats);
        }
        Some(stats)
    }

    fn set_simulation_config(&mut self, config: NetworkSimulationConfig) -> GoudResult<()> {
        config
            .validate()
            .map_err(|message| GoudError::ProviderError {
                subsystem: "network",
                message,
            })?;
        self.config = config.is_enabled().then_some(config);
        Ok(())
    }

    fn clear_simulation_config(&mut self) -> GoudResult<()> {
        self.config = None;
        self.pending.clear();
        Ok(())
    }

    fn simulation_config(&self) -> Option<NetworkSimulationConfig> {
        self.config
    }
}

impl<P: std::fmt::Debug> std::fmt::Debug for NetworkSimProvider<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkSimProvider")
            .field("inner", &self.inner)
            .field("config", &self.config)
            .field("pending", &self.pending.len())
            .finish()
    }
}

#[cfg(test)]
#[path = "network_sim_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "network_contract_tests.rs"]
mod contract_tests;
