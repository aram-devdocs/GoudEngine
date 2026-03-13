//! [`GoudGame`] struct definition, construction, and core API.

mod ecs_scene;

use std::sync::atomic::{AtomicU32, Ordering};
#[cfg(feature = "native")]
use std::sync::atomic::AtomicU64;
#[cfg(feature = "native")]
use std::sync::{Arc, Condvar, Mutex};

use crate::context_registry::scene::SceneManager;
use crate::core::context_id::GoudContextId;
use crate::core::debugger::{self, RuntimeRouteId, RuntimeSurfaceKind, SyntheticInputEventV1};
use crate::core::error::GoudResult;
use crate::core::providers::types::DebugShape;
use crate::core::providers::ProviderRegistry;
use crate::sdk::debug_overlay::DebugOverlay;
use crate::sdk::game_config::{GameConfig, GameContext};
use crate::ui::UiManager;

#[cfg(feature = "native")]
use crate::ecs::InputManager;
#[cfg(feature = "native")]
use crate::libs::graphics::backend::opengl::OpenGLBackend;
#[cfg(feature = "native")]
use crate::libs::graphics::renderer3d::Renderer3D;
#[cfg(feature = "native")]
use crate::libs::platform::PlatformBackend;
#[cfg(feature = "native")]
use crate::rendering::sprite_batch::SpriteBatch;
#[cfg(feature = "native")]
use crate::rendering::text::TextBatch;
#[cfg(feature = "native")]
use crate::rendering::UiRenderSystem;

/// Shared state for deferred framebuffer capture.
///
/// The IPC handler thread sets `requested = true` and waits on the condvar.
/// The main thread (in `swap_buffers`) checks `requested`, does the GL readback,
/// stores the result, and notifies the condvar.
#[cfg(feature = "native")]
pub(crate) struct DeferredCaptureState {
    requested: bool,
    result: Option<Result<debugger::RawFramebufferReadbackV1, String>>,
}

#[cfg(feature = "native")]
type DeferredCapture = Arc<(Mutex<DeferredCaptureState>, Condvar)>;

/// The main game instance managing the ECS world and game loop.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::{GoudGame, GameConfig};
/// use goud_engine::sdk::components::Transform2D;
/// use goud_engine::core::math::Vec2;
///
/// let mut game = GoudGame::new(GameConfig::default()).unwrap();
/// let player = game.spawn()
///     .with(Transform2D::from_position(Vec2::new(400.0, 300.0)))
///     .build();
/// ```
pub struct GoudGame {
    /// Manages multiple isolated ECS worlds (scenes).
    pub(crate) scene_manager: SceneManager,

    /// Game configuration.
    pub(crate) config: GameConfig,

    /// Runtime context for the game loop.
    pub(crate) context: GameContext,

    /// Whether the game has been initialized.
    pub(crate) initialized: bool,

    /// Debug overlay for FPS stats tracking.
    pub(crate) debug_overlay: DebugOverlay,

    /// Provider registry for subsystem backends (render, physics, audio, input).
    pub(crate) providers: ProviderRegistry,

    /// Route registered with the debugger runtime for this game, if enabled.
    pub(crate) debugger_route: Option<RuntimeRouteId>,

    /// Cached physics debug shapes for the most recent frame.
    pub(crate) physics_debug_shapes: Vec<DebugShape>,

    /// Whether debugger runtime control enabled debug draw for the current frame.
    pub(crate) runtime_debug_draw_enabled: bool,

    /// Stores the result of the most recent transition completion, if any.
    /// Use [`take_transition_complete`](Self::take_transition_complete) to consume it.
    pub(crate) last_transition_complete:
        Option<crate::context_registry::scene::transition::TransitionComplete>,

    /// UI manager for immediate-mode UI widgets.
    pub(crate) ui_manager: UiManager,

    // =========================================================================
    // Native-only fields (require windowing + OpenGL)
    // =========================================================================
    /// Platform backend for window management (GLFW).
    #[cfg(feature = "native")]
    pub(crate) platform: Option<Box<dyn PlatformBackend>>,

    /// OpenGL rendering backend.
    #[cfg(feature = "native")]
    pub(crate) render_backend: Option<OpenGLBackend>,

