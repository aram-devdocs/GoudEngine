//! Entity cloning and prefab instantiation.
//!
//! Provides methods to clone entities (with all their cloneable components)
//! and to recursively clone entity hierarchies.

use super::super::component::ComponentId;
use super::super::entity::Entity;
use super::super::Component;
use super::World;
use crate::ecs::components::hierarchy::{Children, Parent};

impl World {
    /// Registers a component type as cloneable.
    ///
    /// After registration, entities with this component can have it cloned
    /// via [`clone_entity`](Self::clone_entity). Components that are not
    /// registered are silently skipped during cloning.
    ///
    /// # Type Parameters
    ///
    /// - `T`: A component type that implements `Clone`
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct Health(f32);
    /// impl Component for Health {}
    ///
    /// let mut world = World::new();
    /// world.register_cloneable::<Health>();
    /// ```
    pub fn register_cloneable<T: Component + Clone>(&mut self) {
        let id = ComponentId::of::<T>();
        self.storages
            .entry(id)
            .or_insert_with(super::storage_entry::ComponentStorageEntry::new::<T>)
            .set_clone_fn::<T>();
    }

    /// Registers built-in engine components as cloneable.
    ///
    /// This registers the following component types:
    /// - `Transform2D`, `GlobalTransform2D` (2D)
    /// - `Transform`, `GlobalTransform` (3D)
    /// - `Parent`, `Children`, `Name`
    ///
    /// Call this method if you intend to use entity cloning with
    /// built-in components. This is opt-in to avoid overhead for
    /// applications that do not need cloning.
    pub fn register_builtin_cloneables(&mut self) {
        use crate::ecs::components::{GlobalTransform, GlobalTransform2D, Name, Transform, Transform2D};
        self.register_cloneable::<Transform2D>();
        self.register_cloneable::<GlobalTransform2D>();
        self.register_cloneable::<Transform>();
        self.register_cloneable::<GlobalTransform>();
        self.register_cloneable::<Parent>();
        self.register_cloneable::<Children>();
        self.register_cloneable::<Name>();
    }

    /// Clones an entity, producing a new entity with copies of all
    /// cloneable components.
    ///
    /// Components whose types were not registered via
    /// [`register_cloneable`](Self::register_cloneable) are silently skipped.
    ///
    /// Hierarchy handling:
    /// - If the source has a `Parent` component, the clone will **not** have one
    ///   (the clone is detached from the hierarchy).
    /// - If the source has a `Children` component, the clone will **not** have one
    ///   (children are not shared). Use [`clone_entity_recursive`](Self::clone_entity_recursive)
    ///   to deep-clone the subtree.
    ///
    /// # Returns
    ///
    /// `Some(new_entity)` if the source entity is alive, `None` otherwise.
    pub fn clone_entity(&mut self, entity: Entity) -> Option<Entity> {
        if !self.is_alive(entity) {
            return None;
        }

        // Get source entity's archetype to find its component set
        let archetype_id = *self.entity_archetypes.get(&entity)?;
        let component_ids: Vec<ComponentId> = self
            .archetypes
            .get(archetype_id)
            .map(|arch| arch.components().iter().copied().collect())
            .unwrap_or_default();

        // Spawn new entity
        let clone = self.spawn_empty();

        // Clone each component from source to clone
        let parent_id = ComponentId::of::<Parent>();
        let children_id = ComponentId::of::<Children>();

        for component_id in &component_ids {
            // Skip hierarchy components -- handled separately below
            if *component_id == parent_id || *component_id == children_id {
                continue;
            }

            if let Some(storage_entry) = self.storages.get_mut(component_id) {
                if storage_entry.clone_to(entity, clone) {
                    // Update archetype: transition clone to include this component
                    let current_arch_id = self.entity_archetypes[&clone];
                    let target_arch_id =
                        self.archetypes.get_add_edge(current_arch_id, *component_id);

                    if let Some(old_arch) = self.archetypes.get_mut(current_arch_id) {
                        old_arch.remove_entity(clone);
                    }
                    if let Some(new_arch) = self.archetypes.get_mut(target_arch_id) {
                        new_arch.add_entity(clone);
                    }
                    self.entity_archetypes.insert(clone, target_arch_id);
                }
            }
        }

        Some(clone)
    }

