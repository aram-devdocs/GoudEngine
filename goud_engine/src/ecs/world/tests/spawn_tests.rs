use super::*;

mod spawn {
    use super::*;

    #[test]
    fn test_spawn_empty_creates_entity() {
        let mut world = World::new();

        let entity = world.spawn_empty();

        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_spawn_empty_adds_to_empty_archetype() {
        let mut world = World::new();

        let entity = world.spawn_empty();

        // Entity should be in the empty archetype
        let archetype_id = world.entity_archetype(entity);
        assert_eq!(archetype_id, Some(ArchetypeId::EMPTY));
    }

    #[test]
    fn test_spawn_empty_multiple_entities() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();
        let e3 = world.spawn_empty();

        assert!(world.is_alive(e1));
        assert!(world.is_alive(e2));
        assert!(world.is_alive(e3));
        assert_eq!(world.entity_count(), 3);

        // All should be in empty archetype
        assert_eq!(world.entity_archetype(e1), Some(ArchetypeId::EMPTY));
        assert_eq!(world.entity_archetype(e2), Some(ArchetypeId::EMPTY));
        assert_eq!(world.entity_archetype(e3), Some(ArchetypeId::EMPTY));
    }

    #[test]
    fn test_spawn_empty_unique_entities() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();

        // Entities should have different indices
        assert_ne!(e1, e2);
        assert_ne!(e1.index(), e2.index());
    }

    #[test]
    fn test_spawn_returns_builder() {
        let mut world = World::new();

        let entity = world.spawn().id();

        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_spawn_builder_id_matches() {
        let mut world = World::new();

        // Get entity ID from builder
        let builder = world.spawn();
        let entity = builder.id();

        // Entity should be alive and valid
        assert!(world.is_alive(entity));
    }

    #[test]
    fn test_spawn_builder_provides_world_access() {
        let mut world = World::new();

        let builder = world.spawn();

        // Should be able to access world read-only
        assert_eq!(builder.world().entity_count(), 1);
    }

    #[test]
    fn test_spawn_batch_empty() {
        let mut world = World::new();

        let entities = world.spawn_batch(0);

        assert!(entities.is_empty());
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_spawn_batch_single() {
        let mut world = World::new();

        let entities = world.spawn_batch(1);

        assert_eq!(entities.len(), 1);
        assert_eq!(world.entity_count(), 1);
        assert!(world.is_alive(entities[0]));
    }

    #[test]
    fn test_spawn_batch_multiple() {
        let mut world = World::new();

        let entities = world.spawn_batch(10);

        assert_eq!(entities.len(), 10);
        assert_eq!(world.entity_count(), 10);

        for entity in &entities {
            assert!(world.is_alive(*entity));
            assert_eq!(world.entity_archetype(*entity), Some(ArchetypeId::EMPTY));
        }
    }

    #[test]
    fn test_spawn_batch_large() {
        let mut world = World::new();

        let entities = world.spawn_batch(10_000);

        assert_eq!(entities.len(), 10_000);
        assert_eq!(world.entity_count(), 10_000);

        // Spot check a few
        assert!(world.is_alive(entities[0]));
        assert!(world.is_alive(entities[5000]));
        assert!(world.is_alive(entities[9999]));
    }

    #[test]
    fn test_spawn_batch_unique_entities() {
        let mut world = World::new();

        let entities = world.spawn_batch(100);

        // All entities should be unique
        let unique: std::collections::HashSet<_> = entities.iter().collect();
        assert_eq!(unique.len(), 100);
    }

    #[test]
    fn test_spawn_mixed_individual_and_batch() {
        let mut world = World::new();

        // Spawn some individually
        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();

        // Spawn a batch
        let batch = world.spawn_batch(5);

        // Spawn more individually
        let e3 = world.spawn_empty();

        assert_eq!(world.entity_count(), 8);
        assert!(world.is_alive(e1));
        assert!(world.is_alive(e2));
        assert!(world.is_alive(e3));
        for entity in &batch {
            assert!(world.is_alive(*entity));
        }
    }

    #[test]
    fn test_spawn_updates_archetype_entity_count() {
        let mut world = World::new();

        world.spawn_batch(5);

        // Empty archetype should have 5 entities
        let empty_archetype = world.archetypes().get(ArchetypeId::EMPTY).unwrap();
        assert_eq!(empty_archetype.len(), 5);
    }
}
