//! [`GoudGame`] struct definition, construction, and core API.

mod capture;
mod debugger_frame;
mod ecs_scene;

#[cfg(feature = "native")]
use std::sync::atomic::AtomicU64;
#[cfg(feature = "native")]
use std::sync::{Arc, Condvar, Mutex};

use crate::context_registry::scene::SceneManager;
use crate::core::debugger::{self, RuntimeRouteId, RuntimeSurfaceKind};
use crate::core::error::GoudResult;
use crate::core::providers::types::DebugShape;
use crate::core::providers::ProviderRegistry;
use crate::sdk::debug_overlay::DebugOverlay;
use crate::sdk::game_config::{GameConfig, GameContext};
use crate::ui::UiManager;

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
pub(crate) use capture::{DeferredCapture, DeferredCaptureState};

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
        use crate::assets::AssetServer;
        use crate::libs::platform::native_runtime::create_native_runtime;
        init_engine_diagnostics(&config);
        use crate::libs::platform::WindowConfig;

        let window_config = WindowConfig {
            width: config.width,
            height: config.height,
            title: config.title.clone(),
            vsync: config.vsync,
            resizable: config.resizable,
        };

        let native_runtime =
            create_native_runtime(&window_config, config.window_backend, config.render_backend)?;
        let window_size = (config.width, config.height);
        let mut debug_overlay = DebugOverlay::new(config.fps_update_interval);
        debug_overlay.set_enabled(config.show_fps_overlay);
        let render_backend = native_runtime.render_backend;
        let renderer_3d = Renderer3D::new(
            Box::new(render_backend.clone()),
            config.width,
            config.height,
        )
        .map_err(crate::core::error::GoudError::InitializationFailed)?;
        let sprite_batch = SpriteBatch::new(render_backend.clone(), SpriteBatchConfig::default())?;

        let audio_manager = crate::assets::AudioManager::new().ok();
        let debugger_route =
            Self::register_debugger_route(&config, RuntimeSurfaceKind::WindowedGame);

        // Register deferred capture hook for framebuffer readback if debugger
        // is enabled. The hook is invoked from the IPC thread, so it signals
        // the main thread (via condvar) to perform readback during
        // `swap_buffers()`.
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
            platform: Some(native_runtime.platform),
            render_backend: Some(render_backend),
            input_manager: InputManager::default(),
            sprite_batch: Some(sprite_batch),
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

#[cfg(test)]
mod tests {
    #[cfg(all(feature = "native", not(target_os = "macos")))]
    use std::thread;
    #[cfg(all(feature = "native", not(target_os = "macos")))]
    use std::time::Duration;

    #[cfg(all(feature = "native", not(target_os = "macos")))]
    use crate::sdk::{RenderBackendKind, WindowBackendKind};

    #[cfg(all(feature = "native", not(target_os = "macos")))]
    use super::*;

    // winit requires the macOS main thread, which the unit-test harness does not provide.
    #[cfg(all(feature = "native", not(target_os = "macos")))]
    #[test]
    fn test_with_platform_default_native_stack_initializes_renderers_and_readback() {
        let mut game = GoudGame::with_platform(
            GameConfig::default().with_title("native-stack-smoke-instance-test"),
        )
        .expect("default native stack should initialize");

        assert_eq!(game.config.render_backend, RenderBackendKind::Wgpu);
        assert_eq!(game.config.window_backend, WindowBackendKind::Winit);
        assert!(game.has_platform());
        assert!(game.has_2d_renderer());
        assert!(game.has_3d_renderer());

        let (fb_width, fb_height) = game.get_framebuffer_size();
        assert!(fb_width > 0);
        assert!(fb_height > 0);

        assert!(game.begin_render());
        game.clear(0.15, 0.25, 0.35, 1.0);
        assert!(game.begin_2d_render().is_ok());
        assert!(game.draw_quad(32.0, 32.0, 24.0, 24.0, 1.0, 0.0, 0.0, 1.0));
        assert!(game.draw_text(
            "/Users/aramhammoudeh/dev/game/GoudEngine-issue-280/goud_engine/test_assets/fonts/test_font.ttf",
            "wgpu",
            12.0,
            18.0,
            14.0,
            0.0,
            1.0,
            1.0,
            1.0,
            1.0,
            1.0,
        ));
        assert!(game.end_2d_render().is_ok());
        let cube = game.create_cube(0, 1.0, 1.0, 1.0);
        assert_ne!(cube, u32::MAX);
        assert!(game.set_object_position(cube, 0.0, 0.0, 0.0));
        assert!(game.configure_grid(true, 4.0, 4));
        assert!(game.render());
        assert!(game.end_render());

        let readback = game
            .read_default_framebuffer_rgba8()
            .expect("framebuffer readback should succeed");
        assert_eq!(readback.len(), (fb_width * fb_height * 4) as usize);

        game.set_window_size(176, 132)
            .expect("window resize request should succeed");
        for _ in 0..120 {
            game.poll_events()
                .expect("poll after resize should succeed");
            if game.get_window_size() == (176, 132) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(game.get_window_size(), (176, 132));
        let (resized_fb_width, resized_fb_height) = game.get_framebuffer_size();
        assert!(resized_fb_width > 0);
        assert!(resized_fb_height > 0);
        let resized_readback = game
            .read_default_framebuffer_rgba8()
            .expect("resized framebuffer readback should succeed");
        assert_eq!(
            resized_readback.len(),
            (resized_fb_width * resized_fb_height * 4) as usize
        );
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
