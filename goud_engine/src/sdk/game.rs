//! Main game abstraction for Rust-native game development.
//!
//! Contains [`GoudGame`], the primary entry point managing the ECS world,
//! game loop, and convenient methods for entity and component operations.

use crate::core::error::GoudResult;
use crate::ecs::{Component, Entity, World};

use super::entity_builder::EntityBuilder;
use super::game_config::{GameConfig, GameContext};

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
    pub fn default_game() -> GoudResult<Self> {
        Self::new(GameConfig::default())
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Vec2;
    use crate::sdk::components::{GlobalTransform2D, Transform2D};

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
