//! Unit and integration tests for component storage traits.

use crate::ecs::{Component, Entity, SparseSet};

use crate::ecs::storage::{AnyComponentStorage, ComponentStorage};

// Test components
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
struct Name(String);
impl Component for Name {}

// Marker component (zero-sized)
#[derive(Debug, Clone, Copy, PartialEq)]
struct Player;
impl Component for Player {}

// =========================================================================
// ComponentStorage Trait Tests
// =========================================================================

mod component_storage_trait {
    use super::*;

    #[test]
    fn test_sparse_set_implements_component_storage() {
        // Verify SparseSet<T: Component> implements ComponentStorage
        fn assert_component_storage<S: ComponentStorage>() {}
        assert_component_storage::<SparseSet<Position>>();
        assert_component_storage::<SparseSet<Velocity>>();
        assert_component_storage::<SparseSet<Name>>();
        assert_component_storage::<SparseSet<Player>>();
    }

    #[test]
    fn test_component_storage_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SparseSet<Position>>();
        assert_send_sync::<SparseSet<Name>>();
    }

    #[test]
    fn test_insert_via_trait() {
        let mut storage: SparseSet<Position> = SparseSet::new();
        let entity = Entity::new(0, 1);

        let old = ComponentStorage::insert(&mut storage, entity, Position { x: 1.0, y: 2.0 });

        assert_eq!(old, None);
        assert!(storage.contains(entity));
    }

    #[test]
    fn test_insert_replace_via_trait() {
        let mut storage: SparseSet<Position> = SparseSet::new();
        let entity = Entity::new(0, 1);

        ComponentStorage::insert(&mut storage, entity, Position { x: 1.0, y: 2.0 });
        let old = ComponentStorage::insert(&mut storage, entity, Position { x: 3.0, y: 4.0 });

        assert_eq!(old, Some(Position { x: 1.0, y: 2.0 }));
        assert_eq!(
            ComponentStorage::get(&storage, entity),
            Some(&Position { x: 3.0, y: 4.0 })
        );
    }

    #[test]
    fn test_remove_via_trait() {
        let mut storage: SparseSet<Position> = SparseSet::new();
        let entity = Entity::new(0, 1);

        storage.insert(entity, Position { x: 1.0, y: 2.0 });
        let removed = ComponentStorage::remove(&mut storage, entity);

        assert_eq!(removed, Some(Position { x: 1.0, y: 2.0 }));
        assert!(!storage.contains(entity));
    }

    #[test]
    fn test_remove_nonexistent_via_trait() {
        let mut storage: SparseSet<Position> = SparseSet::new();
        let entity = Entity::new(0, 1);

        let removed = ComponentStorage::remove(&mut storage, entity);
        assert_eq!(removed, None);
    }

    #[test]
    fn test_get_via_trait() {
        let mut storage: SparseSet<Position> = SparseSet::new();
        let entity = Entity::new(0, 1);

        storage.insert(entity, Position { x: 5.0, y: 10.0 });
        let pos = ComponentStorage::get(&storage, entity);

        assert_eq!(pos, Some(&Position { x: 5.0, y: 10.0 }));
    }

    #[test]
    fn test_get_nonexistent_via_trait() {
        let storage: SparseSet<Position> = SparseSet::new();
        let entity = Entity::new(0, 1);

        assert_eq!(ComponentStorage::get(&storage, entity), None);
    }

    #[test]
    fn test_get_mut_via_trait() {
        let mut storage: SparseSet<Position> = SparseSet::new();
        let entity = Entity::new(0, 1);

        storage.insert(entity, Position { x: 1.0, y: 2.0 });

        if let Some(pos) = ComponentStorage::get_mut(&mut storage, entity) {
            pos.x = 100.0;
            pos.y = 200.0;
        }

        assert_eq!(storage.get(entity), Some(&Position { x: 100.0, y: 200.0 }));
    }

    #[test]
    fn test_contains_via_trait() {
        let mut storage: SparseSet<Position> = SparseSet::new();
        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        storage.insert(e1, Position { x: 0.0, y: 0.0 });

        assert!(ComponentStorage::contains(&storage, e1));
        assert!(!ComponentStorage::contains(&storage, e2));
    }

    #[test]
    fn test_len_via_trait() {
        let mut storage: SparseSet<Position> = SparseSet::new();

        assert_eq!(ComponentStorage::len(&storage), 0);

        storage.insert(Entity::new(0, 1), Position { x: 0.0, y: 0.0 });
        assert_eq!(ComponentStorage::len(&storage), 1);

        storage.insert(Entity::new(1, 1), Position { x: 1.0, y: 1.0 });
        assert_eq!(ComponentStorage::len(&storage), 2);
    }

    #[test]
    fn test_is_empty_via_trait() {
        let mut storage: SparseSet<Position> = SparseSet::new();

        assert!(ComponentStorage::is_empty(&storage));

        storage.insert(Entity::new(0, 1), Position { x: 0.0, y: 0.0 });
        assert!(!ComponentStorage::is_empty(&storage));
    }

    #[test]
    fn test_generic_function_with_component_storage() {
        // Test that we can write generic code over ComponentStorage
        #[allow(dead_code)]
        fn count_matching<S>(storage: &S, predicate: impl Fn(&Position) -> bool) -> usize
        where
            S: ComponentStorage<Item = Position> + AsRef<SparseSet<Position>>,
        {
            storage
                .as_ref()
                .iter()
                .filter(|(_, pos)| predicate(pos))
                .count()
        }

        // Note: This requires a custom wrapper or direct iteration
        // For now, just verify the trait works with the methods
        let mut storage: SparseSet<Position> = SparseSet::new();
        storage.insert(Entity::new(0, 1), Position { x: 1.0, y: 2.0 });
        storage.insert(Entity::new(1, 1), Position { x: 3.0, y: 4.0 });

        assert_eq!(ComponentStorage::len(&storage), 2);
    }
}

