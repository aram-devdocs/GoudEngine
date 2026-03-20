//! Reconnection logic for the TCP transport provider.

use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use std::net::TcpStream;

use crate::core::providers::network_types::{ConnectionId, ConnectionState, DisconnectReason};
use crate::libs::error::GoudResult;

use super::tcp_network_io::{configure_stream, spawn_io_thread};
use super::{InternalTcpEvent, TcpNetProvider};

pub(super) const DEFAULT_RECONNECT_DELAY: Duration = Duration::from_secs(1);
pub(super) const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Configuration for automatic reconnection behavior.
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Whether reconnection is enabled for this connection.
    pub enabled: bool,
    /// Maximum number of reconnection attempts before giving up.
    pub max_attempts: u32,
    /// Delay between reconnection attempts.
    pub delay: Duration,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_attempts: MAX_RECONNECT_ATTEMPTS,
            delay: DEFAULT_RECONNECT_DELAY,
        }
    }
}

impl TcpNetProvider {
    /// Enable automatic reconnection for a connection.
    ///
    /// When reconnect is enabled and the connection drops unexpectedly
    /// (remote close, timeout, or error), the provider will automatically
    /// attempt to re-establish the connection up to `config.max_attempts`
    /// times with `config.delay` between attempts. Only applicable to
    /// client-initiated connections that have a stored remote address.
    pub fn set_reconnect(&mut self, conn: ConnectionId, config: ReconnectConfig) -> GoudResult<()> {
        let connection = self
            .connections
            .get_mut(&conn.0)
            .ok_or_else(|| super::net_err(format!("Unknown connection {:?}", conn)))?;
        if connection.remote_addr.is_none() {
            return Err(super::net_err(
                "Reconnect is only supported for client-initiated connections",
            ));
        }
        connection.reconnect = config;
        connection.reconnect_attempts = 0;
        connection.last_reconnect_at = None;
        Ok(())
    }

    /// Attempt reconnection for connections that have been disconnected
    /// and have reconnect enabled. Called from `update()`.
    pub(super) fn process_reconnects(&mut self) {
        let now = Instant::now();
        let mut to_reconnect: Vec<(ConnectionId, String, u64)> = Vec::new();

        for conn in self.connections.values_mut() {
            if !conn.reconnect.enabled {
                continue;
            }
            if conn.state != ConnectionState::Disconnected && conn.state != ConnectionState::Error {
                continue;
            }
            if conn.reconnect_attempts >= conn.reconnect.max_attempts {
                continue;
            }
            let Some(ref addr) = conn.remote_addr else {
                continue;
            };
            if let Some(last) = conn.last_reconnect_at {
                if now.duration_since(last) < conn.reconnect.delay {
                    continue;
                }
            }
            conn.reconnect_attempts += 1;
            conn.last_reconnect_at = Some(now);
            conn.state = ConnectionState::Connecting;
            // Bump generation so stale events from the old IO thread are ignored.
            let gen = self.next_generation.fetch_add(1, Ordering::Relaxed);
            conn.generation = gen;
            to_reconnect.push((conn.id, addr.clone(), gen));
        }

        for (id, address, gen) in to_reconnect {
            let event_tx = self.event_tx.clone();
            let running = self.running.clone();

            let handle = thread::spawn(move || match TcpStream::connect(&address) {
                Ok(stream) => {
                    configure_stream(&stream);
                    let write_tx = spawn_io_thread(id, gen, stream, event_tx.clone(), running);
                    let _ = event_tx.send(InternalTcpEvent::WriteTxReady(id, write_tx, gen));
                    let _ = event_tx.send(InternalTcpEvent::Connected(id, gen));
                }
                Err(e) => {
                    let _ = event_tx.send(InternalTcpEvent::Error(id, format!("reconnect: {e}")));
                    let _ = event_tx.send(InternalTcpEvent::Disconnected(
                        id,
                        DisconnectReason::Error(e.to_string()),
                        gen,
                    ));
                }
            });

            if let Ok(mut guard) = self.threads.lock() {
                guard.push(handle);
            }
        }
    }
}
