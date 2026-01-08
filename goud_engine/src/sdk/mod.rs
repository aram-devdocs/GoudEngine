//! # Rust Native SDK for GoudEngine
//!
//! This module provides a high-level, ergonomic Rust API for game development.
//! Unlike the FFI layer which is designed for cross-language bindings, this SDK
//! is pure Rust with zero FFI overhead - ideal for Rust-native game development.
//!
//! ## Architecture Philosophy
//!
//! The SDK follows a "Rust-first" design principle:
//!
//! - **All game logic lives in Rust**: Components, systems, and game behavior
//!   are implemented in Rust and exposed through this SDK
//! - **Zero-overhead abstractions**: No FFI marshalling, no runtime type checks
//! - **Type-safe**: Full Rust type safety with compile-time guarantees
//! - **Ergonomic**: Builder patterns, fluent APIs, and sensible defaults
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use goud_engine::sdk::{GoudGame, GameConfig};
//! use goud_engine::sdk::components::{Transform2D, Sprite};
//! use goud_engine::core::math::Vec2;
//!
//! fn main() {
//!     // Create game instance
//!     let mut game = GoudGame::new(GameConfig {
//!         title: "My Game".to_string(),
//!         width: 800,
//!         height: 600,
//!         ..Default::default()
//!     }).expect("Failed to create game");
//!
//!     // Spawn entities with fluent builder API
//!     let player = game.spawn()
//!         .with(Transform2D::from_position(Vec2::new(100.0, 100.0)))
//!         .with(Sprite::default())
//!         .build();
//!
//!     // Run game loop
//!     game.run(|ctx| {
//!         // Update logic here
//!         ctx.delta_time(); // Get frame delta
//!         ctx.input();      // Access input state
//!     });
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`components`]: Re-exports of all ECS components (Transform2D, Sprite, etc.)
//! - [`GoudGame`]: High-level game abstraction managing world, window, and loop
//! - [`EntityBuilder`]: Fluent builder for spawning entities with components
//! - [`GameContext`]: Runtime context passed to update callbacks
//!
//! ## Comparison with FFI Layer
//!
//! | Feature | SDK (this module) | FFI Layer |
//! |---------|-------------------|-----------|
//! | Target | Rust games | C#/Python/etc |
//! | Overhead | Zero | Marshalling cost |
//! | Type Safety | Compile-time | Runtime checks |
//! | API Style | Idiomatic Rust | C-compatible |

pub mod components;

// Re-export commonly used types for convenience
pub use crate::core::error::{GoudError, GoudResult};
pub use crate::core::math::{Color, Rect, Vec2, Vec3, Vec4};
pub use crate::ecs::{Component, Entity, EntityAllocator, SparseSet, World};

// Re-export components module contents at sdk level for convenience
// Note: We explicitly re-export to avoid shadowing issues
pub use components::{
    // Propagation
    propagate_transforms,
    propagate_transforms_2d,
    // Audio
    AttenuationModel,
    AudioChannel,
    AudioSource,
    // Hierarchy
    Children,
    // Physics
    Collider,
    ColliderShape,
    // Transforms
    GlobalTransform,
    GlobalTransform2D,
    Mat3x3,
    Name,
    Parent,
    RigidBody,
    RigidBodyType,
    // Rendering
    Sprite,
    Transform,
    Transform2D,
};

// =============================================================================
// Game Configuration
// =============================================================================

/// Configuration for creating a GoudGame instance.
///
/// This struct holds all the settings needed to initialize the game engine,
/// including window properties, rendering options, and engine settings.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::GameConfig;
///
/// let config = GameConfig {
///     title: "My Awesome Game".to_string(),
///     width: 1280,
///     height: 720,
///     vsync: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Window title displayed in the title bar.
    pub title: String,

    /// Window width in pixels.
    pub width: u32,

    /// Window height in pixels.
    pub height: u32,

    /// Enable vertical sync to prevent screen tearing.
    pub vsync: bool,

    /// Enable fullscreen mode.
    pub fullscreen: bool,

    /// Enable window resizing.
    pub resizable: bool,

    /// Target frames per second (0 = unlimited).
    pub target_fps: u32,

    /// Enable debug rendering (collision boxes, etc.).
    pub debug_rendering: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            title: "GoudEngine Game".to_string(),
            width: 800,
            height: 600,
            vsync: true,
            fullscreen: false,
            resizable: true,
            target_fps: 60,
            debug_rendering: false,
        }
    }
}

