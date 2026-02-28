//! WebAssembly bindings for browser-based game development.
//!
//! This module exposes the engine's ECS, input, and timing through
//! `wasm_bindgen` for consumption by the TypeScript web SDK. The game
//! loop runs in JavaScript via `requestAnimationFrame`; each tick calls
//! [`WasmGame::begin_frame`] and [`WasmGame::end_frame`].
//!
//! # Architecture
//!
//! ```text
//! Browser JS  ──▶  wasm_bindgen exports  ──▶  ECS World
//!   (input,          (this module)            (entities,
//!    rAF loop,                                 components,
//!    Web Audio)                                 timing)
//! ```

use std::collections::HashSet;
use wasm_bindgen::prelude::*;

use crate::core::math::Vec2;
use crate::ecs::components::{Name, Transform2D};
use crate::ecs::{Entity, World};

// ---------------------------------------------------------------------------
// Transform2D data transfer object
// ---------------------------------------------------------------------------

/// Flat representation of [`Transform2D`] for transfer across the wasm boundary.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmTransform2D {
    /// X position in world units.
    pub position_x: f32,
    /// Y position in world units.
    pub position_y: f32,
    /// Rotation in radians.
    pub rotation: f32,
    /// Horizontal scale factor.
    pub scale_x: f32,
    /// Vertical scale factor.
    pub scale_y: f32,
}

// ---------------------------------------------------------------------------
// Main game handle exposed to JavaScript
// ---------------------------------------------------------------------------

/// The primary wasm entry-point wrapping the ECS [`World`] and frame state.
///
/// Created once by the TypeScript SDK's `init()`, then driven each frame by
/// the `requestAnimationFrame` loop on the JS side.
#[wasm_bindgen]
pub struct WasmGame {
    world: World,

    // Frame timing
    delta_time: f32,
    total_time: f32,
    frame_count: u64,

    // Window dimensions (logical, set from JS canvas size)
    width: u32,
    height: u32,
    title: String,

    // Input state — updated from JS event handlers via setter methods.
    // Uses raw integer key codes matching `KeyboardEvent.keyCode`.
    keys_current: HashSet<u32>,
    keys_previous: HashSet<u32>,
    mouse_buttons_current: HashSet<u32>,
    mouse_buttons_previous: HashSet<u32>,
    mouse_x: f32,
    mouse_y: f32,
    scroll_dx: f32,
    scroll_dy: f32,
}

#[wasm_bindgen]
impl WasmGame {
    // ======================================================================
    // Construction
    // ======================================================================

