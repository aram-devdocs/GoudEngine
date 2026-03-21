//! RPC (Remote Procedure Call) framework for GoudEngine networking.
//!
//! Sits above the [`NetworkProvider`] transport layer and provides a
//! higher-level abstraction for calling functions across the network.
//!
//! # Wire Format
//!
//! All RPC messages are sent over Channel 0 (reliable-ordered) with the
//! following layout:
//!
//! ```text
//! [2 bytes: rpc_id][8 bytes: call_id][1 byte: msg_type][N bytes: payload]
//! ```
//!
//! Where `msg_type` is:
//! - `0` = call (request)
//! - `1` = response (success)
//! - `2` = error response
//!
//! # Example
//!
//! ```rust
//! use goud_engine::libs::networking::rpc::{RpcFramework, RpcConfig, RpcDirection};
//!
//! let mut rpc = RpcFramework::new(RpcConfig::default());
//! rpc.register(
//!     1,
//!     "ping".to_string(),
//!     RpcDirection::Bidirectional,
//!     Box::new(|payload| {
//!         // Echo the payload back
//!         payload.to_vec()
//!     }),
//! ).unwrap();
//! ```

use std::collections::HashMap;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Public Types
// ---------------------------------------------------------------------------

/// Identifier for a registered RPC procedure.
pub type RpcId = u16;

/// A callback that processes an incoming RPC call payload and returns the
/// response payload. Must be `Send + Sync` so the framework can be used
/// from any thread that holds the mutex.
pub type RpcHandler = Box<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>;

/// Configuration knobs for the RPC framework.
#[derive(Debug, Clone)]
pub struct RpcConfig {
    /// How many milliseconds to wait before a pending call is considered
    /// timed out. Default: 5000.
    pub timeout_ms: u64,
    /// Maximum payload size in bytes (header excluded). Default: 65535.
    pub max_payload_size: usize,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            max_payload_size: 65535,
        }
    }
}

/// Which direction an RPC is allowed to flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcDirection {
    /// Only the server may call this RPC on a client.
    ServerToClient,
    /// Only a client may call this RPC on the server.
    ClientToServer,
    /// Either side may invoke the RPC.
    Bidirectional,
}

/// The outcome of an RPC call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcResult {
    /// The remote handler returned a payload.
    Success(Vec<u8>),
    /// The call timed out before a response arrived.
    Timeout,
    /// The peer could not be reached.
    PeerUnreachable,
    /// A generic error occurred (message from the remote side).
    Error(String),
}

// ---------------------------------------------------------------------------
// Wire‑format constants
// ---------------------------------------------------------------------------

/// Size of the RPC wire header: rpc_id(2) + call_id(8) + msg_type(1).
pub const RPC_HEADER_SIZE: usize = 2 + 8 + 1;

const MSG_TYPE_CALL: u8 = 0;
const MSG_TYPE_RESPONSE: u8 = 1;
const MSG_TYPE_ERROR: u8 = 2;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

struct RpcRegistration {
    _id: RpcId,
    _name: String,
    handler: RpcHandler,
    _direction: RpcDirection,
}

/// A pending outbound call awaiting a response.
struct PendingRpc {
    _call_id: u64,
    _rpc_id: RpcId,
    sent_at: Instant,
    result: Option<RpcResult>,
}

/// A fully formed outbound message ready to be sent by the caller.
#[derive(Debug, Clone)]
pub struct OutgoingRpcMessage {
    /// The peer this message should be sent to. `u64` mirrors `ConnectionId.0`.
    pub peer_id: u64,
    /// The serialized wire bytes to send on Channel 0.
    pub data: Vec<u8>,
}

// ---------------------------------------------------------------------------
// RpcFramework
// ---------------------------------------------------------------------------

/// The core RPC system.
///
/// The framework does **not** own a network provider directly. Instead, it
/// produces [`OutgoingRpcMessage`] values that the caller is responsible for
/// sending via whatever transport is active.  Incoming raw bytes are fed to
/// [`process_incoming`](Self::process_incoming) which dispatches to
/// registered handlers and queues response messages.
pub struct RpcFramework {
    config: RpcConfig,
    handlers: HashMap<RpcId, RpcRegistration>,
    pending_calls: HashMap<u64, PendingRpc>,
    next_call_id: u64,
    /// Outbound messages produced by `call()` and `process_incoming()`.
    outbox: Vec<OutgoingRpcMessage>,
}

