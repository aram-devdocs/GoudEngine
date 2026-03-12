//! Browser WebSocket networking methods for `WasmGame`.
//!
//! This module implements client-side WebSocket networking for WASM targets.
//! Hosting/server behavior is intentionally unsupported in browser mode.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{BinaryType, CloseEvent, Event, MessageEvent, WebSocket};

use super::WasmGame;

const PROTOCOL_WS: i32 = 1;
const ERR_UNSUPPORTED: i32 = -1;
const ERR_INVALID_HANDLE: i32 = -2;
const ERR_SEND_FAILED: i32 = -3;
const WEB_MAX_MESSAGE_SIZE: u32 = 16_777_216;

#[derive(Debug, Clone, Copy, Default)]
struct StatsSnapshot {
    bytes_sent: u64,
    bytes_received: u64,
    packets_sent: u64,
    packets_received: u64,
    packets_lost: u64,
}

#[derive(Default)]
struct WasmSocketShared {
    connected: bool,
    closed: bool,
    inbound: VecDeque<Vec<u8>>,
    stats: StatsSnapshot,
}

struct WasmWebSocketConnection {
    socket: WebSocket,
    peer_id: u64,
    shared: Rc<RefCell<WasmSocketShared>>,
    _on_open: Closure<dyn FnMut(Event)>,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_error: Closure<dyn FnMut(Event)>,
    _on_close: Closure<dyn FnMut(CloseEvent)>,
}

impl WasmWebSocketConnection {
    fn disconnect(self) {
        self.socket.set_onopen(None);
        self.socket.set_onmessage(None);
        self.socket.set_onerror(None);
        self.socket.set_onclose(None);
        let _ = self.socket.close();
    }
}

pub(super) struct WasmNetworkState {
    next_handle: i32,
    connections: HashMap<i32, WasmWebSocketConnection>,
}

impl WasmNetworkState {
    pub(super) fn new() -> Self {
        Self {
            next_handle: 1,
            connections: HashMap::new(),
        }
    }

    fn allocate_handle(&mut self) -> i32 {
        let handle = self.next_handle.max(1);
        self.next_handle = self.next_handle.saturating_add(1);
        handle
    }

    fn get(&self, handle: i32) -> Option<&WasmWebSocketConnection> {
        self.connections.get(&handle)
    }

    fn get_mut(&mut self, handle: i32) -> Option<&mut WasmWebSocketConnection> {
        self.connections.get_mut(&handle)
    }

