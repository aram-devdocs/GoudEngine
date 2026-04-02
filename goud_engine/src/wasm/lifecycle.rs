use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use crate::ecs::World;
use crate::rendering::text::GlyphAtlas;

use super::audio;
use super::network;
use super::sprite_renderer::{TextureEntry, WgpuSpriteRenderer};
use super::WasmGame;

// ---------------------------------------------------------------------------
// wgpu rendering state (owned by WasmGame when canvas is provided)
// ---------------------------------------------------------------------------

pub(super) struct WgpuRenderState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub renderer: WgpuSpriteRenderer,
    pub textures: Vec<Option<TextureEntry>>,
    pub clear_color: [f64; 4],
    pub current_frame: Option<wgpu::SurfaceTexture>,
    pub current_view: Option<wgpu::TextureView>,
}

pub(super) struct WasmFontAtlas {
    pub atlas: GlyphAtlas,
    pub texture_handle: Option<u32>,
    pub synced_version: u64,
}

pub(super) struct WasmFontEntry {
    pub font: fontdue::Font,
    pub bytes: Vec<u8>,
    pub atlases: HashMap<u32, WasmFontAtlas>,
}

// ---------------------------------------------------------------------------
// Canvas-based construction
// ---------------------------------------------------------------------------

#[wasm_bindgen]
impl WasmGame {
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
            std::panic::set_hook(Box::new(super::console_error_panic_hook));
        }

        let mut instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
        instance_desc.backends = wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL;
        let instance = wgpu::Instance::new(instance_desc);

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
            network_state: network::WasmNetworkState::new(),
            debugger_route: None,
            pending_textures: Rc::new(RefCell::new(Vec::new())),
        })
    }
}
