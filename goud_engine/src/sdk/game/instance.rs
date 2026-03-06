//! [`GoudGame`] struct definition, construction, and core API.

use crate::core::error::GoudResult;
use crate::ecs::{Component, Entity, World};

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

use crate::sdk::entity_builder::EntityBuilder;
use crate::sdk::game_config::{GameConfig, GameContext};

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
    /// The ECS world containing all game state.
    pub(crate) world: World,

    /// Game configuration.
    pub(crate) config: GameConfig,

    /// Runtime context for the game loop.
    pub(crate) context: GameContext,

    /// Whether the game has been initialized.
    pub(crate) initialized: bool,

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
}

impl GoudGame {
    /// Creates a new game instance with the given configuration.
    ///
    /// This creates a headless game instance suitable for testing and
    /// non-graphical use. For a windowed game with rendering, use
    /// [`with_platform`](Self::with_platform) instead.
    pub fn new(config: GameConfig) -> GoudResult<Self> {
        let window_size = (config.width, config.height);
        Ok(Self {
            world: World::new(),
            config,
            context: GameContext::new(window_size),
            initialized: false,
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

        Ok(Self {
            world: World::new(),
            config,
            context: GameContext::new(window_size),
            initialized: false,
            platform: Some(Box::new(platform)),
            render_backend: None,
            input_manager: InputManager::default(),
            sprite_batch: None,
            asset_server: None,
            renderer_3d: None,
            immediate_state: None,
        })
    }

    /// Returns a reference to the ECS world.
    #[inline]
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Returns a mutable reference to the ECS world.
    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Creates an entity builder for fluent entity creation.
    #[inline]
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        EntityBuilder::new(&mut self.world)
    }

    /// Spawns an empty entity with no components.
    #[inline]
    pub fn spawn_empty(&mut self) -> Entity {
        self.world.spawn_empty()
    }

    /// Spawns multiple empty entities at once.
    #[inline]
    pub fn spawn_batch(&mut self, count: usize) -> Vec<Entity> {
        self.world.spawn_batch(count)
    }

    /// Despawns an entity and removes all its components.
    #[inline]
    pub fn despawn(&mut self, entity: Entity) -> bool {
        self.world.despawn(entity)
    }

    /// Gets a reference to a component on an entity.
    #[inline]
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.world.get::<T>(entity)
    }

    /// Gets a mutable reference to a component on an entity.
    #[inline]
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.world.get_mut::<T>(entity)
    }

    /// Adds or replaces a component on an entity.
    #[inline]
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        self.world.insert(entity, component);
    }

    /// Removes a component from an entity.
    #[inline]
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Option<T> {
        self.world.remove::<T>(entity)
    }

    /// Checks if an entity has a specific component.
    #[inline]
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        self.world.has::<T>(entity)
    }

    /// Returns the number of entities in the world.
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.world.entity_count()
    }

    /// Checks if an entity is alive.
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.world.is_alive(entity)
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

    /// Runs the game loop with the given update callback.
    pub fn run<F>(&mut self, mut update: F)
    where
        F: FnMut(&mut GameContext, &mut World),
    {
        self.initialized = true;

        // Simple game loop (actual implementation would use GLFW/window events)
        let frame_time = if self.config.target_fps > 0 {
            1.0 / self.config.target_fps as f32
        } else {
            1.0 / 60.0 // Default to 60 FPS for simulation
        };

        // For now, just run a few frames to demonstrate the API
        // Real implementation would integrate with windowing system
        while self.context.is_running() {
            self.context.update(frame_time);
            update(&mut self.context, &mut self.world);

            // Safety: Limit iterations in tests/examples without actual window
            if self.context.frame_count() > 10000 {
                break;
            }
        }
    }

    /// Runs a single frame update.
    pub fn update_frame<F>(&mut self, delta_time: f32, mut update: F)
    where
        F: FnMut(&mut GameContext, &mut World),
    {
        self.context.update(delta_time);
        update(&mut self.context, &mut self.world);
    }

    /// Returns the current frame count.
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.context.frame_count()
    }

    /// Returns the total time elapsed since game start.
    #[inline]
    pub fn total_time(&self) -> f32 {
        self.context.total_time()
    }

    /// Returns the current FPS.
    #[inline]
    pub fn fps(&self) -> f32 {
        self.context.fps()
    }

    /// Returns true if the game has been initialized.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized
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
            .field("entity_count", &self.world.entity_count())
            .field("initialized", &self.initialized)
            .finish()
    }
}
