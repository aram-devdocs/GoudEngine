use super::super::archetype::ArchetypeId;
use super::super::component::ComponentId;
use super::super::entity::Entity;
use super::entity_world_mut::EntityWorldMut;
use super::World;
use crate::ecs::components::hierarchy::Children;

impl World {
    // =========================================================================
    // Entity Statistics
    // =========================================================================

    /// Returns the number of currently alive entities.
    ///
    /// This is the total number of entities that have been spawned and not
    /// yet despawned.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let world = World::new();
    /// assert_eq!(world.entity_count(), 0);
    /// ```
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Returns `true` if there are no entities in the world.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let world = World::new();
    /// assert!(world.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    // =========================================================================
    // Archetype Statistics
    // =========================================================================

    /// Returns the number of archetypes in the world.
    ///
    /// This includes the EMPTY archetype which always exists. As entities
    /// gain different combinations of components, new archetypes are created.
    ///
    /// # Performance Note
    ///
    /// A high archetype count can indicate fragmentation. If you have many
    /// archetypes with few entities each, consider consolidating component
    /// combinations.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let world = World::new();
    /// assert_eq!(world.archetype_count(), 1); // Empty archetype
    /// ```
    #[inline]
    pub fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }

    // =========================================================================
    // Entity Spawning
    // =========================================================================

    /// Spawns a new entity with no components.
    ///
    /// The entity is allocated, added to the empty archetype, and returned.
    /// Use this when you want to add components later or need a simple entity.
    ///
    /// # Returns
    ///
    /// The newly created [`Entity`].
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    ///
    /// let entity = world.spawn_empty();
    ///
    /// assert!(world.is_alive(entity));
    /// assert_eq!(world.entity_count(), 1);
    /// ```
    ///
    /// # Performance
    ///
    /// This is an O(1) operation. The entity is immediately available for
    /// component additions.
    #[inline]
    pub fn spawn_empty(&mut self) -> Entity {
        // Allocate a new entity ID
        let entity = self.entities.allocate();

        // Add to the empty archetype
        let empty_archetype = self
            .archetypes
            .get_mut(ArchetypeId::EMPTY)
            .expect("Empty archetype should always exist");
        empty_archetype.add_entity(entity);

        // Track entity -> archetype mapping
        self.entity_archetypes.insert(entity, ArchetypeId::EMPTY);

        entity
    }

    /// Spawns a new entity and returns a builder for adding components.
    ///
    /// This is the primary way to create entities with components using a
    /// fluent builder pattern. The entity is immediately alive and starts
    /// in the empty archetype.
    ///
    /// # Returns
    ///
    /// An [`EntityWorldMut`] builder for adding components to the entity.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use goud_engine::ecs::{World, Component};
    ///
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut world = World::new();
    ///
    /// // Spawn with components (insert will be available in step 2.4.5)
    /// let entity = world.spawn()
    ///     .insert(Position { x: 0.0, y: 0.0 })
    ///     .id();
    ///
    /// assert!(world.is_alive(entity));
    /// ```
    ///
    /// # Builder Pattern
    ///
    /// The returned `EntityWorldMut` holds a mutable borrow of the world.
    /// Call [`.id()`](EntityWorldMut::id) at the end of your chain to get
    /// the entity ID and release the borrow.
    ///
    /// # Performance
    ///
    /// Entity spawning is O(1). Adding components via the builder triggers
    /// archetype transitions as needed.
    #[inline]
    pub fn spawn(&mut self) -> EntityWorldMut<'_> {
        let entity = self.spawn_empty();
        EntityWorldMut::new(self, entity)
    }

    /// Spawns multiple entities at once and returns their IDs.
    ///
    /// This is more efficient than calling [`spawn_empty()`](Self::spawn_empty)
    /// in a loop because it can batch allocations.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of entities to spawn
    ///
    /// # Returns
    ///
    /// A vector of the newly created [`Entity`] IDs.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    ///
    /// let entities = world.spawn_batch(100);
    ///
    /// assert_eq!(entities.len(), 100);
    /// assert_eq!(world.entity_count(), 100);
    ///
    /// for entity in &entities {
    ///     assert!(world.is_alive(*entity));
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// Batch spawning uses the entity allocator's batch allocation, which
    /// is more efficient than individual allocations. The empty archetype
    /// is also updated in bulk.
    pub fn spawn_batch(&mut self, count: usize) -> Vec<Entity> {
        if count == 0 {
            return Vec::new();
        }

        // Batch allocate entity IDs
        let entities = self.entities.allocate_batch(count);

        // Get the empty archetype
        let empty_archetype = self
            .archetypes
            .get_mut(ArchetypeId::EMPTY)
            .expect("Empty archetype should always exist");

        // Add all entities to empty archetype and track mappings
        for &entity in &entities {
            empty_archetype.add_entity(entity);
            self.entity_archetypes.insert(entity, ArchetypeId::EMPTY);
        }

        entities
    }

    // =========================================================================
    // Entity Despawning
    // =========================================================================

    /// Despawns an entity, removing it and all its components from the world.
    ///
    /// This completely removes the entity from the ECS:
    /// - The entity is removed from its archetype
    /// - All components attached to the entity are dropped
    /// - The entity ID is deallocated and may be recycled with a new generation
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to despawn
    ///
    /// # Returns
    ///
    /// `true` if the entity was successfully despawned, `false` if it was
    /// already dead or invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    ///
    /// let entity = world.spawn_empty();
    /// assert!(world.is_alive(entity));
    ///
    /// let despawned = world.despawn(entity);
    /// assert!(despawned);
    /// assert!(!world.is_alive(entity));
    ///
    /// // Despawning again returns false
    /// let despawned_again = world.despawn(entity);
    /// assert!(!despawned_again);
    /// ```
    ///
    /// # Component Cleanup
    ///
    /// All components attached to the entity are dropped when the entity is
    /// despawned. This ensures resources are properly released.
    ///
    /// # Performance
    ///
    /// Despawning is O(n) where n is the number of component types registered
    /// in the world, as we must check each storage for the entity's components.
    /// For better performance with many entities, use [`despawn_batch`](Self::despawn_batch).
    pub fn despawn(&mut self, entity: Entity) -> bool {
        // Check if entity is alive
        if !self.entities.is_alive(entity) {
            return false;
        }

        // Get and remove the archetype mapping
        let archetype_id = match self.entity_archetypes.remove(&entity) {
            Some(id) => id,
            None => return false,
        };

        // Get the components this entity has from its archetype
        // We need to clone the component set to avoid borrowing issues
        let component_ids: Vec<ComponentId> = self
            .archetypes
            .get(archetype_id)
            .map(|arch| arch.components().iter().copied().collect())
            .unwrap_or_default();

        // Remove from archetype
        if let Some(archetype) = self.archetypes.get_mut(archetype_id) {
            archetype.remove_entity(entity);
        }

        // Remove components using type-erased removal
        // We only check storages for components the entity actually has (from its archetype)
        for component_id in component_ids {
            if let Some(storage_entry) = self.storages.get_mut(&component_id) {
                storage_entry.remove_entity(entity);
            }
        }

        // Deallocate the entity ID
        self.entities.deallocate(entity);

        true
    }

    /// Despawns multiple entities at once.
    ///
    /// This is more efficient than calling [`despawn`](Self::despawn) repeatedly
    /// because it can batch some operations.
    ///
    /// # Arguments
    ///
    /// * `entities` - A slice of entities to despawn
    ///
    /// # Returns
    ///
    /// The number of entities successfully despawned. Entities that were already
    /// dead or invalid are not counted.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    ///
    /// let entities = world.spawn_batch(10);
    /// assert_eq!(world.entity_count(), 10);
    ///
    /// let despawned = world.despawn_batch(&entities);
    /// assert_eq!(despawned, 10);
    /// assert_eq!(world.entity_count(), 0);
    /// ```
    ///
    /// # Performance
    ///
    /// Batch despawning is more efficient when removing many entities, but
    /// the complexity is still O(n * m) where n is the number of entities
    /// and m is the number of component types.
    pub fn despawn_batch(&mut self, entities: &[Entity]) -> usize {
        let mut count = 0;
        for &entity in entities {
            if self.despawn(entity) {
                count += 1;
            }
        }
        count
    }

    /// Despawns the entity and all its descendants recursively.
    ///
    /// Walks the hierarchy via [`Children`] components, despawning from
    /// leaves up to the root. Returns `true` if the root entity was alive
    /// and successfully despawned.
    ///
    /// # Arguments
    ///
    /// * `entity` - The root entity to despawn along with its entire subtree
    ///
    /// # Returns
    ///
    /// `true` if the root entity was alive and successfully despawned,
    /// `false` if the root entity was already dead or invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let mut world = World::new();
    /// let parent = world.spawn_empty();
    /// let child = world.spawn_empty();
    /// world.insert(parent, Children::from_slice(&[child]));
    ///
    /// assert!(world.despawn_recursive(parent));
    /// assert!(!world.is_alive(parent));
    /// assert!(!world.is_alive(child));
    /// ```
    ///
    /// # Performance
    ///
    /// This is O(n) in the number of descendants. Descendants are collected
    /// first to avoid borrow conflicts, then despawned from leaves toward
    /// the root.
    pub fn despawn_recursive(&mut self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        // Collect all descendants first to avoid borrow conflicts
        let mut to_despawn = Vec::new();
        self.collect_descendants(entity, &mut to_despawn);

        // Despawn descendants in reverse (leaves first)
        for descendant in to_despawn.into_iter().rev() {
            self.despawn(descendant);
        }

        // Despawn the root entity
        self.despawn(entity)
    }

    /// Recursively collects all descendant entities into `out`.
    ///
    /// Performs a depth-first traversal of the hierarchy rooted at `entity`,
    /// appending each child (and its descendants) to `out` in pre-order.
    /// The root itself is not included.
    fn collect_descendants(&self, entity: Entity, out: &mut Vec<Entity>) {
        if let Some(children) = self.get::<Children>(entity) {
            let child_entities: Vec<Entity> = children.as_slice().to_vec();
            for child in child_entities {
                out.push(child);
                self.collect_descendants(child, out);
            }
        }
    }

    // =========================================================================
    // Entity Lifecycle Helpers
    // =========================================================================

    /// Checks if an entity is currently alive in this world.
    ///
    /// An entity is alive if:
    /// - It was allocated by this world's entity allocator
    /// - It has not been despawned
    /// - Its generation matches the current slot generation
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    ///
    /// # Returns
    ///
    /// `true` if the entity is valid and alive in this world.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    /// use goud_engine::ecs::Entity;
    ///
    /// let world = World::new();
    ///
    /// // Placeholder entities are never alive
    /// assert!(!world.is_alive(Entity::PLACEHOLDER));
    ///
    /// // Fabricated entities are not alive
    /// let fake = Entity::new(999, 1);
    /// assert!(!world.is_alive(fake));
    /// ```
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    /// Returns the archetype ID for the given entity.
    ///
    /// Returns `None` if the entity is not alive in this world.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to look up
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    /// use goud_engine::ecs::Entity;
    ///
    /// let world = World::new();
    ///
    /// // Non-existent entity has no archetype
    /// let fake = Entity::new(0, 1);
    /// assert!(world.entity_archetype(fake).is_none());
    /// ```
    #[inline]
    pub fn entity_archetype(&self, entity: Entity) -> Option<ArchetypeId> {
        if self.is_alive(entity) {
            self.entity_archetypes.get(&entity).copied()
        } else {
            None
        }
    }
}