impl RpcFramework {
    /// Creates a new RPC framework with the given configuration.
    pub fn new(config: RpcConfig) -> Self {
        Self {
            config,
            handlers: HashMap::new(),
            pending_calls: HashMap::new(),
            next_call_id: 1,
            outbox: Vec::new(),
        }
    }

    // -- Registration -------------------------------------------------------

    /// Registers an RPC handler.
    ///
    /// Returns `Err` if `id` is already registered.
    pub fn register(
        &mut self,
        id: RpcId,
        name: String,
        direction: RpcDirection,
        handler: RpcHandler,
    ) -> Result<(), String> {
        if self.handlers.contains_key(&id) {
            return Err(format!("RPC id {} is already registered", id));
        }
        self.handlers.insert(
            id,
            RpcRegistration {
                _id: id,
                _name: name,
                handler,
                _direction: direction,
            },
        );
        Ok(())
    }

    /// Returns `true` if an RPC with the given id is registered.
    pub fn is_registered(&self, id: RpcId) -> bool {
        self.handlers.contains_key(&id)
    }

    // -- Outbound calls -----------------------------------------------------

    /// Initiates an asynchronous RPC call to `peer_id`.
    ///
    /// The serialized request is placed in the outbox (retrieve with
    /// [`drain_outbox`](Self::drain_outbox)).  Returns the `call_id` that
    /// can later be used to poll for the response via
    /// [`take_result`](Self::take_result).
    ///
    /// Returns `Err` if the payload exceeds `max_payload_size`.
    pub fn call(&mut self, peer_id: u64, rpc_id: RpcId, payload: &[u8]) -> Result<u64, String> {
        if payload.len() > self.config.max_payload_size {
            return Err(format!(
                "Payload size {} exceeds max {}",
                payload.len(),
                self.config.max_payload_size,
            ));
        }

        let call_id = self.next_call_id;
        self.next_call_id += 1;

        let data = encode_message(rpc_id, call_id, MSG_TYPE_CALL, payload);
        self.outbox.push(OutgoingRpcMessage { peer_id, data });

        self.pending_calls.insert(
            call_id,
            PendingRpc {
                _call_id: call_id,
                _rpc_id: rpc_id,
                sent_at: Instant::now(),
                result: None,
            },
        );

        Ok(call_id)
    }

    /// Blocking convenience wrapper around [`call`](Self::call).
    ///
    /// **NOTE:** Because `RpcFramework` does not own the transport, this
    /// method cannot actually block on the network.  It initiates the call
    /// and returns `RpcResult::Timeout` immediately.  In practice callers
    /// should use `call()` + `update()` + `take_result()`.
    pub fn call_sync(
        &mut self,
        peer_id: u64,
        rpc_id: RpcId,
        payload: &[u8],
    ) -> Result<u64, String> {
        self.call(peer_id, rpc_id, payload)
    }

    // -- Incoming data ------------------------------------------------------

    /// Feed raw bytes received from `peer_id` into the framework.
    ///
    /// If the message is a **call**, the corresponding handler is invoked
    /// and a response message is placed in the outbox.
    ///
    /// If the message is a **response** or **error**, the matching pending
    /// call is resolved.
    pub fn process_incoming(&mut self, peer_id: u64, data: &[u8]) -> Result<(), String> {
        if data.len() < RPC_HEADER_SIZE {
            return Err("RPC message too short".to_string());
        }

        let (rpc_id, call_id, msg_type, payload) = decode_message(data)?;

        match msg_type {
            MSG_TYPE_CALL => {
                let response_payload = match self.handlers.get(&rpc_id) {
                    Some(reg) => (reg.handler)(payload),
                    None => {
                        // Unknown RPC -- send error back.
                        let err_msg = format!("Unknown RPC id {}", rpc_id);
                        let err_bytes =
                            encode_message(rpc_id, call_id, MSG_TYPE_ERROR, err_msg.as_bytes());
                        self.outbox.push(OutgoingRpcMessage {
                            peer_id,
                            data: err_bytes,
                        });
                        return Ok(());
                    }
                };

                let resp_bytes =
                    encode_message(rpc_id, call_id, MSG_TYPE_RESPONSE, &response_payload);
                self.outbox.push(OutgoingRpcMessage {
                    peer_id,
                    data: resp_bytes,
                });
            }
            MSG_TYPE_RESPONSE => {
                if let Some(pending) = self.pending_calls.get_mut(&call_id) {
                    pending.result = Some(RpcResult::Success(payload.to_vec()));
                }
            }
            MSG_TYPE_ERROR => {
                if let Some(pending) = self.pending_calls.get_mut(&call_id) {
                    let msg = String::from_utf8_lossy(payload).to_string();
                    pending.result = Some(RpcResult::Error(msg));
                }
            }
            _ => {
                return Err(format!("Unknown RPC msg_type {}", msg_type));
            }
        }

        Ok(())
    }

