//! Tests for With and Without filter types.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::ecs::archetype::{Archetype, ArchetypeId};
    use crate::ecs::component::ComponentId;
    use crate::ecs::query::fetch::{ReadOnlyWorldQuery, With, Without, WorldQuery};
    use crate::ecs::{Component, World};

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

    #[derive(Debug, Clone, Copy)]
    struct Player;
    impl Component for Player {}

    // =========================================================================
    // With Filter Tests
    // =========================================================================

    mod with_filter {
        use super::*;

        #[test]
        fn test_with_init_state() {
            let world = World::new();
            let state = With::<Position>::init_state(&world);
            assert_eq!(state, ComponentId::of::<Position>());
        }

        #[test]
        fn test_with_matches_archetype_with_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(With::<Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_with_does_not_match_archetype_without_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Velocity>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(!With::<Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_with_fetch_entity_with_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = With::<Position>::init_state(&world);
            let result = With::<Position>::fetch(&state, &world, entity);

            assert_eq!(result, Some(()));
        }

        #[test]
        fn test_with_fetch_entity_without_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 1.0, y: 2.0 });

            let state = With::<Position>::init_state(&world);
            let result = With::<Position>::fetch(&state, &world, entity);

            assert_eq!(result, None);
        }

        #[test]
        fn test_with_is_read_only() {
            fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            requires_read_only::<With<Position>>();
        }

        #[test]
        fn test_with_component_access_empty() {
            let state = ComponentId::of::<Position>();
            let access = With::<Position>::component_access(&state);
            assert!(access.is_empty());
        }

        #[test]
        fn test_with_filter_usage_documentation() {
            let mut world = World::new();
            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 0.0, y: 0.0 });
            world.insert(e1, Player);

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 1.0, y: 1.0 });

            let state = With::<Player>::init_state(&world);

            assert!(With::<Player>::fetch(&state, &world, e1).is_some());
            assert!(With::<Player>::fetch(&state, &world, e2).is_none());
        }
    }

    // =========================================================================
    // Without Filter Tests
    // =========================================================================

    mod without_filter {
        use super::*;

        #[test]
        fn test_without_init_state() {
            let world = World::new();
            let state = Without::<Position>::init_state(&world);
            assert_eq!(state, ComponentId::of::<Position>());
        }

        #[test]
        fn test_without_matches_archetype_without_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Velocity>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(Without::<Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_without_does_not_match_archetype_with_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(!Without::<Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_without_fetch_entity_without_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 1.0, y: 2.0 });

            let state = Without::<Position>::init_state(&world);
            let result = Without::<Position>::fetch(&state, &world, entity);

            assert_eq!(result, Some(()));
        }

        #[test]
        fn test_without_fetch_entity_with_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = Without::<Position>::init_state(&world);
            let result = Without::<Position>::fetch(&state, &world, entity);

            assert_eq!(result, None);
        }

        #[test]
        fn test_without_is_read_only() {
            fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            requires_read_only::<Without<Position>>();
        }

        #[test]
        fn test_without_matches_empty_archetype() {
            let archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
            let state = ComponentId::of::<Position>();
            assert!(Without::<Position>::matches_archetype(&state, &archetype));
        }
    }
}
