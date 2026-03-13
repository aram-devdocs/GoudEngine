use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::core::providers::diagnostics::NetworkDiagnosticsV1;
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionId, ConnectionState, ConnectionStats, DisconnectReason, HostConfig,
    NetworkCapabilities, NetworkEvent, NetworkStats, NetworkStatsTracker,
};
use crate::core::providers::{Provider, ProviderLifecycle};
use crate::libs::error::{GoudError, GoudResult};

use super::io::{set_stream_timeout, spawn_io_thread, InternalWsEvent, WsConnection, READ_TIMEOUT};
use super::tls::{connect_with_optional_custom_ca, load_server_tls_config};

fn net_err(msg: String) -> GoudError {
    GoudError::ProviderError {
        subsystem: "network",
        message: msg,
    }
}

/// WebSocket transport provider (native only).
pub struct WsNetProvider {
    capabilities: NetworkCapabilities,
    connections: HashMap<u64, WsConnection>,
    event_tx: mpsc::Sender<InternalWsEvent>,
    event_rx: Mutex<Option<mpsc::Receiver<InternalWsEvent>>>,
    events: Vec<NetworkEvent>,
    send_txs: HashMap<u64, mpsc::Sender<Vec<u8>>>,
    stats: NetworkStatsTracker,
    next_id: Arc<AtomicU64>,
    conn_count: Arc<AtomicU32>,
    threads: Mutex<Vec<JoinHandle<()>>>,
    running: Arc<AtomicBool>,
    is_hosting: bool,
    local_addr: Option<std::net::SocketAddr>,
}

// SAFETY: WsNetProvider is only accessed via &mut self from the main game
// thread through ProviderRegistry (which is held by GoudGame with no shared
// reference path). The non-Sync fields (send_txs HashMap, event_tx Sender)
// are only mutated via &mut self. Mutex-wrapped fields (event_rx, threads)
// are Sync. The Provider trait requires Sync, but no codepath creates &WsNetProvider
// shared across threads.
unsafe impl Sync for WsNetProvider {}

