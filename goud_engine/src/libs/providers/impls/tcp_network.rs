//! TCP transport provider implementing `NetworkProvider`.
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[path = "tcp_network_io.rs"]
mod tcp_network_io;

use self::tcp_network_io::{configure_stream, spawn_io_thread};
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, DisconnectReason, HostConfig,
    NetworkCapabilities, NetworkEvent, NetworkStats, NetworkStatsTracker,
};
use crate::core::providers::{Provider, ProviderLifecycle};
use crate::libs::error::{GoudError, GoudResult};

const MAX_MESSAGE_SIZE: usize = 16_777_215;

fn net_err(msg: impl Into<String>) -> GoudError {
    GoudError::ProviderError {
        subsystem: "network",
        message: msg.into(),
    }
}

enum InternalTcpEvent {
    Connected(ConnectionId),
    Disconnected(ConnectionId, DisconnectReason),
    Received(ConnectionId, Channel, Vec<u8>),
    Error(ConnectionId, String),
    WriteTxReady(ConnectionId, mpsc::Sender<Vec<u8>>),
}

struct TcpConnection {
    id: ConnectionId,
    state: ConnectionState,
    stats: NetworkStatsTracker,
}

/// TCP transport provider using length-prefixed framed messages.
pub struct TcpNetProvider {
    capabilities: NetworkCapabilities,
    connections: HashMap<u64, TcpConnection>,
    event_tx: mpsc::Sender<InternalTcpEvent>,
    event_rx: Mutex<Option<mpsc::Receiver<InternalTcpEvent>>>,
    events: Vec<NetworkEvent>,
    send_txs: HashMap<u64, mpsc::Sender<Vec<u8>>>,
    stats: NetworkStatsTracker,
    next_id: Arc<AtomicU64>,
    conn_count: Arc<AtomicU32>,
    threads: Mutex<Vec<JoinHandle<()>>>,
    running: Arc<AtomicBool>,
    is_hosting: bool,
    local_addr: Option<SocketAddr>,
}

// SAFETY: TcpNetProvider is only mutated through &mut self on the main thread.
// Non-Sync fields are not shared; Mutex-wrapped fields provide internal sync.
unsafe impl Sync for TcpNetProvider {}

