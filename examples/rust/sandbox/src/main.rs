//! GoudEngine Sandbox -- Rust example

use goudengine::{input::Key, GameConfig, GoudGame};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const MOVE_SPEED: f32 = 220.0;

fn main() {
    let root = std::path::Path::new("examples/shared/sandbox");
    if !root.exists() {
        eprintln!("Run from the repository root: cargo run -p sandbox");
        std::process::exit(1);
    }

    println!("GoudEngine Sandbox (Rust)");
    println!("Controls: 1/2/3 switch 2D/3D/Hybrid, WASD/arrows move sprite, Esc quits");

    let mut game = GoudGame::with_platform(GameConfig::new(
        "GoudEngine Sandbox - Rust",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    ))
    .expect("failed to create game");

    let background = game.load("examples/shared/sandbox/sprites/background-day.png");
    let sprite = game.load("examples/shared/sandbox/sprites/yellowbird-midflap.png");
    let accent = game.load("examples/shared/sandbox/sprites/pipe-green.png");
    let texture3d = game.load("examples/shared/sandbox/textures/default_grey.png") as u32;

    let cube = game.create_cube(texture3d, 1.2, 1.2, 1.2);
    let plane = game.create_plane(texture3d, 8.0, 8.0);
    let _light = game.add_light(
        0, 4.0, 6.0, -4.0, 0.0, -1.0, 0.0, 1.0, 0.95, 0.8, 4.0, 25.0, 0.0,
    );
    game.set_object_position(plane, 0.0, -1.0, 0.0);
    game.configure_grid(true, 12.0, 12);

    let scene_2d = game.create_scene("sandbox-2d").ok();
    let scene_3d = game.create_scene("sandbox-3d").ok();
    let scene_hybrid = game.create_scene("sandbox-hybrid").ok();

    let mut mode_index = 0usize;
    let modes = ["2D", "3D", "Hybrid"];
    let mut player_x = 250.0f32;
    let mut player_y = 300.0f32;
    let mut angle = 0.0f32;
    let smoke_seconds = std::env::var("GOUD_SANDBOX_SMOKE_SECONDS")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| *value > 0.0)
        .unwrap_or(0.0);

    while !game.should_close() {
        let dt = game.poll_events().unwrap_or(0.016);
        angle += dt;

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

        match mode_index {
            0 => {
                if let Some(id) = scene_2d {
                    let _ = game.set_scene_active(id, true);
                }
            }
            1 => {
                if let Some(id) = scene_3d {
                    let _ = game.set_scene_active(id, true);
                }
            }
            _ => {
                if let Some(id) = scene_hybrid {
                    let _ = game.set_scene_active(id, true);
                }
            }
        }

        game.begin_render();
        game.clear(0.07, 0.10, 0.14, 1.0);
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
        game.draw_quad(210.0, 110.0, 320.0, 110.0, 0.05, 0.08, 0.12, 0.88);
        game.draw_quad(620.0, 110.0, 560.0, 110.0, 0.08, 0.12, 0.18, 0.88);
        game.draw_quad(620.0, 630.0, 560.0, 120.0, 0.05, 0.08, 0.12, 0.90);

        let (mx, my) = game.mouse_position();
        game.draw_quad(mx, my, 14.0, 14.0, 0.95, 0.85, 0.20, 0.95);

        let current_mode = modes[mode_index];
        if current_mode != "3D" {
            game.draw_quad(920.0, 260.0, 180.0, 40.0, 0.20, 0.55, 0.95, 0.80);
            game.draw_sprite(
                sprite,
                player_x,
                player_y,
                64.0,
                64.0,
                angle * 0.25,
                1.0,
                1.0,
                1.0,
                1.0,
            );
            game.draw_sprite(accent, 1040.0, 420.0, 72.0, 240.0, 0.0, 1.0, 1.0, 1.0, 1.0);
        }

        if current_mode != "2D" {
            game.set_camera_position(0.0, 3.0, -9.5);
            game.set_camera_rotation(-10.0, angle * 20.0, 0.0);
            game.set_object_position(cube, 0.0, 1.0 + 0.35 * (angle * 2.0).sin(), 0.0);
            game.set_object_rotation(cube, 0.0, angle * 55.0, 0.0);
            game.render();
        }

        game.end_render();
        game.swap_buffers().expect("swap buffers");

        if smoke_seconds > 0.0 && angle >= smoke_seconds {
            break;
        }
    }
}