impl GameConfig {
    /// Creates a new configuration with the given title and dimensions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::sdk::GameConfig;
    ///
    /// let config = GameConfig::new("My Game", 800, 600);
    /// ```
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            ..Default::default()
        }
    }

    /// Sets the window title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the window dimensions.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Enables or disables vsync.
    pub fn with_vsync(mut self, enabled: bool) -> Self {
        self.vsync = enabled;
        self
    }

    /// Enables or disables fullscreen mode.
    pub fn with_fullscreen(mut self, enabled: bool) -> Self {
        self.fullscreen = enabled;
        self
    }

    /// Sets the target FPS (0 for unlimited).
    pub fn with_target_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }
}

// =============================================================================
// Game Context (passed to update callback)
// =============================================================================

/// Runtime context passed to the game update callback.
///
/// This struct provides access to frame timing, input state, and other
/// runtime information needed during the game loop.
///
/// # Example
///
/// ```rust,ignore
/// game.run(|ctx| {
///     let dt = ctx.delta_time();
///     let fps = ctx.fps();
///     
///     // Move something based on time
///     position.x += velocity * dt;
/// });
/// ```
#[derive(Debug)]
pub struct GameContext {
    /// Time elapsed since last frame in seconds.
    delta_time: f32,

    /// Total time elapsed since game start in seconds.
    total_time: f32,

    /// Current frames per second.
    fps: f32,

    /// Current frame number.
    frame_count: u64,

    /// Window dimensions.
    window_size: (u32, u32),

    /// Whether the game should continue running.
    running: bool,
}

impl GameContext {
    /// Creates a new game context with default values.
    fn new(window_size: (u32, u32)) -> Self {
        Self {
            delta_time: 0.0,
            total_time: 0.0,
            fps: 0.0,
            frame_count: 0,
            window_size,
            running: true,
        }
    }

    /// Returns the time elapsed since the last frame in seconds.
    ///
    /// Use this for frame-rate independent movement and animations.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Move at 100 pixels per second regardless of frame rate
    /// position.x += 100.0 * ctx.delta_time();
    /// ```
    #[inline]
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    /// Returns the total time elapsed since game start in seconds.
    #[inline]
    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    /// Returns the current frames per second.
    #[inline]
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Returns the current frame number (0-indexed).
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Returns the window dimensions as (width, height).
    #[inline]
    pub fn window_size(&self) -> (u32, u32) {
        self.window_size
    }

    /// Returns the window width in pixels.
    #[inline]
    pub fn window_width(&self) -> u32 {
        self.window_size.0
    }

    /// Returns the window height in pixels.
    #[inline]
    pub fn window_height(&self) -> u32 {
        self.window_size.1
    }

    /// Returns true if the game is still running.
    #[inline]
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Signals the game to exit after the current frame.
    #[inline]
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Updates the context for a new frame.
    fn update(&mut self, delta_time: f32) {
        self.delta_time = delta_time;
        self.total_time += delta_time;
        self.frame_count += 1;

        // Simple FPS calculation (could be smoothed)
        if delta_time > 0.0 {
            self.fps = 1.0 / delta_time;
        }
    }
}

// =============================================================================
// Entity Builder
// =============================================================================

/// A fluent builder for creating entities with components.
///
/// The `EntityBuilder` provides a convenient way to spawn entities and
/// attach multiple components in a single expression chain.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::EntityBuilder;
/// use goud_engine::sdk::components::{Transform2D, Sprite};
/// use goud_engine::ecs::World;
/// use goud_engine::core::math::Vec2;
/// use goud_engine::assets::AssetServer;
///
/// let mut world = World::new();
/// let mut assets = AssetServer::new();
///
/// // Create a fully configured entity
/// let entity = EntityBuilder::new(&mut world)
///     .with(Transform2D::from_position(Vec2::new(100.0, 200.0)))
///     .build();
/// ```
pub struct EntityBuilder<'w> {
    /// Reference to the world where the entity will be created.
    world: &'w mut World,

    /// The entity being built.
    entity: Entity,
}

impl<'w> EntityBuilder<'w> {
    /// Creates a new entity builder.
    ///
    /// This immediately spawns an empty entity in the world.
    /// Use the `with()` method to add components.
    pub fn new(world: &'w mut World) -> Self {
        let entity = world.spawn_empty();
        Self { world, entity }
    }

