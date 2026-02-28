//! WebAssembly bindings for browser-based game development.
//!
//! This module exposes the engine's ECS, input, rendering, and timing
//! through `wasm_bindgen` for consumption by the TypeScript web SDK.
//!
//! # Architecture
//!
//! ```text
//! Browser JS  ──▶  wasm_bindgen exports  ──▶  ECS World + wgpu renderer
//!   (input,          (this module)            (entities, components,
//!    rAF loop,                                 sprites, textures)
//!    canvas)
//! ```

mod sprite_renderer;
mod texture_loader;

use std::collections::HashSet;
use wasm_bindgen::prelude::*;

use crate::core::math::Vec2;
use crate::ecs::components::{Name, Transform2D};
use crate::ecs::{Entity, World};

use sprite_renderer::{TextureEntry, WgpuSpriteRenderer};

// ---------------------------------------------------------------------------
// Transform2D data transfer object
// ---------------------------------------------------------------------------

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmTransform2D {
    pub position_x: f32,
    pub position_y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

// ---------------------------------------------------------------------------
// wgpu rendering state (owned by WasmGame when canvas is provided)
// ---------------------------------------------------------------------------

struct WgpuRenderState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    renderer: WgpuSpriteRenderer,
    textures: Vec<Option<TextureEntry>>,
    clear_color: [f64; 4],
    current_frame: Option<wgpu::SurfaceTexture>,
    current_view: Option<wgpu::TextureView>,
}

// ---------------------------------------------------------------------------
// Main game handle exposed to JavaScript
// ---------------------------------------------------------------------------

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

    // Input state
    keys_current: HashSet<u32>,
    keys_previous: HashSet<u32>,
    mouse_buttons_current: HashSet<u32>,
    mouse_buttons_previous: HashSet<u32>,
    mouse_x: f32,
    mouse_y: f32,
    scroll_dx: f32,
    scroll_dy: f32,

    // Optional wgpu renderer (None for ECS-only / headless mode)
    render_state: Option<WgpuRenderState>,
}

#[wasm_bindgen]
impl WasmGame {
    // ======================================================================
    // Construction
    // ======================================================================