// =========================================================================
// AnyComponentStorage Tests
// =========================================================================

mod any_component_storage {
    use super::*;

    #[test]
    fn test_sparse_set_implements_any_component_storage() {
        fn assert_any_storage<S: AnyComponentStorage>() {}
        assert_any_storage::<SparseSet<Position>>();
        assert_any_storage::<SparseSet<Name>>();
    }

    #[test]
    fn test_any_storage_is_object_safe() {
        // Verify we can create trait objects
        let _storage: Box<dyn AnyComponentStorage> = Box::new(SparseSet::<Position>::new());
    }

    #[test]
    fn test_contains_entity_via_any() {
        let storage: Box<dyn AnyComponentStorage> = Box::new(SparseSet::<Position>::new());

        let entity = Entity::new(0, 1);

        assert!(!storage.contains_entity(entity));

        // We need to downcast to insert - this is expected
        // Type-erased storage is for querying, not typed operations
    }

    #[test]
    fn test_storage_len_via_any() {
        let mut storage = SparseSet::<Position>::new();
        storage.insert(Entity::new(0, 1), Position { x: 0.0, y: 0.0 });
        storage.insert(Entity::new(1, 1), Position { x: 1.0, y: 1.0 });

        let any_storage: &dyn AnyComponentStorage = &storage;

        assert_eq!(any_storage.storage_len(), 2);
        assert!(!any_storage.storage_is_empty());
    }

    #[test]
    fn test_remove_entity_via_any() {
        let mut storage = SparseSet::<Position>::new();
        let entity = Entity::new(0, 1);

        storage.insert(entity, Position { x: 1.0, y: 2.0 });

        let any_storage: &mut dyn AnyComponentStorage = &mut storage;

        assert!(any_storage.contains_entity(entity));
        let removed = any_storage.remove_entity(entity);

        assert!(removed);
        assert!(!any_storage.contains_entity(entity));
    }

    #[test]
    fn test_remove_nonexistent_via_any() {
        let mut storage = SparseSet::<Position>::new();
        let entity = Entity::new(0, 1);

        let any_storage: &mut dyn AnyComponentStorage = &mut storage;
        let removed = any_storage.remove_entity(entity);

        assert!(!removed);
    }

