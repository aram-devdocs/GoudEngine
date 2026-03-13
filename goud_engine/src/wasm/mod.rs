//! WebAssembly bindings for browser-based game development.
//!
//! This module exposes the engine's ECS, input, rendering, and timing
//! through `wasm_bindgen` for consumption by the TypeScript web SDK.
//!
//! # Architecture
//!
//! ```text
//! Browser JS  -->  wasm_bindgen exports  -->  ECS World + wgpu renderer
//!   (input,          (this module)            (entities, components,
//!    rAF loop,                                 sprites, textures)
//!    canvas)
//! ```

mod audio;
mod collision;
mod ecs_ops;
mod input;
mod lifecycle;
mod network;
mod rendering;
mod sprite_renderer;
mod texture_loader;
mod types;
mod ui;

#[cfg(all(test, target_arch = "wasm32"))]
#[cfg(test)]
mod tests;

pub use collision::WasmContact;
pub use texture_loader::{fetch_bytes, load_texture_from_url};
pub use types::{WasmRenderStats, WasmSprite, WasmTransform2D};
pub use ui::{WasmUiEvent, WasmUiManager};

use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;

use crate::core::debugger::{self, DebuggerConfig, RuntimeRouteId, RuntimeSurfaceKind};
use crate::ecs::World;

use lifecycle::{WasmFontEntry, WgpuRenderState};

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

    // Action map: action name -> list of bound key codes
    action_map: HashMap<String, Vec<u32>>,

    // Loaded runtime fonts (1-based handles for JS API).
    fonts: Vec<Option<WasmFontEntry>>,

    // Optional wgpu renderer (None for ECS-only / headless mode)
    render_state: Option<WgpuRenderState>,

    // WebAudio runtime state.
    audio_state: audio::WasmAudioState,

    // Browser WebSocket networking runtime state.
    network_state: network::WasmNetworkState,

    /// Route registered with the debugger runtime, if enabled.
    debugger_route: Option<RuntimeRouteId>,
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
            network_state: network::WasmNetworkState::new(),
            debugger_route: None,
        }
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

        if let Some(ref route_id) = self.debugger_route {
            debugger::begin_frame(
                route_id,
                self.frame_count,
                self.delta_time,
                self.total_time as f64,
            );
        }

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
        if let Some(ref route_id) = self.debugger_route {
            debugger::end_frame(route_id);
        }

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

    // ======================================================================
    // Debugger support
    // ======================================================================

    /// Initialize the debugger route for this game instance.
    /// Call this once after construction to enable debugger support.
    #[wasm_bindgen(js_name = "initDebugger")]
    pub fn init_debugger(&mut self, route_label: &str) {
        if self.debugger_route.is_some() {
            return; // Already initialized
        }
        let config = DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: Some(route_label.to_string()),
        };
        let context_id = crate::core::context_id::GoudContextId::new(
            (self.frame_count as u32).wrapping_add(1),
            1,
        );
        let route_id =
            debugger::register_context(context_id, RuntimeSurfaceKind::WindowedGame, &config);
        self.debugger_route = Some(route_id);
    }

    /// Dispatch a JSON debugger request and return the JSON response.
    /// Used by the WebSocket relay to forward IPC verbs.
    #[wasm_bindgen(js_name = "dispatchDebuggerRequest")]
    pub fn dispatch_debugger_request(&self, json: &str) -> String {
        let Some(ref route_id) = self.debugger_route else {
            return r#"{"ok":false,"error":{"code":"attach_disabled","message":"debugger not initialized"}}"#.to_string();
        };
        match debugger::dispatch_request_json_for_route(route_id, json) {
            Ok(value) => serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string()),
            Err(err) => format!(
                r#"{{"ok":false,"error":{{"code":"protocol_error","message":"{}"}}}}"#,
                err
            ),
        }
    }

    /// Get the debugger snapshot as JSON.
    #[wasm_bindgen(js_name = "getDebuggerSnapshotJson")]
    pub fn get_debugger_snapshot_json(&self) -> String {
        let Some(ref route_id) = self.debugger_route else {
            return "{}".to_string();
        };
        debugger::snapshot_for_route(route_id)
            .and_then(|s| serde_json::to_string(&s).ok())
            .unwrap_or_else(|| "{}".to_string())
    }

    /// Capture the canvas as a base64-encoded PNG data URL.
    /// The actual canvas capture is done in JS via canvas.toDataURL().
    #[wasm_bindgen(js_name = "captureCanvasBase64")]
    pub fn capture_canvas_base64(&self) -> Result<String, JsValue> {
        Ok(String::new())
    }
}

// ---------------------------------------------------------------------------
// Panic hook -- routes Rust panics to `console.error`
// ---------------------------------------------------------------------------

fn console_error_panic_hook(info: &std::panic::PanicHookInfo<'_>) {
    let msg = info.to_string();
    web_sys::console::error_1(&JsValue::from_str(&msg));
}
