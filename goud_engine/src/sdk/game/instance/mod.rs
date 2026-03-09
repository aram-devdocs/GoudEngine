//! [`GoudGame`] struct definition, construction, and core API.

use crate::context_registry::scene::SceneManager;
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

mod accessors;
mod constructors;
mod runtime;
mod scene_management;
mod traits_impls;
mod world_access;

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