    fn remove(&mut self, handle: i32) -> Option<WasmWebSocketConnection> {
        self.connections.remove(&handle)
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct WasmNetworkConnectResult {
    pub handle: i32,
    pub peer_id: u64,
}

#[wasm_bindgen]
pub struct WasmNetworkPacket {
    peer_id: u64,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl WasmNetworkPacket {
    #[wasm_bindgen(getter)]
    pub fn peer_id(&self) -> u64 {
        self.peer_id
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct WasmNetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
    pub rtt_ms: f32,
    pub send_bandwidth_bytes_per_sec: f32,
    pub receive_bandwidth_bytes_per_sec: f32,
    pub packet_loss_percent: f32,
    pub jitter_ms: f32,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct WasmNetworkCapabilities {
    pub supports_hosting: bool,
    pub max_connections: u32,
    pub max_channels: u8,
    pub max_message_size: u32,
}

fn build_ws_url(address: &str, port: u16) -> String {
    if address.starts_with("ws://") || address.starts_with("wss://") {
        return address.to_string();
    }
    if port == 0 {
        return format!("ws://{}", address);
    }
    format!("ws://{}:{}", address, port)
}

#[wasm_bindgen]
impl WasmGame {
    /// Browser-hosting is not supported for WASM networking.
    pub fn network_host(&mut self, _protocol: i32, _port: u16) -> i32 {
        ERR_UNSUPPORTED
    }

    /// Connects to a remote WebSocket endpoint and returns a connection handle.
    pub fn network_connect_with_peer(
        &mut self,
        protocol: i32,
        address: String,
        port: u16,
    ) -> Result<WasmNetworkConnectResult, JsValue> {
        if protocol != PROTOCOL_WS {
            return Err(JsValue::from_str(
                "WASM networking supports WebSocket protocol only",
            ));
        }

        let url = build_ws_url(&address, port);
        let socket = WebSocket::new(&url)
            .map_err(|_| JsValue::from_str(&format!("Failed to connect to {}", url)))?;
        socket.set_binary_type(BinaryType::Arraybuffer);

        let shared = Rc::new(RefCell::new(WasmSocketShared::default()));
        let open_shared = Rc::clone(&shared);
        let message_shared = Rc::clone(&shared);
        let error_shared = Rc::clone(&shared);
        let close_shared = Rc::clone(&shared);

        let on_open = Closure::wrap(Box::new(move |_event: Event| {
            let mut state = open_shared.borrow_mut();
            state.connected = true;
            state.closed = false;
        }) as Box<dyn FnMut(_)>);

        let on_message = Closure::wrap(Box::new(move |event: MessageEvent| {
            let mut state = message_shared.borrow_mut();
            if let Ok(buffer) = event.data().dyn_into::<ArrayBuffer>() {
                let payload = Uint8Array::new(&buffer).to_vec();
                state.stats.bytes_received = state
                    .stats
                    .bytes_received
                    .saturating_add(payload.len() as u64);
                state.stats.packets_received = state.stats.packets_received.saturating_add(1);
                state.inbound.push_back(payload);
            } else if let Ok(array) = event.data().dyn_into::<Uint8Array>() {
                let payload = array.to_vec();
                state.stats.bytes_received = state
                    .stats
                    .bytes_received
                    .saturating_add(payload.len() as u64);
                state.stats.packets_received = state.stats.packets_received.saturating_add(1);
                state.inbound.push_back(payload);
            }
        }) as Box<dyn FnMut(_)>);

        let on_error = Closure::wrap(Box::new(move |_event: Event| {
            let mut state = error_shared.borrow_mut();
            state.stats.packets_lost = state.stats.packets_lost.saturating_add(1);
        }) as Box<dyn FnMut(_)>);

        let on_close = Closure::wrap(Box::new(move |_event: CloseEvent| {
            let mut state = close_shared.borrow_mut();
            state.connected = false;
            state.closed = true;
        }) as Box<dyn FnMut(_)>);

        socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        socket.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));

        let handle = self.network_state.allocate_handle();
        let peer_id = 1_u64;
        self.network_state.connections.insert(
            handle,
            WasmWebSocketConnection {
                socket,
                peer_id,
                shared,
                _on_open: on_open,
                _on_message: on_message,
                _on_error: on_error,
                _on_close: on_close,
            },
        );

        Ok(WasmNetworkConnectResult { handle, peer_id })
    }

    /// Convenience connect returning the connection handle only.
    pub fn network_connect(
        &mut self,
        protocol: i32,
        address: String,
        port: u16,
    ) -> Result<i32, JsValue> {
        self.network_connect_with_peer(protocol, address, port)
            .map(|result| result.handle)
    }

    /// Sends a binary packet on an existing connection.
    pub fn network_send(&mut self, handle: i32, _peer_id: u64, data: &[u8], _channel: u8) -> i32 {
        let Some(conn) = self.network_state.get_mut(handle) else {
            return ERR_INVALID_HANDLE;
        };

        if conn.socket.ready_state() != WebSocket::OPEN {
            return ERR_SEND_FAILED;
        }

        if conn.socket.send_with_u8_array(data).is_err() {
            let mut state = conn.shared.borrow_mut();
            state.stats.packets_lost = state.stats.packets_lost.saturating_add(1);
            return ERR_SEND_FAILED;
        }

        let mut state = conn.shared.borrow_mut();
        state.stats.bytes_sent = state.stats.bytes_sent.saturating_add(data.len() as u64);
        state.stats.packets_sent = state.stats.packets_sent.saturating_add(1);
        0
    }

    /// Receives the next queued packet with peer metadata.
    pub fn network_receive_packet(&mut self, handle: i32) -> Option<WasmNetworkPacket> {
        let conn = self.network_state.get(handle)?;
        let mut state = conn.shared.borrow_mut();
        let data = state.inbound.pop_front()?;
        Some(WasmNetworkPacket {
            peer_id: conn.peer_id,
            data,
        })
    }

    /// Legacy receive path returning payload bytes only.
    pub fn network_receive(&mut self, handle: i32) -> Vec<u8> {
        self.network_receive_packet(handle)
            .map(|packet| packet.data)
            .unwrap_or_default()
    }

    /// Poll is a no-op for browser sockets (event-driven), but validates the handle.
    pub fn network_poll(&mut self, handle: i32) -> i32 {
        if self.network_state.get(handle).is_some() {
            0
        } else {
            ERR_INVALID_HANDLE
        }
    }

    /// Disconnects and removes a network connection.
    pub fn network_disconnect(&mut self, handle: i32) -> i32 {
        let Some(conn) = self.network_state.remove(handle) else {
            return ERR_INVALID_HANDLE;
        };
        conn.disconnect();
        0
    }

    /// Returns aggregate stats for a specific network handle.
    pub fn get_network_stats(&self, handle: i32) -> WasmNetworkStats {
        let Some(conn) = self.network_state.get(handle) else {
            return WasmNetworkStats {
                bytes_sent: 0,
                bytes_received: 0,
                packets_sent: 0,
                packets_received: 0,
                packets_lost: 0,
                rtt_ms: 0.0,
                send_bandwidth_bytes_per_sec: 0.0,
                receive_bandwidth_bytes_per_sec: 0.0,
                packet_loss_percent: 0.0,
                jitter_ms: 0.0,
            };
        };
        let state = conn.shared.borrow();
        WasmNetworkStats {
            bytes_sent: state.stats.bytes_sent,
            bytes_received: state.stats.bytes_received,
            packets_sent: state.stats.packets_sent,
            packets_received: state.stats.packets_received,
            packets_lost: state.stats.packets_lost,
            rtt_ms: 0.0,
            send_bandwidth_bytes_per_sec: 0.0,
            receive_bandwidth_bytes_per_sec: 0.0,
            packet_loss_percent: 0.0,
            jitter_ms: 0.0,
        }
    }

    /// Returns network capabilities for browser/WebSocket mode.
    pub fn get_network_capabilities(&self) -> WasmNetworkCapabilities {
        WasmNetworkCapabilities {
            supports_hosting: false,
            max_connections: 1,
            max_channels: 1,
            max_message_size: WEB_MAX_MESSAGE_SIZE,
        }
    }

    /// Returns active peer count for the connection.
    pub fn network_peer_count(&self, handle: i32) -> i32 {
        let Some(conn) = self.network_state.get(handle) else {
            return 0;
        };
        let state = conn.shared.borrow();
        if state.connected && !state.closed {
            1
        } else {
            0
        }
    }

    /// Network simulation controls are unsupported in browser networking mode.
    pub fn set_network_simulation(
        &mut self,
        _handle: i32,
        _one_way_latency_ms: u32,
        _jitter_ms: u32,
        _packet_loss_percent: f32,
    ) -> i32 {
        ERR_UNSUPPORTED
    }

    /// Network simulation controls are unsupported in browser networking mode.
    pub fn clear_network_simulation(&mut self, _handle: i32) -> i32 {
        ERR_UNSUPPORTED
    }

    /// Overlay handle selection is unsupported in browser networking mode.
    pub fn set_network_overlay_handle(&mut self, _handle: i32) -> i32 {
        ERR_UNSUPPORTED
    }

    /// Overlay handle selection is unsupported in browser networking mode.
    pub fn clear_network_overlay_handle(&mut self) -> i32 {
        ERR_UNSUPPORTED
    }
}