    /// Recursively clones an entity and its entire subtree.
    ///
    /// Each child (and descendant) is cloned and re-parented under the
    /// corresponding cloned parent, preserving the hierarchy shape.
    ///
    /// # Returns
    ///
    /// `Some(root_clone)` if the source entity is alive, `None` otherwise.
    pub fn clone_entity_recursive(&mut self, entity: Entity) -> Option<Entity> {
        if !self.is_alive(entity) {
            return None;
        }

        // Collect original children before cloning (to avoid borrow issues)
        let original_children: Vec<Entity> = self
            .get::<Children>(entity)
            .map(|c| c.as_slice().to_vec())
            .unwrap_or_default();

        // Clone the root entity (shallow, no hierarchy components)
        let root_clone = self.clone_entity(entity)?;

        // Recursively clone children and re-parent them
        if !original_children.is_empty() {
            let mut cloned_children = Vec::with_capacity(original_children.len());

            for child in &original_children {
                if let Some(child_clone) = self.clone_entity_recursive(*child) {
                    // Set parent on the cloned child
                    self.insert(child_clone, Parent::new(root_clone));
                    cloned_children.push(child_clone);
                }
            }

            if !cloned_children.is_empty() {
                self.insert(root_clone, Children::from_slice(&cloned_children));
            }
        }

        Some(root_clone)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::component::ComponentId;
    use super::super::super::entity::Entity;
    use super::super::super::Component;
    use super::super::World;
    use crate::ecs::components::hierarchy::{Children, Parent};

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    #[derive(Debug, Clone, PartialEq)]
    struct Health(f32);
    impl Component for Health {}

    /// Component that does NOT implement Clone, to verify graceful skipping.
    struct NonCloneable {
        _data: Vec<u8>,
    }
    impl Component for NonCloneable {}

    // =========================================================================
    // clone_entity tests
    // =========================================================================

    #[test]
    fn test_clone_entity_single_component() {
        let mut world = World::new();
        world.register_cloneable::<Position>();

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });

        let clone = world.clone_entity(entity).unwrap();