    /// Input manager for keyboard/mouse/gamepad state.
    #[cfg(feature = "native")]
    pub(crate) input_manager: InputManager,

    /// 2D sprite batch renderer.
    #[cfg(feature = "native")]
    pub(crate) sprite_batch: Option<SpriteBatch<OpenGLBackend>>,

    /// Asset server for loading and managing assets.
    #[cfg(feature = "native")]
    pub(crate) asset_server: Option<crate::assets::AssetServer>,

    /// Native UI renderer that consumes `UiManager` command streams.
    #[cfg(feature = "native")]
    pub(crate) ui_render_system: Option<UiRenderSystem>,

    /// Immediate text batch renderer for native SDK text draws.
    #[cfg(feature = "native")]
    pub(crate) text_batch: Option<TextBatch>,

    /// 3D renderer for primitives, lighting, and camera.
    #[cfg(feature = "native")]
    pub(crate) renderer_3d: Option<Renderer3D>,

    /// GPU resources for immediate-mode sprite/quad rendering.
    #[cfg(feature = "native")]
    pub(crate) immediate_state: Option<crate::sdk::rendering::ImmediateRenderState>,

    /// Centralized audio playback manager.
    #[cfg(feature = "native")]
    pub(crate) audio_manager: Option<crate::assets::AudioManager>,

    /// Shared framebuffer dimensions for the debugger capture hook.
    /// Packed as `(width << 32) | height`. Read by the capture hook closure.
    #[cfg(feature = "native")]
    #[allow(dead_code)]
    pub(crate) capture_dimensions: Option<Arc<AtomicU64>>,

    /// Deferred capture coordination between IPC thread and main GL thread.
    #[cfg(feature = "native")]
    pub(crate) deferred_capture: Option<DeferredCapture>,
}

/// Initializes the logger, diagnostic mode from environment, and optionally
/// enables diagnostic mode based on the game configuration.
fn init_engine_diagnostics(config: &GameConfig) {
    crate::core::error::init_logger();
    crate::core::error::init_diagnostic_from_env();
    if config.diagnostic_mode {
        crate::core::error::set_diagnostic_enabled(true);
    }
}

impl GoudGame {
    fn next_debugger_context_id() -> GoudContextId {
        static NEXT_ID: AtomicU32 = AtomicU32::new(1_000_000);
        GoudContextId::new(NEXT_ID.fetch_add(1, Ordering::Relaxed), 1)
    }

    fn register_debugger_route(
        config: &GameConfig,
        surface: RuntimeSurfaceKind,
    ) -> Option<RuntimeRouteId> {
        config.debugger.enabled.then(|| {
            debugger::register_context(Self::next_debugger_context_id(), surface, &config.debugger)
        })
    }

    #[cfg(feature = "native")]
    pub(crate) fn apply_synthetic_inputs(&mut self, events: &[SyntheticInputEventV1]) {
        use glfw::{Key, MouseButton};

        fn parse_key(key: &str) -> Option<Key> {
            match key.to_ascii_lowercase().as_str() {
                "space" => Some(Key::Space),
                "enter" => Some(Key::Enter),
                "escape" => Some(Key::Escape),
                "tab" => Some(Key::Tab),
                "left" => Some(Key::Left),
                "right" => Some(Key::Right),
                "up" => Some(Key::Up),
                "down" => Some(Key::Down),
                "a" => Some(Key::A),
                "d" => Some(Key::D),
                "s" => Some(Key::S),
                "w" => Some(Key::W),
                _ => None,
            }
        }

        fn parse_mouse_button(button: &str) -> Option<MouseButton> {
            match button.to_ascii_lowercase().as_str() {
                "left" => Some(MouseButton::Button1),
                "right" => Some(MouseButton::Button2),
                "middle" => Some(MouseButton::Button3),
                _ => None,
            }
        }

        for event in events {
            match (
                event.device.as_str(),
                event.action.as_str(),
                event.key.as_deref(),
                event.button.as_deref(),
            ) {
                ("keyboard", "press", Some(key), _) => {
                    if let Some(key) = parse_key(key) {
                        self.input_manager.press_key(key);
                    }
                }
                ("keyboard", "release", Some(key), _) => {
                    if let Some(key) = parse_key(key) {
                        self.input_manager.release_key(key);
                    }
                }
                ("mouse", "press", _, Some(button)) => {
                    if let Some(button) = parse_mouse_button(button) {
                        self.input_manager.press_mouse_button(button);
                    }
                }
                ("mouse", "release", _, Some(button)) => {
                    if let Some(button) = parse_mouse_button(button) {
                        self.input_manager.release_mouse_button(button);
                    }
                }
                _ => {}
            }
        }
    }

