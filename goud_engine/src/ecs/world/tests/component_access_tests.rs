use super::*;

mod component_access {
    use super::super::super::super::entity::Entity;
    use super::*;

    // Helper function to insert a component directly via storage
    // (This simulates what insert() will do in Step 2.4.5)
    fn insert_component<T: Component>(world: &mut World, entity: Entity, component: T) {
        world.get_storage_mut::<T>().insert(entity, component);
    }

    // =====================================================================
    // get() Tests
    // =====================================================================

    #[test]
    fn test_get_returns_none_for_dead_entity() {
        let world = World::new();

        // Entity that was never allocated
        let fake = Entity::new(0, 1);
        assert!(world.get::<Position>(fake).is_none());
    }

    #[test]
    fn test_get_returns_none_for_placeholder() {
        let world = World::new();
        assert!(world.get::<Position>(Entity::PLACEHOLDER).is_none());
    }

    #[test]
    fn test_get_returns_none_when_no_storage_exists() {
        let mut world = World::new();

        // Spawn entity but don't add any components
        let entity = world.spawn_empty();

        // No storage for Position exists yet
        assert!(world.get::<Position>(entity).is_none());
    }

    #[test]
    fn test_get_returns_none_when_entity_lacks_component() {
        let mut world = World::new();

        // Spawn two entities
        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();

        // Add component to e1 only
        insert_component(&mut world, e1, Position { x: 1.0, y: 2.0 });

        // e1 has component
        assert!(world.get::<Position>(e1).is_some());

        // e2 does not have component
        assert!(world.get::<Position>(e2).is_none());
    }

    #[test]
    fn test_get_returns_correct_component() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        let pos = Position { x: 10.0, y: 20.0 };
        insert_component(&mut world, entity, pos);