        assert_ne!(entity, clone);
        assert!(world.is_alive(clone));
        assert_eq!(
            world.get::<Position>(clone),
            Some(&Position { x: 1.0, y: 2.0 })
        );
    }

    #[test]
    fn test_clone_entity_multiple_components() {
        let mut world = World::new();
        world.register_cloneable::<Position>();
        world.register_cloneable::<Velocity>();
        world.register_cloneable::<Health>();

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 5.0, y: 10.0 });
        world.insert(entity, Velocity { x: 1.0, y: -1.0 });
        world.insert(entity, Health(100.0));

        let clone = world.clone_entity(entity).unwrap();

        assert_eq!(
            world.get::<Position>(clone),
            Some(&Position { x: 5.0, y: 10.0 })
        );
        assert_eq!(
            world.get::<Velocity>(clone),
            Some(&Velocity { x: 1.0, y: -1.0 })
        );
        assert_eq!(world.get::<Health>(clone), Some(&Health(100.0)));
    }

    #[test]
    fn test_clone_independence_modify_clone_not_original() {
        let mut world = World::new();
        world.register_cloneable::<Position>();

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });

        let clone = world.clone_entity(entity).unwrap();

        // Modify clone
        if let Some(pos) = world.get_mut::<Position>(clone) {
            pos.x = 99.0;
            pos.y = 99.0;
        }

        // Original unchanged
        assert_eq!(
            world.get::<Position>(entity),
            Some(&Position { x: 1.0, y: 2.0 })
        );
        assert_eq!(
            world.get::<Position>(clone),
            Some(&Position { x: 99.0, y: 99.0 })
        );
    }

    #[test]
    fn test_clone_entity_with_parent_is_removed() {
        let mut world = World::new();
        world.register_cloneable::<Position>();
        world.register_cloneable::<Parent>();

        let parent = world.spawn_empty();
        let child = world.spawn_empty();
        world.insert(child, Position { x: 1.0, y: 2.0 });
        world.insert(child, Parent::new(parent));

        let clone = world.clone_entity(child).unwrap();

        // Clone should have Position but NOT Parent
        assert!(world.has::<Position>(clone));
        assert!(!world.has::<Parent>(clone));
    }

    #[test]
    fn test_clone_entity_with_children_is_removed() {
        let mut world = World::new();
        world.register_cloneable::<Position>();
        world.register_cloneable::<Children>();

        let parent = world.spawn_empty();
        let child = world.spawn_empty();
        world.insert(parent, Position { x: 1.0, y: 2.0 });
        world.insert(parent, Children::from_slice(&[child]));

        let clone = world.clone_entity(parent).unwrap();

        // Clone should have Position but NOT Children
        assert!(world.has::<Position>(clone));
        assert!(!world.has::<Children>(clone));
    }

    #[test]
    fn test_clone_nonexistent_entity_returns_none() {
        let mut world = World::new();
        let fake = Entity::new(999, 1);

        assert!(world.clone_entity(fake).is_none());
    }

    #[test]
    fn test_clone_with_unregistered_component_skips_gracefully() {
        let mut world = World::new();
        world.register_cloneable::<Position>();
        // NonCloneable is NOT registered

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });
        world.insert(
            entity,
            NonCloneable {
                _data: vec![1, 2, 3],
            },
        );

        let clone = world.clone_entity(entity).unwrap();

        // Position should be cloned, NonCloneable should be skipped
        assert!(world.has::<Position>(clone));
        assert!(!world.has::<NonCloneable>(clone));
    }

    // =========================================================================
    // clone_entity_recursive tests
    // =========================================================================

    #[test]
    fn test_clone_entity_recursive_children_cloned_and_reparented() {
        let mut world = World::new();
        world.register_cloneable::<Position>();
        world.register_cloneable::<Parent>();
        world.register_cloneable::<Children>();

        // Build hierarchy: parent -> child1, child2
        let parent = world.spawn_empty();
        let child1 = world.spawn_empty();
        let child2 = world.spawn_empty();

        world.insert(parent, Position { x: 0.0, y: 0.0 });
        world.insert(child1, Position { x: 1.0, y: 1.0 });
        world.insert(child1, Parent::new(parent));
        world.insert(child2, Position { x: 2.0, y: 2.0 });
        world.insert(child2, Parent::new(parent));
        world.insert(parent, Children::from_slice(&[child1, child2]));

        let clone_root = world.clone_entity_recursive(parent).unwrap();

        // Clone root should have Position
        assert_eq!(
            world.get::<Position>(clone_root),
            Some(&Position { x: 0.0, y: 0.0 })
        );

        // Clone root should have Children
        let clone_children = world.get::<Children>(clone_root).unwrap();
        assert_eq!(clone_children.len(), 2);

        let cloned_child1 = clone_children.get(0).unwrap();
        let cloned_child2 = clone_children.get(1).unwrap();

        // Cloned children should have Position
        assert_eq!(
            world.get::<Position>(cloned_child1),
            Some(&Position { x: 1.0, y: 1.0 })
        );
        assert_eq!(
            world.get::<Position>(cloned_child2),
            Some(&Position { x: 2.0, y: 2.0 })
        );

        // Cloned children should point to clone_root as parent
        assert_eq!(
            world.get::<Parent>(cloned_child1),
            Some(&Parent::new(clone_root))
        );
        assert_eq!(
            world.get::<Parent>(cloned_child2),
            Some(&Parent::new(clone_root))
        );

        // Original children are unaffected
        assert_eq!(world.get::<Parent>(child1), Some(&Parent::new(parent)));
        assert_eq!(world.get::<Parent>(child2), Some(&Parent::new(parent)));
    }

    #[test]
    fn test_clone_entity_recursive_nonexistent_returns_none() {
        let mut world = World::new();
        let fake = Entity::new(999, 1);

        assert!(world.clone_entity_recursive(fake).is_none());
    }

    #[test]
    fn test_clone_entity_recursive_no_children() {
        let mut world = World::new();
        world.register_cloneable::<Position>();

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 5.0, y: 5.0 });

        let clone = world.clone_entity_recursive(entity).unwrap();

        assert_eq!(
            world.get::<Position>(clone),
            Some(&Position { x: 5.0, y: 5.0 })
        );
        assert!(!world.has::<Children>(clone));
    }

    #[test]
    fn test_clone_entity_recursive_deep_hierarchy() {
        let mut world = World::new();
        world.register_cloneable::<Position>();
        world.register_cloneable::<Parent>();
        world.register_cloneable::<Children>();

        // Build: grandparent -> parent -> child
        let grandparent = world.spawn_empty();
        let parent = world.spawn_empty();
        let child = world.spawn_empty();

        world.insert(grandparent, Position { x: 0.0, y: 0.0 });
        world.insert(parent, Position { x: 1.0, y: 1.0 });
        world.insert(parent, Parent::new(grandparent));
        world.insert(child, Position { x: 2.0, y: 2.0 });
        world.insert(child, Parent::new(parent));

        world.insert(grandparent, Children::from_slice(&[parent]));
        world.insert(parent, Children::from_slice(&[child]));

        let clone_root = world.clone_entity_recursive(grandparent).unwrap();

        // Verify 3-level deep clone
        let clone_children = world.get::<Children>(clone_root).unwrap();
        assert_eq!(clone_children.len(), 1);

        let clone_parent = clone_children.get(0).unwrap();
        assert_eq!(
            world.get::<Position>(clone_parent),
            Some(&Position { x: 1.0, y: 1.0 })
        );

        let clone_grandchildren = world.get::<Children>(clone_parent).unwrap();
        assert_eq!(clone_grandchildren.len(), 1);

        let clone_child = clone_grandchildren.get(0).unwrap();
        assert_eq!(
            world.get::<Position>(clone_child),
            Some(&Position { x: 2.0, y: 2.0 })
        );

        // Verify parent references
        assert_eq!(
            world.get::<Parent>(clone_parent),
            Some(&Parent::new(clone_root))
        );
        assert_eq!(
            world.get::<Parent>(clone_child),
            Some(&Parent::new(clone_parent))
        );
    }
}