    #[cfg(not(feature = "native"))]
    pub(crate) fn apply_synthetic_inputs(&mut self, _events: &[SyntheticInputEventV1]) {}

    pub(crate) fn prepare_runtime_frame(&mut self, raw_delta_seconds: f32) -> f32 {
        let Some(route_id) = self.debugger_route.clone() else {
            self.runtime_debug_draw_enabled = false;
            return raw_delta_seconds;
        };

        let frame_plan = debugger::take_frame_control_for_route(&route_id, raw_delta_seconds)
            .unwrap_or_default();
        self.runtime_debug_draw_enabled = frame_plan.debug_draw_enabled;
        self.apply_synthetic_inputs(&frame_plan.synthetic_inputs);

        let (next_index, total_seconds) = debugger::snapshot_for_route(&route_id)
            .map(|snapshot| {
                (
                    snapshot.frame.index.saturating_add(1),
                    snapshot.frame.total_seconds + frame_plan.effective_delta_seconds as f64,
                )
            })
            .unwrap_or((1, frame_plan.effective_delta_seconds as f64));
        debugger::begin_frame(
            &route_id,
            next_index,
            frame_plan.effective_delta_seconds,
            total_seconds,
        );
        frame_plan.effective_delta_seconds
    }

    pub(crate) fn finish_runtime_frame(&mut self) {
        if let Some(route_id) = self.debugger_route.as_ref() {
            // Push render stats from sprite batch if available.
            #[cfg(feature = "native")]
            if let Some(batch) = &self.sprite_batch {
                let (sprite_count, batch_count, _ratio) = batch.stats();
                // Each sprite is a quad = 2 triangles; one draw call per batch.
                let _ = debugger::update_render_stats_for_context(
                    GoudContextId::new(
                        route_id.context_id as u32,
                        (route_id.context_id >> 32) as u32,
                    ),
                    batch_count as u32,
                    (sprite_count * 2) as u32,
                    batch_count as u32,
                    1,
                );
            }

            // Eagerly push entity/scene data so IPC snapshots are fresh.
            self.refresh_debugger_snapshot_data(route_id);

            debugger::end_frame(route_id);
        }
    }

    /// Services a pending deferred capture request by performing the GL
    /// readback on the current (main) thread, then notifying the waiting
    /// IPC thread.
    #[cfg(feature = "native")]
    pub(crate) fn service_deferred_capture(&self) {
        let Some(ref deferred) = self.deferred_capture else {
            return;
        };
        let (lock, cvar) = &**deferred;
        let mut guard = match lock.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if !guard.requested {
            return;
        }
        let (w, h) = self.get_framebuffer_size();
        let result = OpenGLBackend::read_framebuffer_standalone(w, h)
            .map(|rgba8| debugger::RawFramebufferReadbackV1 {
                width: w,
                height: h,
                rgba8,
            })
            .map_err(|e| format!("framebuffer readback failed: {e}"));
        guard.result = Some(result);
        cvar.notify_all();
    }

