//! WebSocket transport provider implementing `NetworkProvider`.
//!
//! Uses `tungstenite` for synchronous WebSocket I/O on native targets.
//! Each connection runs a single I/O thread that handles both reading
//! (non-blocking with short timeouts) and writing (from an mpsc channel).

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::collections::HashMap;
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    use tungstenite::protocol::Message;

    use crate::core::providers::network::NetworkProvider;
    use crate::core::providers::network_types::{
        Channel, ConnectionId, ConnectionState, ConnectionStats,
        DisconnectReason, HostConfig, NetworkCapabilities, NetworkEvent,
        NetworkStats,
    };
    use crate::core::providers::{Provider, ProviderLifecycle};
    use crate::libs::error::{GoudError, GoudResult};

    /// Read timeout for non-blocking polling of the WebSocket.
    const READ_TIMEOUT: Duration = Duration::from_millis(10);

    fn net_err(msg: String) -> GoudError {
        GoudError::ProviderError { subsystem: "network", message: msg }
    }

    enum InternalWsEvent {
        Connected(ConnectionId),
        Disconnected(ConnectionId, DisconnectReason),
        Received(ConnectionId, Vec<u8>),
        Error(ConnectionId, String),
        WriteTxReady(ConnectionId, mpsc::Sender<Vec<u8>>),
    }

    // SAFETY: Only sent across threads via mpsc (requires Send, not Sync).
    // The Sender in WriteTxReady is consumed on the receiving main thread.
    unsafe impl Send for InternalWsEvent {}

    struct WsConnection {
        id: ConnectionId,
        state: ConnectionState,
        stats: ConnectionStats,
    }

    /// Spawn a single I/O thread that handles both reading and writing for
    /// a WebSocket connection. Returns the write sender for outbound data.
    fn spawn_io_thread<S>(
        cid: ConnectionId,
        mut ws: tungstenite::WebSocket<S>,
        event_tx: mpsc::Sender<InternalWsEvent>,
        running: Arc<AtomicBool>,
    ) -> mpsc::Sender<Vec<u8>>
    where
        S: std::io::Read + std::io::Write + Send + 'static,
    {
        let (write_tx, write_rx) = mpsc::channel::<Vec<u8>>();

        thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                // Try to read (non-blocking via read timeout on stream).
                match ws.read() {
                    Ok(Message::Binary(d)) => {
                        let _ = event_tx.send(InternalWsEvent::Received(cid, d.to_vec()));
                    }
                    Ok(Message::Text(t)) => {
                        let _ = event_tx.send(InternalWsEvent::Received(cid, t.as_bytes().to_vec()));
                    }
                    Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) => {
                        let _ = event_tx.send(InternalWsEvent::Disconnected(cid, DisconnectReason::RemoteClose));
                        break;
                    }
                    Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_)) => {}
                    Err(tungstenite::Error::Io(ref e))
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        // No data available; fall through to check writes.
                    }
                    Err(e) => {
                        let _ = event_tx.send(InternalWsEvent::Error(cid, format!("read: {}", e)));
                        let _ = event_tx.send(InternalWsEvent::Disconnected(cid, DisconnectReason::Error(e.to_string())));
                        break;
                    }
                }
                // Drain all pending writes.
                loop {
                    match write_rx.try_recv() {
                        Ok(data) => {
                            if let Err(e) = ws.send(Message::Binary(data.into())) {
                                let _ = event_tx.send(InternalWsEvent::Error(cid, format!("write: {}", e)));
                                return;
                            }
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => return,
                    }
                }
            }
        });
        write_tx
    }

    /// WebSocket transport provider (native only).
    pub struct WsNetProvider {
        capabilities: NetworkCapabilities,
        connections: HashMap<u64, WsConnection>,
        event_tx: mpsc::Sender<InternalWsEvent>,
        event_rx: Mutex<Option<mpsc::Receiver<InternalWsEvent>>>,
        events: Vec<NetworkEvent>,
        send_txs: HashMap<u64, mpsc::Sender<Vec<u8>>>,
        stats: NetworkStats,
        next_id: Arc<AtomicU64>,
        threads: Mutex<Vec<JoinHandle<()>>>,
        running: Arc<AtomicBool>,
        is_hosting: bool,
        local_addr: Option<std::net::SocketAddr>,
    }

    // SAFETY: Only accessed via &mut self from the main game thread. Non-Sync
    // fields (mpsc::Sender, HashMap<mpsc::Sender>) are never shared across
    // threads. Mutex wrappers on event_rx and threads provide Sync.
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
                    max_message_size: 16_777_216,
                },
                connections: HashMap::new(),
                event_tx,
                event_rx: Mutex::new(Some(event_rx)),
                events: Vec::new(),
                send_txs: HashMap::new(),
                stats: NetworkStats::default(),
                next_id: Arc::new(AtomicU64::new(1)),
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
        fn default() -> Self { Self::new() }
    }

    impl Provider for WsNetProvider {
        fn name(&self) -> &str { "websocket" }
        fn version(&self) -> &str { "0.1.0" }
        fn capabilities(&self) -> Box<dyn std::any::Any> {
            Box::new(self.capabilities.clone())
        }
    }

    impl ProviderLifecycle for WsNetProvider {
        fn init(&mut self) -> GoudResult<()> { Ok(()) }

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
                            if let Some(c) = self.connections.get_mut(&id.0) {
                                c.state = ConnectionState::Disconnected;
                            }
                            self.send_txs.remove(&id.0);
                            self.events.push(NetworkEvent::Disconnected { conn: id, reason });
                        }
                        InternalWsEvent::Received(id, data) => {
                            self.stats.bytes_received += data.len() as u64;
                            self.stats.packets_received += 1;
                            if let Some(c) = self.connections.get_mut(&id.0) {
                                c.stats.bytes_received += data.len() as u64;
                            }
                            self.events.push(NetworkEvent::Received {
                                conn: id, channel: Channel(0), data,
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
                            self.connections.entry(id.0).or_insert_with(|| WsConnection {
                                id, state: ConnectionState::Connecting,
                                stats: ConnectionStats::default(),
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
            if let Ok(mut g) = self.event_rx.lock() { g.take(); }
            if let Ok(mut g) = self.threads.lock() {
                for h in std::mem::take(&mut *g) { let _ = h.join(); }
            }
            self.connections.clear();
        }
    }

    /// Set read timeout on TcpStream for non-blocking read polling.
    fn set_stream_timeout(stream: &std::net::TcpStream) {
        let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    }

    impl NetworkProvider for WsNetProvider {
        fn host(&mut self, config: &HostConfig) -> GoudResult<()> {
            if self.is_hosting { return Err(net_err("Already hosting".into())); }
            let bind = format!("{}:{}", config.bind_address, config.port);
            let listener = TcpListener::bind(&bind)
                .map_err(|e| net_err(format!("bind {}: {}", bind, e)))?;
            self.local_addr = Some(
                listener.local_addr().map_err(|e| net_err(e.to_string()))?,
            );
            listener.set_nonblocking(true).map_err(|e| net_err(e.to_string()))?;
            self.is_hosting = true;

            let running = self.running.clone();
            let event_tx = self.event_tx.clone();
            let next_id = self.next_id.clone();
            let max_conns = config.max_connections;

            let h = thread::spawn(move || {
                let mut count = 0u32;
                while running.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            if count >= max_conns { continue; }
                            if stream.set_nonblocking(false).is_err() { continue; }
                            match tungstenite::accept(stream) {
                                Ok(ws) => {
                                    set_stream_timeout(ws.get_ref());
                                    let id = ConnectionId(
                                        next_id.fetch_add(1, Ordering::Relaxed),
                                    );
                                    count += 1;
                                    let wtx = spawn_io_thread(
                                        id, ws, event_tx.clone(), running.clone(),
                                    );
                                    let _ = event_tx.send(InternalWsEvent::WriteTxReady(id, wtx));
                                    let _ = event_tx.send(InternalWsEvent::Connected(id));
                                }
                                Err(e) => log::warn!("WS handshake failed: {}", e),
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(_) => thread::sleep(Duration::from_millis(5)),
                    }
                }
            });
            if let Ok(mut g) = self.threads.lock() { g.push(h); }
            Ok(())
        }

        fn connect(&mut self, addr: &str) -> GoudResult<ConnectionId> {
            let id = self.allocate_id();
            self.connections.insert(id.0, WsConnection {
                id, state: ConnectionState::Connecting, stats: ConnectionStats::default(),
            });
            let url = if addr.starts_with("ws://") || addr.starts_with("wss://") {
                addr.to_string()
            } else {
                format!("ws://{}", addr)
            };
            let event_tx = self.event_tx.clone();
            let running = self.running.clone();

            let h = thread::spawn(move || {
                match tungstenite::connect(&url) {
                    Ok((ws, _)) => {
                        if let tungstenite::stream::MaybeTlsStream::Plain(s) = ws.get_ref() {
                            let _ = s.set_read_timeout(Some(READ_TIMEOUT));
                        }
                        let wtx = spawn_io_thread(id, ws, event_tx.clone(), running);
                        let _ = event_tx.send(InternalWsEvent::WriteTxReady(id, wtx));
                        let _ = event_tx.send(InternalWsEvent::Connected(id));
                    }
                    Err(e) => {
                        let _ = event_tx.send(InternalWsEvent::Error(
                            id, format!("connect: {}", e),
                        ));
                        let _ = event_tx.send(InternalWsEvent::Disconnected(
                            id, DisconnectReason::Error(e.to_string()),
                        ));
                    }
                }
            });
            if let Ok(mut g) = self.threads.lock() { g.push(h); }
            Ok(id)
        }

        fn disconnect(&mut self, conn_id: ConnectionId) -> GoudResult<()> {
            self.send_txs.remove(&conn_id.0);
            if let Some(c) = self.connections.get_mut(&conn_id.0) {
                c.state = ConnectionState::Disconnected;
            }
            self.events.push(NetworkEvent::Disconnected {
                conn: conn_id, reason: DisconnectReason::LocalClose,
            });
            Ok(())
        }

        fn disconnect_all(&mut self) -> GoudResult<()> {
            let ids: Vec<_> = self.connections.keys().map(|k| ConnectionId(*k)).collect();
            for id in ids { let _ = self.disconnect(id); }
            Ok(())
        }

        fn send(&mut self, conn_id: ConnectionId, _ch: Channel, data: &[u8]) -> GoudResult<()> {
            let conn = self.connections.get(&conn_id.0)
                .ok_or_else(|| net_err(format!("Unknown connection {:?}", conn_id)))?;
            if conn.state != ConnectionState::Connected {
                return Err(net_err("Connection not established".into()));
            }
            let tx = self.send_txs.get(&conn_id.0)
                .ok_or_else(|| net_err("No write channel".into()))?;
            let len = data.len();
            tx.send(data.to_vec()).map_err(|e| net_err(format!("enqueue: {}", e)))?;
            self.stats.bytes_sent += len as u64;
            self.stats.packets_sent += 1;
            if let Some(c) = self.connections.get_mut(&conn_id.0) {
                c.stats.bytes_sent += len as u64;
            }
            Ok(())
        }

        fn broadcast(&mut self, ch: Channel, data: &[u8]) -> GoudResult<()> {
            let ids: Vec<_> = self.connections.values()
                .filter(|c| c.state == ConnectionState::Connected).map(|c| c.id).collect();
            for id in ids { let _ = self.send(id, ch, data); }
            Ok(())
        }

        fn drain_events(&mut self) -> Vec<NetworkEvent> { std::mem::take(&mut self.events) }

        fn connections(&self) -> Vec<ConnectionId> {
            self.connections.keys().map(|k| ConnectionId(*k)).collect()
        }

        fn connection_state(&self, conn: ConnectionId) -> ConnectionState {
            self.connections.get(&conn.0).map(|c| c.state).unwrap_or(ConnectionState::Disconnected)
        }

        fn local_id(&self) -> Option<ConnectionId> { None }
        fn network_capabilities(&self) -> &NetworkCapabilities { &self.capabilities }
        fn stats(&self) -> NetworkStats { self.stats.clone() }

        fn connection_stats(&self, conn: ConnectionId) -> Option<ConnectionStats> {
            self.connections.get(&conn.0).map(|c| c.stats.clone())
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
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::WsNetProvider;

// WASM WebSocket support deferred to future PR
#[cfg(target_arch = "wasm32")]
compile_error!("WebSocket provider is not yet implemented for WASM targets");

#[cfg(test)]
#[path = "ws_network_tests.rs"]
mod tests;
