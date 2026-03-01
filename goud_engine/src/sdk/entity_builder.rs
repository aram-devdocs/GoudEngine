//! Fluent entity builder for spawning entities with components.
//!
//! Provides [`EntityBuilder`] for ergonomic entity creation using
//! method chaining.

use crate::ecs::{Component, Entity, World};

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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Vec2;
    use crate::sdk::components::{GlobalTransform2D, Transform2D};

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
}
