use super::*;

// =========================================================================
// Despawn Tests
// =========================================================================

mod despawn {
    use super::super::super::super::entity::Entity;
    use super::*;

    #[test]
    fn test_despawn_single_entity() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);

        let despawned = world.despawn(entity);
        assert!(despawned);
        assert!(!world.is_alive(entity));
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_despawn_returns_false_for_dead_entity() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        world.despawn(entity);

        // Despawning again should return false
        let despawned_again = world.despawn(entity);
        assert!(!despawned_again);
    }

    #[test]
    fn test_despawn_returns_false_for_never_allocated() {
        let mut world = World::new();

        // Entity that was never allocated
        let fake = Entity::new(999, 1);
        let despawned = world.despawn(fake);
        assert!(!despawned);
    }

    #[test]
    fn test_despawn_returns_false_for_placeholder() {
        let mut world = World::new();

        let despawned = world.despawn(Entity::PLACEHOLDER);
        assert!(!despawned);
    }

    #[test]
    fn test_despawn_removes_from_archetype() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();
        let e3 = world.spawn_empty();

        // Empty archetype should have 3 entities
        let empty_archetype = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
        assert_eq!(empty_archetype.len(), 3);

        // Despawn middle entity
        world.despawn(e2);

        // Empty archetype should now have 2 entities
        let empty_archetype = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
        assert_eq!(empty_archetype.len(), 2);

        // e1 and e3 should still be alive
        assert!(world.is_alive(e1));
        assert!(!world.is_alive(e2));
        assert!(world.is_alive(e3));
    }

    #[test]
    fn test_despawn_multiple_entities() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();
        let e3 = world.spawn_empty();

        world.despawn(e1);
        assert!(!world.is_alive(e1));
        assert_eq!(world.entity_count(), 2);

        world.despawn(e3);
        assert!(!world.is_alive(e3));
        assert_eq!(world.entity_count(), 1);

        world.despawn(e2);
        assert!(!world.is_alive(e2));
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_despawn_stale_entity() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        let stale = entity; // Copy of the entity

        // Despawn the original
        world.despawn(entity);

        // Spawn a new entity (may reuse the same index with new generation)
        let new_entity = world.spawn_empty();

        // The stale reference should not despawn the new entity
        if stale.index() == new_entity.index() {
            // Same index, different generation
            assert_ne!(stale.generation(), new_entity.generation());
        }

        // Despawning stale entity should fail
        let despawned = world.despawn(stale);
        assert!(!despawned);

        // New entity should still be alive
        assert!(world.is_alive(new_entity));
    }
}

// =========================================================================
// Despawn Batch Tests
// =========================================================================

mod despawn_batch {
    use super::super::super::super::entity::Entity;
    use super::*;

    #[test]
    fn test_despawn_batch_empty() {
        let mut world = World::new();

        let despawned = world.despawn_batch(&[]);
        assert_eq!(despawned, 0);
    }

    #[test]
    fn test_despawn_batch_single() {
        let mut world = World::new();

        let entity = world.spawn_empty();
        let despawned = world.despawn_batch(&[entity]);

        assert_eq!(despawned, 1);
        assert!(!world.is_alive(entity));
    }

    #[test]
    fn test_despawn_batch_multiple() {
        let mut world = World::new();

        let entities = world.spawn_batch(10);
        assert_eq!(world.entity_count(), 10);

        let despawned = world.despawn_batch(&entities);
        assert_eq!(despawned, 10);
        assert_eq!(world.entity_count(), 0);

        for entity in &entities {
            assert!(!world.is_alive(*entity));
        }
    }

    #[test]
    fn test_despawn_batch_partial_invalid() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();
        let e3 = world.spawn_empty();

        // Despawn e2 individually
        world.despawn(e2);

        // Batch despawn including already-dead entity
        let despawned = world.despawn_batch(&[e1, e2, e3]);

        // Only e1 and e3 should count as despawned
        assert_eq!(despawned, 2);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_despawn_batch_with_placeholder() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();

        // Batch despawn with placeholder
        let despawned = world.despawn_batch(&[e1, Entity::PLACEHOLDER, e2]);

        // Placeholder doesn't count
        assert_eq!(despawned, 2);
    }

    #[test]
    fn test_despawn_batch_large() {
        let mut world = World::new();

        let entities = world.spawn_batch(10_000);
        assert_eq!(world.entity_count(), 10_000);

        let despawned = world.despawn_batch(&entities);
        assert_eq!(despawned, 10_000);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_despawn_batch_duplicate_entities() {
        let mut world = World::new();

        let entity = world.spawn_empty();

        // Batch with duplicate - only first despawn should succeed
        let despawned = world.despawn_batch(&[entity, entity, entity]);

        assert_eq!(despawned, 1);
        assert!(!world.is_alive(entity));
    }

    #[test]
    fn test_despawn_batch_preserves_other_entities() {
        let mut world = World::new();

        let keep = world.spawn_batch(5);
        let remove = world.spawn_batch(5);

        assert_eq!(world.entity_count(), 10);

        world.despawn_batch(&remove);

        assert_eq!(world.entity_count(), 5);
        for entity in &keep {
            assert!(world.is_alive(*entity));
        }
        for entity in &remove {
            assert!(!world.is_alive(*entity));
        }
    }
}