    /// Populates the debugger snapshot with current entity and scene data.
    fn refresh_debugger_snapshot_data(&self, route_id: &RuntimeRouteId) {
        let active_scene_name = self
            .scene_manager
            .get_scene_name(self.scene_manager.default_scene())
            .unwrap_or("default")
            .to_string();

        let selected_entity = debugger::snapshot_for_route(route_id).and_then(|snapshot| {
            snapshot.selection.entity_id.map(|entity_id| {
                let scene_id = if snapshot.selection.scene_id.is_empty() {
                    active_scene_name.clone()
                } else {
                    snapshot.selection.scene_id.clone()
                };
                (scene_id, entity_id)
            })
        });

        let mut entities = Vec::new();
        let mut entity_count = 0usize;
        for scene_id in self.scene_manager.active_scenes() {
            let Some(world) = self.scene_manager.get_scene(*scene_id) else {
                continue;
            };
            entity_count = entity_count.saturating_add(world.entity_count());
            let scene_name = self
                .scene_manager
                .get_scene_name(*scene_id)
                .unwrap_or("unknown")
                .to_string();
            let scene_entities =
                crate::context_registry::scene::collect_debugger_entities(
                    world,
                    scene_name,
                    selected_entity
                        .as_ref()
                        .map(|(scene_id, entity_id)| (scene_id.as_str(), *entity_id)),
                );
            entities.extend(scene_entities);
        }

        let _ = debugger::with_snapshot_mut(route_id, |snapshot| {
            snapshot.scene.active_scene = active_scene_name.clone();
            snapshot.scene.entity_count = entity_count as u32;
            if snapshot.selection.entity_id.is_none() || snapshot.selection.scene_id.is_empty() {
                snapshot.selection.scene_id = active_scene_name;
            }
            snapshot.entities = entities;
        });
    }

    /// Updates cached physics debug shapes according to runtime config.
    ///
    /// When disabled, this avoids querying the physics provider entirely.
    pub(crate) fn update_physics_debug_shapes(&mut self) {
        if !self.config.physics_debug.enabled && !self.runtime_debug_draw_enabled {
            self.physics_debug_shapes.clear();
            return;
        }

        self.physics_debug_shapes = self.providers.physics.debug_shapes();
    }

    /// Creates a new game instance with the given configuration.
    ///
    /// This creates a headless game instance suitable for testing and
    /// non-graphical use. For a windowed game with rendering, use
    /// [`with_platform`](Self::with_platform) instead.
    pub fn new(config: GameConfig) -> GoudResult<Self> {
        init_engine_diagnostics(&config);

        let window_size = (config.width, config.height);
        let mut debug_overlay = DebugOverlay::new(config.fps_update_interval);
        debug_overlay.set_enabled(config.show_fps_overlay);
        let debugger_route =
            Self::register_debugger_route(&config, RuntimeSurfaceKind::HeadlessContext);
        Ok(Self {
            scene_manager: SceneManager::new(),
            config,
            context: GameContext::new(window_size),
            initialized: false,
            debug_overlay,
            providers: ProviderRegistry::default(),
            debugger_route,
            physics_debug_shapes: Vec::new(),
            runtime_debug_draw_enabled: false,
            last_transition_complete: None,
            ui_manager: UiManager::new(),
            #[cfg(feature = "native")]
            platform: None,
            #[cfg(feature = "native")]
            render_backend: None,
            #[cfg(feature = "native")]
            input_manager: InputManager::default(),
            #[cfg(feature = "native")]
            sprite_batch: None,
            #[cfg(feature = "native")]
            asset_server: None,
            #[cfg(feature = "native")]
            ui_render_system: None,
            #[cfg(feature = "native")]
            text_batch: None,
            #[cfg(feature = "native")]
            renderer_3d: None,
            #[cfg(feature = "native")]
            immediate_state: None,
            #[cfg(feature = "native")]
            audio_manager: None,
            #[cfg(feature = "native")]
            capture_dimensions: None,
            #[cfg(feature = "native")]
            deferred_capture: None,
        })
    }

    /// Creates a game with default configuration.
    pub fn default_game() -> GoudResult<Self> {
        Self::new(GameConfig::default())
    }

