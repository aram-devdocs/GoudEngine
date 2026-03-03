use super::*;

mod insert {
    use super::super::super::super::entity::Entity;
    use super::*;

    // =====================================================================
    // World::insert() Basic Tests
    // =====================================================================

    #[test]
    fn test_insert_first_component_returns_none() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        let old = world.insert(entity, Position { x: 1.0, y: 2.0 });
        assert!(old.is_none());
    }

    #[test]
    fn test_insert_makes_component_accessible() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        world.insert(entity, Position { x: 1.0, y: 2.0 });

        assert!(world.has::<Position>(entity));
        assert_eq!(
            world.get::<Position>(entity),
            Some(&Position { x: 1.0, y: 2.0 })
        );
    }

    #[test]
    fn test_insert_replace_returns_old_value() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        world.insert(entity, Position { x: 1.0, y: 2.0 });
        let old = world.insert(entity, Position { x: 10.0, y: 20.0 });

        assert_eq!(old, Some(Position { x: 1.0, y: 2.0 }));
        assert_eq!(
            world.get::<Position>(entity),
            Some(&Position { x: 10.0, y: 20.0 })
        );
    }

    #[test]
    fn test_insert_on_dead_entity_returns_none() {
        let mut world = World::new();
        let entity = world.spawn_empty();
        world.despawn(entity);

        let result = world.insert(entity, Position { x: 1.0, y: 2.0 });
        assert!(result.is_none());
    }

    #[test]
    fn test_insert_on_placeholder_returns_none() {
        let mut world = World::new();

        let result = world.insert(Entity::PLACEHOLDER, Position { x: 1.0, y: 2.0 });
        assert!(result.is_none());
    }

    #[test]
    fn test_insert_on_never_allocated_entity_returns_none() {
        let mut world = World::new();

        let fake = Entity::new(999, 1);
        let result = world.insert(fake, Position { x: 1.0, y: 2.0 });
        assert!(result.is_none());
    }

    // =====================================================================
    // Archetype Transition Tests
    // =====================================================================

    #[test]
    fn test_insert_triggers_archetype_transition() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        // Entity starts in empty archetype
        assert_eq!(world.entity_archetype(entity), Some(ArchetypeId::EMPTY));

        // Insert component
        world.insert(entity, Position { x: 1.0, y: 2.0 });

        // Entity should now be in a different archetype
        let archetype_id = world.entity_archetype(entity).unwrap();
        assert_ne!(archetype_id, ArchetypeId::EMPTY);
    }

    #[test]
    fn test_insert_second_component_creates_new_archetype() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        world.insert(entity, Position { x: 1.0, y: 2.0 });
        let arch_with_pos = world.entity_archetype(entity).unwrap();

        world.insert(entity, Velocity { x: 3.0, y: 4.0 });
        let arch_with_both = world.entity_archetype(entity).unwrap();

        // Should be a different archetype
        assert_ne!(arch_with_pos, arch_with_both);

        // Both components should be accessible
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
    fn test_insert_removes_entity_from_old_archetype() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        // Empty archetype should have 1 entity
        let empty_archetype = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
        assert_eq!(empty_archetype.len(), 1);

        world.insert(entity, Position { x: 1.0, y: 2.0 });

        // Empty archetype should now be empty
        let empty_archetype = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
        assert_eq!(empty_archetype.len(), 0);
    }

    #[test]
    fn test_insert_adds_entity_to_new_archetype() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        world.insert(entity, Position { x: 1.0, y: 2.0 });

        let archetype_id = world.entity_archetype(entity).unwrap();
        let archetype = world.archetypes().get(archetype_id).unwrap();

        // New archetype should contain the entity
        assert!(archetype.contains_entity(entity));
        assert_eq!(archetype.len(), 1);
    }

    #[test]
    fn test_insert_same_component_does_not_change_archetype() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        world.insert(entity, Position { x: 1.0, y: 2.0 });
        let arch_before = world.entity_archetype(entity).unwrap();

        world.insert(entity, Position { x: 10.0, y: 20.0 });
        let arch_after = world.entity_archetype(entity).unwrap();

        // Should be same archetype
        assert_eq!(arch_before, arch_after);
    }

    #[test]
    fn test_insert_creates_correct_archetype_count() {
        let mut world = World::new();

        // Start with 1 archetype (empty)
        assert_eq!(world.archetype_count(), 1);

        let entity = world.spawn_empty();

        // Add Position - creates archetype with Position
        world.insert(entity, Position { x: 0.0, y: 0.0 });
        assert_eq!(world.archetype_count(), 2);

        // Add Velocity - creates archetype with Position+Velocity
        world.insert(entity, Velocity { x: 0.0, y: 0.0 });
        assert_eq!(world.archetype_count(), 3);

        // Replace Position - no new archetype
        world.insert(entity, Position { x: 1.0, y: 1.0 });
        assert_eq!(world.archetype_count(), 3);
    }

    // =====================================================================
    // Multiple Entity Tests
    // =====================================================================

    #[test]
    fn test_insert_multiple_entities_same_components() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();
        let e3 = world.spawn_empty();

        world.insert(e1, Position { x: 1.0, y: 1.0 });
        world.insert(e2, Position { x: 2.0, y: 2.0 });
        world.insert(e3, Position { x: 3.0, y: 3.0 });

        // All should have Position
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

        // All should be in same archetype
        assert_eq!(world.entity_archetype(e1), world.entity_archetype(e2));
        assert_eq!(world.entity_archetype(e2), world.entity_archetype(e3));
    }

    #[test]
    fn test_insert_multiple_entities_different_components() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();
        let e3 = world.spawn_empty();

        world.insert(e1, Position { x: 1.0, y: 1.0 });
        world.insert(e2, Velocity { x: 2.0, y: 2.0 });
        world.insert(e3, Player);

        // Each should have its own component
        assert!(world.has::<Position>(e1));
        assert!(!world.has::<Velocity>(e1));

        assert!(world.has::<Velocity>(e2));
        assert!(!world.has::<Position>(e2));

        assert!(world.has::<Player>(e3));
        assert!(!world.has::<Position>(e3));

        // All should be in different archetypes
        assert_ne!(world.entity_archetype(e1), world.entity_archetype(e2));
        assert_ne!(world.entity_archetype(e2), world.entity_archetype(e3));
    }

    // =====================================================================
    // Component Type Registration Tests
    // =====================================================================

    #[test]
    fn test_insert_registers_component_type() {
        let mut world = World::new();

        assert!(!world.has_component_type::<Position>());

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 0.0, y: 0.0 });

        assert!(world.has_component_type::<Position>());
    }

    #[test]
    fn test_insert_multiple_types_registers_all() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 0.0, y: 0.0 });
        world.insert(entity, Velocity { x: 0.0, y: 0.0 });
        world.insert(entity, Player);

        assert_eq!(world.component_type_count(), 3);
        assert!(world.has_component_type::<Position>());
        assert!(world.has_component_type::<Velocity>());
        assert!(world.has_component_type::<Player>());
    }

    // =====================================================================
    // Edge Cases
    // =====================================================================

    #[test]
    fn test_insert_after_despawn_and_respawn() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });
        world.despawn(entity);

        // Spawn new entity (might reuse slot)
        let new_entity = world.spawn_empty();
        world.insert(new_entity, Position { x: 10.0, y: 20.0 });

        // New entity should have its component
        assert_eq!(
            world.get::<Position>(new_entity),
            Some(&Position { x: 10.0, y: 20.0 })
        );

        // Old entity (stale) should not
        if entity.index() == new_entity.index() {
            assert!(world.get::<Position>(entity).is_none());
        }
    }

    #[test]
    fn test_insert_large_component() {
        #[derive(Debug, Clone, PartialEq)]
        struct LargeComponent {
            data: [u8; 1024],
        }
        impl Component for LargeComponent {}

        let mut world = World::new();
        let entity = world.spawn_empty();

        let large = LargeComponent { data: [42; 1024] };
        world.insert(entity, large.clone());

        assert_eq!(world.get::<LargeComponent>(entity), Some(&large));
    }

    #[test]
    fn test_insert_string_component() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        world.insert(entity, Name("Hello, World!".to_string()));

        assert_eq!(
            world.get::<Name>(entity),
            Some(&Name("Hello, World!".to_string()))
        );
    }

    #[test]
    fn test_insert_stress_many_entities() {
        let mut world = World::new();

        let entities = world.spawn_batch(10_000);

        for (i, &entity) in entities.iter().enumerate() {
            world.insert(
                entity,
                Position {
                    x: i as f32,
                    y: (i * 2) as f32,
                },
            );
        }

        // Spot check
        assert_eq!(
            world.get::<Position>(entities[0]),
            Some(&Position { x: 0.0, y: 0.0 })
        );
        assert_eq!(
            world.get::<Position>(entities[5000]),
            Some(&Position {
                x: 5000.0,
                y: 10000.0
            })
        );
        assert_eq!(
            world.get::<Position>(entities[9999]),
            Some(&Position {
                x: 9999.0,
                y: 19998.0
            })
        );
    }

    #[test]
    fn test_insert_stress_many_component_types() {
        // Test with 10 different component types on single entity
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct C1(u32);
        impl Component for C1 {}
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct C2(u32);
        impl Component for C2 {}
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct C3(u32);
        impl Component for C3 {}
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct C4(u32);
        impl Component for C4 {}
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct C5(u32);
        impl Component for C5 {}
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct C6(u32);
        impl Component for C6 {}
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct C7(u32);
        impl Component for C7 {}
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct C8(u32);
        impl Component for C8 {}
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct C9(u32);
        impl Component for C9 {}
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct C10(u32);
        impl Component for C10 {}

        let mut world = World::new();
        let entity = world.spawn_empty();

        world.insert(entity, C1(1));
        world.insert(entity, C2(2));
        world.insert(entity, C3(3));
        world.insert(entity, C4(4));
        world.insert(entity, C5(5));
        world.insert(entity, C6(6));
        world.insert(entity, C7(7));
        world.insert(entity, C8(8));
        world.insert(entity, C9(9));
        world.insert(entity, C10(10));

        assert_eq!(world.get::<C1>(entity), Some(&C1(1)));
        assert_eq!(world.get::<C5>(entity), Some(&C5(5)));
        assert_eq!(world.get::<C10>(entity), Some(&C10(10)));
        assert_eq!(world.component_type_count(), 10);
    }
}
