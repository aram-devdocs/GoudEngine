mod config;
mod network;

use std::path::Path;

use config::{mode_color, parse_start_mode, read_manifest};
use goudengine::{input::Key, GameConfig, GoudGame};
use network::NetworkState;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const MOVE_SPEED: f32 = 220.0;

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