    // -- Tick / timeout management ------------------------------------------

    /// Advances the framework by `_delta_secs` (currently unused) and
    /// expires any pending calls that have exceeded the configured timeout.
    pub fn update(&mut self, _delta_secs: f32) {
        let timeout = std::time::Duration::from_millis(self.config.timeout_ms);
        let now = Instant::now();

        for pending in self.pending_calls.values_mut() {
            if pending.result.is_none() && now.duration_since(pending.sent_at) >= timeout {
                pending.result = Some(RpcResult::Timeout);
            }
        }
    }

    // -- Result retrieval ---------------------------------------------------

    /// Takes the result for `call_id` if it is ready, removing it from the
    /// pending set.  Returns `None` if the call is still in-flight.
    pub fn take_result(&mut self, call_id: u64) -> Option<RpcResult> {
        if let Some(pending) = self.pending_calls.get(&call_id) {
            if pending.result.is_some() {
                return self.pending_calls.remove(&call_id).and_then(|p| p.result);
            }
        }
        None
    }

    /// Returns `true` if a result is available for `call_id`.
    pub fn has_result(&self, call_id: u64) -> bool {
        self.pending_calls
            .get(&call_id)
            .is_some_and(|p| p.result.is_some())
    }

    /// Returns the number of calls still awaiting a response.
    pub fn pending_count(&self) -> usize {
        self.pending_calls
            .values()
            .filter(|p| p.result.is_none())
            .count()
    }

    // -- Outbox -------------------------------------------------------------

    /// Drains all outbound messages produced since the last drain.
    pub fn drain_outbox(&mut self) -> Vec<OutgoingRpcMessage> {
        std::mem::take(&mut self.outbox)
    }

    /// Returns the current configuration (read-only).
    pub fn config(&self) -> &RpcConfig {
        &self.config
    }
}

// ---------------------------------------------------------------------------
// Wire encoding / decoding
// ---------------------------------------------------------------------------

/// Encode an RPC message into its wire representation.
fn encode_message(rpc_id: RpcId, call_id: u64, msg_type: u8, payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(RPC_HEADER_SIZE + payload.len());
    buf.extend_from_slice(&rpc_id.to_le_bytes());
    buf.extend_from_slice(&call_id.to_le_bytes());
    buf.push(msg_type);
    buf.extend_from_slice(payload);
    buf
}

/// Decode an RPC message, returning `(rpc_id, call_id, msg_type, payload)`.
fn decode_message(data: &[u8]) -> Result<(RpcId, u64, u8, &[u8]), String> {
    if data.len() < RPC_HEADER_SIZE {
        return Err("RPC message too short for header".to_string());
    }
    let rpc_id = u16::from_le_bytes([data[0], data[1]]);
    let call_id = u64::from_le_bytes([
        data[2], data[3], data[4], data[5], data[6], data[7], data[8], data[9],
    ]);
    let msg_type = data[10];
    let payload = &data[RPC_HEADER_SIZE..];
    Ok((rpc_id, call_id, msg_type, payload))
}

// ---------------------------------------------------------------------------
// Publicly expose encode/decode for FFI and tests
// ---------------------------------------------------------------------------

/// Encode an RPC message (public for FFI layer).
pub fn rpc_encode_message(rpc_id: RpcId, call_id: u64, msg_type: u8, payload: &[u8]) -> Vec<u8> {
    encode_message(rpc_id, call_id, msg_type, payload)
}

/// Decode an RPC message (public for FFI layer).
pub fn rpc_decode_message(data: &[u8]) -> Result<(RpcId, u64, u8, &[u8]), String> {
    decode_message(data)
}