    /// Adds a component to the entity.
    ///
    /// This is the primary method for attaching components to the entity
    /// being built. Components can be chained.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::sdk::EntityBuilder;
    /// use goud_engine::sdk::components::Transform2D;
    /// use goud_engine::ecs::World;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let mut world = World::new();
    /// let entity = EntityBuilder::new(&mut world)
    ///     .with(Transform2D::from_position(Vec2::new(10.0, 20.0)))
    ///     .build();
    /// ```
    pub fn with<T: Component>(self, component: T) -> Self {
        self.world.insert(self.entity, component);
        self
    }

    /// Conditionally adds a component to the entity.
    ///
    /// The component is only added if `condition` is true.
    /// Useful for optional components based on game state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::sdk::EntityBuilder;
    /// use goud_engine::sdk::components::Transform2D;
    /// use goud_engine::ecs::World;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let mut world = World::new();
    /// let has_physics = true;
    ///
    /// let entity = EntityBuilder::new(&mut world)
    ///     .with(Transform2D::default())
    ///     .with_if(has_physics, Transform2D::from_scale(Vec2::one()))
    ///     .build();
    /// ```
    pub fn with_if<T: Component>(self, condition: bool, component: T) -> Self {
        if condition {
            self.world.insert(self.entity, component);
        }
        self
    }

    /// Conditionally adds a component using a closure.
    ///
    /// The closure is only called if `condition` is true, avoiding
    /// unnecessary component construction.
    pub fn with_if_else<T: Component>(
        self,
        condition: bool,
        if_true: impl FnOnce() -> T,
        if_false: impl FnOnce() -> T,
    ) -> Self {
        let component = if condition { if_true() } else { if_false() };
        self.world.insert(self.entity, component);
        self
    }

    /// Finalizes the builder and returns the created entity.
    ///
    /// After calling `build()`, the builder is consumed and the entity
    /// is ready for use.
    pub fn build(self) -> Entity {
        self.entity
    }

    /// Returns a reference to the entity being built.
    ///
    /// Useful for accessing the entity ID before finalizing.
    #[inline]
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Provides mutable access to the world during building.
    ///
    /// Use this for advanced scenarios where you need to perform
    /// world operations while building an entity.
    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }
}

// =============================================================================
// GoudGame - High-Level Game Abstraction
// =============================================================================

/// The main game instance managing the ECS world and game loop.
///
/// `GoudGame` is the primary entry point for Rust-native game development.
/// It manages the ECS world, handles the game loop, and provides convenient
/// methods for common game operations.
///
/// # Lifecycle
///
/// 1. Create with `GoudGame::new(config)`
/// 2. Set up initial game state (spawn entities, load assets)
/// 3. Run the game loop with `game.run(callback)`
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::{GoudGame, GameConfig};
/// use goud_engine::sdk::components::Transform2D;
/// use goud_engine::core::math::Vec2;
///
/// // Create game
/// let mut game = GoudGame::new(GameConfig::default()).unwrap();
///
/// // Spawn player
/// let player = game.spawn()
///     .with(Transform2D::from_position(Vec2::new(400.0, 300.0)))
///     .build();
///
/// // Game loop would be:
/// // game.run(|ctx| { /* update logic */ });
/// ```
pub struct GoudGame {
    /// The ECS world containing all game state.
    world: World,

    /// Game configuration.
    config: GameConfig,

    /// Runtime context for the game loop.
    context: GameContext,

    /// Whether the game has been initialized.
    initialized: bool,
}

impl GoudGame {
    /// Creates a new game instance with the given configuration.
    ///
    /// This initializes the ECS world but does not open a window or
    /// start the game loop. Call `run()` to begin.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails (e.g., graphics not available).
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::sdk::{GoudGame, GameConfig};
    ///
    /// let game = GoudGame::new(GameConfig {
    ///     title: "My Game".to_string(),
    ///     width: 1280,
    ///     height: 720,
    ///     ..Default::default()
    /// });
    /// ```
    pub fn new(config: GameConfig) -> GoudResult<Self> {
        let window_size = (config.width, config.height);
        Ok(Self {
            world: World::new(),
            config,
            context: GameContext::new(window_size),
            initialized: false,
        })
    }

    /// Creates a game with default configuration.
    ///
    /// Equivalent to `GoudGame::new(GameConfig::default())`.
    pub fn default_game() -> GoudResult<Self> {
        Self::new(GameConfig::default())
    }