    /// Creates a new game instance.
    ///
    /// Call this once from the TypeScript SDK's `init()` function.
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        #[cfg(feature = "web")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook));
        }

        Self {
            world: World::new(),
            delta_time: 0.0,
            total_time: 0.0,
            frame_count: 0,
            width,
            height,
            title: title.to_string(),
            keys_current: HashSet::new(),
            keys_previous: HashSet::new(),
            mouse_buttons_current: HashSet::new(),
            mouse_buttons_previous: HashSet::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            scroll_dx: 0.0,
            scroll_dy: 0.0,
        }
    }

    // ======================================================================
    // Frame lifecycle
    // ======================================================================

    /// Advances timing state. Call at the start of each `requestAnimationFrame`.
    ///
    /// `delta_time` is seconds since the previous frame (typically computed
    /// from `performance.now()` in the JS layer).
    pub fn begin_frame(&mut self, delta_time: f32) {
        self.keys_previous = self.keys_current.clone();
        self.mouse_buttons_previous = self.mouse_buttons_current.clone();
        self.scroll_dx = 0.0;
        self.scroll_dy = 0.0;

        self.delta_time = delta_time;
        self.total_time += delta_time;
        self.frame_count += 1;
    }

    /// Marks the end of the current frame. Reserved for future use (e.g.
    /// flushing deferred render commands).
    pub fn end_frame(&mut self) {
        // Placeholder — wgpu present calls will go here once rendering
        // is wired through the wasm surface.
    }

    // ======================================================================
    // Timing queries
    // ======================================================================

    /// Seconds elapsed since the last frame.
    #[wasm_bindgen(getter)]
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    /// Total seconds since [`WasmGame::new`].
    #[wasm_bindgen(getter)]
    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    /// Instantaneous frames-per-second derived from the last delta.
    #[wasm_bindgen(getter)]
    pub fn fps(&self) -> f32 {
        if self.delta_time > 0.0 {
            1.0 / self.delta_time
        } else {
            0.0
        }
    }

    /// Number of frames processed so far.
    #[wasm_bindgen(getter)]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Configured window title.
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    /// Logical canvas width.
    #[wasm_bindgen(getter)]
    pub fn window_width(&self) -> u32 {
        self.width
    }

    /// Logical canvas height.
    #[wasm_bindgen(getter)]
    pub fn window_height(&self) -> u32 {
        self.height
    }

    /// Updates the stored canvas dimensions (call on browser resize).
    pub fn set_canvas_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    // ======================================================================
    // Entity operations
    // ======================================================================

    /// Spawns an empty entity and returns its packed `u64` id.
    pub fn spawn_empty(&mut self) -> u64 {
        self.world.spawn_empty().to_bits()
    }

    /// Spawns `count` empty entities and returns their ids as a `Uint32Array`-
    /// compatible flat buffer (pairs of `[lo, hi]` words).
    pub fn spawn_batch(&mut self, count: u32) -> Vec<u64> {
        self.world
            .spawn_batch(count as usize)
            .into_iter()
            .map(|e| e.to_bits())
            .collect()
    }

    /// Removes an entity and all its components. Returns `true` on success.
    pub fn despawn(&mut self, entity_bits: u64) -> bool {
        self.world.despawn(Entity::from_bits(entity_bits))
    }

    /// Total number of live entities.
    pub fn entity_count(&self) -> u32 {
        self.world.entity_count() as u32
    }

    /// Whether the entity is still alive.
    pub fn is_alive(&self, entity_bits: u64) -> bool {
        self.world.is_alive(Entity::from_bits(entity_bits))
    }

    // ======================================================================
    // Transform2D component
    // ======================================================================

    /// Attaches a [`Transform2D`] to `entity`.
    pub fn add_transform2d(
        &mut self,
        entity_bits: u64,
        px: f32,
        py: f32,
        rotation: f32,
        sx: f32,
        sy: f32,
    ) {
        let entity = Entity::from_bits(entity_bits);
        self.world.insert(
            entity,
            Transform2D {
                position: Vec2::new(px, py),
                rotation,
                scale: Vec2::new(sx, sy),
            },
        );
    }

    /// Returns the entity's [`Transform2D`], or `undefined` if absent.
    pub fn get_transform2d(&self, entity_bits: u64) -> Option<WasmTransform2D> {
        let entity = Entity::from_bits(entity_bits);
        self.world
            .get::<Transform2D>(entity)
            .map(|t| WasmTransform2D {
                position_x: t.position.x,
                position_y: t.position.y,
                rotation: t.rotation,
                scale_x: t.scale.x,
                scale_y: t.scale.y,
            })
    }

    /// Overwrites the entity's [`Transform2D`].
    pub fn set_transform2d(
        &mut self,
        entity_bits: u64,
        px: f32,
        py: f32,
        rotation: f32,
        sx: f32,
        sy: f32,
    ) {
        let entity = Entity::from_bits(entity_bits);
        if let Some(t) = self.world.get_mut::<Transform2D>(entity) {
            t.position = Vec2::new(px, py);
            t.rotation = rotation;
            t.scale = Vec2::new(sx, sy);
        }
    }

    /// Whether `entity` has a [`Transform2D`].
    pub fn has_transform2d(&self, entity_bits: u64) -> bool {
        self.world
            .has::<Transform2D>(Entity::from_bits(entity_bits))
    }

    /// Removes the [`Transform2D`] from `entity`. Returns `true` if it was present.
    pub fn remove_transform2d(&mut self, entity_bits: u64) -> bool {
        self.world
            .remove::<Transform2D>(Entity::from_bits(entity_bits))
            .is_some()
    }

    // ======================================================================
    // Name component
    // ======================================================================

    /// Attaches a [`Name`] component.
    pub fn add_name(&mut self, entity_bits: u64, name: &str) {
        let entity = Entity::from_bits(entity_bits);
        self.world.insert(entity, Name::new(name));
    }

    /// Returns the entity's [`Name`], or `undefined` if absent.
    pub fn get_name(&self, entity_bits: u64) -> Option<String> {
        let entity = Entity::from_bits(entity_bits);
        self.world
            .get::<Name>(entity)
            .map(|n| n.as_str().to_string())
    }

    /// Whether `entity` has a [`Name`].
    pub fn has_name(&self, entity_bits: u64) -> bool {
        self.world.has::<Name>(Entity::from_bits(entity_bits))
    }

    /// Removes the [`Name`] from `entity`.
    pub fn remove_name(&mut self, entity_bits: u64) -> bool {
        self.world
            .remove::<Name>(Entity::from_bits(entity_bits))
            .is_some()
    }

    // ======================================================================
    // Input — setters (called from JS event handlers)
    // ======================================================================

    /// Records a key press. `key_code` matches `KeyboardEvent.keyCode`.
    pub fn press_key(&mut self, key_code: u32) {
        self.keys_current.insert(key_code);
    }

    /// Records a key release.
    pub fn release_key(&mut self, key_code: u32) {
        self.keys_current.remove(&key_code);
    }

    /// Records a mouse button press (0 = left, 1 = middle, 2 = right).
    pub fn press_mouse_button(&mut self, button: u32) {
        self.mouse_buttons_current.insert(button);
    }

    /// Records a mouse button release.
    pub fn release_mouse_button(&mut self, button: u32) {
        self.mouse_buttons_current.remove(&button);
    }

    /// Updates the stored mouse position (in CSS pixels relative to canvas).
    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    /// Accumulates scroll delta for the current frame.
    pub fn add_scroll_delta(&mut self, dx: f32, dy: f32) {
        self.scroll_dx += dx;
        self.scroll_dy += dy;
    }

    // ======================================================================
    // Input — queries (called from game logic)
    // ======================================================================

    /// Whether the key is held down right now.
    pub fn is_key_pressed(&self, key_code: u32) -> bool {
        self.keys_current.contains(&key_code)
    }

    /// Whether the key transitioned to pressed this frame.
    pub fn is_key_just_pressed(&self, key_code: u32) -> bool {
        self.keys_current.contains(&key_code) && !self.keys_previous.contains(&key_code)
    }

    /// Whether the key transitioned to released this frame.
    pub fn is_key_just_released(&self, key_code: u32) -> bool {
        !self.keys_current.contains(&key_code) && self.keys_previous.contains(&key_code)
    }

    /// Whether a mouse button is held down.
    pub fn is_mouse_button_pressed(&self, button: u32) -> bool {
        self.mouse_buttons_current.contains(&button)
    }

    /// Whether a mouse button was just pressed this frame.
    pub fn is_mouse_button_just_pressed(&self, button: u32) -> bool {
        self.mouse_buttons_current.contains(&button)
            && !self.mouse_buttons_previous.contains(&button)
    }

    /// Mouse X in CSS pixels relative to the canvas.
    pub fn mouse_x(&self) -> f32 {
        self.mouse_x
    }

    /// Mouse Y in CSS pixels relative to the canvas.
    pub fn mouse_y(&self) -> f32 {
        self.mouse_y
    }

    /// Horizontal scroll delta accumulated this frame.
    pub fn scroll_dx(&self) -> f32 {
        self.scroll_dx
    }

    /// Vertical scroll delta accumulated this frame.
    pub fn scroll_dy(&self) -> f32 {
        self.scroll_dy
    }
}

// ---------------------------------------------------------------------------
// Panic hook — routes Rust panics to `console.error`
// ---------------------------------------------------------------------------

fn console_error_panic_hook(info: &std::panic::PanicHookInfo<'_>) {
    let msg = info.to_string();
    web_sys::console::error_1(&JsValue::from_str(&msg));
}

// ---------------------------------------------------------------------------
// Tests — wasm_bindgen types only work on wasm32; use `wasm-pack test` to run.
// ---------------------------------------------------------------------------

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
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
        assert!(game.get_transform2d(bits).is_none());

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
        game.press_key(32); // Space
        assert!(game.is_key_pressed(32));
        assert!(!game.is_key_just_pressed(32)); // just_pressed needs frame boundary

        game.begin_frame(0.016);
        // Space was pressed last frame and this frame → not just_pressed
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
}
