use super::*;

mod entity_world_mut {
    use super::*;

    #[test]
    fn test_entity_world_mut_id() {
        let mut world = World::new();

        let builder = world.spawn();
        let id = builder.id();

        // ID should be valid
        assert!(!id.is_placeholder());
    }

    #[test]
    fn test_entity_world_mut_world_ref() {
        let mut world = World::new();

        let builder = world.spawn();

        // Should have read access to world
        let count = builder.world().entity_count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_entity_world_mut_world_mut() {
        let mut world = World::new();

        let mut builder = world.spawn();

        // Can access world mutably through builder
        let _world_mut = builder.world_mut();
    }

    #[test]
    fn test_entity_world_mut_debug() {
        let mut world = World::new();

        let builder = world.spawn();
        let debug_str = format!("{:?}", builder);

        assert!(debug_str.contains("EntityWorldMut"));
        assert!(debug_str.contains("entity"));
    }
}
