//! STUN protocol helpers for WebRTC NAT traversal.

use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use super::{InternalWebRtcEvent, WebRtcNetProvider};

pub(super) const STUN_BINDING_REQUEST: u16 = 0x0001;
pub(super) const STUN_MAGIC_COOKIE: u32 = 0x2112A442;

/// Minimal STUN binding request. Returns a 20-byte STUN header with a random
/// transaction ID. This is enough to discover our public-facing address from
/// a STUN server.
pub(super) fn build_stun_binding_request() -> ([u8; 20], [u8; 12]) {
    let mut msg = [0u8; 20];
    // Type: Binding Request (0x0001)
    msg[0] = (STUN_BINDING_REQUEST >> 8) as u8;
    msg[1] = (STUN_BINDING_REQUEST & 0xFF) as u8;
    // Length: 0 (no attributes)
    msg[2] = 0;
    msg[3] = 0;
    // Magic Cookie
    let cookie = STUN_MAGIC_COOKIE.to_be_bytes();
    msg[4..8].copy_from_slice(&cookie);
    // Transaction ID: 12 random bytes (use simple counter for determinism in tests)
    let tid: [u8; 12] = {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut t = [0u8; 12];
        let bytes = ts.to_le_bytes();
        let copy_len = bytes.len().min(12);
        t[..copy_len].copy_from_slice(&bytes[..copy_len]);
        t
    };
    msg[8..20].copy_from_slice(&tid);
    (msg, tid)
}

/// Parse a STUN binding response to extract the XOR-MAPPED-ADDRESS.
/// Returns `Some(SocketAddr)` if a valid address was found.
pub(super) fn parse_stun_response(data: &[u8], tid: &[u8; 12]) -> Option<SocketAddr> {
    if data.len() < 20 {
        return None;
    }
    // Verify it is a binding success response (0x0101)
    let msg_type = u16::from_be_bytes([data[0], data[1]]);
    if msg_type != 0x0101 {
        return None;
    }
    // Verify magic cookie
    let cookie = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    if cookie != STUN_MAGIC_COOKIE {
        return None;
    }
    // Verify transaction ID matches
    if data[8..20] != tid[..] {
        return None;
    }

    let msg_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    let attr_data = &data[20..20 + msg_len.min(data.len() - 20)];

    // Walk attributes looking for XOR-MAPPED-ADDRESS (0x0020) or
    // MAPPED-ADDRESS (0x0001).
    let mut offset = 0;
    while offset + 4 <= attr_data.len() {
        let attr_type = u16::from_be_bytes([attr_data[offset], attr_data[offset + 1]]);
        let attr_len = u16::from_be_bytes([attr_data[offset + 2], attr_data[offset + 3]]) as usize;
        let val_start = offset + 4;

        if attr_type == 0x0020 && attr_len >= 8 {
            // XOR-MAPPED-ADDRESS: family(1) + padding(1) + port(2) + ip(4)
            let family = attr_data[val_start + 1];
            if family == 0x01 {
                // IPv4
                let xor_port =
                    u16::from_be_bytes([attr_data[val_start + 2], attr_data[val_start + 3]]);
                let port = xor_port ^ (STUN_MAGIC_COOKIE >> 16) as u16;
                let xor_ip = u32::from_be_bytes([
                    attr_data[val_start + 4],
                    attr_data[val_start + 5],
                    attr_data[val_start + 6],
                    attr_data[val_start + 7],
                ]);
                let ip = xor_ip ^ STUN_MAGIC_COOKIE;
                let ip_bytes = ip.to_be_bytes();
                let addr = SocketAddr::from((ip_bytes, port));
                return Some(addr);
            }
        }

        // Attributes are padded to 4-byte boundaries
        let padded_len = (attr_len + 3) & !3;
        offset = val_start + padded_len;
    }

    None
}

impl WebRtcNetProvider {
    /// Perform a STUN binding request to discover the public-facing address.
    /// This is a blocking operation run on a background thread.
    pub(super) fn start_stun_discovery(&mut self) {
        if self.config.stun_servers.is_empty() {
            return;
        }
        let stun_server = self.config.stun_servers[0].clone();
        let socket = match &self.socket {
            Some(s) => match s.try_clone() {
                Ok(s) => s,
                Err(_) => return,
            },
            None => return,
        };

        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        let conn_id = self.allocate_id();

        let handle = thread::spawn(move || {
            // Strip the "stun:" prefix if present.
            let addr_str = stun_server.strip_prefix("stun:").unwrap_or(&stun_server);

            let stun_addr: SocketAddr = match addr_str.parse() {
                Ok(a) => a,
                Err(e) => {
                    let _ = event_tx.send(InternalWebRtcEvent::StunResult(
                        conn_id,
                        Err(format!("parse STUN addr: {e}")),
                    ));
                    return;
                }
            };

            let (request, tid) = build_stun_binding_request();
            // Set a blocking timeout for the STUN request.
            let _ = socket.set_read_timeout(Some(Duration::from_secs(3)));
            let _ = socket.set_nonblocking(false);
            if let Err(e) = socket.send_to(&request, stun_addr) {
                let _ = event_tx.send(InternalWebRtcEvent::StunResult(
                    conn_id,
                    Err(format!("STUN send: {e}")),
                ));
                return;
            }

            let mut buf = [0u8; 256];
            match socket.recv_from(&mut buf) {
                Ok((len, _)) => {
                    if !running.load(Ordering::Relaxed) {
                        return;
                    }
                    match parse_stun_response(&buf[..len], &tid) {
                        Some(addr) => {
                            let _ =
                                event_tx.send(InternalWebRtcEvent::StunResult(conn_id, Ok(addr)));
                        }
                        None => {
                            let _ = event_tx.send(InternalWebRtcEvent::StunResult(
                                conn_id,
                                Err("Failed to parse STUN response".into()),
                            ));
                        }
                    }
                }
                Err(e) => {
                    let _ = event_tx.send(InternalWebRtcEvent::StunResult(
                        conn_id,
                        Err(format!("STUN recv: {e}")),
                    ));
                }
            }

            // Restore non-blocking mode.
            let _ = socket.set_nonblocking(true);
        });

        if let Ok(mut guard) = self.threads.lock() {
            guard.push(handle);
        }
    }
}
