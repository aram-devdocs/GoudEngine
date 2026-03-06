//! Tests for WorldQuery, ReadOnlyWorldQuery, QueryState trait impls, and unit/entity queries.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::ecs::archetype::{Archetype, ArchetypeId};
    use crate::ecs::component::ComponentId;
    use crate::ecs::entity::Entity;
    use crate::ecs::query::fetch::{QueryState, ReadOnlyWorldQuery, WorldQuery};
    use crate::ecs::{Component, World};

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    // =========================================================================
    // WorldQuery / Entity Tests
    // =========================================================================

    mod world_query_trait {
        use super::*;

        #[test]
        fn test_entity_query_init_state() {
            let world = World::new();
            let _state: () = Entity::init_state(&world);
        }

        #[test]
        fn test_entity_query_matches_all_archetypes() {
            let state = ();

            let empty_arch = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
            assert!(Entity::matches_archetype(&state, &empty_arch));

            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            let pos_arch = Archetype::new(ArchetypeId::new(1), components);
            assert!(Entity::matches_archetype(&state, &pos_arch));
        }

        #[test]
        fn test_entity_query_fetch_alive() {
            let mut world = World::new();
            let entity = world.spawn_empty();

            let fetched = Entity::fetch(&(), &world, entity);
            assert_eq!(fetched, Some(entity));
        }

        #[test]
        fn test_entity_query_fetch_dead() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.despawn(entity);

            let fetched = Entity::fetch(&(), &world, entity);
            assert_eq!(fetched, None);
        }

        #[test]
        fn test_entity_query_is_read_only() {
            fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            requires_read_only::<Entity>();
        }

        #[test]
        fn test_entity_query_component_access_empty() {
            let state = ();
            let access = Entity::component_access(&state);
            assert!(access.is_empty());
        }
    }

    // =========================================================================
    // Unit Query Tests
    // =========================================================================

    mod unit_query {
        use super::*;

        #[test]
        fn test_unit_query_matches_all() {
            let state = ();
            let empty_arch = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
            assert!(<()>::matches_archetype(&state, &empty_arch));
        }

        #[test]
        fn test_unit_query_fetch_always_succeeds() {
            let world = World::new();
            let entity = Entity::new(0, 1);
            let fetched = <()>::fetch(&(), &world, entity);
            assert!(fetched.is_some());
        }

        #[test]
        fn test_unit_query_is_read_only() {
            fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            requires_read_only::<()>();
        }
    }

    // =========================================================================
    // QueryState Trait Tests
    // =========================================================================

    mod query_state_trait {
        use super::*;

        #[test]
        fn test_component_id_implements_query_state() {
            fn requires_query_state<S: QueryState>() {}
            requires_query_state::<ComponentId>();
        }

        #[test]
        fn test_unit_implements_query_state() {
            fn requires_query_state<S: QueryState>() {}
            requires_query_state::<()>();
        }

        #[test]
        fn test_query_state_is_send_sync() {
            fn requires_send_sync<S: Send + Sync>() {}
            requires_send_sync::<ComponentId>();
            requires_send_sync::<()>();
        }
    }
}
