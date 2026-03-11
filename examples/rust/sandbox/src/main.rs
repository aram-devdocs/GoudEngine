//! GoudEngine Sandbox -- Rust example

use std::fs;
use std::path::Path;

use goud_engine::ffi::context::{
    goud_context_create, goud_context_destroy, GoudContextId, GOUD_INVALID_CONTEXT_ID,
};
use goud_engine::ffi::network::{
    goud_network_connect_with_peer, goud_network_disconnect, goud_network_host,
    goud_network_peer_count, goud_network_poll, goud_network_receive, goud_network_send,
};
use goudengine::{input::Key, GameConfig, GoudGame};
use serde_json::Value;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const MOVE_SPEED: f32 = 220.0;
const NETWORK_SEND_INTERVAL: f32 = 0.10;

#[derive(Clone)]
struct Assets {
    background: String,
    sprite: String,
    accent: String,
    texture3d: String,
    font: String,
}

#[derive(Clone)]
struct HudConfig {
    overview_title: String,
    status_title: String,
    next_steps_title: String,
    tagline: String,
    overview: Vec<String>,
    next_steps: Vec<String>,
}

#[derive(Clone)]
struct SceneEntry {
    key: String,
    mode: String,
    label: String,
}

struct SandboxConfig {
    title: String,
    assets: Assets,
    network_port: u16,
    packet_version: String,
    hud: HudConfig,
    scenes: Vec<SceneEntry>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Host,
    Client,
    Offline,
}

impl Role {
    fn as_str(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::Client => "client",
            Self::Offline => "offline",
        }
    }
}

#[derive(Clone)]
struct RemoteState {
    x: f32,
    y: f32,
    mode: String,
    label: String,
}

struct NetworkState {
    context_id: GoudContextId,
    handle: Option<i64>,
    role: Role,
    label: String,
    peer_count: u32,
    default_peer_id: Option<u64>,
    known_peer_id: Option<u64>,
    remote: Option<RemoteState>,
    send_timer: f32,
    packet_version: String,
    port: u16,
    exit_on_peer: bool,
    expect_peer: bool,
}

impl NetworkState {
    fn new(config: &SandboxConfig) -> Self {
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

    fn update(&mut self, dt: f32, x: f32, y: f32, mode: &str) {
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
            if let Some(remote) =
                Self::parse_packet(&buf[..size as usize], &self.packet_version, self.role, mode)
            {
                self.known_peer_id = Some(peer_id);
                self.peer_count = self.peer_count.max(1);
                self.label = "connected".to_string();
                self.remote = Some(remote);
                // Fast-follow with an outbound sync so the peer process can also
                // observe remote state before smoke exit checks trigger.
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

    fn parse_packet(
        payload: &[u8],
        expected_version: &str,
        local_role: Role,
        local_mode: &str,
    ) -> Option<RemoteState> {
        let text = std::str::from_utf8(payload).ok()?;
        let parts: Vec<&str> = text.split('|').collect();
        if parts.len() != 7 || parts[0] != "sandbox" || parts[1] != expected_version {
            return None;
        }
        let mode = parts[3].to_string();
        let x = parts[4].parse::<f32>().ok()?;
        let y = parts[5].parse::<f32>().ok()?;
        let mut label = parts[6].to_string();
        if local_role == Role::Host && label == "connected" && local_mode == mode {
            label = "connected".to_string();
        }
        Some(RemoteState { x, y, mode, label })
    }

    fn should_exit_on_peer(&self) -> bool {
        self.exit_on_peer && self.remote.is_some()
    }

    fn should_fail_expectation(&self, elapsed: f32, smoke_seconds: f32) -> bool {
        self.expect_peer && smoke_seconds > 0.0 && elapsed >= smoke_seconds && self.peer_count == 0
    }

    fn detail(&self) -> String {
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

fn parse_start_mode(scene_count: usize) -> usize {
    std::env::var("GOUD_SANDBOX_START_MODE")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| (1..=scene_count).contains(value))
        .map(|value| value - 1)
        .unwrap_or(0)
}

fn read_manifest() -> SandboxConfig {
    let path = "examples/shared/sandbox/manifest.json";
    let raw = fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    let root: Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"));
    let assets = root
        .get("assets")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("missing assets in {path}"));
    let network = root
        .get("network")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("missing network in {path}"));
    let hud = root
        .get("hud")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("missing hud in {path}"));
    let scenes = root
        .get("scenes")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("missing scenes in {path}"));

