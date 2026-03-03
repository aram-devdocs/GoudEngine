use super::super::entity::Entity;
use super::super::Component;
use super::World;

// =============================================================================
// EntityWorldMut - Entity Builder
// =============================================================================

/// A mutable reference to an entity's data within a [`World`].
///
/// `EntityWorldMut` provides a fluent builder API for constructing entities
/// with components. It holds a mutable borrow of the world, allowing chained
/// component insertions.
///
/// # Builder Pattern
///
/// The primary use is via [`World::spawn()`] for fluent entity construction:
///
/// ```ignore
/// let entity = world.spawn()
///     .insert(Position { x: 0.0, y: 0.0 })
///     .insert(Velocity { x: 1.0, y: 0.0 })
///     .id();
/// ```
///
/// # Lifetime
///
/// The builder holds a mutable borrow of the [`World`], so you cannot access
/// the world while an `EntityWorldMut` exists. Call [`id()`](Self::id) to
/// get the entity ID and release the borrow.
///
/// # Thread Safety
///
/// `EntityWorldMut` is not `Send` or `Sync` - it's designed for single-threaded
/// entity construction. For batch spawning, use [`World::spawn_batch()`] (future).
#[derive(Debug)]
pub struct EntityWorldMut<'w> {
    /// The world containing this entity.
    world: &'w mut World,

    /// The entity being built.
    entity: Entity,
}

impl<'w> EntityWorldMut<'w> {
    /// Creates a new `EntityWorldMut` for an entity in the given world.
    ///
    /// # Safety Note
    ///
    /// The entity must already be allocated and registered in the world.
    /// This is an internal constructor - use [`World::spawn()`] instead.
    #[inline]
    pub(crate) fn new(world: &'w mut World, entity: Entity) -> Self {
        Self { world, entity }
    }

    /// Returns the [`Entity`] ID of the entity being built.
    ///
    /// This is commonly used at the end of a builder chain to get the
    /// entity ID for later reference.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let entity = world.spawn().id();
    /// assert!(world.is_alive(entity));
    /// ```
    #[inline]
    pub fn id(&self) -> Entity {
        self.entity
    }

    /// Returns a reference to the [`World`] containing this entity.
    ///
    /// This allows read-only access to world state while building an entity.
    /// For mutable access, you'll need to finish building and drop this
    /// `EntityWorldMut` first.
    #[inline]
    pub fn world(&self) -> &World {
        self.world
    }

    /// Returns a mutable reference to the [`World`] containing this entity.
    ///
    /// # Warning
    ///
    /// Be careful when accessing the world mutably - ensure you don't
    /// invalidate this entity or its archetype in unexpected ways.
    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }

    // =========================================================================
    // Component Operations
    // =========================================================================

    /// Inserts a component on this entity.
    ///
    /// If the entity already has a component of this type, it is replaced.
    /// Returns `self` for method chaining.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to insert
    ///
    /// # Arguments
    ///
    /// * `component` - The component value to insert
    ///
    /// # Returns
    ///
    /// `&mut Self` for fluent method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// struct Velocity { x: f32, y: f32 }
    /// impl Component for Velocity {}
    ///
    /// let mut world = World::new();
    ///
    /// // Fluent builder pattern
    /// let entity = world.spawn()
    ///     .insert(Position { x: 0.0, y: 0.0 })
    ///     .insert(Velocity { x: 1.0, y: 0.0 })
    ///     .id();
    ///
    /// assert!(world.has::<Position>(entity));
    /// assert!(world.has::<Velocity>(entity));
    /// assert_eq!(world.get::<Position>(entity), Some(&Position { x: 0.0, y: 0.0 }));
    /// assert_eq!(world.get::<Velocity>(entity), Some(&Velocity { x: 1.0, y: 0.0 }));
    /// ```
    ///
    /// # Archetype Transitions
    ///
    /// Each `insert` call may trigger an archetype transition if the entity
    /// doesn't already have the component type. Multiple inserts in sequence
    /// will create intermediate archetypes.
    #[inline]
    pub fn insert<T: Component>(&mut self, component: T) -> &mut Self {
        self.world.insert(self.entity, component);
        self
    }
}
