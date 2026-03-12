use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use tungstenite::protocol::Message;

use crate::core::providers::network_types::{
    ConnectionId, ConnectionState, DisconnectReason, NetworkStatsTracker,
};

/// Read timeout for non-blocking polling of the WebSocket.
pub const READ_TIMEOUT: Duration = Duration::from_millis(10);

pub enum InternalWsEvent {
    Connected(ConnectionId),
    Disconnected(ConnectionId, DisconnectReason),
    Received(ConnectionId, Vec<u8>),
    Error(ConnectionId, String),
    WriteTxReady(ConnectionId, mpsc::Sender<Vec<u8>>),
}

// InternalWsEvent is Send because all variants contain only Send types.
// (ConnectionId, String, Vec<u8>, DisconnectReason, mpsc::Sender<Vec<u8>> are all Send.)
const _: () = {
    fn _assert_send<T: Send>() {}
    fn _check() {
        _assert_send::<InternalWsEvent>();
    }
};

pub struct WsConnection {
    pub id: ConnectionId,
    pub state: ConnectionState,
    pub stats: NetworkStatsTracker,
}

/// Spawn a single I/O thread that handles both reading and writing for
/// a WebSocket connection. Returns the write sender for outbound data.
pub fn spawn_io_thread<S>(
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
            match ws.read() {
                Ok(Message::Binary(d)) => {
                    let _ = event_tx.send(InternalWsEvent::Received(cid, d.to_vec()));
                }
                Ok(Message::Text(t)) => {
                    let _ = event_tx.send(InternalWsEvent::Received(cid, t.as_bytes().to_vec()));
                }
                Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) => {
                    let _ = event_tx.send(InternalWsEvent::Disconnected(
                        cid,
                        DisconnectReason::RemoteClose,
                    ));
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
                    let _ = event_tx.send(InternalWsEvent::Disconnected(
                        cid,
                        DisconnectReason::Error(e.to_string()),
                    ));
                    break;
                }
            }

            loop {
                match write_rx.try_recv() {
                    Ok(data) => {
                        if let Err(e) = ws.send(Message::Binary(data.into())) {
                            let _ =
                                event_tx.send(InternalWsEvent::Error(cid, format!("write: {}", e)));
                            return;
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        let _ = ws.send(Message::Close(None));
                        return;
                    }
                }
            }
        }
    });

    write_tx
}

/// Set read timeout on TcpStream for non-blocking read polling.
pub fn set_stream_timeout(stream: &std::net::TcpStream) {
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
}
