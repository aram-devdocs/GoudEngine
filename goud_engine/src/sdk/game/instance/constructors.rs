use crate::core::error::GoudResult;
use crate::sdk::engine_config::EngineConfig;
use crate::sdk::game_config::GameConfig;

#[cfg(feature = "native")]
use crate::ecs::InputManager;

use super::GoudGame;

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
        let mut debug_overlay =
            crate::sdk::debug_overlay::DebugOverlay::new(config.fps_update_interval);
        debug_overlay.set_enabled(config.show_fps_overlay);
        Ok(Self {
            scene_manager: crate::context_registry::scene::SceneManager::new(),
            config,
            context: crate::sdk::game_config::GameContext::new(window_size),
            initialized: false,
            debug_overlay,
            providers: crate::core::providers::ProviderRegistry::default(),
            last_transition_complete: None,
            ui_manager: crate::ui::UiManager::new(),
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
        let mut debug_overlay =
            crate::sdk::debug_overlay::DebugOverlay::new(config.fps_update_interval);
        debug_overlay.set_enabled(config.show_fps_overlay);

        let audio_manager = crate::assets::AudioManager::new().ok();

        Ok(Self {
            scene_manager: crate::context_registry::scene::SceneManager::new(),
            config,
            context: crate::sdk::game_config::GameContext::new(window_size),
            initialized: false,
            debug_overlay,
            providers: crate::core::providers::ProviderRegistry::default(),
            last_transition_complete: None,
            ui_manager: crate::ui::UiManager::new(),
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
}