impl WsNetProvider {
    /// Create a new WebSocket network provider.
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        Self {
            capabilities: NetworkCapabilities {
                supports_hosting: true,
                max_connections: 64,
                max_channels: 1,
                max_message_size: 16_777_216_u32,
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

    fn allocate_id(&self) -> ConnectionId {
        ConnectionId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    /// Local address the provider is listening on (for tests with port 0).
    pub fn local_addr(&self) -> Option<std::net::SocketAddr> {
        self.local_addr
    }
}

impl Default for WsNetProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for WsNetProvider {
    fn name(&self) -> &str {
        "websocket"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for WsNetProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: f32) -> GoudResult<()> {
        let guard = self.event_rx.lock().map_err(|e| net_err(e.to_string()))?;
        if let Some(ref rx) = *guard {
            while let Ok(evt) = rx.try_recv() {
                match evt {
                    InternalWsEvent::Connected(id) => {
                        if let Some(c) = self.connections.get_mut(&id.0) {
                            c.state = ConnectionState::Connected;
                        }
                        self.events.push(NetworkEvent::Connected { conn: id });
                    }
                    InternalWsEvent::Disconnected(id, reason) => {
                        self.connections.remove(&id.0);
                        self.send_txs.remove(&id.0);
                        self.conn_count.fetch_sub(1, Ordering::Relaxed);
                        self.events
                            .push(NetworkEvent::Disconnected { conn: id, reason });
                    }
                    InternalWsEvent::Received(id, data) => {
                        self.stats.record_received_packet(data.len());
                        if let Some(c) = self.connections.get_mut(&id.0) {
                            c.stats.record_received_packet(data.len());
                        }
                        self.events.push(NetworkEvent::Received {
                            conn: id,
                            channel: Channel(0),
                            data,
                        });
                    }
                    InternalWsEvent::Error(id, message) => {
                        if let Some(c) = self.connections.get_mut(&id.0) {
                            c.state = ConnectionState::Error;
                        }
                        self.events.push(NetworkEvent::Error { conn: id, message });
                    }
                    InternalWsEvent::WriteTxReady(id, write_tx) => {
                        self.send_txs.insert(id.0, write_tx);
                        self.connections
                            .entry(id.0)
                            .or_insert_with(|| WsConnection {
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
        if let Ok(mut g) = self.event_rx.lock() {
            g.take();
        }
        if let Ok(mut g) = self.threads.lock() {
            for h in std::mem::take(&mut *g) {
                let _ = h.join();
            }
        }
        self.connections.clear();
    }
}

impl NetworkProvider for WsNetProvider {
    fn host(&mut self, config: &HostConfig) -> GoudResult<()> {
        if self.is_hosting {
            return Err(net_err("Already hosting".into()));
        }
        let tls_config = match (&config.tls_cert_path, &config.tls_key_path) {
            (Some(cert), Some(key)) => Some(load_server_tls_config(cert, key)?),
            (None, None) => None,
            (Some(_), None) | (None, Some(_)) => {
                return Err(net_err(
                    "WebSocket TLS requires both tls_cert_path and tls_key_path".into(),
                ));
            }
        };
        let bind = format!("{}:{}", config.bind_address, config.port);
        let listener =
            TcpListener::bind(&bind).map_err(|e| net_err(format!("bind {}: {}", bind, e)))?;
        self.local_addr = Some(listener.local_addr().map_err(|e| net_err(e.to_string()))?);
        listener
            .set_nonblocking(true)
            .map_err(|e| net_err(e.to_string()))?;
        self.is_hosting = true;

        let running = self.running.clone();
        let event_tx = self.event_tx.clone();
        let next_id = self.next_id.clone();
        let max_conns = config.max_connections;
        let conn_count = self.conn_count.clone();
        let tls_config = tls_config.clone();

        let h = thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if conn_count.load(Ordering::Relaxed) >= max_conns {
                            continue;
                        }
                        if stream.set_nonblocking(false).is_err() {
                            continue;
                        }
                        if let Some(tls_config) = &tls_config {
                            let server_conn =
                                match rustls::ServerConnection::new(tls_config.clone()) {
                                    Ok(conn) => conn,
                                    Err(e) => {
                                        log::warn!("WS TLS server connection setup failed: {}", e);
                                        continue;
                                    }
                                };
                            let tls_stream = rustls::StreamOwned::new(server_conn, stream);
                            match tungstenite::accept(tls_stream) {
                                Ok(ws) => {
                                    let _ = ws.get_ref().sock.set_read_timeout(Some(READ_TIMEOUT));
                                    let id = ConnectionId(next_id.fetch_add(1, Ordering::Relaxed));
                                    conn_count.fetch_add(1, Ordering::Relaxed);
                                    let wtx =
                                        spawn_io_thread(id, ws, event_tx.clone(), running.clone());
                                    let _ = event_tx.send(InternalWsEvent::WriteTxReady(id, wtx));
                                    let _ = event_tx.send(InternalWsEvent::Connected(id));
                                }
                                Err(e) => log::warn!("WS TLS handshake failed: {}", e),
                            }
                        } else {
                            match tungstenite::accept(stream) {
                                Ok(ws) => {
                                    set_stream_timeout(ws.get_ref());
                                    let id = ConnectionId(next_id.fetch_add(1, Ordering::Relaxed));
                                    conn_count.fetch_add(1, Ordering::Relaxed);
                                    let wtx =
                                        spawn_io_thread(id, ws, event_tx.clone(), running.clone());
                                    let _ = event_tx.send(InternalWsEvent::WriteTxReady(id, wtx));
                                    let _ = event_tx.send(InternalWsEvent::Connected(id));
                                }
                                Err(e) => log::warn!("WS handshake failed: {}", e),
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => thread::sleep(Duration::from_millis(5)),
                }
            }
        });
        if let Ok(mut g) = self.threads.lock() {
            g.push(h);
        }
        Ok(())
    }

    fn connect(&mut self, addr: &str) -> GoudResult<ConnectionId> {
        let id = self.allocate_id();
        self.connections.insert(
            id.0,
            WsConnection {
                id,
                state: ConnectionState::Connecting,
                stats: NetworkStatsTracker::new(),
            },
        );
        let url = if addr.starts_with("ws://") || addr.starts_with("wss://") {
            addr.to_string()
        } else {
            format!("ws://{}", addr)
        };
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();

        let h = thread::spawn(move || match connect_with_optional_custom_ca(&url) {
            Ok((ws, _)) => {
                match ws.get_ref() {
                    tungstenite::stream::MaybeTlsStream::Plain(s) => {
                        let _ = s.set_read_timeout(Some(READ_TIMEOUT));
                    }
                    tungstenite::stream::MaybeTlsStream::Rustls(s) => {
                        let _ = s.sock.set_read_timeout(Some(READ_TIMEOUT));
                    }
                    _ => {}
                }
                let wtx = spawn_io_thread(id, ws, event_tx.clone(), running);
                let _ = event_tx.send(InternalWsEvent::WriteTxReady(id, wtx));
                let _ = event_tx.send(InternalWsEvent::Connected(id));
            }
            Err(e) => {
                let _ = event_tx.send(InternalWsEvent::Error(id, format!("connect: {}", e)));
                let _ = event_tx.send(InternalWsEvent::Disconnected(
                    id,
                    DisconnectReason::Error(e.to_string()),
                ));
            }
        });
        if let Ok(mut g) = self.threads.lock() {
            g.push(h);
        }
        Ok(id)
    }

    fn disconnect(&mut self, conn_id: ConnectionId) -> GoudResult<()> {
        self.send_txs.remove(&conn_id.0);
        self.connections.remove(&conn_id.0);
        self.conn_count.fetch_sub(1, Ordering::Relaxed);
        self.events.push(NetworkEvent::Disconnected {
            conn: conn_id,
            reason: DisconnectReason::LocalClose,
        });
        Ok(())
    }

    fn disconnect_all(&mut self) -> GoudResult<()> {
        let ids: Vec<_> = self.connections.keys().map(|k| ConnectionId(*k)).collect();
        for id in ids {
            let _ = self.disconnect(id);
        }
        Ok(())
    }

    fn send(&mut self, conn_id: ConnectionId, _ch: Channel, data: &[u8]) -> GoudResult<()> {
        let conn = self
            .connections
            .get(&conn_id.0)
            .ok_or_else(|| net_err(format!("Unknown connection {:?}", conn_id)))?;
        if conn.state != ConnectionState::Connected {
            return Err(net_err("Connection not established".into()));
        }
        let tx = self
            .send_txs
            .get(&conn_id.0)
            .ok_or_else(|| net_err("No write channel".into()))?;
        let len = data.len();
        tx.send(data.to_vec())
            .map_err(|e| net_err(format!("enqueue: {}", e)))?;
        self.stats.record_sent_packet(len);
        if let Some(c) = self.connections.get_mut(&conn_id.0) {
            c.stats.record_sent_packet(len);
        }
        Ok(())
    }

    fn broadcast(&mut self, ch: Channel, data: &[u8]) -> GoudResult<()> {
        let ids: Vec<_> = self
            .connections
            .values()
            .filter(|c| c.state == ConnectionState::Connected)
            .map(|c| c.id)
            .collect();
        for id in ids {
            let _ = self.send(id, ch, data);
        }
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<NetworkEvent> {
        std::mem::take(&mut self.events)
    }

    fn connections(&self) -> Vec<ConnectionId> {
        self.connections.keys().map(|k| ConnectionId(*k)).collect()
    }

    fn connection_state(&self, conn: ConnectionId) -> ConnectionState {
        self.connections
            .get(&conn.0)
            .map(|c| c.state)
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
            .map(|c| c.stats.snapshot_connection())
    }

    fn network_diagnostics(&self) -> NetworkDiagnosticsV1 {
        let s = self.stats();
        NetworkDiagnosticsV1 {
            bytes_sent: s.bytes_sent,
            bytes_received: s.bytes_received,
            packets_sent: s.packets_sent,
            packets_received: s.packets_received,
            rtt_ms: s.rtt_ms,
            active_connections: self.connections().len() as u32,
        }
    }
}

impl std::fmt::Debug for WsNetProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsNetProvider")
            .field("is_hosting", &self.is_hosting)
            .field("connections", &self.connections.len())
            .finish()
    }
}