impl TcpNetProvider {
    /// Creates a new TCP transport provider.
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        Self {
            capabilities: NetworkCapabilities {
                supports_hosting: true,
                max_connections: 64,
                max_channels: u8::MAX,
                max_message_size: MAX_MESSAGE_SIZE as u32,
            },
            connections: HashMap::new(),
            event_tx,
            event_rx: Mutex::new(Some(event_rx)),
            events: Vec::new(),
            send_txs: HashMap::new(),
            stats: NetworkStatsTracker::new(),
            next_id: Arc::new(AtomicU64::new(1)),
            conn_count: Arc::new(AtomicU32::new(0)),
            threads: Mutex::new(Vec::new()),
            running: Arc::new(AtomicBool::new(true)),
            is_hosting: false,
            local_addr: None,
        }
    }

    /// Returns the bound local listener address, if hosting has started.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    fn allocate_id(&self) -> ConnectionId {
        ConnectionId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    fn encode_frame(channel: Channel, data: &[u8]) -> GoudResult<Vec<u8>> {
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(net_err(format!("message too large: {} bytes", data.len())));
        }

        let frame_len = (data.len() + 1) as u32;
        let mut frame = Vec::with_capacity(4 + frame_len as usize);
        frame.extend_from_slice(&frame_len.to_be_bytes());
        frame.push(channel.0);
        frame.extend_from_slice(data);
        Ok(frame)
    }

    fn decrement_conn_count_if_needed(&self, removed: bool) {
        if removed && self.conn_count.load(Ordering::Relaxed) > 0 {
            self.conn_count.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

impl Default for TcpNetProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for TcpNetProvider {
    fn name(&self) -> &str {
        "tcp"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for TcpNetProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        let guard = self.event_rx.lock().map_err(|e| net_err(e.to_string()))?;
        if let Some(ref rx) = *guard {
            while let Ok(evt) = rx.try_recv() {
                match evt {
                    InternalTcpEvent::Connected(id) => {
                        if let Some(conn) = self.connections.get_mut(&id.0) {
                            conn.state = ConnectionState::Connected;
                        }
                        self.events.push(NetworkEvent::Connected { conn: id });
                    }
                    InternalTcpEvent::Disconnected(id, reason) => {
                        let removed =
                            self.connections.remove(&id.0).is_some() | self.send_txs.remove(&id.0).is_some();
                        self.decrement_conn_count_if_needed(removed);
                        self.events.push(NetworkEvent::Disconnected { conn: id, reason });
                    }
                    InternalTcpEvent::Received(id, channel, data) => {
                        self.stats.record_received_packet(data.len());
                        if let Some(conn) = self.connections.get_mut(&id.0) {
                            conn.stats.record_received_packet(data.len());
                        }
                        self.events.push(NetworkEvent::Received { conn: id, channel, data });
                    }
                    InternalTcpEvent::Error(id, message) => {
                        if let Some(conn) = self.connections.get_mut(&id.0) {
                            conn.state = ConnectionState::Error;
                        }
                        self.events.push(NetworkEvent::Error { conn: id, message });
                    }
                    InternalTcpEvent::WriteTxReady(id, write_tx) => {
                        self.send_txs.insert(id.0, write_tx);
                        self.connections.entry(id.0).or_insert_with(|| TcpConnection {
                            id,
                            state: ConnectionState::Connecting,
                            stats: NetworkStatsTracker::new(),
                        });
                    }
                }
            }
        }
        drop(guard);
        Ok(())
    }

    fn shutdown(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.send_txs.clear();
        if let Ok(mut guard) = self.event_rx.lock() {
            guard.take();
        }
        if let Ok(mut guard) = self.threads.lock() {
            for handle in std::mem::take(&mut *guard) {
                let _ = handle.join();
            }
        }
        self.connections.clear();
    }
}

impl NetworkProvider for TcpNetProvider {
    fn host(&mut self, config: &HostConfig) -> GoudResult<()> {
        if self.is_hosting {
            return Err(net_err("Already hosting"));
        }

        let bind = format!("{}:{}", config.bind_address, config.port);
        let listener = TcpListener::bind(&bind).map_err(|e| net_err(format!("bind {bind}: {e}")))?;
        self.local_addr = Some(listener.local_addr().map_err(|e| net_err(e.to_string()))?);
        listener.set_nonblocking(true).map_err(|e| net_err(e.to_string()))?;
        self.is_hosting = true;

        let running = self.running.clone();
        let event_tx = self.event_tx.clone();
        let next_id = self.next_id.clone();
        let max_conns = config.max_connections;
        let conn_count = self.conn_count.clone();

        let handle = thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if conn_count.load(Ordering::Relaxed) >= max_conns {
                            continue;
                        }
                        configure_stream(&stream);
                        let id = ConnectionId(next_id.fetch_add(1, Ordering::Relaxed));
                        conn_count.fetch_add(1, Ordering::Relaxed);
                        let write_tx = spawn_io_thread(id, stream, event_tx.clone(), running.clone());
                        let _ = event_tx.send(InternalTcpEvent::WriteTxReady(id, write_tx));
                        let _ = event_tx.send(InternalTcpEvent::Connected(id));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => thread::sleep(Duration::from_millis(5)),
                }
            }
        });

        if let Ok(mut guard) = self.threads.lock() {
            guard.push(handle);
        }
        Ok(())
    }

    fn connect(&mut self, addr: &str) -> GoudResult<ConnectionId> {
        let id = self.allocate_id();
        self.connections.insert(
            id.0,
            TcpConnection {
                id,
                state: ConnectionState::Connecting,
                stats: NetworkStatsTracker::new(),
            },
        );

        let address = addr.to_string();
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();

        let handle = thread::spawn(move || match TcpStream::connect(&address) {
            Ok(stream) => {
                configure_stream(&stream);
                let write_tx = spawn_io_thread(id, stream, event_tx.clone(), running);
                let _ = event_tx.send(InternalTcpEvent::WriteTxReady(id, write_tx));
                let _ = event_tx.send(InternalTcpEvent::Connected(id));
            }
            Err(e) => {
                let _ = event_tx.send(InternalTcpEvent::Error(id, format!("connect: {e}")));
                let _ = event_tx.send(InternalTcpEvent::Disconnected(
                    id,
                    DisconnectReason::Error(e.to_string()),
                ));
            }
        });

        if let Ok(mut guard) = self.threads.lock() {
            guard.push(handle);
        }
        Ok(id)
    }

    fn disconnect(&mut self, conn: ConnectionId) -> GoudResult<()> {
        let removed = self.connections.remove(&conn.0).is_some() | self.send_txs.remove(&conn.0).is_some();
        self.decrement_conn_count_if_needed(removed);
        self.events.push(NetworkEvent::Disconnected {
            conn,
            reason: DisconnectReason::LocalClose,
        });
        Ok(())
    }

    fn disconnect_all(&mut self) -> GoudResult<()> {
        let ids: Vec<_> = self.connections.keys().map(|id| ConnectionId(*id)).collect();
        for id in ids {
            let _ = self.disconnect(id);
        }
        Ok(())
    }

    fn send(&mut self, conn: ConnectionId, channel: Channel, data: &[u8]) -> GoudResult<()> {
        let conn_state = self
            .connections
            .get(&conn.0)
            .ok_or_else(|| net_err(format!("Unknown connection {:?}", conn)))?;
        if conn_state.state != ConnectionState::Connected {
            return Err(net_err("Connection not established"));
        }

        let frame = Self::encode_frame(channel, data)?;
        let tx = self
            .send_txs
            .get(&conn.0)
            .ok_or_else(|| net_err("No write channel"))?;
        tx.send(frame).map_err(|e| net_err(format!("enqueue: {e}")))?;
        self.stats.record_sent_packet(data.len());
        if let Some(connection) = self.connections.get_mut(&conn.0) {
            connection.stats.record_sent_packet(data.len());
        }
        Ok(())
    }

    fn broadcast(&mut self, channel: Channel, data: &[u8]) -> GoudResult<()> {
        let ids: Vec<_> = self
            .connections
            .values()
            .filter(|conn| conn.state == ConnectionState::Connected)
            .map(|conn| conn.id)
            .collect();
        for id in ids {
            let _ = self.send(id, channel, data);
        }
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<NetworkEvent> {
        std::mem::take(&mut self.events)
    }

    fn connections(&self) -> Vec<ConnectionId> {
        self.connections.keys().map(|id| ConnectionId(*id)).collect()
    }

    fn connection_state(&self, conn: ConnectionId) -> ConnectionState {
        self.connections
            .get(&conn.0)
            .map(|connection| connection.state)
            .unwrap_or(ConnectionState::Disconnected)
    }

    fn local_id(&self) -> Option<ConnectionId> {
        None
    }

    fn network_capabilities(&self) -> &NetworkCapabilities {
        &self.capabilities
    }

    fn stats(&self) -> NetworkStats {
        self.stats.snapshot_network()
    }

    fn connection_stats(&self, conn: ConnectionId) -> Option<ConnectionStats> {
        self.connections
            .get(&conn.0)
            .map(|connection| connection.stats.snapshot_connection())
    }
}

impl std::fmt::Debug for TcpNetProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TcpNetProvider")
            .field("is_hosting", &self.is_hosting)
            .field("connections", &self.connections.len())
            .finish()
    }
}

#[cfg(test)]
#[path = "tcp_network_tests.rs"]
mod tests;