    // =========================================================================
    // World Access
    // =========================================================================

    /// Returns a reference to the ECS world.
    ///
    /// Use this for read-only queries and iteration.
    #[inline]
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Returns a mutable reference to the ECS world.
    ///
    /// Use this for spawning entities, adding components, and modifications.
    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    // =========================================================================
    // Entity Spawning
    // =========================================================================

    /// Creates an entity builder for fluent entity creation.
    ///
    /// This is the recommended way to spawn entities with multiple components.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::sdk::{GoudGame, GameConfig};
    /// use goud_engine::sdk::components::Transform2D;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let mut game = GoudGame::new(GameConfig::default()).unwrap();
    ///
    /// let entity = game.spawn()
    ///     .with(Transform2D::from_position(Vec2::new(100.0, 100.0)))
    ///     .build();
    /// ```
    #[inline]
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        EntityBuilder::new(&mut self.world)
    }

    /// Spawns an empty entity with no components.
    ///
    /// Components can be added later with `world_mut().insert()`.
    #[inline]
    pub fn spawn_empty(&mut self) -> Entity {
        self.world.spawn_empty()
    }

    /// Spawns multiple empty entities at once.
    ///
    /// More efficient than calling `spawn_empty()` in a loop.
    #[inline]
    pub fn spawn_batch(&mut self, count: usize) -> Vec<Entity> {
        self.world.spawn_batch(count)
    }

    /// Despawns an entity and all its components.
    ///
    /// Returns true if the entity existed and was despawned.
    #[inline]
    pub fn despawn(&mut self, entity: Entity) -> bool {
        self.world.despawn(entity)
    }

    // =========================================================================
    // Component Access (convenience methods)
    // =========================================================================

    /// Gets a reference to a component on an entity.
    ///
    /// Returns `None` if the entity doesn't exist or doesn't have the component.
    #[inline]
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.world.get::<T>(entity)
    }

    /// Gets a mutable reference to a component on an entity.
    #[inline]
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.world.get_mut::<T>(entity)
    }

    /// Adds a component to an entity.
    ///
    /// If the entity already has a component of this type, it's replaced.
    #[inline]
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        self.world.insert(entity, component);
    }

    /// Removes a component from an entity.
    ///
    /// Returns the removed component, or `None` if it didn't exist.
    #[inline]
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Option<T> {
        self.world.remove::<T>(entity)
    }

    /// Checks if an entity has a specific component.
    #[inline]
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        self.world.has::<T>(entity)
    }

    // =========================================================================
    // Entity Queries
    // =========================================================================

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

    // =========================================================================
    // Configuration Access
    // =========================================================================

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

    // =========================================================================
    // Game Loop
    // =========================================================================

    /// Runs the game loop with the given update callback.
    ///
    /// The callback is called once per frame with a `GameContext` that provides
    /// timing information and input state. The loop continues until the context's
    /// `quit()` method is called or the window is closed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// game.run(|ctx| {
    ///     // Update game logic
    ///     if ctx.frame_count() > 1000 {
    ///         ctx.quit();
    ///     }
    /// });
    /// ```
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
    ///
    /// Useful for testing or manual frame stepping.
    pub fn update_frame<F>(&mut self, delta_time: f32, mut update: F)
    where
        F: FnMut(&mut GameContext, &mut World),
    {
        self.context.update(delta_time);
        update(&mut self.context, &mut self.world);
    }

    // =========================================================================
    // Statistics
    // =========================================================================

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

// =============================================================================
// Prelude - Convenient imports
// =============================================================================

/// Prelude module for convenient imports.
///
/// ```rust
/// use goud_engine::sdk::prelude::*;
/// ```
pub mod prelude {
    // Math types
    pub use crate::core::math::{Color, Rect, Vec2, Vec3, Vec4};

    // ECS core
    pub use crate::ecs::{Component, Entity, World};

    // SDK types
    pub use super::{EntityBuilder, GameConfig, GameContext, GoudError, GoudGame, GoudResult};