    let scene_entries = scenes
        .iter()
        .map(|entry| SceneEntry {
            key: value_str(entry, "key"),
            mode: value_str(entry, "mode"),
            label: value_str(entry, "label"),
        })
        .collect::<Vec<_>>();

    SandboxConfig {
        title: value_str(&root, "title"),
        assets: Assets {
            background: value_str_obj(assets, "background"),
            sprite: value_str_obj(assets, "sprite"),
            accent: value_str_obj(assets, "accent_sprite"),
            texture3d: value_str_obj(assets, "texture3d"),
            font: value_str_obj(assets, "font"),
        },
        network_port: network
            .get("port")
            .and_then(Value::as_u64)
            .and_then(|v| u16::try_from(v).ok())
            .unwrap_or(38491),
        packet_version: network
            .get("packet_version")
            .and_then(Value::as_str)
            .unwrap_or("v1")
            .to_string(),
        hud: HudConfig {
            overview_title: value_str_obj(hud, "overview_title"),
            status_title: value_str_obj(hud, "status_title"),
            next_steps_title: value_str_obj(hud, "next_steps_title"),
            tagline: value_str_obj(hud, "tagline"),
            overview: value_str_array_obj(hud, "overview"),
            next_steps: value_str_array_obj(hud, "next_steps"),
        },
        scenes: scene_entries,
    }
}

fn value_str(root: &Value, key: &str) -> String {
    root.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing key {key}"))
        .to_string()
}

fn value_str_obj(root: &serde_json::Map<String, Value>, key: &str) -> String {
    root.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing key {key}"))
        .to_string()
}

fn value_str_array_obj(root: &serde_json::Map<String, Value>, key: &str) -> Vec<String> {
    root.get(key)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("missing array key {key}"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("non-string entry in array key {key}"))
                .to_string()
        })
        .collect()
}

fn mode_color(mode: &str) -> (f32, f32, f32, f32) {
    match mode {
        "2D" => (0.20, 0.55, 0.95, 0.85),
        "3D" => (0.62, 0.35, 0.90, 0.85),
        _ => (0.30, 0.72, 0.50, 0.85),
    }
}

