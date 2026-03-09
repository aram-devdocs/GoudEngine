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

mod audio;
mod collision;
mod ecs_ops;
mod input;
mod rendering;
mod sprite_renderer;
mod texture_loader;

#[cfg(all(test, target_arch = "wasm32"))]
#[cfg(test)]
mod tests;

pub use collision::WasmContact;

use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;

use crate::ecs::World;
use crate::rendering::text::GlyphAtlas;

use sprite_renderer::{TextureEntry, WgpuSpriteRenderer};

// ---------------------------------------------------------------------------
// Transform2D data transfer object
// ---------------------------------------------------------------------------

/// A plain-data snapshot of a `Transform2D` component, safe to pass across
/// the wasm-bindgen boundary.
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
// Sprite data transfer object
// ---------------------------------------------------------------------------

/// A plain-data snapshot of a `Sprite` component, safe to pass across
/// the wasm-bindgen boundary.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmSprite {
    pub texture_handle: u32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub anchor_x: f32,
    pub anchor_y: f32,
}

// ---------------------------------------------------------------------------
// Render statistics data transfer object
// ---------------------------------------------------------------------------

/// Per-frame rendering statistics exposed to JavaScript.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmRenderStats {
    pub draw_calls: u32,
    pub triangles: u32,
    pub texture_binds: u32,
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

struct WasmFontAtlas {
    atlas: GlyphAtlas,
    texture_handle: Option<u32>,
    synced_version: u64,
}

struct WasmFontEntry {
    font: fontdue::Font,
    bytes: Vec<u8>,
    atlases: HashMap<u32, WasmFontAtlas>,
}

// ---------------------------------------------------------------------------
// Main game handle exposed to JavaScript
// ---------------------------------------------------------------------------

/// The primary game handle for browser-based games.
///
/// Owns the ECS world, input state, frame timing, and an optional wgpu
/// rendering backend attached to an HTML canvas element.
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
    mouse_buttons_current: HashSet<u32>,
    mouse_x: f32,
    mouse_y: f32,
    scroll_dx: f32,
    scroll_dy: f32,

    // Event accumulation buffers (filled by JS event handlers between frames)
    keys_pressed_buffer: HashSet<u32>,
    keys_released_buffer: HashSet<u32>,
    mouse_pressed_buffer: HashSet<u32>,
    mouse_released_buffer: HashSet<u32>,

    // Frame snapshots (swapped in at begin_frame, read by queries)
    frame_keys_just_pressed: HashSet<u32>,
    frame_keys_just_released: HashSet<u32>,
    frame_mouse_just_pressed: HashSet<u32>,
    frame_mouse_just_released: HashSet<u32>,

    // Action map: action name → list of bound key codes
    action_map: HashMap<String, Vec<u32>>,

    // Loaded runtime fonts (1-based handles for JS API).
    fonts: Vec<Option<WasmFontEntry>>,

    // Optional wgpu renderer (None for ECS-only / headless mode)
    render_state: Option<WgpuRenderState>,

    // WebAudio runtime state.
    audio_state: audio::WasmAudioState,
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
            mouse_buttons_current: HashSet::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            scroll_dx: 0.0,
            scroll_dy: 0.0,
            keys_pressed_buffer: HashSet::new(),
            keys_released_buffer: HashSet::new(),
            mouse_pressed_buffer: HashSet::new(),
            mouse_released_buffer: HashSet::new(),
            frame_keys_just_pressed: HashSet::new(),
            frame_keys_just_released: HashSet::new(),
            frame_mouse_just_pressed: HashSet::new(),
            frame_mouse_just_released: HashSet::new(),
            action_map: HashMap::new(),
            fonts: Vec::new(),
            render_state: None,
            audio_state: audio::WasmAudioState::new(),
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
            .map_err(|e| JsValue::from_str(&format!("No suitable GPU adapter found: {}", e)))?;

        let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
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
            mouse_buttons_current: HashSet::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            scroll_dx: 0.0,
            scroll_dy: 0.0,
            keys_pressed_buffer: HashSet::new(),
            keys_released_buffer: HashSet::new(),
            mouse_pressed_buffer: HashSet::new(),
            mouse_released_buffer: HashSet::new(),
            frame_keys_just_pressed: HashSet::new(),
            frame_keys_just_released: HashSet::new(),
            frame_mouse_just_pressed: HashSet::new(),
            frame_mouse_just_released: HashSet::new(),
            action_map: HashMap::new(),
            fonts: Vec::new(),
            render_state: Some(render_state),
            audio_state: audio::WasmAudioState::new(),
        })
    }

    // ======================================================================
    // Frame lifecycle
    // ======================================================================

    /// Advances frame timing and prepares the wgpu surface for drawing.
    ///
    /// Call this at the start of each `requestAnimationFrame` callback.
    pub fn begin_frame(&mut self, delta_time: f32) {
        self.frame_keys_just_pressed = std::mem::take(&mut self.keys_pressed_buffer);
        self.frame_keys_just_released = std::mem::take(&mut self.keys_released_buffer);
        self.frame_mouse_just_pressed = std::mem::take(&mut self.mouse_pressed_buffer);
        self.frame_mouse_just_released = std::mem::take(&mut self.mouse_released_buffer);
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

    /// Flushes queued draw calls and presents the frame to the canvas.
    ///
    /// Call this at the end of each `requestAnimationFrame` callback.
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

    /// Sets the background clear color.
    ///
    /// Takes effect on the next `begin_frame` / `end_frame` pair.
    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        if let Some(rs) = &mut self.render_state {
            rs.clear_color = [r as f64, g as f64, b as f64, a as f64];
        }
    }

    // ======================================================================
    // Timing queries
    // ======================================================================

    /// Seconds elapsed since the previous frame.
    #[wasm_bindgen(getter)]
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    /// Total seconds elapsed since the game started.
    #[wasm_bindgen(getter)]
    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    /// Approximate frames per second based on the most recent delta.
    #[wasm_bindgen(getter)]
    pub fn fps(&self) -> f32 {
        if self.delta_time > 0.0 {
            1.0 / self.delta_time
        } else {
            0.0
        }
    }

    /// Number of frames rendered since the game started.
    #[wasm_bindgen(getter)]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// The title string passed at construction time.
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    /// Logical canvas width in pixels.
    #[wasm_bindgen(getter)]
    pub fn window_width(&self) -> u32 {
        self.width
    }

    /// Logical canvas height in pixels.
    #[wasm_bindgen(getter)]
    pub fn window_height(&self) -> u32 {
        self.height
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

    /// Returns `true` when wgpu rendering is active (canvas was provided at
    /// construction time).
    pub fn has_renderer(&self) -> bool {
        self.render_state.is_some()
    }
}

// ---------------------------------------------------------------------------
// Panic hook — routes Rust panics to `console.error`
// ---------------------------------------------------------------------------

fn console_error_panic_hook(info: &std::panic::PanicHookInfo<'_>) {
    let msg = info.to_string();
    web_sys::console::error_1(&JsValue::from_str(&msg));
}
