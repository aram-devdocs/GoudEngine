use crate::ecs::{Component, Entity, World};
use crate::sdk::entity_builder::EntityBuilder;

use super::GoudGame;

impl GoudGame {
    // =========================================================================
    // Default-scene World Access (backward-compatible)
    // =========================================================================

    /// Returns a reference to the default scene's ECS world.
    #[inline]
    pub fn world(&self) -> &World {
        self.scene_manager
            .get_scene(self.scene_manager.default_scene())
            .expect("default scene must exist")
    }

    /// Returns a mutable reference to the default scene's ECS world.
    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        let default = self.scene_manager.default_scene();
        self.scene_manager
            .get_scene_mut(default)
            .expect("default scene must exist")
    }

    /// Creates an entity builder for fluent entity creation (default scene).
    #[inline]
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        let default = self.scene_manager.default_scene();
        let world = self
            .scene_manager
            .get_scene_mut(default)
            .expect("default scene must exist");
        EntityBuilder::new(world)
    }

    /// Spawns an empty entity with no components (default scene).
    #[inline]
    pub fn spawn_empty(&mut self) -> Entity {
        self.world_mut().spawn_empty()
    }

    /// Spawns multiple empty entities at once (default scene).
    #[inline]
    pub fn spawn_batch(&mut self, count: usize) -> Vec<Entity> {
        self.world_mut().spawn_batch(count)
    }

    /// Despawns an entity and removes all its components (default scene).
    #[inline]
    pub fn despawn(&mut self, entity: Entity) -> bool {
        self.world_mut().despawn(entity)
    }

    /// Gets a reference to a component on an entity (default scene).
    #[inline]
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.world().get::<T>(entity)
    }

    /// Gets a mutable reference to a component on an entity (default scene).
    #[inline]
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.world_mut().get_mut::<T>(entity)
    }

    /// Adds or replaces a component on an entity (default scene).
    #[inline]
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        self.world_mut().insert(entity, component);
    }

    /// Removes a component from an entity (default scene).
    #[inline]
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Option<T> {
        self.world_mut().remove::<T>(entity)
    }

    /// Checks if an entity has a specific component (default scene).
    #[inline]
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        self.world().has::<T>(entity)
    }

    /// Returns the number of entities in the default scene.
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.world().entity_count()
    }

    /// Checks if an entity is alive (default scene).
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.world().is_alive(entity)
    }
}
