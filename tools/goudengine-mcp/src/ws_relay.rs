//! WebSocket relay for browser-based debugger connections.
//!
//! Browser games connect to this relay via WebSocket. The relay registers
//! their routes in a shared registry and forwards IPC JSON between the
//! MCP server tool handlers and the browser game.

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

/// Default WebSocket relay port.
pub const DEFAULT_WS_PORT: u16 = 9229;

/// Registration message sent by the browser game on connect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsRegistration {
    pub route_label: String,
    pub surface_kind: String,
    pub capabilities: Vec<String>,
}

/// One connected browser game.
struct WsClient {
    registration: WsRegistration,
    /// Send IPC request, receive response via oneshot.
    request_tx: tokio::sync::mpsc::Sender<(Value, oneshot::Sender<Value>)>,
}

/// Shared state for all WebSocket-connected browser routes.
#[derive(Clone)]
pub struct WsRelayState {
    inner: Arc<Mutex<WsRelayInner>>,
}

struct WsRelayInner {
    clients: HashMap<String, WsClient>,
    next_id: u64,
}

impl Default for WsRelayState {
    fn default() -> Self {
        Self::new()
    }
}

impl WsRelayState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(WsRelayInner {
                clients: HashMap::new(),
                next_id: 1,
            })),
        }
    }

    /// Lists currently connected browser routes.
    pub async fn list_routes(&self) -> Vec<WsRouteInfo> {
        let inner = self.inner.lock().await;
        inner
            .clients
            .iter()
            .map(|(id, client)| WsRouteInfo {
                route_id: id.clone(),
                label: client.registration.route_label.clone(),
                surface_kind: client.registration.surface_kind.clone(),
            })
            .collect()
    }

    /// Sends an IPC request to a browser route and awaits the response.
    pub async fn request(&self, route_id: &str, request: Value) -> Result<Value, String> {
        let tx = {
            let inner = self.inner.lock().await;
            let client = inner
                .clients
                .get(route_id)
                .ok_or_else(|| "browser route not connected".to_string())?;
            client.request_tx.clone()
        };
        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send((request, resp_tx))
            .await
            .map_err(|_| "browser route disconnected".to_string())?;
        resp_rx
            .await
            .map_err(|_| "browser route did not respond".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsRouteInfo {
    pub route_id: String,
    pub label: String,
    pub surface_kind: String,
}

/// Spawns the WebSocket relay listener.
pub async fn start_ws_relay(state: WsRelayState, port: u16) -> Result<(), String> {
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("ws relay bind failed: {e}"))?;
    eprintln!("[ws-relay] listening on ws://{addr}");

    tokio::spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            let state = state.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_ws_connection(state, stream).await {
                    eprintln!("[ws-relay] connection from {addr} error: {e}");
                }
            });
        }
    });

    Ok(())
}

async fn handle_ws_connection(
    state: WsRelayState,
    stream: tokio::net::TcpStream,
) -> Result<(), String> {
    let ws = accept_async(stream)
        .await
        .map_err(|e| format!("ws handshake failed: {e}"))?;
    let (mut ws_tx, mut ws_rx) = ws.split();

    // First message must be registration.
    let reg_msg = ws_rx
        .next()
        .await
        .ok_or("connection closed before registration")?
        .map_err(|e| format!("ws read error: {e}"))?;

    let registration: WsRegistration = match reg_msg {
        Message::Text(text) => {
            serde_json::from_str(&text).map_err(|e| format!("invalid registration: {e}"))?
        }
        _ => return Err("expected text registration message".to_string()),
    };

    let (req_tx, mut req_rx) = tokio::sync::mpsc::channel::<(Value, oneshot::Sender<Value>)>(16);

    let route_id = {
        let mut inner = state.inner.lock().await;
        let id = format!("ws-{}", inner.next_id);
        inner.next_id += 1;
        inner.clients.insert(
            id.clone(),
            WsClient {
                registration: registration.clone(),
                request_tx: req_tx,
            },
        );
        id
    };

    eprintln!(
        "[ws-relay] browser registered: route_id={route_id}, label={}",
        registration.route_label
    );

    // Send ack to browser.
    let ack = json!({
        "type": "registration_ack",
        "route_id": route_id,
    });
    ws_tx
        .send(Message::Text(serde_json::to_string(&ack).unwrap()))
        .await
        .map_err(|e| format!("ws send ack failed: {e}"))?;

    // Main relay loop: forward requests from MCP server to browser and back.
    loop {
        tokio::select! {
            // MCP server wants to send a request to the browser
            Some((request, resp_tx)) = req_rx.recv() => {
                let msg = serde_json::to_string(&request).unwrap_or_default();
                if ws_tx.send(Message::Text(msg)).await.is_err() {
                    let _ = resp_tx.send(json!({
                        "ok": false,
                        "error": {"code": "protocol_error", "message": "browser disconnected"}
                    }));
                    break;
                }
                // Wait for browser response
                match ws_rx.next().await {
                    Some(Ok(Message::Text(text))) => {
                        let response: Value = serde_json::from_str(&text).unwrap_or(json!({
                            "ok": false,
                            "error": {"code": "protocol_error", "message": "invalid response from browser"}
                        }));
                        let _ = resp_tx.send(response);
                    }
                    _ => {
                        let _ = resp_tx.send(json!({
                            "ok": false,
                            "error": {"code": "protocol_error", "message": "browser disconnected"}
                        }));
                        break;
                    }
                }
            }
            // Browser sends unsolicited message (e.g. disconnect)
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {} // Ignore other unsolicited messages
                }
            }
        }
    }

    // Cleanup
    {
        let mut inner = state.inner.lock().await;
        inner.clients.remove(&route_id);
    }
    eprintln!("[ws-relay] browser disconnected: route_id={route_id}");

    Ok(())
}