        let result = world.get::<Position>(entity);
        assert_eq!(result, Some(&Position { x: 10.0, y: 20.0 }));
    }

    #[test]
    fn test_get_returns_correct_component_for_multiple_entities() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();
        let e3 = world.spawn_empty();

        insert_component(&mut world, e1, Position { x: 1.0, y: 1.0 });
        insert_component(&mut world, e2, Position { x: 2.0, y: 2.0 });
        insert_component(&mut world, e3, Position { x: 3.0, y: 3.0 });

        assert_eq!(
            world.get::<Position>(e1),
            Some(&Position { x: 1.0, y: 1.0 })
        );
        assert_eq!(
            world.get::<Position>(e2),
            Some(&Position { x: 2.0, y: 2.0 })
        );
        assert_eq!(
            world.get::<Position>(e3),
            Some(&Position { x: 3.0, y: 3.0 })
        );
    }

    #[test]
    fn test_get_different_component_types() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });
        insert_component(&mut world, entity, Velocity { x: 3.0, y: 4.0 });

        assert_eq!(
            world.get::<Position>(entity),
            Some(&Position { x: 1.0, y: 2.0 })
        );
        assert_eq!(
            world.get::<Velocity>(entity),
            Some(&Velocity { x: 3.0, y: 4.0 })
        );
    }

    #[test]
    fn test_get_returns_none_after_despawn() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });

        // Component exists before despawn
        assert!(world.get::<Position>(entity).is_some());

        // Despawn the entity
        world.despawn(entity);

        // Component no longer accessible (entity is dead)
        assert!(world.get::<Position>(entity).is_none());
    }

    // =====================================================================
    // get_mut() Tests
    // =====================================================================

    #[test]
    fn test_get_mut_returns_none_for_dead_entity() {
        let mut world = World::new();

        let fake = Entity::new(999, 1);
        assert!(world.get_mut::<Position>(fake).is_none());
    }

    #[test]
    fn test_get_mut_returns_none_for_placeholder() {
        let mut world = World::new();
        assert!(world.get_mut::<Position>(Entity::PLACEHOLDER).is_none());
    }

    #[test]
    fn test_get_mut_returns_none_when_no_storage_exists() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        assert!(world.get_mut::<Velocity>(entity).is_none());
    }

    #[test]
    fn test_get_mut_returns_mutable_reference() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });

        // Modify via mutable reference
        if let Some(pos) = world.get_mut::<Position>(entity) {
            pos.x = 100.0;
            pos.y = 200.0;
        }

        // Verify modification persisted
        assert_eq!(
            world.get::<Position>(entity),
            Some(&Position { x: 100.0, y: 200.0 })
        );
    }

    #[test]
    fn test_get_mut_returns_none_after_despawn() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });

        world.despawn(entity);

        assert!(world.get_mut::<Position>(entity).is_none());
    }

    // =====================================================================
    // has() Tests
    // =====================================================================

    #[test]
    fn test_has_returns_false_for_dead_entity() {
        let world = World::new();

        let fake = Entity::new(42, 1);
        assert!(!world.has::<Position>(fake));
    }

    #[test]
    fn test_has_returns_false_for_placeholder() {
        let world = World::new();
        assert!(!world.has::<Position>(Entity::PLACEHOLDER));
    }

    #[test]
    fn test_has_returns_false_when_no_storage_exists() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        assert!(!world.has::<Position>(entity));
    }

    #[test]
    fn test_has_returns_false_when_entity_lacks_component() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();

        insert_component(&mut world, e1, Position { x: 0.0, y: 0.0 });

        assert!(world.has::<Position>(e1));
        assert!(!world.has::<Position>(e2));
    }

    #[test]
    fn test_has_returns_true_when_entity_has_component() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        insert_component(&mut world, entity, Player);

        assert!(world.has::<Player>(entity));
    }

    #[test]
    fn test_has_distinguishes_different_component_types() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        insert_component(&mut world, entity, Position { x: 0.0, y: 0.0 });

        assert!(world.has::<Position>(entity));
        assert!(!world.has::<Velocity>(entity));
        assert!(!world.has::<Name>(entity));
        assert!(!world.has::<Player>(entity));
    }

    #[test]
    fn test_has_returns_false_after_despawn() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });

        assert!(world.has::<Position>(entity));

        world.despawn(entity);

        assert!(!world.has::<Position>(entity));
    }

    // =====================================================================
    // Component Access with Multiple Component Types
    // =====================================================================

    #[test]
    fn test_access_multiple_component_types_on_same_entity() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });
        insert_component(&mut world, entity, Velocity { x: 3.0, y: 4.0 });
        insert_component(&mut world, entity, Name("Test".to_string()));
        insert_component(&mut world, entity, Player);

        // All should be accessible
        assert!(world.has::<Position>(entity));
        assert!(world.has::<Velocity>(entity));
        assert!(world.has::<Name>(entity));
        assert!(world.has::<Player>(entity));

        assert_eq!(
            world.get::<Position>(entity),
            Some(&Position { x: 1.0, y: 2.0 })
        );
        assert_eq!(
            world.get::<Velocity>(entity),
            Some(&Velocity { x: 3.0, y: 4.0 })
        );
        assert_eq!(world.get::<Name>(entity), Some(&Name("Test".to_string())));
        assert_eq!(world.get::<Player>(entity), Some(&Player));
    }

    // =====================================================================
    // Type Safety Tests
    // =====================================================================

    #[test]
    fn test_type_safety_different_types_same_layout() {
        // Two components with same memory layout should not conflict
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct TypeA(f32, f32);
        impl Component for TypeA {}

        #[derive(Debug, Clone, Copy, PartialEq)]
        struct TypeB(f32, f32);
        impl Component for TypeB {}

        let mut world = World::new();
        let entity = world.spawn_empty();

        insert_component(&mut world, entity, TypeA(1.0, 2.0));
        insert_component(&mut world, entity, TypeB(3.0, 4.0));

        // Types should be distinct
        assert_eq!(world.get::<TypeA>(entity), Some(&TypeA(1.0, 2.0)));
        assert_eq!(world.get::<TypeB>(entity), Some(&TypeB(3.0, 4.0)));
    }

    // =====================================================================
    // Stale Entity Reference Tests
    // =====================================================================

    #[test]
    fn test_get_returns_none_for_stale_entity() {
        let mut world = World::new();

        // Spawn and despawn an entity
        let entity = world.spawn_empty();
        insert_component(&mut world, entity, Position { x: 1.0, y: 2.0 });
        let stale = entity;
        world.despawn(entity);

        // Spawn a new entity at the same index
        let new_entity = world.spawn_empty();
        insert_component(&mut world, new_entity, Position { x: 99.0, y: 99.0 });

        // If same index, generations should differ
        if stale.index() == new_entity.index() {
            // Stale reference should not access new entity's component
            assert!(world.get::<Position>(stale).is_none());
            // New entity should have its own component
            assert_eq!(
                world.get::<Position>(new_entity),
                Some(&Position { x: 99.0, y: 99.0 })
            );
        }
    }

    #[test]
    fn test_has_returns_false_for_stale_entity() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        insert_component(&mut world, entity, Player);
        let stale = entity;
        world.despawn(entity);

        let new_entity = world.spawn_empty();
        insert_component(&mut world, new_entity, Player);

        if stale.index() == new_entity.index() {
            assert!(!world.has::<Player>(stale));
            assert!(world.has::<Player>(new_entity));
        }
    }
}