    // Components - explicitly list to avoid shadowing
    pub use super::components::{
        // Propagation
        propagate_transforms,
        propagate_transforms_2d,
        // Audio
        AttenuationModel,
        AudioChannel,
        AudioSource,
        // Hierarchy
        Children,
        // Physics
        Collider,
        ColliderShape,
        // Transforms
        GlobalTransform,
        GlobalTransform2D,
        Mat3x3,
        Name,
        Parent,
        RigidBody,
        RigidBodyType,
        // Rendering
        Sprite,
        Transform,
        Transform2D,
    };
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // GameConfig Tests
    // =========================================================================

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert_eq!(config.title, "GoudEngine Game");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(config.vsync);
        assert!(!config.fullscreen);
    }

    #[test]
    fn test_game_config_new() {
        let config = GameConfig::new("Test Game", 1920, 1080);
        assert_eq!(config.title, "Test Game");
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn test_game_config_builder() {
        let config = GameConfig::default()
            .with_title("Builder Game")
            .with_size(640, 480)
            .with_vsync(false)
            .with_fullscreen(true)
            .with_target_fps(144);

        assert_eq!(config.title, "Builder Game");
        assert_eq!(config.width, 640);
        assert_eq!(config.height, 480);
        assert!(!config.vsync);
        assert!(config.fullscreen);
        assert_eq!(config.target_fps, 144);
    }

    // =========================================================================
    // GameContext Tests
    // =========================================================================

    #[test]
    fn test_game_context_new() {
        let ctx = GameContext::new((800, 600));
        assert_eq!(ctx.delta_time(), 0.0);
        assert_eq!(ctx.total_time(), 0.0);
        assert_eq!(ctx.frame_count(), 0);
        assert_eq!(ctx.window_size(), (800, 600));
        assert!(ctx.is_running());
    }

    #[test]
    fn test_game_context_update() {
        let mut ctx = GameContext::new((800, 600));
        ctx.update(0.016); // ~60 FPS

        assert!((ctx.delta_time() - 0.016).abs() < 0.001);
        assert!((ctx.total_time() - 0.016).abs() < 0.001);
        assert_eq!(ctx.frame_count(), 1);
        assert!((ctx.fps() - 62.5).abs() < 1.0);
    }

    #[test]
    fn test_game_context_quit() {
        let mut ctx = GameContext::new((800, 600));
        assert!(ctx.is_running());

        ctx.quit();
        assert!(!ctx.is_running());
    }

    // =========================================================================
    // EntityBuilder Tests
    // =========================================================================

    #[test]
    fn test_entity_builder_basic() {
        let mut world = World::new();
        let entity = EntityBuilder::new(&mut world).build();

        assert!(world.is_alive(entity));
    }

    #[test]
    fn test_entity_builder_with_component() {
        let mut world = World::new();
        let entity = EntityBuilder::new(&mut world)
            .with(Transform2D::from_position(Vec2::new(10.0, 20.0)))
            .build();

        assert!(world.has::<Transform2D>(entity));

        let transform = world.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.position, Vec2::new(10.0, 20.0));
    }

    #[test]
    fn test_entity_builder_with_multiple_components() {
        let mut world = World::new();
        let entity = EntityBuilder::new(&mut world)
            .with(Transform2D::from_position(Vec2::new(10.0, 20.0)))
            .with(GlobalTransform2D::IDENTITY)
            .build();

        assert!(world.has::<Transform2D>(entity));
        assert!(world.has::<GlobalTransform2D>(entity));
    }

    #[test]
    fn test_entity_builder_with_if() {
        let mut world = World::new();

        // Condition true
        let e1 = EntityBuilder::new(&mut world)
            .with_if(true, Transform2D::default())
            .build();
        assert!(world.has::<Transform2D>(e1));

        // Condition false
        let e2 = EntityBuilder::new(&mut world)
            .with_if(false, Transform2D::default())
            .build();
        assert!(!world.has::<Transform2D>(e2));
    }

    #[test]
    fn test_entity_builder_entity_access() {
        let mut world = World::new();
        let builder = EntityBuilder::new(&mut world);
        let entity = builder.entity();

        // Entity should be alive even before build()
        assert!(world.is_alive(entity));
    }

    // =========================================================================
    // GoudGame Tests
    // =========================================================================

    #[test]
    fn test_goud_game_new() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        assert_eq!(game.entity_count(), 0);
        assert!(!game.is_initialized());
    }

    #[test]
    fn test_goud_game_default() {
        let game = GoudGame::default();
        assert_eq!(game.title(), "GoudEngine Game");
        assert_eq!(game.window_size(), (800, 600));
    }

    #[test]
    fn test_goud_game_spawn() {
        let mut game = GoudGame::default();

        let entity = game
            .spawn()
            .with(Transform2D::from_position(Vec2::new(100.0, 100.0)))
            .build();

        assert!(game.is_alive(entity));
        assert!(game.has::<Transform2D>(entity));
        assert_eq!(game.entity_count(), 1);
    }

    #[test]
    fn test_goud_game_spawn_empty() {
        let mut game = GoudGame::default();

        let entity = game.spawn_empty();
        assert!(game.is_alive(entity));
        assert_eq!(game.entity_count(), 1);
    }

    #[test]
    fn test_goud_game_spawn_batch() {
        let mut game = GoudGame::default();

        let entities = game.spawn_batch(100);
        assert_eq!(entities.len(), 100);
        assert_eq!(game.entity_count(), 100);

        for entity in entities {
            assert!(game.is_alive(entity));
        }
    }

    #[test]
    fn test_goud_game_despawn() {
        let mut game = GoudGame::default();

        let entity = game.spawn_empty();
        assert!(game.is_alive(entity));

        let despawned = game.despawn(entity);
        assert!(despawned);
        assert!(!game.is_alive(entity));
    }

    #[test]
    fn test_goud_game_component_operations() {
        let mut game = GoudGame::default();
        let entity = game.spawn_empty();

        // Insert
        game.insert(entity, Transform2D::from_position(Vec2::new(5.0, 10.0)));
        assert!(game.has::<Transform2D>(entity));

        // Get
        let transform = game.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.position, Vec2::new(5.0, 10.0));

        // Get mut
        {
            let transform = game.get_mut::<Transform2D>(entity).unwrap();
            transform.position.x = 100.0;
        }
        let transform = game.get::<Transform2D>(entity).unwrap();
        assert_eq!(transform.position.x, 100.0);

        // Remove
        let removed = game.remove::<Transform2D>(entity);
        assert!(removed.is_some());
        assert!(!game.has::<Transform2D>(entity));
    }

    #[test]
    fn test_goud_game_world_access() {
        let mut game = GoudGame::default();

        // Immutable access
        assert_eq!(game.world().entity_count(), 0);

        // Mutable access
        let entity = game.world_mut().spawn_empty();
        assert!(game.world().is_alive(entity));
    }

    #[test]
    fn test_goud_game_update_frame() {
        let mut game = GoudGame::default();
        let mut update_count = 0;

        game.update_frame(0.016, |_ctx, _world| {
            update_count += 1;
        });

        assert_eq!(update_count, 1);
        assert_eq!(game.frame_count(), 1);
    }

    #[test]
    fn test_goud_game_run_with_quit() {
        let mut game = GoudGame::default();
        let mut frame_count = 0;

        game.run(|ctx, _world| {
            frame_count += 1;
            if frame_count >= 5 {
                ctx.quit();
            }
        });

        assert_eq!(frame_count, 5);
        assert!(game.is_initialized());
    }

    #[test]
    fn test_goud_game_debug() {
        let game = GoudGame::default();
        let debug = format!("{:?}", game);

        assert!(debug.contains("GoudGame"));
        assert!(debug.contains("config"));
        assert!(debug.contains("entity_count"));
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    fn test_full_game_workflow() {
        // Create game
        let mut game = GoudGame::new(GameConfig::new("Test", 800, 600)).unwrap();

        // Spawn player with components
        let player = game
            .spawn()
            .with(Transform2D::from_position(Vec2::new(400.0, 300.0)))
            .with(GlobalTransform2D::IDENTITY)
            .build();

        // Spawn enemies
        let mut enemies = Vec::new();
        for i in 0..10 {
            let enemy = game
                .spawn()
                .with(Transform2D::from_position(Vec2::new(
                    i as f32 * 50.0,
                    100.0,
                )))
                .build();
            enemies.push(enemy);
        }

        assert_eq!(game.entity_count(), 11); // 1 player + 10 enemies

        // Run a few frames
        let mut player_moved = false;
        game.run(|ctx, world| {
            // Move player
            if let Some(transform) = world.get_mut::<Transform2D>(player) {
                transform.position.x += 10.0 * ctx.delta_time();
                player_moved = true;
            }

            // Quit after a few frames
            if ctx.frame_count() >= 3 {
                ctx.quit();
            }
        });

        assert!(player_moved);
        assert!(game.is_alive(player));
    }
}
