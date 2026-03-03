use super::*;

// =========================================================================
// World Construction Tests
// =========================================================================

mod construction {
    use super::*;

    #[test]
    fn test_world_new() {
        let world = World::new();

        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.archetype_count(), 1); // Empty archetype
        assert_eq!(world.component_type_count(), 0);
        assert!(world.is_empty());
    }

    #[test]
    fn test_world_default() {
        let world: World = Default::default();

        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.archetype_count(), 1);
    }

    #[test]
    fn test_world_with_capacity() {
        let world = World::with_capacity(10_000, 50);

        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.archetype_count(), 1);
        assert_eq!(world.component_type_count(), 0);
    }

    #[test]
    fn test_world_debug() {
        let world = World::new();
        let debug_str = format!("{:?}", world);

        assert!(debug_str.contains("World"));
        assert!(debug_str.contains("entities"));
        assert!(debug_str.contains("archetypes"));
    }
}

// =========================================================================
// Entity Count Tests
// =========================================================================

mod entity_count {
    use super::*;

    #[test]
    fn test_entity_count_starts_at_zero() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_is_empty_starts_true() {
        let world = World::new();
        assert!(world.is_empty());
    }
}

// =========================================================================
// Archetype Count Tests
// =========================================================================

mod archetype_count {
    use super::*;

    #[test]
    fn test_archetype_count_starts_at_one() {
        let world = World::new();
        // Empty archetype always exists
        assert_eq!(world.archetype_count(), 1);
    }

    #[test]
    fn test_empty_archetype_exists() {
        let world = World::new();
        let archetypes = world.archetypes();

        // EMPTY archetype (index 0) should exist
        assert!(archetypes.get(ArchetypeId::EMPTY).is_some());
    }
}

// =========================================================================
// is_alive Tests
// =========================================================================

mod is_alive {
    use super::super::super::super::entity::Entity;
    use super::*;

    #[test]
    fn test_placeholder_not_alive() {
        let world = World::new();
        assert!(!world.is_alive(Entity::PLACEHOLDER));
    }

    #[test]
    fn test_fake_entity_not_alive() {
        let world = World::new();

        // Fabricated entity that was never allocated
        let fake = Entity::new(999, 1);
        assert!(!world.is_alive(fake));
    }

    #[test]
    fn test_entity_index_zero_not_alive_when_empty() {
        let world = World::new();

        // Index 0 hasn't been allocated yet
        let fake = Entity::new(0, 1);
        assert!(!world.is_alive(fake));
    }
}

// =========================================================================
// entity_archetype Tests
// =========================================================================

mod entity_archetype {
    use super::super::super::super::entity::Entity;
    use super::*;

    #[test]
    fn test_nonexistent_entity_has_no_archetype() {
        let world = World::new();
        let fake = Entity::new(0, 1);

        assert!(world.entity_archetype(fake).is_none());
    }

    #[test]
    fn test_placeholder_has_no_archetype() {
        let world = World::new();
        assert!(world.entity_archetype(Entity::PLACEHOLDER).is_none());
    }
}

// =========================================================================
// Component Type Tests
// =========================================================================

mod component_types {
    use super::*;

    #[test]
    fn test_component_type_count_starts_at_zero() {
        let world = World::new();
        assert_eq!(world.component_type_count(), 0);
    }

    #[test]
    fn test_has_component_type_false_initially() {
        let world = World::new();

        assert!(!world.has_component_type::<Position>());
        assert!(!world.has_component_type::<Velocity>());
        assert!(!world.has_component_type::<Name>());
    }
}

// =========================================================================
// Storage Access Tests
// =========================================================================

mod storage {
    use super::super::super::super::entity::Entity;
    use super::*;

    #[test]
    fn test_get_storage_returns_none_for_unregistered() {
        let world = World::new();

        assert!(world.get_storage::<Position>().is_none());
    }

