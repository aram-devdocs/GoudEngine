use crate::components::{SpriteData, Transform2DData};
use crate::entity::Entity;
use goud_engine::ecs::components::{Name, Sprite, Transform2D};
use goud_engine::sdk::{GameConfig as EngineGameConfig, GoudGame as EngineGoudGame};
use napi::bindgen_prelude::*;
use napi_derive::napi;

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
        let game =
            EngineGoudGame::new(engine_config).map_err(|e| Error::from_reason(format!("{}", e)))?;
        Ok(Self {
            inner: game,
            last_delta_time: 0.0,
        })
    }

    // =========================================================================
    // Entity Operations
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
    // Game Loop
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
        self.inner.fps() as f64
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
}
