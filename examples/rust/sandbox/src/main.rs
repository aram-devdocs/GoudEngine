mod config;
mod hud;
mod network;

use std::path::Path;

use config::{parse_start_mode, read_manifest};
use goudengine::{input::Key, DebuggerConfig, GameConfig, GoudGame};
use hud::{draw as draw_hud, HudSnapshot};
use network::NetworkState;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const MOVE_SPEED: f32 = 220.0;

fn setup_3d_once(game: &mut GoudGame, texture_path: &str) -> Option<(u32, u32)> {
    let texture3d = game.load(texture_path) as u32;
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
    game.set_object_position(plane, 0.0, -1.0, 0.0);
    game.configure_grid(true, 12.0, 12);

    if cube != 0 && plane != 0 {
        Some((cube, plane))
    } else {
        None
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

    let mut game = GoudGame::with_platform(
        GameConfig::new("GoudEngine Sandbox - Rust", WINDOW_WIDTH, WINDOW_HEIGHT).with_debugger(
            DebuggerConfig {
                enabled: true,
                publish_local_attach: true,
                route_label: Some("sandbox-rust".to_string()),
            },
        ),
    )
    .expect("failed to create game");

    let background = game.load(&config.assets.background);
    let sprite = game.load(&config.assets.sprite);
    let accent = game.load(&config.assets.accent);
    let mut scene3d: Option<(u32, u32)> = None;
    let mut has_3d_setup = false;
    game.enable_blending();

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
    let mut audio_activated = false;

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
        if game.is_key_just_pressed(Key::Space) {
            audio_activated = true;
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
        let cube_yaw = (elapsed * 46.0).rem_euclid(360.0);

        if is_3d_family_mode && !has_3d_setup {
            scene3d = setup_3d_once(&mut game, &config.assets.texture3d);
            has_3d_setup = scene3d.is_some();
        }

        if is_3d_family_mode && has_3d_setup {
            let (cube, plane) = scene3d.expect("3D setup promised handles");
            game.enable_depth_test();
            game.set_camera_position(0.0, 2.2, if current_mode == "3D" { -7.0 } else { -7.8 });
            game.set_camera_rotation(-7.0, if current_mode == "3D" { 0.0 } else { 8.0 }, 0.0);
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

        let (mx, my) = game.mouse_position();
        let render_caps = game.render_capabilities();
        let physics_caps = game.physics_capabilities();
        let audio_caps = game.audio_capabilities();
        let hud_snapshot = HudSnapshot {
            current_mode: current_mode.to_string(),
            mode_index,
            mouse_x: mx,
            mouse_y: my,
            render_texture_size: render_caps.max_texture_size,
            render_supports_instancing: render_caps.supports_instancing,
            physics_supports_joints: physics_caps.supports_joints,
            physics_max_bodies: physics_caps.max_bodies,
            audio_supports_spatial: audio_caps.supports_spatial,
            audio_max_channels: audio_caps.max_channels,
            audio_activated,
        };
        draw_hud(
            &mut game,
            &config.assets.font,
            &config,
            &network,
            &hud_snapshot,
        );
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

        game.end_render();
        game.swap_buffers().expect("swap buffers");

        if smoke_seconds > 0.0 && elapsed >= smoke_seconds {
            break;
        }
    }
}