    /// Creates a windowed game instance with a GLFW platform backend.
    ///
    /// This initializes a GLFW window with an OpenGL 3.3 Core context,
    /// sets up the sprite batch renderer, and prepares the asset server.
    ///
    /// # Errors
    ///
    /// Returns an error if GLFW initialization or window creation fails.
    #[cfg(feature = "native")]
    pub fn with_platform(config: GameConfig) -> GoudResult<Self> {
        use crate::assets::AssetServer;
        use crate::libs::graphics::backend::StateOps;
        init_engine_diagnostics(&config);

        use crate::libs::platform::glfw_platform::GlfwPlatform;
        use crate::libs::platform::WindowConfig;

        let window_config = WindowConfig {
            width: config.width,
            height: config.height,
            title: config.title.clone(),
            vsync: config.vsync,
            resizable: config.resizable,
        };

        let platform = GlfwPlatform::new(&window_config)?;
        let window_size = (config.width, config.height);
        let mut debug_overlay = DebugOverlay::new(config.fps_update_interval);
        debug_overlay.set_enabled(config.show_fps_overlay);
        let mut render_backend = OpenGLBackend::new()?;
        render_backend.set_viewport(0, 0, config.width, config.height);
        let renderer_3d =
            Renderer3D::new(Box::new(OpenGLBackend::new()?), config.width, config.height)
                .map_err(crate::core::error::GoudError::InitializationFailed)?;

        let audio_manager = crate::assets::AudioManager::new().ok();
        let debugger_route =
            Self::register_debugger_route(&config, RuntimeSurfaceKind::WindowedGame);

        // Register deferred capture hook for framebuffer readback if debugger
        // is enabled.  The hook is invoked from the IPC thread which has no GL
        // context, so it signals the main thread (via condvar) to do the actual
        // glReadPixels in swap_buffers().
        let (capture_dimensions, deferred_capture) = if let Some(ref route_id) = debugger_route {
            let dims = Arc::new(AtomicU64::new(
                ((config.width as u64) << 32) | config.height as u64,
            ));
            let deferred: DeferredCapture = Arc::new((
                Mutex::new(DeferredCaptureState {
                    requested: false,
                    result: None,
                }),
                Condvar::new(),
            ));
            let deferred_clone = Arc::clone(&deferred);
            debugger::register_capture_hook_for_route(route_id.clone(), move || {
                let (lock, cvar) = &*deferred_clone;
                let mut guard = lock
                    .lock()
                    .map_err(|e| format!("capture lock poisoned: {e}"))?;
                guard.requested = true;
                guard.result = None;
                // Wait up to 5 seconds for the main thread to service the readback.
                let timeout = std::time::Duration::from_secs(5);
                loop {
                    let (new_guard, wait_result) = cvar
                        .wait_timeout(guard, timeout)
                        .map_err(|e| format!("capture condvar error: {e}"))?;
                    guard = new_guard;
                    if guard.result.is_some() {
                        break;
                    }
                    if wait_result.timed_out() {
                        guard.requested = false;
                        return Err(
                            "capture timed out waiting for main thread readback".to_string()
                        );
                    }
                }
                guard.requested = false;
                guard.result.take().unwrap()
            });
            (Some(dims), Some(deferred))
        } else {
            (None, None)
        };

        Ok(Self {
            scene_manager: SceneManager::new(),
            config,
            context: GameContext::new(window_size),
            initialized: false,
            debug_overlay,
            providers: ProviderRegistry::default(),
            debugger_route,
            physics_debug_shapes: Vec::new(),
            runtime_debug_draw_enabled: false,
            last_transition_complete: None,
            ui_manager: UiManager::new(),
            platform: Some(Box::new(platform)),
            render_backend: Some(render_backend),
            input_manager: InputManager::default(),
            sprite_batch: None,
            asset_server: Some(AssetServer::with_root(".")),
            ui_render_system: Some(UiRenderSystem::new()),
            text_batch: Some(TextBatch::new()),
            renderer_3d: Some(renderer_3d),
            immediate_state: None,
            audio_manager,
            capture_dimensions,
            deferred_capture,
        })
    }

    /// Returns the game configuration.
    #[inline]
    pub fn config(&self) -> &GameConfig {
        &self.config
    }

    /// Returns the window title.
    #[inline]
    pub fn title(&self) -> &str {
        &self.config.title
    }

    /// Returns the window dimensions.
    #[inline]
    pub fn window_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}

impl Drop for GoudGame {
    fn drop(&mut self) {
        if let Some(route_id) = self.debugger_route.take() {
            debugger::unregister_capture_hook_for_route(&route_id);
            debugger::unregister_context(GoudContextId::new(
                route_id.context_id as u32,
                (route_id.context_id >> 32) as u32,
            ));
        }
    }
}

impl Default for GoudGame {
    fn default() -> Self {
        Self::new(GameConfig::default()).expect("Failed to create default GoudGame")
    }
}

impl std::fmt::Debug for GoudGame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoudGame")
            .field("config", &self.config)
            .field("entity_count", &self.entity_count())
            .field("initialized", &self.initialized)
            .finish()
    }
}
