//! [`GoudGame`] struct definition, construction, and core API.

mod capture;
mod debugger_frame;
mod ecs_scene;
#[cfg(feature = "lua")]
mod lua_bindings;
#[cfg(all(feature = "lua", feature = "native"))]
mod lua_bridge;
#[cfg(all(feature = "lua", feature = "native"))]
pub(crate) mod lua_hot_reload;
#[cfg(feature = "lua")]
pub(crate) mod lua_runtime;
#[cfg(test)]
mod tests;

#[cfg(feature = "native")]
use std::collections::HashMap;
#[cfg(feature = "native")]
use std::sync::atomic::AtomicU64;
#[cfg(feature = "native")]
use std::sync::Arc;

use crate::context_registry::scene::SceneManager;
use crate::core::debugger::{self, RuntimeRouteId, RuntimeSurfaceKind};
use crate::core::error::GoudResult;
use crate::core::event::Events;
use crate::core::events::WindowResized;
use crate::core::providers::types::DebugShape;
use crate::core::providers::ProviderRegistry;
use crate::rendering::{compute_render_viewport, RenderViewport, ViewportScaleMode};
use crate::sdk::debug_overlay::DebugOverlay;
use crate::sdk::game_config::{GameConfig, GameContext};
use crate::ui::UiManager;
#[cfg(feature = "lua")]
use lua_runtime::LuaRuntime;

#[cfg(feature = "native")]
use crate::ecs::InputManager;
#[cfg(feature = "native")]
use crate::libs::graphics::backend::native_backend::SharedNativeRenderBackend;
#[cfg(feature = "native")]
use crate::libs::graphics::renderer3d::Renderer3D;
#[cfg(feature = "native")]
use crate::libs::platform::PlatformBackend;
#[cfg(feature = "native")]
use crate::rendering::sprite_batch::SpriteBatch;
#[cfg(feature = "native")]
use crate::rendering::sprite_batch::SpriteBatchConfig;
#[cfg(feature = "native")]
use crate::rendering::text::TextBatch;
#[cfg(feature = "native")]
use crate::rendering::UiRenderSystem;

