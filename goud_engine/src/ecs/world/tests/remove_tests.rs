use super::*;

mod remove {
    use super::super::super::super::entity::Entity;
    use super::*;

    // =====================================================================
    // Basic Removal
    // =====================================================================

    #[test]
    fn test_remove_returns_component() {
        let mut world = World::new();
        let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

        let removed = world.remove::<Position>(entity);
        assert_eq!(removed, Some(Position { x: 1.0, y: 2.0 }));
    }

    #[test]
    fn test_remove_entity_no_longer_has_component() {
        let mut world = World::new();
        let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

        world.remove::<Position>(entity);

        assert!(!world.has::<Position>(entity));
        assert!(world.get::<Position>(entity).is_none());
    }

    #[test]
    fn test_remove_nonexistent_component_returns_none() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        let removed = world.remove::<Position>(entity);
        assert!(removed.is_none());
    }

    #[test]
    fn test_remove_twice_returns_none_second_time() {
        let mut world = World::new();
        let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

        let first = world.remove::<Position>(entity);
        let second = world.remove::<Position>(entity);

        assert_eq!(first, Some(Position { x: 1.0, y: 2.0 }));
        assert!(second.is_none());
    }

    #[test]
    fn test_remove_dead_entity_returns_none() {
        let mut world = World::new();
        let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();
        world.despawn(entity);

        let removed = world.remove::<Position>(entity);
        assert!(removed.is_none());
    }

    #[test]
    fn test_remove_placeholder_returns_none() {
        let mut world = World::new();

        let removed = world.remove::<Position>(Entity::PLACEHOLDER);
        assert!(removed.is_none());
    }

    #[test]
    fn test_remove_never_allocated_entity_returns_none() {
        let mut world = World::new();
        let fake = Entity::new(999, 1);

        let removed = world.remove::<Position>(fake);
        assert!(removed.is_none());
    }

    // =====================================================================
    // Archetype Transitions
    // =====================================================================

    #[test]
    fn test_remove_triggers_archetype_transition() {
        let mut world = World::new();
        let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

        // Entity should be in archetype with Position
        let archetype_before = world.entity_archetype(entity).unwrap();
        assert!(!archetype_before.is_empty()); // Not in empty archetype

        // Remove the component
        world.remove::<Position>(entity);

        // Entity should be in empty archetype
        let archetype_after = world.entity_archetype(entity).unwrap();
        assert!(archetype_after.is_empty()); // Now in empty archetype
    }

    #[test]
    fn test_remove_to_empty_archetype() {
        let mut world = World::new();
        let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

        world.remove::<Position>(entity);

        // Entity should still be alive
        assert!(world.is_alive(entity));

        // Entity should be in empty archetype
        let archetype_id = world.entity_archetype(entity).unwrap();
        assert_eq!(archetype_id, ArchetypeId::EMPTY);
    }

    #[test]
    fn test_remove_one_of_multiple_components() {
        let mut world = World::new();
        let entity = world
            .spawn()
            .insert(Position { x: 1.0, y: 2.0 })
            .insert(Velocity { x: 3.0, y: 4.0 })
            .id();

        // Remove only Position
        world.remove::<Position>(entity);

        // Should still have Velocity
        assert!(!world.has::<Position>(entity));
        assert!(world.has::<Velocity>(entity));
        assert_eq!(
            world.get::<Velocity>(entity),
            Some(&Velocity { x: 3.0, y: 4.0 })
        );
    }

    #[test]
    fn test_remove_creates_correct_target_archetype() {
        let mut world = World::new();

        // Create entity with Position + Velocity
        let entity1 = world
            .spawn()
            .insert(Position { x: 0.0, y: 0.0 })
            .insert(Velocity { x: 0.0, y: 0.0 })
            .id();

        // Create another entity with just Velocity
        let entity2 = world.spawn().insert(Velocity { x: 1.0, y: 1.0 }).id();

        // Get archetype for entity with just Velocity
        let velocity_archetype = world.entity_archetype(entity2).unwrap();

        // Remove Position from entity1
        world.remove::<Position>(entity1);

        // Entity1 should now be in same archetype as entity2 (just Velocity)
        assert_eq!(world.entity_archetype(entity1), Some(velocity_archetype));
    }

    #[test]
    fn test_remove_removes_from_old_archetype() {
        let mut world = World::new();
        let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

        let archetype_before = world.entity_archetype(entity).unwrap();

        // Verify entity is in old archetype
        {
            let arch = world.archetypes().get(archetype_before).unwrap();
            assert!(arch.contains_entity(entity));
        }

        // Remove component
        world.remove::<Position>(entity);

        // Verify entity is NOT in old archetype anymore
        {
            let arch = world.archetypes().get(archetype_before).unwrap();
            assert!(!arch.contains_entity(entity));
        }
    }

    #[test]
    fn test_remove_adds_to_new_archetype() {
        let mut world = World::new();
        let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

        world.remove::<Position>(entity);

        // Verify entity is in empty archetype
        let arch = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
        assert!(arch.contains_entity(entity));
    }

    // =====================================================================
    // Take (alias for remove)
    // =====================================================================

    #[test]
    fn test_take_returns_component() {
        let mut world = World::new();
        let entity = world.spawn().insert(Position { x: 5.0, y: 10.0 }).id();

        let taken = world.take::<Position>(entity);
        assert_eq!(taken, Some(Position { x: 5.0, y: 10.0 }));
    }

    #[test]
    fn test_take_removes_component() {
        let mut world = World::new();
        let entity = world.spawn().insert(Velocity { x: 1.0, y: 2.0 }).id();

        world.take::<Velocity>(entity);

        assert!(!world.has::<Velocity>(entity));
    }

    #[test]
    fn test_take_dead_entity_returns_none() {
        let mut world = World::new();
        let entity = world.spawn().insert(Position { x: 0.0, y: 0.0 }).id();
        world.despawn(entity);

        assert!(world.take::<Position>(entity).is_none());
    }

    // =====================================================================
    // Edge Cases
    // =====================================================================

    #[test]
    fn test_remove_after_despawn_and_respawn() {
        let mut world = World::new();

        let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();
        let old_entity = entity;
        world.despawn(entity);

        // Spawn new entity (might reuse slot)
        let new_entity = world.spawn().insert(Position { x: 10.0, y: 20.0 }).id();

        // New entity's component can be removed
        let removed = world.remove::<Position>(new_entity);
        assert_eq!(removed, Some(Position { x: 10.0, y: 20.0 }));

        // Old entity (stale) cannot have components removed
        if old_entity.index() == new_entity.index() {
            assert!(world.remove::<Position>(old_entity).is_none());
        }
    }

    #[test]
    fn test_remove_string_component() {
        let mut world = World::new();
        let entity = world.spawn().insert(Name("Test Name".to_string())).id();

        let removed = world.remove::<Name>(entity);
        assert_eq!(removed, Some(Name("Test Name".to_string())));
        assert!(!world.has::<Name>(entity));
    }

    #[test]
    fn test_remove_large_component() {
        #[derive(Debug, Clone, PartialEq)]
        struct LargeComponent {
            data: [u8; 1024],
        }
        impl Component for LargeComponent {}

        let mut world = World::new();
        let entity = world.spawn_empty();

        let large = LargeComponent { data: [42; 1024] };
        world.insert(entity, large.clone());

        let removed = world.remove::<LargeComponent>(entity);
        assert_eq!(removed, Some(large));
    }

    #[test]
    fn test_remove_stale_entity() {
        let mut world = World::new();

        let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();
        let stale = entity;

        world.despawn(entity);

        // Spawn enough entities to potentially reuse the slot
        let _new_entity = world.spawn().insert(Position { x: 99.0, y: 99.0 }).id();

        // Stale entity should return None
        assert!(world.remove::<Position>(stale).is_none());
    }

    // =====================================================================
    // Stress Tests
    // =====================================================================

    #[test]
    fn test_remove_stress_many_entities() {
        let mut world = World::new();

        // Spawn entities with Position
        let entities: Vec<Entity> = (0..1000)
            .map(|i| {
                world
                    .spawn()
                    .insert(Position {
                        x: i as f32,
                        y: (i * 2) as f32,
                    })
                    .id()
            })
            .collect();

        // Remove Position from all entities
        for (i, &entity) in entities.iter().enumerate() {
            let removed = world.remove::<Position>(entity);
            assert_eq!(
                removed,
                Some(Position {
                    x: i as f32,
                    y: (i * 2) as f32
                })
            );
        }

        // All entities should be alive but without Position
        for &entity in &entities {
            assert!(world.is_alive(entity));
            assert!(!world.has::<Position>(entity));
            assert!(world.entity_archetype(entity) == Some(ArchetypeId::EMPTY));
        }
    }

    #[test]
    fn test_remove_add_cycle() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        // Add and remove the same component multiple times
        for i in 0..100 {
            world.insert(
                entity,
                Position {
                    x: i as f32,
                    y: 0.0,
                },
            );
            assert!(world.has::<Position>(entity));

            let removed = world.remove::<Position>(entity);
            assert_eq!(
                removed,
                Some(Position {
                    x: i as f32,
                    y: 0.0
                })
            );
            assert!(!world.has::<Position>(entity));
        }

        // Entity should be alive and in empty archetype
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_archetype(entity), Some(ArchetypeId::EMPTY));
    }

    #[test]
    fn test_remove_preserves_other_entities_components() {
        let mut world = World::new();

        // Create two entities with the same components
        let entity1 = world
            .spawn()
            .insert(Position { x: 1.0, y: 1.0 })
            .insert(Velocity { x: 1.0, y: 1.0 })
            .id();

        let entity2 = world
            .spawn()
            .insert(Position { x: 2.0, y: 2.0 })
            .insert(Velocity { x: 2.0, y: 2.0 })
            .id();

        // Remove Position from entity1 only
        world.remove::<Position>(entity1);

        // Entity1 should no longer have Position
        assert!(!world.has::<Position>(entity1));
        assert!(world.has::<Velocity>(entity1));

        // Entity2 should still have both components unchanged
        assert!(world.has::<Position>(entity2));
        assert!(world.has::<Velocity>(entity2));
        assert_eq!(
            world.get::<Position>(entity2),
            Some(&Position { x: 2.0, y: 2.0 })
        );
        assert_eq!(
            world.get::<Velocity>(entity2),
            Some(&Velocity { x: 2.0, y: 2.0 })
        );
    }

    #[test]
    fn test_remove_different_types_same_entity() {
        let mut world = World::new();

        let entity = world
            .spawn()
            .insert(Position { x: 1.0, y: 2.0 })
            .insert(Velocity { x: 3.0, y: 4.0 })
            .insert(Player)
            .id();

        // Remove each component type in sequence
        let pos = world.remove::<Position>(entity);
        assert_eq!(pos, Some(Position { x: 1.0, y: 2.0 }));
        assert!(world.has::<Velocity>(entity));
        assert!(world.has::<Player>(entity));

        let vel = world.remove::<Velocity>(entity);
        assert_eq!(vel, Some(Velocity { x: 3.0, y: 4.0 }));
        assert!(world.has::<Player>(entity));

        let player = world.remove::<Player>(entity);
        assert!(player.is_some()); // Player is a unit struct

        // Entity should be in empty archetype
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_archetype(entity), Some(ArchetypeId::EMPTY));
    }
}