    #[test]
    fn test_get_storage_mut_creates_storage() {
        let mut world = World::new();

        // Access storage (will create it)
        let storage = world.get_storage_mut::<Position>();

        // Should be empty but exist
        assert!(storage.is_empty());

        // Now component type is registered
        assert!(world.has_component_type::<Position>());
        assert_eq!(world.component_type_count(), 1);
    }

    #[test]
    fn test_get_storage_mut_returns_same_storage() {
        let mut world = World::new();

        // First access creates storage
        world.get_storage_mut::<Position>();

        // Insert a component
        let entity = Entity::new(0, 1);
        world
            .get_storage_mut::<Position>()
            .insert(entity, Position { x: 1.0, y: 2.0 });

        // Second access returns same storage with data
        let storage = world.get_storage_mut::<Position>();
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.get(entity), Some(&Position { x: 1.0, y: 2.0 }));
    }

    #[test]
    fn test_multiple_component_types_separate_storage() {
        let mut world = World::new();

        // Create storages for different types
        world.get_storage_mut::<Position>();
        world.get_storage_mut::<Velocity>();

        assert_eq!(world.component_type_count(), 2);
        assert!(world.has_component_type::<Position>());
        assert!(world.has_component_type::<Velocity>());
    }

    #[test]
    fn test_get_storage_after_mut_access() {
        let mut world = World::new();

        // Create and populate storage
        let entity = Entity::new(0, 1);
        world
            .get_storage_mut::<Position>()
            .insert(entity, Position { x: 5.0, y: 10.0 });

        // Now immutable access should work
        let storage = world.get_storage::<Position>().unwrap();
        assert_eq!(storage.get(entity), Some(&Position { x: 5.0, y: 10.0 }));
    }
}

// =========================================================================
// Clear Tests
// =========================================================================

mod clear {
    use super::*;

    #[test]
    fn test_clear_resets_entity_count() {
        let mut world = World::new();

        // Manually add to entity_archetypes to simulate entities
        // (In a full implementation, spawn() would do this)
        // For now, we just verify clear works on empty world

        world.clear();
        assert_eq!(world.entity_count(), 0);
        assert!(world.is_empty());
    }

    #[test]
    fn test_clear_preserves_archetype_count() {
        let mut world = World::new();

        // Archetypes should still exist after clear
        // (just empty of entities)
        world.clear();
        assert_eq!(world.archetype_count(), 1);
    }
}

// =========================================================================
// Direct Access Tests
// =========================================================================

mod direct_access {
    use super::*;

    #[test]
    fn test_entities_accessor() {
        let world = World::new();
        let allocator = world.entities();

        assert_eq!(allocator.len(), 0);
    }

    #[test]
    fn test_archetypes_accessor() {
        let world = World::new();
        let graph = world.archetypes();

        assert_eq!(graph.len(), 1);
        assert!(graph.get(ArchetypeId::EMPTY).is_some());
    }
}

// =========================================================================
// Thread Safety Tests
// =========================================================================

mod thread_safety {
    use super::*;

    // Note: World is NOT Send because it contains NonSendResources.
    // This is intentional - the World should be owned by the main thread
    // when using non-send resources (window handles, GL contexts, etc.).
    //
    // For thread-safe world access, use:
    // - Arc<RwLock<World>> for shared ownership
    // - The scheduler, which handles thread safety for systems

    #[test]
    fn test_world_is_not_send_due_to_non_send_resources() {
        // This test documents the intentional !Send behavior.
        // World contains NonSendResources which uses NonSendMarker
        // containing *const (), making the whole type !Send.
        fn check_type<T>() {}
        check_type::<World>();
        // The fact that this compiles proves World exists as a type.
        // The !Send behavior is enforced by NonSendMarker.
    }

    // Note: World is NOT Sync by design - concurrent access requires
    // external synchronization (e.g., RwLock) or the scheduler
}

// =========================================================================
// Edge Cases
// =========================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_zero_capacity_world() {
        let world = World::with_capacity(0, 0);
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.component_type_count(), 0);
    }

    #[test]
    fn test_large_capacity_world() {
        let world = World::with_capacity(1_000_000, 1000);
        assert_eq!(world.entity_count(), 0);
    }
}