#[cfg(feature = "native")]
pub(crate) use crate::core::debugger::DeferredCapture;

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

    /// Window resize events emitted through the runtime path.
    pub(crate) window_resized_events: Events<WindowResized>,

    #[cfg(feature = "lua")]
    // Kept alive for Drop: the embedded Lua VM lives as long as GoudGame.
    lua_runtime: LuaRuntime,

    /// Optional Lua script hot-reload watcher (native + lua only).
    #[cfg(all(feature = "lua", feature = "native"))]
    lua_watcher: Option<lua_hot_reload::LuaScriptWatcher>,

    // =========================================================================
    // Native-only fields (require a desktop windowing and render backend)
    // =========================================================================
    /// Native window backend for event pumping and lifecycle.
    #[cfg(feature = "native")]
    pub(crate) platform: Option<Box<dyn PlatformBackend>>,

    /// Active native rendering backend.
    #[cfg(feature = "native")]
    pub(crate) render_backend: Option<SharedNativeRenderBackend>,

    /// Input manager for keyboard/mouse/gamepad state.
    #[cfg(feature = "native")]
    pub(crate) input_manager: InputManager,

    /// 2D sprite batch renderer.
    #[cfg(feature = "native")]
    pub(crate) sprite_batch: Option<SpriteBatch<SharedNativeRenderBackend>>,

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

    /// Deferred capture coordination between the IPC thread and the main thread.
    #[cfg(feature = "native")]
    pub(crate) deferred_capture: Option<DeferredCapture>,

    /// Logical resolution used for 2D/UI projection and viewport policy.
    #[cfg(feature = "native")]
    pub(crate) design_resolution: (u32, u32),

    /// Current viewport scaling policy.
    #[cfg(feature = "native")]
    pub(crate) viewport_scale_mode: ViewportScaleMode,

    /// Resolved render viewport for the current framebuffer.
    #[cfg(feature = "native")]
    pub(crate) render_viewport: RenderViewport,

    /// Active offscreen render-target viewport override, when bound.
    #[cfg(feature = "native")]
    pub(crate) bound_render_target_viewport: Option<(u64, RenderViewport)>,

    /// Packed texture handles owned by active render targets.
    #[cfg(feature = "native")]
    pub(crate) render_target_attachment_owners: HashMap<u64, u64>,
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
        #[cfg(feature = "lua")]
        let lua_runtime = LuaRuntime::new(0)?;
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
            window_resized_events: Events::new(),
            #[cfg(feature = "lua")]
            lua_runtime,
            #[cfg(all(feature = "lua", feature = "native"))]
            lua_watcher: None,
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
            #[cfg(feature = "native")]
            design_resolution: window_size,
            #[cfg(feature = "native")]
            viewport_scale_mode: ViewportScaleMode::Stretch,
            #[cfg(feature = "native")]
            render_viewport: RenderViewport::fullscreen(window_size),
            #[cfg(feature = "native")]
            bound_render_target_viewport: None,
            #[cfg(feature = "native")]
            render_target_attachment_owners: HashMap::new(),
        })
    }

    /// Creates a game with default configuration.
    pub fn default_game() -> GoudResult<Self> {
        Self::new(GameConfig::default())
    }

    /// Creates a windowed game instance using the configured native backend pair.
    ///
    /// This initializes the selected window backend and render backend, then
    /// prepares the native 2D, 3D, UI, and asset subsystems.
    ///
    /// # Errors
    ///
    /// Returns an error if native runtime initialization fails.
    #[cfg(feature = "native")]
    pub fn with_platform(config: GameConfig) -> GoudResult<Self> {
        use crate::assets::loaders::ensure_3d_asset_loaders;
        use crate::assets::AssetServer;
        use crate::libs::platform::native_runtime::create_native_runtime;
        use crate::rendering::sprite_batch::{
            ensure_default_sprite_shader_loaded, ensure_sprite_asset_loaders,
        };
        init_engine_diagnostics(&config);
        use crate::libs::platform::WindowConfig;

        let window_config = WindowConfig {
            width: config.width,
            height: config.height,
            title: config.title.clone(),
            vsync: config.vsync,
            resizable: config.resizable,
            msaa_samples: config.msaa_samples,
        };

        let native_runtime =
            create_native_runtime(&window_config, config.window_backend, config.render_backend)?;
        let window_size = (config.width, config.height);
        let mut debug_overlay = DebugOverlay::new(config.fps_update_interval);
        debug_overlay.set_enabled(config.show_fps_overlay);
        let render_backend = native_runtime.render_backend;
        let mut renderer_3d = Renderer3D::new(
            Box::new(render_backend.clone()),
            config.width,
            config.height,
        )
        .map_err(crate::core::error::GoudError::InitializationFailed)?;
        renderer_3d.set_msaa_samples(config.msaa_samples);
        renderer_3d
            .set_anti_aliasing_mode(config.anti_aliasing_mode)
            .map_err(crate::core::error::GoudError::InitializationFailed)?;
        let mut asset_server = AssetServer::with_root(".");
        ensure_sprite_asset_loaders(&mut asset_server);
        ensure_3d_asset_loaders(&mut asset_server);
        let sprite_shader = ensure_default_sprite_shader_loaded(&mut asset_server);
        let sprite_batch = SpriteBatch::new(
            render_backend.clone(),
            SpriteBatchConfig {
                shader_asset: sprite_shader,
                ..SpriteBatchConfig::default()
            },
        )?;
        let framebuffer_size = native_runtime.platform.get_framebuffer_size();
        let render_viewport =
            compute_render_viewport(framebuffer_size, window_size, ViewportScaleMode::Stretch);

        let audio_manager = crate::assets::AudioManager::new().ok();
        let debugger_route =
            Self::register_debugger_route(&config, RuntimeSurfaceKind::WindowedGame);
        // TODO: pass actual context ID once context registry supports it
        #[cfg(feature = "lua")]
        let lua_runtime = LuaRuntime::new(0)?;

        // Register deferred capture hook for framebuffer readback if debugger
        // is enabled. The hook is invoked from the IPC thread, so it signals
        // the main thread (via condvar) to perform readback during
        // `swap_buffers()`.
        let (capture_dimensions, deferred_capture) = if let Some(ref route_id) = debugger_route {
            let dims = Arc::new(AtomicU64::new(
                ((config.width as u64) << 32) | config.height as u64,
            ));
            let deferred = debugger::new_deferred_capture();
            debugger::register_deferred_capture_hook_for_route(route_id.clone(), deferred.clone());
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
            window_resized_events: Events::new(),
            #[cfg(feature = "lua")]
            lua_runtime,
            #[cfg(feature = "lua")]
            lua_watcher: None,
            platform: Some(native_runtime.platform),
            render_backend: Some(render_backend),
            input_manager: InputManager::default(),
            sprite_batch: Some(sprite_batch),
            asset_server: Some(asset_server),
            ui_render_system: Some(UiRenderSystem::new()),
            text_batch: Some(TextBatch::new()),
            renderer_3d: Some(renderer_3d),
            immediate_state: None,
            audio_manager,
            capture_dimensions,
            deferred_capture,
            design_resolution: window_size,
            viewport_scale_mode: ViewportScaleMode::Stretch,
            render_viewport,
            bound_render_target_viewport: None,
            render_target_attachment_owners: HashMap::new(),
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
        self.context.window_size()
    }

    #[cfg(feature = "native")]
    pub(crate) fn sync_render_viewport(
        &mut self,
        logical_size: (u32, u32),
        framebuffer_size: (u32, u32),
    ) {
        self.render_viewport = compute_render_viewport(
            framebuffer_size,
            self.design_resolution,
            self.viewport_scale_mode,
        );
        self.context.set_window_size(logical_size);
        self.apply_render_viewport();
    }

    #[cfg(feature = "native")]
    pub(crate) fn apply_render_viewport(&mut self) {
        let viewport = self
            .bound_render_target_viewport
            .map(|(_, viewport)| viewport)
            .unwrap_or(self.render_viewport);

        if let Some(batch) = self.sprite_batch.as_mut() {
            batch.set_viewport(viewport);
        }
        if let Some(renderer) = self.renderer_3d.as_mut() {
            renderer.resize(viewport.width, viewport.height);
            renderer.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        }
    }

    /// Executes a Lua script in the embedded runtime.
    ///
    /// # Errors
    ///
    /// Returns a `GoudError` if the script has syntax or runtime errors.
    #[cfg(feature = "lua")]
    pub fn execute_lua(&self, source: &str, name: &str) -> GoudResult<()> {
        self.lua_runtime.execute_script(source, name)
    }

    /// Calls a Lua global function by name, if it exists.
    ///
    /// If the global is not defined this is a no-op and returns `Ok(())`.
    #[cfg(feature = "lua")]
    pub fn call_lua_global(&self, name: &str) -> GoudResult<()> {
        self.lua_runtime.call_global(name)
    }

    /// Calls `on_update(dt)` if defined in the Lua environment.
    #[cfg(feature = "lua")]
    pub fn call_lua_update(&self, dt: f32) -> GoudResult<()> {
        self.lua_runtime.call_update(dt)
    }

    /// Checks if a global Lua function exists.
    #[cfg(feature = "lua")]
    pub fn has_lua_global(&self, name: &str) -> bool {
        self.lua_runtime.has_global(name)
    }

    /// Starts watching a directory for `.lua` file changes.
    ///
    /// Changed scripts will be automatically re-executed when
    /// [`process_lua_hot_reload`](Self::process_lua_hot_reload) is called
    /// each frame.
    ///
    /// # Errors
    ///
    /// Returns an error if the file-system watcher cannot be initialised.
    #[cfg(all(feature = "lua", feature = "native"))]
    pub fn watch_lua_dir(&mut self, path: impl AsRef<std::path::Path>) -> GoudResult<()> {
        let watcher = lua_hot_reload::LuaScriptWatcher::new(path.as_ref())?;
        self.lua_watcher = Some(watcher);
        Ok(())
    }

    /// Polls the Lua hot-reload watcher and re-executes any changed scripts.
    ///
    /// Call this once per frame (e.g., at the start of the update loop).
    /// If no watcher is active this is a no-op.
    #[cfg(all(feature = "lua", feature = "native"))]
    pub fn process_lua_hot_reload(&mut self) {
        let changed = match self.lua_watcher.as_mut() {
            Some(w) => w.poll_changes(),
            None => return,
        };

        for path in changed {
            let source = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("Failed to read changed Lua file {:?}: {}", path, e);
                    continue;
                }
            };
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("<unknown>");
            if let Err(e) = self.lua_runtime.reload_script(&source, name) {
                log::error!("Lua hot-reload error for {:?}: {}", path, e);
            }
        }
    }
}

impl Drop for GoudGame {
    fn drop(&mut self) {
        if let Some(route_id) = self.debugger_route.take() {
            debugger::unregister_capture_hook_for_route(&route_id);
            debugger::unregister_context(crate::core::context_id::GoudContextId::new(
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
