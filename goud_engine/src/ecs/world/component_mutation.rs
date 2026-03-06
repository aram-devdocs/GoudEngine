use super::super::component::ComponentId;
use super::super::entity::Entity;
use super::super::Component;
use super::World;

impl World {
    // =========================================================================
    // Component Insertion
    // =========================================================================

    /// Inserts a component on an entity.
    ///
    /// If the entity already has a component of this type, it is replaced and
    /// the old value is returned. If the entity doesn't have this component,
    /// it is added and the entity transitions to a new archetype.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to insert
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add the component to
    /// * `component` - The component value to insert
    ///
    /// # Returns
    ///
    /// - `Some(old_component)` if the entity already had this component
    /// - `None` if this is a new component for the entity
    ///
    /// # Panics
    ///
    /// This method does not panic. If the entity is dead, the operation is a no-op
    /// and returns `None`.
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
    /// let mut world = World::new();
    /// let entity = world.spawn_empty();
    ///
    /// // First insert returns None
    /// let old = world.insert(entity, Position { x: 1.0, y: 2.0 });
    /// assert!(old.is_none());
    /// assert_eq!(world.get::<Position>(entity), Some(&Position { x: 1.0, y: 2.0 }));
    ///
    /// // Second insert returns old value
    /// let old = world.insert(entity, Position { x: 10.0, y: 20.0 });
    /// assert_eq!(old, Some(Position { x: 1.0, y: 2.0 }));
    /// assert_eq!(world.get::<Position>(entity), Some(&Position { x: 10.0, y: 20.0 }));
    /// ```
    ///
    /// # Archetype Transitions
    ///
    /// When a new component type is added, the entity moves from its current
    /// archetype to a new archetype that includes the new component. This is
    /// handled automatically by the archetype graph.
    ///
    /// # Performance
    ///
    /// - If replacing an existing component: O(1)
    /// - If adding a new component type: O(k) where k is the number of components
    ///   on the entity (archetype transition)
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) -> Option<T> {
        // Check if entity is alive
        if !self.is_alive(entity) {
            return None;
        }

        let component_id = ComponentId::of::<T>();

        // Get current archetype
        let current_archetype_id = *self.entity_archetypes.get(&entity)?;

        // Check if entity already has this component
        let has_component = self
            .archetypes
            .get(current_archetype_id)
            .map(|arch| arch.has_component(component_id))
            .unwrap_or(false);

        if has_component {
            // Entity already has this component - just replace in storage
            let tick = self.change_tick;
            let storage = self.get_or_create_storage_mut::<T>();
            storage.insert_with_tick(entity, component, tick)
        } else {
            // Entity doesn't have this component - need archetype transition
            // 1. Get the target archetype (with the new component)
            let target_archetype_id = self
                .archetypes
                .get_add_edge(current_archetype_id, component_id);

            // 2. Remove entity from old archetype
            if let Some(old_arch) = self.archetypes.get_mut(current_archetype_id) {
                old_arch.remove_entity(entity);
            }

            // 3. Add entity to new archetype
            if let Some(new_arch) = self.archetypes.get_mut(target_archetype_id) {
                new_arch.add_entity(entity);
            }

            // 4. Update entity -> archetype mapping
            self.entity_archetypes.insert(entity, target_archetype_id);

            // 5. Insert the component into storage
            let tick = self.change_tick;
            let storage = self.get_or_create_storage_mut::<T>();
            storage.insert_with_tick(entity, component, tick);

            // No old value to return
            None
        }
    }

    /// Inserts components for multiple entities in batch.
    ///
    /// This is more efficient than calling [`insert`](Self::insert) repeatedly
    /// because it can batch archetype lookups and transitions.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to insert
    ///
    /// # Arguments
    ///
    /// * `batch` - An iterator of (entity, component) pairs
    ///
    /// # Returns
    ///
    /// The number of components successfully inserted (entities that were alive).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// struct Health(i32);
    /// impl Component for Health {}
    ///
    /// let mut world = World::new();
    /// let entities = world.spawn_batch(3);
    ///
    /// // Insert health for all entities
    /// let components = vec![
    ///     (entities[0], Health(100)),
    ///     (entities[1], Health(80)),
    ///     (entities[2], Health(50)),
    /// ];
    ///
    /// let count = world.insert_batch(components);
    /// assert_eq!(count, 3);
    ///
    /// assert_eq!(world.get::<Health>(entities[0]), Some(&Health(100)));
    /// assert_eq!(world.get::<Health>(entities[1]), Some(&Health(80)));
    /// assert_eq!(world.get::<Health>(entities[2]), Some(&Health(50)));
    /// ```
    ///
    /// # Performance
    ///
    /// Batch insertion is more efficient because:
    /// - Archetype edge lookups are cached after the first transition
    /// - Storage is accessed once and reused for all insertions
    pub fn insert_batch<T: Component>(
        &mut self,
        batch: impl IntoIterator<Item = (Entity, T)>,
    ) -> usize {
        let mut count = 0;
        for (entity, component) in batch {
            // Check if entity is alive before insertion
            if self.is_alive(entity) {
                self.insert(entity, component);
                count += 1;
            }
        }
        count
    }

    // =========================================================================
    // Component Removal
    // =========================================================================

    /// Removes a component from an entity and returns it.
    ///
    /// If the entity has the component, it is removed, the entity transitions
    /// to a new archetype (without this component), and the old value is returned.
    /// If the entity doesn't have this component, returns `None`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to remove
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the component from
    ///
    /// # Returns
    ///
    /// - `Some(component)` if the entity had this component
    /// - `None` if the entity doesn't exist, is dead, or doesn't have this component
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
    /// let mut world = World::new();
    /// let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();
    ///
    /// // Remove returns the component
    /// let removed = world.remove::<Position>(entity);
    /// assert_eq!(removed, Some(Position { x: 1.0, y: 2.0 }));
    ///
    /// // Entity no longer has the component
    /// assert!(!world.has::<Position>(entity));
    ///
    /// // Removing again returns None
    /// let removed_again = world.remove::<Position>(entity);
    /// assert!(removed_again.is_none());
    /// ```
    ///
    /// # Archetype Transitions
    ///
    /// When a component is removed, the entity moves from its current archetype
    /// to a new archetype that doesn't include this component. If this was the
    /// entity's last component, it transitions to the empty archetype.
    ///
    /// # Performance
    ///
    /// This is O(1) for the storage removal plus O(1) archetype transition
    /// (using cached edge lookup). The archetype graph caches remove edges
    /// for efficient repeated transitions.
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Option<T> {
        // Check if entity is alive
        if !self.is_alive(entity) {
            return None;
        }

        let component_id = ComponentId::of::<T>();

        // Get current archetype
        let current_archetype_id = *self.entity_archetypes.get(&entity)?;

        // Check if entity has this component
        let has_component = self
            .archetypes
            .get(current_archetype_id)
            .map(|arch| arch.has_component(component_id))
            .unwrap_or(false);

        if !has_component {
            // Entity doesn't have this component - nothing to remove
            return None;
        }

        // Remove component from storage first (before archetype transition)
        let removed_component = self.get_storage_option_mut::<T>()?.remove(entity)?;

        // Get the target archetype (without this component)
        // Note: get_remove_edge returns None only if the component wasn't in the source archetype,
        // but we've already verified the entity has this component above
        let target_archetype_id = self
            .archetypes
            .get_remove_edge(current_archetype_id, component_id)
            .expect("Archetype should have remove edge for component it contains");

        // Remove entity from old archetype
        if let Some(old_arch) = self.archetypes.get_mut(current_archetype_id) {
            old_arch.remove_entity(entity);
        }

        // Add entity to new archetype
        if let Some(new_arch) = self.archetypes.get_mut(target_archetype_id) {
            new_arch.add_entity(entity);
        }

        // Update entity -> archetype mapping
        self.entity_archetypes.insert(entity, target_archetype_id);

        Some(removed_component)
    }

    /// Removes a component from an entity and returns it.
    ///
    /// This is an alias for [`remove`](Self::remove). The name `take` follows
    /// Rust's convention for methods that consume/take ownership of data.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// struct Health(i32);
    /// impl Component for Health {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn().insert(Health(100)).id();
    ///
    /// // take() is identical to remove()
    /// let health = world.take::<Health>(entity);
    /// assert_eq!(health, Some(Health(100)));
    /// ```
    #[inline]
    pub fn take<T: Component>(&mut self, entity: Entity) -> Option<T> {
        self.remove(entity)
    }
}
