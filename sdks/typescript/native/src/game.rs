use crate::components::{SpriteData, Transform2DData};
use crate::entity::Entity;
use goud_engine::ecs::components::{Name, Sprite, Transform2D};
use goud_engine::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use goud_engine::ffi::input::{
    goud_input_get_mouse_delta, goud_input_get_mouse_position, goud_input_get_scroll_delta,
    goud_input_key_just_pressed, goud_input_key_just_released, goud_input_key_pressed,
    goud_input_mouse_button_just_pressed, goud_input_mouse_button_just_released,
    goud_input_mouse_button_pressed,
};
use goud_engine::ffi::renderer::{
    goud_renderer_begin, goud_renderer_draw_quad, goud_renderer_draw_sprite,
    goud_renderer_enable_blending, goud_renderer_end, goud_texture_destroy, goud_texture_load,
};
use goud_engine::ffi::window::{
    goud_window_clear, goud_window_create, goud_window_destroy, goud_window_get_delta_time,
    goud_window_poll_events, goud_window_set_should_close, goud_window_should_close,
    goud_window_swap_buffers,
};
use goud_engine::sdk::{GameConfig as EngineGameConfig, GoudGame as EngineGoudGame};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::ffi::CString;

// =============================================================================
// GameConfig
// =============================================================================

#[napi(object)]
#[derive(Clone, Debug)]
pub struct GameConfig {
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub vsync: Option<bool>,
    pub fullscreen: Option<bool>,
    pub resizable: Option<bool>,
    pub target_fps: Option<u32>,
    pub debug_rendering: Option<bool>,
}

impl From<&GameConfig> for EngineGameConfig {
    fn from(cfg: &GameConfig) -> Self {
        let defaults = EngineGameConfig::default();
        EngineGameConfig {
            title: cfg.title.clone().unwrap_or(defaults.title),
            width: cfg.width.unwrap_or(defaults.width),
            height: cfg.height.unwrap_or(defaults.height),
            vsync: cfg.vsync.unwrap_or(defaults.vsync),
            fullscreen: cfg.fullscreen.unwrap_or(defaults.fullscreen),
            resizable: cfg.resizable.unwrap_or(defaults.resizable),
            target_fps: cfg.target_fps.unwrap_or(defaults.target_fps),
            debug_rendering: cfg.debug_rendering.unwrap_or(defaults.debug_rendering),
        }
    }
}

// =============================================================================
// GoudGame
// =============================================================================

#[napi]
pub struct GoudGame {
    inner: EngineGoudGame,
    context_id: GoudContextId,
    last_delta_time: f32,
}

#[napi]
impl GoudGame {
    #[napi(constructor)]
    pub fn new(config: Option<GameConfig>) -> Result<Self> {
        let engine_config = match &config {
            Some(cfg) => EngineGameConfig::from(cfg),
            None => EngineGameConfig::default(),
        };

        let width = config.as_ref().and_then(|c| c.width).unwrap_or(800);
        let height = config.as_ref().and_then(|c| c.height).unwrap_or(600);
        let title_str = config
            .as_ref()
            .and_then(|c| c.title.clone())
            .unwrap_or_else(|| "GoudEngine".to_string());

        let c_title = CString::new(title_str)
            .map_err(|e| Error::from_reason(format!("Invalid title string: {}", e)))?;

        // SAFETY: CString guarantees a valid null-terminated pointer.
        let context_id = unsafe { goud_window_create(width, height, c_title.as_ptr()) };
        if context_id == GOUD_INVALID_CONTEXT_ID {
            return Err(Error::from_reason("Failed to create GLFW window"));
        }

        let game =
            EngineGoudGame::new(engine_config).map_err(|e| Error::from_reason(format!("{}", e)))?;

        Ok(Self {
            inner: game,
            context_id,
            last_delta_time: 0.0,
        })
    }

    // =========================================================================
    // Lifecycle
    // =========================================================================

    #[napi]
    pub fn should_close(&self) -> bool {
        goud_window_should_close(self.context_id)
    }

    #[napi]
    pub fn close(&self) {
        goud_window_set_should_close(self.context_id, true);
    }

    #[napi]
    pub fn destroy(&self) -> bool {
        goud_window_destroy(self.context_id)
    }

    // =========================================================================
    // Frame Management
    // =========================================================================

    #[napi]
    pub fn begin_frame(&mut self, r: Option<f64>, g: Option<f64>, b: Option<f64>, a: Option<f64>) {
        let dt = goud_window_poll_events(self.context_id);
        self.last_delta_time = dt;
        goud_window_clear(
            self.context_id,
            r.unwrap_or(0.0) as f32,
            g.unwrap_or(0.0) as f32,
            b.unwrap_or(0.0) as f32,
            a.unwrap_or(1.0) as f32,
        );
        goud_renderer_begin(self.context_id);
        goud_renderer_enable_blending(self.context_id);
    }

