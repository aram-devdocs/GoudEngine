//! [`GoudGame`] struct definition, construction, and core API.

mod ecs_scene;
mod runtime;

use crate::context_registry::scene::SceneManager;
use crate::core::error::GoudResult;
use crate::core::providers::ProviderRegistry;
use crate::sdk::debug_overlay::DebugOverlay;
use crate::sdk::engine_config::EngineConfig;
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

    /// 3D renderer for primitives, lighting, and camera.
    #[cfg(feature = "native")]
    pub(crate) renderer_3d: Option<Renderer3D>,

    /// GPU resources for immediate-mode sprite/quad rendering.
    #[cfg(feature = "native")]
    pub(crate) immediate_state: Option<crate::sdk::rendering::ImmediateRenderState>,

    /// Centralized audio playback manager.
    #[cfg(feature = "native")]
    pub(crate) audio_manager: Option<crate::assets::AudioManager>,
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
        Ok(Self {
            scene_manager: SceneManager::new(),
            config,
            context: GameContext::new(window_size),
            initialized: false,
            debug_overlay,
            providers: ProviderRegistry::default(),
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
            renderer_3d: None,
            #[cfg(feature = "native")]
            immediate_state: None,
            #[cfg(feature = "native")]
            audio_manager: None,
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
        init_engine_diagnostics(&config);

        use crate::libs::platform::WindowConfig;
        use crate::libs::platform::glfw_platform::GlfwPlatform;

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

        let audio_manager = crate::assets::AudioManager::new().ok();

        Ok(Self {
            scene_manager: SceneManager::new(),
            config,
            context: GameContext::new(window_size),
            initialized: false,
            debug_overlay,
            providers: ProviderRegistry::default(),
            last_transition_complete: None,
            ui_manager: UiManager::new(),
            platform: Some(Box::new(platform)),
            render_backend: None,
            input_manager: InputManager::default(),
            sprite_batch: None,
            asset_server: None,
            renderer_3d: None,
            immediate_state: None,
            audio_manager,
        })
    }

    /// Creates a headless game from an [`EngineConfig`] builder.
    pub fn from_engine_config(config: EngineConfig) -> GoudResult<Self> {
        let (game_config, providers) = config.build();
        let mut game = Self::new(game_config)?;
        game.providers = providers;
        Ok(game)
    }

    /// Creates a windowed game from an [`EngineConfig`] builder.
    #[cfg(feature = "native")]
    pub fn from_engine_config_with_platform(config: EngineConfig) -> GoudResult<Self> {
        let (game_config, providers) = config.build();
        let mut game = Self::with_platform(game_config)?;
        game.providers = providers;
        Ok(game)
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

    /// Returns true if the game has been initialized.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns a reference to the provider registry.
    #[inline]
    pub fn providers(&self) -> &ProviderRegistry {
        &self.providers
    }

    /// Returns a reference to the audio manager, if available.
    #[cfg(feature = "native")]
    #[inline]
    pub fn audio_manager(&self) -> Option<&crate::assets::AudioManager> {
        self.audio_manager.as_ref()
    }

    /// Returns a mutable reference to the audio manager, if available.
    #[cfg(feature = "native")]
    #[inline]
    pub fn audio_manager_mut(&mut self) -> Option<&mut crate::assets::AudioManager> {
        self.audio_manager.as_mut()
    }

    /// Returns a reference to the UI manager.
    #[inline]
    pub fn ui_manager(&self) -> &UiManager {
        &self.ui_manager
    }

    /// Returns a mutable reference to the UI manager.
    #[inline]
    pub fn ui_manager_mut(&mut self) -> &mut UiManager {
        &mut self.ui_manager
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