    #[test]
    fn test_clear_via_any() {
        let mut storage = SparseSet::<Position>::new();
        storage.insert(Entity::new(0, 1), Position { x: 0.0, y: 0.0 });
        storage.insert(Entity::new(1, 1), Position { x: 1.0, y: 1.0 });

        let any_storage: &mut dyn AnyComponentStorage = &mut storage;
        any_storage.clear();

        assert!(any_storage.storage_is_empty());
        assert_eq!(any_storage.storage_len(), 0);
    }

    #[test]
    fn test_component_type_name_via_any() {
        let storage = SparseSet::<Position>::new();
        let any_storage: &dyn AnyComponentStorage = &storage;

        let name = any_storage.component_type_name();
        assert!(name.contains("Position"));
    }

    #[test]
    fn test_multiple_storages_as_any() {
        let mut positions = SparseSet::<Position>::new();
        let mut velocities = SparseSet::<Velocity>::new();

        positions.insert(Entity::new(0, 1), Position { x: 1.0, y: 2.0 });
        velocities.insert(Entity::new(0, 1), Velocity { x: 3.0, y: 4.0 });

        let storages: Vec<&dyn AnyComponentStorage> = vec![&positions, &velocities];

        assert_eq!(storages[0].storage_len(), 1);
        assert_eq!(storages[1].storage_len(), 1);

        assert!(storages[0].component_type_name().contains("Position"));
        assert!(storages[1].component_type_name().contains("Velocity"));
    }

    #[test]
    fn test_any_storage_is_empty() {
        let storage: Box<dyn AnyComponentStorage> = Box::new(SparseSet::<Position>::new());
        assert!(storage.storage_is_empty());
    }
}

// =========================================================================
// Integration Tests
// =========================================================================

mod integration {
    use super::*;

    #[test]
    fn test_component_storage_workflow() {
        let mut storage: SparseSet<Position> = SparseSet::new();

        // Create entities
        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        // Insert via trait
        ComponentStorage::insert(&mut storage, e1, Position { x: 0.0, y: 0.0 });
        ComponentStorage::insert(&mut storage, e2, Position { x: 10.0, y: 20.0 });
        ComponentStorage::insert(&mut storage, e3, Position { x: 100.0, y: 200.0 });

        assert_eq!(ComponentStorage::len(&storage), 3);

        // Modify via trait
        if let Some(pos) = ComponentStorage::get_mut(&mut storage, e2) {
            pos.x += 5.0;
            pos.y += 5.0;
        }

        // Verify
        assert_eq!(
            ComponentStorage::get(&storage, e2),
            Some(&Position { x: 15.0, y: 25.0 })
        );

        // Remove via trait
        let removed = ComponentStorage::remove(&mut storage, e1);
        assert_eq!(removed, Some(Position { x: 0.0, y: 0.0 }));
        assert_eq!(ComponentStorage::len(&storage), 2);

        // Remaining entities still accessible
        assert!(ComponentStorage::contains(&storage, e2));
        assert!(ComponentStorage::contains(&storage, e3));
    }

    #[test]
    fn test_zero_sized_component_storage() {
        let mut storage: SparseSet<Player> = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        ComponentStorage::insert(&mut storage, e1, Player);
        ComponentStorage::insert(&mut storage, e2, Player);

        assert_eq!(ComponentStorage::len(&storage), 2);
        assert!(ComponentStorage::contains(&storage, e1));
        assert!(ComponentStorage::contains(&storage, e2));

        // ZST storage works correctly
        assert_eq!(ComponentStorage::get(&storage, e1), Some(&Player));
    }

    #[test]
    fn test_string_component_storage() {
        let mut storage: SparseSet<Name> = SparseSet::new();

        let entity = Entity::new(0, 1);
        ComponentStorage::insert(&mut storage, entity, Name("Player".to_string()));

        assert_eq!(
            ComponentStorage::get(&storage, entity),
            Some(&Name("Player".to_string()))
        );

        // Replace
        let old = ComponentStorage::insert(&mut storage, entity, Name("Enemy".to_string()));
        assert_eq!(old, Some(Name("Player".to_string())));
    }
}
