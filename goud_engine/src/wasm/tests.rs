//! Tests for the `wasm` module.
//!
//! All tests are gated with `#[cfg(all(test, target_arch = "wasm32"))]` and
//! require a browser environment for the rendering-dependent cases.

use super::*;

#[test]
fn wasm_game_creation() {
    let game = WasmGame::new(800, 600, "Test");
    assert_eq!(game.window_width(), 800);
    assert_eq!(game.window_height(), 600);
    assert_eq!(game.entity_count(), 0);
    assert_eq!(game.frame_count(), 0);
}

#[test]
fn entity_lifecycle() {
    let mut game = WasmGame::new(800, 600, "Test");
    let bits = game.spawn_empty();
    assert!(game.is_alive(bits));
    assert_eq!(game.entity_count(), 1);
    assert!(game.despawn(bits));
    assert!(!game.is_alive(bits));
    assert_eq!(game.entity_count(), 0);
}

#[test]
fn spawn_batch_returns_correct_count() {
    let mut game = WasmGame::new(800, 600, "Test");
    let entities = game.spawn_batch(10);
    assert_eq!(entities.len(), 10);
    assert_eq!(game.entity_count(), 10);
}

#[test]
fn transform2d_crud() {
    let mut game = WasmGame::new(800, 600, "Test");
    let bits = game.spawn_empty();
    assert!(!game.has_transform2d(bits));
    game.add_transform2d(bits, 10.0, 20.0, 0.5, 1.0, 1.0);
    assert!(game.has_transform2d(bits));
    let t = game.get_transform2d(bits).unwrap();
    assert!((t.position_x - 10.0).abs() < f32::EPSILON);
    assert!((t.position_y - 20.0).abs() < f32::EPSILON);
    assert!((t.rotation - 0.5).abs() < f32::EPSILON);
    game.set_transform2d(bits, 30.0, 40.0, 1.0, 2.0, 2.0);
    let t = game.get_transform2d(bits).unwrap();
    assert!((t.position_x - 30.0).abs() < f32::EPSILON);
    assert!(game.remove_transform2d(bits));
    assert!(!game.has_transform2d(bits));
}

#[test]
fn name_crud() {
    let mut game = WasmGame::new(800, 600, "Test");
    let bits = game.spawn_empty();
    assert!(!game.has_name(bits));
    game.add_name(bits, "Player");
    assert!(game.has_name(bits));
    assert_eq!(game.get_name(bits).unwrap(), "Player");
    assert!(game.remove_name(bits));
    assert!(!game.has_name(bits));
}

#[test]
fn frame_timing() {
    let mut game = WasmGame::new(800, 600, "Test");
    game.begin_frame(0.016);
    assert!((game.delta_time() - 0.016).abs() < 0.001);
    assert_eq!(game.frame_count(), 1);
    game.begin_frame(0.016);
    assert!((game.total_time() - 0.032).abs() < 0.001);
    assert_eq!(game.frame_count(), 2);
}

#[test]
fn input_key_state() {
    let mut game = WasmGame::new(800, 600, "Test");
    game.begin_frame(0.016);
    game.press_key(32);
    assert!(game.is_key_pressed(32));
    assert!(!game.is_key_just_pressed(32));
    game.begin_frame(0.016);
    assert!(game.is_key_pressed(32));
    assert!(!game.is_key_just_pressed(32));
    game.release_key(32);
    game.begin_frame(0.016);
    assert!(!game.is_key_pressed(32));
    assert!(game.is_key_just_released(32));
}

#[test]
fn mouse_state() {
    let mut game = WasmGame::new(800, 600, "Test");
    game.set_mouse_position(100.0, 200.0);
    assert!((game.mouse_x() - 100.0).abs() < f32::EPSILON);
    assert!((game.mouse_y() - 200.0).abs() < f32::EPSILON);
    game.press_mouse_button(0);
    assert!(game.is_mouse_button_pressed(0));
}

#[test]
fn scroll_delta_resets_per_frame() {
    let mut game = WasmGame::new(800, 600, "Test");
    game.add_scroll_delta(0.0, 3.0);
    assert!((game.scroll_dy() - 3.0).abs() < f32::EPSILON);
    game.begin_frame(0.016);
    assert!((game.scroll_dy()).abs() < f32::EPSILON);
}

#[test]
fn canvas_resize() {
    let mut game = WasmGame::new(800, 600, "Test");
    game.set_canvas_size(1920, 1080);
    assert_eq!(game.window_width(), 1920);
    assert_eq!(game.window_height(), 1080);
}
