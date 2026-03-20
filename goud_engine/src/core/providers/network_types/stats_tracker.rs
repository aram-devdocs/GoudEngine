//! Rolling statistics accumulator shared by network providers.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::{ConnectionStats, NetworkStats};

// This helper is consumed only by feature-gated native transport backends.
#[derive(Debug, Clone, Copy)]
struct TimedValue {
    at: Instant,
    value: u64,
}

/// Rolling statistics accumulator shared by network providers.
#[derive(Debug, Clone)]
pub(crate) struct NetworkStatsTracker {
    total_bytes_sent: u64,
    total_bytes_received: u64,
    total_packets_sent: u64,
    total_packets_received: u64,
    total_packets_lost: u64,
    send_window: VecDeque<TimedValue>,
    recv_window: VecDeque<TimedValue>,
    sent_packet_window: VecDeque<TimedValue>,
    lost_packet_window: VecDeque<TimedValue>,
    rtt_samples_ms: VecDeque<f32>,
    latest_rtt_ms: f32,
}

impl Default for NetworkStatsTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkStatsTracker {
    const BANDWIDTH_WINDOW: Duration = Duration::from_secs(1);
    const RTT_SAMPLE_CAPACITY: usize = 32;

    pub(crate) fn new() -> Self {
        Self {
            total_bytes_sent: 0,
            total_bytes_received: 0,
            total_packets_sent: 0,
            total_packets_received: 0,
            total_packets_lost: 0,
            send_window: VecDeque::new(),
            recv_window: VecDeque::new(),
            sent_packet_window: VecDeque::new(),
            lost_packet_window: VecDeque::new(),
            rtt_samples_ms: VecDeque::with_capacity(Self::RTT_SAMPLE_CAPACITY),
            latest_rtt_ms: 0.0,
        }
    }

    pub(crate) fn record_sent_packet(&mut self, bytes: usize) {
        let now = Instant::now();
        self.total_bytes_sent += bytes as u64;
        self.total_packets_sent += 1;
        self.send_window.push_back(TimedValue {
            at: now,
            value: bytes as u64,
        });
        self.sent_packet_window
            .push_back(TimedValue { at: now, value: 1 });
        self.prune(now);
    }

    pub(crate) fn record_received_packet(&mut self, bytes: usize) {
        let now = Instant::now();
        self.total_bytes_received += bytes as u64;
        self.total_packets_received += 1;
        self.recv_window.push_back(TimedValue {
            at: now,
            value: bytes as u64,
        });
        self.prune(now);
    }

    pub(crate) fn record_packets_lost(&mut self, count: u64) {
        if count == 0 {
            return;
        }
        let now = Instant::now();
        self.total_packets_lost += count;
        self.lost_packet_window.push_back(TimedValue {
            at: now,
            value: count,
        });
        self.prune(now);
    }

    pub(crate) fn record_rtt_sample(&mut self, rtt_ms: f32) {
        self.latest_rtt_ms = rtt_ms.max(0.0);
        if self.rtt_samples_ms.len() >= Self::RTT_SAMPLE_CAPACITY {
            self.rtt_samples_ms.pop_front();
        }
        self.rtt_samples_ms.push_back(self.latest_rtt_ms);
    }

    pub(crate) fn snapshot_network(&self) -> NetworkStats {
        let now = Instant::now();
        NetworkStats {
            bytes_sent: self.total_bytes_sent,
            bytes_received: self.total_bytes_received,
            packets_sent: self.total_packets_sent,
            packets_received: self.total_packets_received,
            packets_lost: self.total_packets_lost,
            rtt_ms: self.latest_rtt_ms,
            send_bandwidth_bytes_per_sec: self.window_sum(&self.send_window, now) as f32,
            receive_bandwidth_bytes_per_sec: self.window_sum(&self.recv_window, now) as f32,
            packet_loss_percent: self.packet_loss_percent(now),
            jitter_ms: self.jitter_ms(),
        }
    }