    /// Creates a new game instance without rendering (ECS-only mode).
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
            render_state: None,
        }
    }

    /// Creates a game instance with wgpu rendering attached to a canvas.
    #[wasm_bindgen(js_name = "createWithCanvas")]
    pub async fn create_with_canvas(
        canvas: web_sys::HtmlCanvasElement,
        width: u32,
        height: u32,
        title: &str,
    ) -> Result<WasmGame, JsValue> {
        #[cfg(feature = "web")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook));
        }

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| JsValue::from_str(&format!("Surface creation failed: {}", e)))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .ok_or_else(|| JsValue::from_str("No suitable GPU adapter found"))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(|e| JsValue::from_str(&format!("Device request failed: {}", e)))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .first()
            .copied()
            .ok_or_else(|| JsValue::from_str("No surface format available"))?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let renderer = WgpuSpriteRenderer::new(&device, &queue, format);

        let render_state = WgpuRenderState {
            device,
            queue,
            surface,
            surface_config: config,
            renderer,
            textures: Vec::new(),
            clear_color: [0.0, 0.0, 0.0, 1.0],
            current_frame: None,
            current_view: None,
        };

        Ok(Self {
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
            render_state: Some(render_state),
        })
    }

    // ======================================================================
    // Frame lifecycle
    // ======================================================================

    pub fn begin_frame(&mut self, delta_time: f32) {
        self.keys_previous = self.keys_current.clone();
        self.mouse_buttons_previous = self.mouse_buttons_current.clone();
        self.scroll_dx = 0.0;
        self.scroll_dy = 0.0;
        self.delta_time = delta_time;
        self.total_time += delta_time;
        self.frame_count += 1;

        if let Some(rs) = &mut self.render_state {
            rs.renderer.begin_frame();
            if let Ok(frame) = rs.surface.get_current_texture() {
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                rs.current_view = Some(view);
                rs.current_frame = Some(frame);
            }
        }
    }

    pub fn end_frame(&mut self) {
        if let Some(rs) = &mut self.render_state {
            if let Some(view) = rs.current_view.take() {
                rs.renderer.flush(
                    &rs.device,
                    &rs.queue,
                    &view,
                    &rs.textures,
                    self.width,
                    self.height,
                    rs.clear_color,
                );
            }
            if let Some(frame) = rs.current_frame.take() {
                frame.present();
            }
        }
    }

    /// Sets the background clear color (called before begin_frame or via
    /// the TypeScript SDK's beginFrame parameters).
    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        if let Some(rs) = &mut self.render_state {
            rs.clear_color = [r as f64, g as f64, b as f64, a as f64];
        }
    }

    // ======================================================================
    // Rendering
    // ======================================================================

    pub fn draw_sprite(
        &mut self,
        texture: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rotation: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        if let Some(rs) = &mut self.render_state {
            rs.renderer
                .draw_sprite(texture, x, y, w, h, rotation, r, g, b, a);
        }
    }

    pub fn draw_quad(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, g: f32, b: f32, a: f32) {
        if let Some(rs) = &mut self.render_state {
            rs.renderer.draw_quad(x, y, w, h, r, g, b, a);
        }
    }

    /// Loads a texture from a URL. Returns the texture handle (1-based;
    /// 0 is reserved for the white fallback texture used by draw_quad).
    pub async fn load_texture(&mut self, url: String) -> Result<u32, JsValue> {
        let rs = self
            .render_state
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Rendering not initialized"))?;

        let entry = texture_loader::load_texture_from_url(
            &rs.device,
            &rs.queue,
            &rs.renderer.texture_bind_group_layout,
            &rs.renderer.sampler,
            &url,
        )
        .await?;

        let idx = rs.textures.len();
        rs.textures.push(Some(entry));
        Ok((idx + 1) as u32)
    }

    pub fn destroy_texture(&mut self, handle: u32) {
        if handle == 0 {
            return;
        }
        if let Some(rs) = &mut self.render_state {
            let idx = (handle - 1) as usize;
            if idx < rs.textures.len() {
                rs.textures[idx] = None;
            }
        }
    }

    /// Reconfigures the wgpu surface after a canvas resize.
    pub fn set_canvas_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        if let Some(rs) = &mut self.render_state {
            rs.surface_config.width = width.max(1);
            rs.surface_config.height = height.max(1);
            rs.surface.configure(&rs.device, &rs.surface_config);
        }
    }

    /// Whether wgpu rendering is active.
    pub fn has_renderer(&self) -> bool {
        self.render_state.is_some()
    }

    // ======================================================================
    // Timing queries
    // ======================================================================

    #[wasm_bindgen(getter)]
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    #[wasm_bindgen(getter)]
    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    #[wasm_bindgen(getter)]
    pub fn fps(&self) -> f32 {
        if self.delta_time > 0.0 {
            1.0 / self.delta_time
        } else {
            0.0
        }
    }

    #[wasm_bindgen(getter)]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn window_width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn window_height(&self) -> u32 {
        self.height
    }

    // ======================================================================
    // Entity operations
    // ======================================================================

    pub fn spawn_empty(&mut self) -> u64 {
        self.world.spawn_empty().to_bits()
    }

    pub fn spawn_batch(&mut self, count: u32) -> Vec<u64> {
        self.world
            .spawn_batch(count as usize)
            .into_iter()
            .map(|e| e.to_bits())
            .collect()
    }

    pub fn despawn(&mut self, entity_bits: u64) -> bool {
        self.world.despawn(Entity::from_bits(entity_bits))
    }

    pub fn entity_count(&self) -> u32 {
        self.world.entity_count() as u32
    }

    pub fn is_alive(&self, entity_bits: u64) -> bool {
        self.world.is_alive(Entity::from_bits(entity_bits))
    }

    // ======================================================================
    // Transform2D component
    // ======================================================================

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

    pub fn has_transform2d(&self, entity_bits: u64) -> bool {
        self.world
            .has::<Transform2D>(Entity::from_bits(entity_bits))
    }

    pub fn remove_transform2d(&mut self, entity_bits: u64) -> bool {
        self.world
            .remove::<Transform2D>(Entity::from_bits(entity_bits))
            .is_some()
    }

    // ======================================================================
    // Name component
    // ======================================================================

    pub fn add_name(&mut self, entity_bits: u64, name: &str) {
        let entity = Entity::from_bits(entity_bits);
        self.world.insert(entity, Name::new(name));
    }

    pub fn get_name(&self, entity_bits: u64) -> Option<String> {
        let entity = Entity::from_bits(entity_bits);
        self.world
            .get::<Name>(entity)
            .map(|n| n.as_str().to_string())
    }

    pub fn has_name(&self, entity_bits: u64) -> bool {
        self.world.has::<Name>(Entity::from_bits(entity_bits))
    }

    pub fn remove_name(&mut self, entity_bits: u64) -> bool {
        self.world
            .remove::<Name>(Entity::from_bits(entity_bits))
            .is_some()
    }

    // ======================================================================
    // Input — setters (called from JS event handlers)
    // ======================================================================

    pub fn press_key(&mut self, key_code: u32) {
        self.keys_current.insert(key_code);
    }

    pub fn release_key(&mut self, key_code: u32) {
        self.keys_current.remove(&key_code);
    }

    pub fn press_mouse_button(&mut self, button: u32) {
        self.mouse_buttons_current.insert(button);
    }

    pub fn release_mouse_button(&mut self, button: u32) {
        self.mouse_buttons_current.remove(&button);
    }

    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn add_scroll_delta(&mut self, dx: f32, dy: f32) {
        self.scroll_dx += dx;
        self.scroll_dy += dy;
    }

    // ======================================================================
    // Input — queries (called from game logic)
    // ======================================================================

    pub fn is_key_pressed(&self, key_code: u32) -> bool {
        self.keys_current.contains(&key_code)
    }

    pub fn is_key_just_pressed(&self, key_code: u32) -> bool {
        self.keys_current.contains(&key_code) && !self.keys_previous.contains(&key_code)
    }

    pub fn is_key_just_released(&self, key_code: u32) -> bool {
        !self.keys_current.contains(&key_code) && self.keys_previous.contains(&key_code)
    }

    pub fn is_mouse_button_pressed(&self, button: u32) -> bool {
        self.mouse_buttons_current.contains(&button)
    }

    pub fn is_mouse_button_just_pressed(&self, button: u32) -> bool {
        self.mouse_buttons_current.contains(&button)
            && !self.mouse_buttons_previous.contains(&button)
    }

    pub fn mouse_x(&self) -> f32 {
        self.mouse_x
    }

    pub fn mouse_y(&self) -> f32 {
        self.mouse_y
    }

    pub fn scroll_dx(&self) -> f32 {
        self.scroll_dx
    }

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
// Tests
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
}