fn main() {
    let root = Path::new("examples/shared/sandbox");
    if !root.exists() {
        eprintln!("Run from the repository root: cargo run -p sandbox");
        std::process::exit(1);
    }

    let config = read_manifest();
    let mut network = NetworkState::new(&config);
    let smoke_seconds = std::env::var("GOUD_SANDBOX_SMOKE_SECONDS")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| *value > 0.0)
        .unwrap_or(0.0);

    println!("{}", config.title);
    println!(
        "Controls: 1/2/3 scene switch, WASD/arrows move, Esc quits. Network {}",
        network.detail()
    );

    let mut game = GoudGame::with_platform(GameConfig::new(
        "GoudEngine Sandbox - Rust",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    ))
    .expect("failed to create game");

    let background = game.load(&config.assets.background);
    let sprite = game.load(&config.assets.sprite);
    let accent = game.load(&config.assets.accent);
    let texture3d = game.load(&config.assets.texture3d) as u32;

    let cube = game.create_cube(texture3d, 1.2, 1.2, 1.2);
    let plane = game.create_plane(texture3d, 8.0, 8.0);
    let _ = game.add_light(
        0, 4.0, 6.0, -4.0, 0.0, -1.0, 0.0, 1.0, 0.95, 0.80, 5.0, 28.0, 0.0,
    );
    let _ = game.add_light(
        0, -3.5, 3.5, -2.0, 0.0, -0.65, 0.35, 0.70, 0.85, 1.0, 2.5, 18.0, 0.0,
    );
    let _ = game.add_light(
        0, 0.0, 2.4, 7.0, 0.0, -0.25, -1.0, 0.55, 0.65, 0.90, 1.8, 20.0, 0.0,
    );
    game.enable_blending();
    game.set_object_position(plane, 0.0, -1.0, 0.0);
    game.configure_grid(true, 12.0, 12);

    let mode_order = if config.scenes.len() == 3 {
        config
            .scenes
            .iter()
            .map(|scene| scene.mode.clone())
            .collect::<Vec<_>>()
    } else {
        vec!["2D".to_string(), "3D".to_string(), "Hybrid".to_string()]
    };

    let mut mode_index = parse_start_mode(mode_order.len());
    let mut player_x = 250.0f32;
    let mut player_y = 300.0f32;
    let mut elapsed = 0.0f32;
    let mut last_report_mode = String::new();

    while !game.should_close() {
        let dt = game.poll_events().unwrap_or(0.016);
        elapsed += dt;

        if game.is_key_just_pressed(Key::Escape) {
            break;
        }
        if game.is_key_just_pressed(Key::Num1) {
            mode_index = 0;
        }
        if game.is_key_just_pressed(Key::Num2) {
            mode_index = 1;
        }
        if game.is_key_just_pressed(Key::Num3) {
            mode_index = 2;
        }
        if game.is_key_pressed(Key::A) || game.is_key_pressed(Key::Left) {
            player_x -= MOVE_SPEED * dt;
        }
        if game.is_key_pressed(Key::D) || game.is_key_pressed(Key::Right) {
            player_x += MOVE_SPEED * dt;
        }
        if game.is_key_pressed(Key::W) || game.is_key_pressed(Key::Up) {
            player_y -= MOVE_SPEED * dt;
        }
        if game.is_key_pressed(Key::S) || game.is_key_pressed(Key::Down) {
            player_y += MOVE_SPEED * dt;
        }

        let current_mode = mode_order
            .get(mode_index)
            .map(String::as_str)
            .unwrap_or("2D");
        let is_3d_family_mode = matches!(current_mode, "3D" | "Hybrid");
        let scene_label = config
            .scenes
            .get(mode_index)
            .map(|s| format!("{} {}", s.key, s.label))
            .unwrap_or_else(|| current_mode.to_string());
        let scene_name = config
            .scenes
            .get(mode_index)
            .map(|s| s.label.as_str())
            .unwrap_or(current_mode);
        if current_mode != last_report_mode {
            println!(
                "Mode {} ({}) -- role={} peers={}",
                current_mode,
                scene_label,
                network.role.as_str(),
                network.peer_count
            );
            last_report_mode = current_mode.to_string();
        }

        network.update(dt, player_x, player_y, current_mode);

        if network.should_exit_on_peer() {
            println!("Peer discovered; GOUD_SANDBOX_EXIT_ON_PEER requested early exit.");
            break;
        }
        if network.should_fail_expectation(elapsed, smoke_seconds) {
            eprintln!("Expected peer was not discovered before smoke timeout.");
            std::process::exit(1);
        }

        game.begin_render();
        game.clear(0.07, 0.10, 0.14, 1.0);

        let bob_phase = (elapsed * 2.0).rem_euclid(std::f32::consts::TAU);
        let sprite_rotation = (elapsed * 0.25).rem_euclid(std::f32::consts::TAU);
        let camera_yaw = (elapsed * 15.0).rem_euclid(360.0);
        let cube_yaw = (elapsed * 46.0).rem_euclid(360.0);

        if is_3d_family_mode {
            game.set_camera_position(0.0, 2.2, if current_mode == "3D" { -7.0 } else { -7.8 });
            game.set_camera_rotation(-7.0, camera_yaw, 0.0);
            game.set_object_position(cube, 0.85, 1.2 + 0.26 * bob_phase.sin(), 2.1);
            game.set_object_rotation(cube, 20.0, cube_yaw, 0.0);
            game.set_object_position(plane, 0.0, -1.2, 2.5);
            game.render();
            game.disable_depth_test();
        }

        if current_mode == "2D" {
            game.draw_sprite(
                background,
                WINDOW_WIDTH as f32 / 2.0,
                WINDOW_HEIGHT as f32 / 2.0,
                WINDOW_WIDTH as f32,
                WINDOW_HEIGHT as f32,
                0.0,
                1.0,
                1.0,
                1.0,
                1.0,
            );
            game.draw_sprite(
                sprite,
                player_x,
                player_y,
                64.0,
                64.0,
                sprite_rotation,
                1.0,
                1.0,
                1.0,
                1.0,
            );
            game.draw_sprite(accent, 1040.0, 420.0, 72.0, 240.0, 0.0, 1.0, 1.0, 1.0, 1.0);
            game.draw_quad(920.0, 260.0, 180.0, 40.0, 0.20, 0.55, 0.95, 0.80);
        }

        if current_mode == "Hybrid" {
            game.draw_sprite(
                background,
                WINDOW_WIDTH as f32 / 2.0,
                WINDOW_HEIGHT as f32 / 2.0,
                WINDOW_WIDTH as f32,
                WINDOW_HEIGHT as f32,
                0.0,
                1.0,
                1.0,
                1.0,
                0.26,
            );
            game.draw_quad(640.0, 360.0, 1280.0, 720.0, 0.08, 0.17, 0.24, 0.10);
            game.draw_quad(640.0, 654.0, 1280.0, 132.0, 0.03, 0.10, 0.12, 0.18);
            game.draw_sprite(
                sprite,
                player_x,
                player_y,
                72.0,
                72.0,
                sprite_rotation,
                1.0,
                1.0,
                1.0,
                1.0,
            );
            game.draw_sprite(accent, 1044.0, 420.0, 78.0, 250.0, 0.0, 1.0, 1.0, 1.0, 1.0);
            game.draw_quad(920.0, 260.0, 180.0, 40.0, 0.20, 0.55, 0.95, 0.62);
        }

        if current_mode != "3D" {
            if let Some(remote) = &network.remote {
                game.draw_quad(
                    remote.x,
                    remote.y - 50.0,
                    84.0,
                    18.0,
                    0.96,
                    0.70,
                    0.20,
                    0.92,
                );
                let _ = game.draw_text(
                    &config.assets.font,
                    &format!("Peer {}", remote.mode),
                    remote.x - 32.0,
                    remote.y - 56.0,
                    13.0,
                    0.0,
                    1.0,
                    0.04,
                    0.05,
                    0.08,
                    1.0,
                );
                game.draw_sprite(
                    sprite,
                    remote.x,
                    remote.y,
                    52.0,
                    52.0,
                    -sprite_rotation * 0.72,
                    1.0,
                    1.0,
                    1.0,
                    1.0,
                );
            }
        }

        let panel_alpha = if is_3d_family_mode { 0.62 } else { 0.88 };
        let bottom_alpha = if is_3d_family_mode { 0.70 } else { 0.92 };
        game.draw_quad(332.0, 192.0, 620.0, 318.0, 0.05, 0.08, 0.12, panel_alpha);
        game.draw_quad(1006.0, 192.0, 520.0, 318.0, 0.08, 0.12, 0.18, panel_alpha);
        game.draw_quad(640.0, 620.0, 1168.0, 182.0, 0.05, 0.08, 0.12, bottom_alpha);

        // Scene badge + network pulse badges.
        let (mr, mg, mb, ma) = mode_color(current_mode);
        game.draw_quad(980.0, 312.0, 220.0, 42.0, mr, mg, mb, ma);
        game.draw_quad(
            1040.0,
            210.0,
            28.0 + (network.peer_count as f32 * 10.0),
            16.0,
            0.96,
            0.74,
            0.20,
            0.9,
        );

        let (mx, my) = game.mouse_position();
        game.draw_quad(mx, my, 12.0, 12.0, 0.95, 0.85, 0.20, 0.95);

        let render_texture_size = game.render_capabilities().max_texture_size;
        let render_supports_instancing = game.render_capabilities().supports_instancing;
        let physics_max_bodies = game.physics_capabilities().max_bodies;
        let audio_max_channels = game.audio_capabilities().max_channels;
        let network_cap = game
            .network_capabilities()
            .map(|caps| caps.max_connections.to_string())
            .unwrap_or_else(|| "n/a".to_string());

        let _ = game.draw_text(
            &config.assets.font,
            &config.hud.overview_title,
            60.0,
            52.0,
            30.0,
            0.0,
            1.12,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let _ = game.draw_text(
            &config.assets.font,
            &config.hud.tagline,
            60.0,
            96.0,
            18.0,
            520.0,
            1.12,
            0.94,
            0.97,
            1.0,
            1.0,
        );

        let mut overview_y = 142.0;
        for line in config.hud.overview.iter().take(3) {
            let _ = game.draw_text(
                &config.assets.font,
                line,
                60.0,
                overview_y,
                15.0,
                520.0,
                1.12,
                0.94,
                0.97,
                1.0,
                1.0,
            );
            overview_y += 27.0;
        }

        let _ = game.draw_text(
            &config.assets.font,
            &config.hud.status_title,
            768.0,
            52.0,
            26.0,
            0.0,
            1.12,
            0.95,
            0.90,
            0.40,
            1.0,
        );
        let status_lines = [
            format!("Mode {current_mode}  (1/2/3)"),
            format!("Mouse {:.0}, {:.0}", mx, my),
            format!(
                "Render tex={} inst={}",
                render_texture_size, render_supports_instancing
            ),
            format!(
                "Physics max={}  audio ch={}",
                physics_max_bodies, audio_max_channels
            ),
            format!(
                "Net {}/{} peers={} cap={}",
                network.role.as_str(),
                network.label,
                network.peer_count,
                network_cap
            ),
            format!("Networking: {}", network.detail()),
        ];
        let mut status_y = 90.0;
        for line in &status_lines {
            let _ = game.draw_text(
                &config.assets.font,
                line,
                768.0,
                status_y,
                15.0,
                430.0,
                1.12,
                0.94,
                0.97,
                1.0,
                1.0,
            );
            status_y += 27.0;
        }

        let _ = game.draw_text(
            &config.assets.font,
            &config.hud.next_steps_title,
            94.0,
            526.0,
            26.0,
            0.0,
            1.12,
            0.95,
            0.90,
            0.40,
            1.0,
        );
        let mut next_y = 564.0;
        for line in config.hud.next_steps.iter().take(3) {
            let _ = game.draw_text(
                &config.assets.font,
                line,
                94.0,
                next_y,
                15.0,
                1060.0,
                1.12,
                0.94,
                0.97,
                1.0,
                1.0,
            );
            next_y += 25.0;
        }
        let _ = game.draw_text(
            &config.assets.font,
            &format!("Networking: {}", network.detail()),
            94.0,
            next_y,
            15.0,
            1060.0,
            1.12,
            0.94,
            0.97,
            1.0,
            1.0,
        );
        let _ = game.draw_text(
            &config.assets.font,
            scene_name,
            900.0,
            320.0,
            20.0,
            190.0,
            1.10,
            1.0,
            1.0,
            1.0,
            1.0,
        );

        game.end_render();
        game.swap_buffers().expect("swap buffers");

        if smoke_seconds > 0.0 && elapsed >= smoke_seconds {
            break;
        }
    }
}