    pub(crate) fn snapshot_connection(&self) -> ConnectionStats {
        let now = Instant::now();
        ConnectionStats {
            rtt_ms: self.latest_rtt_ms,
            bytes_sent: self.total_bytes_sent,
            bytes_received: self.total_bytes_received,
            packets_sent: self.total_packets_sent,
            packets_received: self.total_packets_received,
            packets_lost: self.total_packets_lost,
            send_bandwidth_bytes_per_sec: self.window_sum(&self.send_window, now) as f32,
            receive_bandwidth_bytes_per_sec: self.window_sum(&self.recv_window, now) as f32,
            packet_loss_percent: self.packet_loss_percent(now),
            jitter_ms: self.jitter_ms(),
        }
    }

    fn prune(&mut self, now: Instant) {
        Self::prune_window(&mut self.send_window, now);
        Self::prune_window(&mut self.recv_window, now);
        Self::prune_window(&mut self.sent_packet_window, now);
        Self::prune_window(&mut self.lost_packet_window, now);
    }

    fn prune_window(window: &mut VecDeque<TimedValue>, now: Instant) {
        while let Some(front) = window.front() {
            if now.duration_since(front.at) <= Self::BANDWIDTH_WINDOW {
                break;
            }
            window.pop_front();
        }
    }

    fn window_sum(&self, window: &VecDeque<TimedValue>, now: Instant) -> u64 {
        window
            .iter()
            .filter(|sample| now.duration_since(sample.at) <= Self::BANDWIDTH_WINDOW)
            .map(|sample| sample.value)
            .sum()
    }

    fn packet_loss_percent(&self, now: Instant) -> f32 {
        let sent = self.window_sum(&self.sent_packet_window, now);
        let lost = self.window_sum(&self.lost_packet_window, now);
        if sent + lost == 0 {
            return 0.0;
        }
        (lost as f32 / (sent + lost) as f32) * 100.0
    }

    fn jitter_ms(&self) -> f32 {
        if self.rtt_samples_ms.len() < 2 {
            return 0.0;
        }

        let mean = self.rtt_samples_ms.iter().sum::<f32>() / self.rtt_samples_ms.len() as f32;
        let variance = self
            .rtt_samples_ms
            .iter()
            .map(|sample| {
                let delta = sample - mean;
                delta * delta
            })
            .sum::<f32>()
            / self.rtt_samples_ms.len() as f32;

        variance.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_stats_tracker_computes_bandwidth_and_loss_windows() {
        let mut tracker = NetworkStatsTracker::new();
        tracker.record_sent_packet(128);
        tracker.record_sent_packet(64);
        tracker.record_received_packet(256);
        tracker.record_packets_lost(1);

        let snapshot = tracker.snapshot_network();
        assert_eq!(snapshot.bytes_sent, 192);
        assert_eq!(snapshot.bytes_received, 256);
        assert_eq!(snapshot.packets_sent, 2);
        assert_eq!(snapshot.packets_received, 1);
        assert_eq!(snapshot.packets_lost, 1);
        assert_eq!(snapshot.send_bandwidth_bytes_per_sec, 192.0);
        assert_eq!(snapshot.receive_bandwidth_bytes_per_sec, 256.0);
        assert!((snapshot.packet_loss_percent - (100.0 / 3.0)).abs() < 0.001);
    }

    #[test]
    fn test_network_stats_tracker_computes_jitter_from_rtt_samples() {
        let mut tracker = NetworkStatsTracker::new();
        tracker.record_rtt_sample(10.0);
        tracker.record_rtt_sample(20.0);
        tracker.record_rtt_sample(30.0);

        let snapshot = tracker.snapshot_connection();
        assert_eq!(snapshot.rtt_ms, 30.0);
        assert!((snapshot.jitter_ms - 8.164966).abs() < 0.001);
    }
}
