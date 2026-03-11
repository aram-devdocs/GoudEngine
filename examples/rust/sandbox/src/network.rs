use crate::config::SandboxConfig;
use goud_engine::ffi::context::{
    goud_context_create, goud_context_destroy, GoudContextId, GOUD_INVALID_CONTEXT_ID,
};
use goud_engine::ffi::network::{
    goud_network_connect_with_peer, goud_network_disconnect, goud_network_host,
    goud_network_peer_count, goud_network_poll, goud_network_receive, goud_network_send,
};

const NETWORK_SEND_INTERVAL: f32 = 0.10;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    Host,
    Client,
    Offline,
}

impl Role {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::Client => "client",
            Self::Offline => "offline",
        }
    }
}

#[derive(Clone)]
pub(crate) struct RemoteState {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) mode: String,
}

pub(crate) struct NetworkState {
    context_id: GoudContextId,
    handle: Option<i64>,
    pub(crate) role: Role,
    pub(crate) label: String,
    pub(crate) peer_count: u32,
    default_peer_id: Option<u64>,
    known_peer_id: Option<u64>,
    pub(crate) remote: Option<RemoteState>,
    send_timer: f32,
    packet_version: String,
    port: u16,
    exit_on_peer: bool,
    expect_peer: bool,
}

impl NetworkState {
    pub(crate) fn new(config: &SandboxConfig) -> Self {
        let role_pref = std::env::var("GOUD_SANDBOX_NETWORK_ROLE")
            .unwrap_or_else(|_| "auto".to_string())
            .to_lowercase();
        let port = std::env::var("GOUD_SANDBOX_NETWORK_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(config.network_port);

        let context_id = goud_context_create();
        let mut state = match role_pref.as_str() {
            "host" => Self::connect_host(context_id, port, &config.packet_version)
                .unwrap_or_else(|| Self::offline(context_id, port, &config.packet_version)),
            "client" => Self::connect_client(context_id, port, &config.packet_version)
                .unwrap_or_else(|| Self::offline(context_id, port, &config.packet_version)),
            _ => Self::connect_host(context_id, port, &config.packet_version)
                .or_else(|| Self::connect_client(context_id, port, &config.packet_version))
                .unwrap_or_else(|| Self::offline(context_id, port, &config.packet_version)),
        };

        state.exit_on_peer = env_flag("GOUD_SANDBOX_EXIT_ON_PEER");
        state.expect_peer = env_flag("GOUD_SANDBOX_EXPECT_PEER");
        state
    }

    fn connect_host(context_id: GoudContextId, port: u16, packet_version: &str) -> Option<Self> {
        if context_id == GOUD_INVALID_CONTEXT_ID {
            return None;
        }
        let handle = goud_network_host(context_id, 2, port);
        if handle < 0 {
            return None;
        }
        Some(Self {
            context_id,
            handle: Some(handle),
            role: Role::Host,
            label: "waiting".to_string(),
            peer_count: 0,
            default_peer_id: None,
            known_peer_id: None,
            remote: None,
            send_timer: 0.0,
            packet_version: packet_version.to_string(),
            port,
            exit_on_peer: false,
            expect_peer: false,
        })
    }

    fn connect_client(context_id: GoudContextId, port: u16, packet_version: &str) -> Option<Self> {
        if context_id == GOUD_INVALID_CONTEXT_ID {
            return None;
        }
        let mut handle = -1_i64;
        let mut peer_id = 0_u64;
        let host = b"127.0.0.1";
        let status = unsafe {
            // SAFETY: `host` points to valid UTF-8 bytes for the duration of the call,
            // and both output pointers are valid mutable references to initialized locals.
            goud_network_connect_with_peer(
                context_id,
                2,
                host.as_ptr(),
                i32::try_from(host.len()).ok()?,
                port,
                &mut handle as *mut i64,
                &mut peer_id as *mut u64,
            )
        };
        if status < 0 || handle < 0 {
            return None;
        }
        Some(Self {
            context_id,
            handle: Some(handle),
            role: Role::Client,
            label: "connected".to_string(),
            peer_count: 0,
            default_peer_id: Some(peer_id),
            known_peer_id: None,
            remote: None,
            send_timer: 0.0,
            packet_version: packet_version.to_string(),
            port,
            exit_on_peer: false,
            expect_peer: false,
        })
    }

    fn offline(context_id: GoudContextId, port: u16, packet_version: &str) -> Self {
        Self {
            context_id,
            handle: None,
            role: Role::Offline,
            label: "offline".to_string(),
            peer_count: 0,
            default_peer_id: None,
            known_peer_id: None,
            remote: None,
            send_timer: 0.0,
            packet_version: packet_version.to_string(),
            port,
            exit_on_peer: false,
            expect_peer: false,
        }
    }

    pub(crate) fn update(&mut self, dt: f32, x: f32, y: f32, mode: &str) {
        let Some(handle) = self.handle else {
            return;
        };

        if goud_network_poll(self.context_id, handle) < 0 {
            return;
        }
        let count = goud_network_peer_count(self.context_id, handle);
        self.peer_count = if count > 0 { count as u32 } else { 0 };
        if self.role == Role::Host {
            self.label = if self.peer_count > 0 {
                "connected".to_string()
            } else {
                "waiting".to_string()
            };
        }

        let mut buf = [0_u8; 512];
        loop {
            let mut peer_id = 0_u64;
            let size = unsafe {
                // SAFETY: `buf` is valid writable storage for `buf.len()` bytes and `peer_id`
                // is a valid writable `u64` for the duration of the call.
                goud_network_receive(
                    self.context_id,
                    handle,
                    buf.as_mut_ptr(),
                    buf.len() as i32,
                    &mut peer_id as *mut u64,
                )
            };
            if size <= 0 {
                break;
            }
            if let Some(remote) = Self::parse_packet(&buf[..size as usize], &self.packet_version) {
                self.known_peer_id = Some(peer_id);
                self.peer_count = self.peer_count.max(1);
                self.label = "connected".to_string();
                self.remote = Some(remote);
                self.send_timer = NETWORK_SEND_INTERVAL;
            }
        }

        self.send_timer += dt;
        if self.send_timer < NETWORK_SEND_INTERVAL {
            return;
        }
        self.send_timer = 0.0;
        let peer_id = self.default_peer_id.or(self.known_peer_id);
        if let Some(peer_id) = peer_id {
            let payload = format!(
                "sandbox|{}|{}|{}|{:.1}|{:.1}|{}",
                self.packet_version,
                self.role.as_str(),
                mode,
                x,
                y,
                self.label
            );
            let _ = unsafe {
                // SAFETY: `payload` points to valid initialized bytes for the duration
                // of the call, and the handle/peer_id values originate from engine FFI.
                goud_network_send(
                    self.context_id,
                    handle,
                    peer_id,
                    payload.as_ptr(),
                    payload.len() as i32,
                    0,
                )
            };
        }
    }

    fn parse_packet(payload: &[u8], expected_version: &str) -> Option<RemoteState> {
        let text = std::str::from_utf8(payload).ok()?;
        let parts: Vec<&str> = text.split('|').collect();
        if parts.len() != 7 || parts[0] != "sandbox" || parts[1] != expected_version {
            return None;
        }
        let mode = parts[3].to_string();
        let x = parts[4].parse::<f32>().ok()?;
        let y = parts[5].parse::<f32>().ok()?;
        Some(RemoteState { x, y, mode })
    }

    pub(crate) fn should_exit_on_peer(&self) -> bool {
        self.exit_on_peer && self.remote.is_some()
    }

    pub(crate) fn should_fail_expectation(&self, elapsed: f32, smoke_seconds: f32) -> bool {
        self.expect_peer && smoke_seconds > 0.0 && elapsed >= smoke_seconds && self.peer_count == 0
    }

    pub(crate) fn detail(&self) -> String {
        match self.role {
            Role::Host => format!("host:{} ({})", self.port, self.label),
            Role::Client => format!("client:{} ({})", self.port, self.label),
            Role::Offline => "offline".to_string(),
        }
    }
}

impl Drop for NetworkState {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = goud_network_disconnect(self.context_id, handle);
        }
        if self.context_id != GOUD_INVALID_CONTEXT_ID {
            let _ = goud_context_destroy(self.context_id);
        }
    }
}

fn env_flag(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}