    #[napi]
    pub fn end_frame(&self) {
        goud_renderer_end(self.context_id);
        goud_window_swap_buffers(self.context_id);
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    #[napi]
    pub fn load_texture(&self, path: String) -> Result<f64> {
        let c_path =
            CString::new(path).map_err(|e| Error::from_reason(format!("Invalid path: {}", e)))?;
        // SAFETY: CString guarantees a valid null-terminated pointer.
        let handle = unsafe { goud_texture_load(self.context_id, c_path.as_ptr()) };
        Ok(handle as f64)
    }

    #[napi]
    pub fn destroy_texture(&self, handle: f64) -> bool {
        goud_texture_destroy(self.context_id, handle as u64)
    }

    #[napi]
    pub fn draw_sprite(
        &self,
        texture: f64,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        rotation: Option<f64>,
        r: Option<f64>,
        g: Option<f64>,
        b: Option<f64>,
        a: Option<f64>,
    ) -> bool {
        goud_renderer_draw_sprite(
            self.context_id,
            texture as u64,
            x as f32,
            y as f32,
            w as f32,
            h as f32,
            rotation.unwrap_or(0.0) as f32,
            r.unwrap_or(1.0) as f32,
            g.unwrap_or(1.0) as f32,
            b.unwrap_or(1.0) as f32,
            a.unwrap_or(1.0) as f32,
        )
    }

    #[napi]
    pub fn draw_quad(
        &self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        r: Option<f64>,
        g: Option<f64>,
        b: Option<f64>,
        a: Option<f64>,
    ) -> bool {
        goud_renderer_draw_quad(
            self.context_id,
            x as f32,
            y as f32,
            w as f32,
            h as f32,
            r.unwrap_or(1.0) as f32,
            g.unwrap_or(1.0) as f32,
            b.unwrap_or(1.0) as f32,
            a.unwrap_or(1.0) as f32,
        )
    }

    // =========================================================================
    // Input
    // =========================================================================

    #[napi]
    pub fn is_key_pressed(&self, key: i32) -> bool {
        goud_input_key_pressed(self.context_id, key)
    }

    #[napi]
    pub fn is_key_just_pressed(&self, key: i32) -> bool {
        goud_input_key_just_pressed(self.context_id, key)
    }

    #[napi]
    pub fn is_key_just_released(&self, key: i32) -> bool {
        goud_input_key_just_released(self.context_id, key)
    }

    #[napi]
    pub fn is_mouse_button_pressed(&self, button: i32) -> bool {
        goud_input_mouse_button_pressed(self.context_id, button)
    }

    #[napi]
    pub fn is_mouse_button_just_pressed(&self, button: i32) -> bool {
        goud_input_mouse_button_just_pressed(self.context_id, button)
    }

    #[napi]
    pub fn is_mouse_button_just_released(&self, button: i32) -> bool {
        goud_input_mouse_button_just_released(self.context_id, button)
    }

    #[napi]
    pub fn get_mouse_position(&self) -> Vec<f64> {
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        // SAFETY: Passing valid mutable references as out-pointers.
        unsafe { goud_input_get_mouse_position(self.context_id, &mut x, &mut y) };
        vec![x as f64, y as f64]
    }

    #[napi]
    pub fn get_mouse_delta(&self) -> Vec<f64> {
        let mut dx: f32 = 0.0;
        let mut dy: f32 = 0.0;
        // SAFETY: Passing valid mutable references as out-pointers.
        unsafe { goud_input_get_mouse_delta(self.context_id, &mut dx, &mut dy) };
        vec![dx as f64, dy as f64]
    }

    #[napi]
    pub fn get_scroll_delta(&self) -> Vec<f64> {
        let mut dx: f32 = 0.0;
        let mut dy: f32 = 0.0;
        // SAFETY: Passing valid mutable references as out-pointers.
        unsafe { goud_input_get_scroll_delta(self.context_id, &mut dx, &mut dy) };
        vec![dx as f64, dy as f64]
    }

    // =========================================================================
    // Entity Operations (ECS)
    // =========================================================================

    #[napi]
    pub fn spawn_empty(&mut self) -> Entity {
        Entity {
            inner: self.inner.spawn_empty(),
        }
    }

    #[napi]
    pub fn spawn_batch(&mut self, count: u32) -> Vec<Entity> {
        self.inner
            .spawn_batch(count as usize)
            .into_iter()
            .map(|e| Entity { inner: e })
            .collect()
    }

    #[napi]
    pub fn despawn(&mut self, entity: &Entity) -> bool {
        self.inner.despawn(entity.inner)
    }

    #[napi]
    pub fn entity_count(&self) -> u32 {
        self.inner.entity_count() as u32
    }

    #[napi]
    pub fn is_alive(&self, entity: &Entity) -> bool {
        self.inner.is_alive(entity.inner)
    }

    // =========================================================================
    // Transform2D Component
    // =========================================================================

    #[napi]
    pub fn add_transform2d(&mut self, entity: &Entity, data: Transform2DData) {
        let transform = Transform2D::from(&data);
        self.inner.insert(entity.inner, transform);
    }

    #[napi]
    pub fn get_transform2d(&self, entity: &Entity) -> Option<Transform2DData> {
        self.inner
            .get::<Transform2D>(entity.inner)
            .map(Transform2DData::from)
    }

    #[napi]
    pub fn set_transform2d(&mut self, entity: &Entity, data: Transform2DData) {
        if let Some(t) = self.inner.get_mut::<Transform2D>(entity.inner) {
            t.position.x = data.position_x as f32;
            t.position.y = data.position_y as f32;
            t.rotation = data.rotation as f32;
            t.scale.x = data.scale_x as f32;
            t.scale.y = data.scale_y as f32;
        }
    }

    #[napi]
    pub fn has_transform2d(&self, entity: &Entity) -> bool {
        self.inner.has::<Transform2D>(entity.inner)
    }

    #[napi]
    pub fn remove_transform2d(&mut self, entity: &Entity) -> bool {
        self.inner.remove::<Transform2D>(entity.inner).is_some()
    }

    // =========================================================================
    // Sprite Component
    // =========================================================================

    #[napi]
    pub fn add_sprite(&mut self, entity: &Entity, data: SpriteData) {
        let sprite = Sprite::from(&data);
        self.inner.insert(entity.inner, sprite);
    }

    #[napi]
    pub fn get_sprite(&self, entity: &Entity) -> Option<SpriteData> {
        self.inner.get::<Sprite>(entity.inner).map(SpriteData::from)
    }

    #[napi]
    pub fn has_sprite(&self, entity: &Entity) -> bool {
        self.inner.has::<Sprite>(entity.inner)
    }

    #[napi]
    pub fn remove_sprite(&mut self, entity: &Entity) -> bool {
        self.inner.remove::<Sprite>(entity.inner).is_some()
    }

    // =========================================================================
    // Name Component
    // =========================================================================

    #[napi]
    pub fn add_name(&mut self, entity: &Entity, name: String) {
        self.inner.insert(entity.inner, Name::new(&name));
    }

    #[napi]
    pub fn get_name(&self, entity: &Entity) -> Option<String> {
        self.inner
            .get::<Name>(entity.inner)
            .map(|n| n.as_str().to_string())
    }

    #[napi]
    pub fn has_name(&self, entity: &Entity) -> bool {
        self.inner.has::<Name>(entity.inner)
    }

    #[napi]
    pub fn remove_name(&mut self, entity: &Entity) -> bool {
        self.inner.remove::<Name>(entity.inner).is_some()
    }

    // =========================================================================
    // Legacy Game Loop (ECS-only)
    // =========================================================================

    #[napi]
    pub fn update_frame(&mut self, delta_time: f64) {
        let dt = delta_time as f32;
        self.last_delta_time = dt;
        self.inner.update_frame(dt, |_, _| {});
    }

    // =========================================================================
    // Timing / Stats (getters)
    // =========================================================================

    #[napi(getter)]
    pub fn delta_time(&self) -> f64 {
        self.last_delta_time as f64
    }

    #[napi(getter)]
    pub fn total_time(&self) -> f64 {
        self.inner.total_time() as f64
    }

    #[napi(getter)]
    pub fn fps(&self) -> f64 {
        if self.last_delta_time > 0.0 {
            (1.0 / self.last_delta_time) as f64
        } else {
            0.0
        }
    }

    #[napi(getter)]
    pub fn frame_count(&self) -> u32 {
        self.inner.frame_count() as u32
    }

    #[napi(getter)]
    pub fn is_initialized(&self) -> bool {
        self.inner.is_initialized()
    }

    // =========================================================================
    // Configuration (getters)
    // =========================================================================

    #[napi(getter)]
    pub fn title(&self) -> String {
        self.inner.title().to_string()
    }

    #[napi(getter)]
    pub fn window_width(&self) -> u32 {
        self.inner.window_size().0
    }

    #[napi(getter)]
    pub fn window_height(&self) -> u32 {
        self.inner.window_size().1
    }

    #[napi(getter)]
    pub fn context_valid(&self) -> bool {
        self.context_id != GOUD_INVALID_CONTEXT_ID
    }

    /// Returns the raw FFI delta time from the last poll_events call.
    #[napi(getter)]
    pub fn ffi_delta_time(&self) -> f64 {
        goud_window_get_delta_time(self.context_id) as f64
    }
}
