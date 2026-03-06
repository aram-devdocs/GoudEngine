//! Tests for &T, &mut T, MutState, and WriteAccess.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::ecs::archetype::{Archetype, ArchetypeId};
    use crate::ecs::component::ComponentId;
    use crate::ecs::entity::Entity;
    use crate::ecs::query::fetch::{
        MutState, QueryState, ReadOnlyWorldQuery, WorldQuery, WriteAccess,
    };
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
    // Component Reference (&T) Tests
    // =========================================================================

    mod component_ref {
        use super::*;

        #[test]
        fn test_ref_init_state() {
            let world = World::new();
            let state = <&Position>::init_state(&world);
            assert_eq!(state, ComponentId::of::<Position>());
        }

        #[test]
        fn test_ref_matches_archetype_with_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(<&Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_ref_does_not_match_archetype_without_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Velocity>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = ComponentId::of::<Position>();
            assert!(!<&Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_ref_does_not_match_empty_archetype() {
            let archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
            let state = ComponentId::of::<Position>();
            assert!(!<&Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_ref_fetch_entity_with_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <&Position>::init_state(&world);
            let result = <&Position>::fetch(&state, &world, entity);

            assert!(result.is_some());
            let pos = result.unwrap();
            assert_eq!(pos.x, 1.0);
            assert_eq!(pos.y, 2.0);
        }

        #[test]
        fn test_ref_fetch_entity_without_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 1.0, y: 2.0 });

            let state = <&Position>::init_state(&world);
            let result = <&Position>::fetch(&state, &world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_ref_fetch_dead_entity() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.despawn(entity);

            let state = <&Position>::init_state(&world);
            let result = <&Position>::fetch(&state, &world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_ref_fetch_placeholder_entity() {
            let world = World::new();
            let state = <&Position>::init_state(&world);
            let result = <&Position>::fetch(&state, &world, Entity::PLACEHOLDER);
            assert!(result.is_none());
        }

        #[test]
        fn test_ref_is_read_only() {
            fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            requires_read_only::<&Position>();
            requires_read_only::<&Velocity>();
        }

        #[test]
        fn test_ref_component_access_contains_component_id() {
            let state = ComponentId::of::<Position>();
            let access = <&Position>::component_access(&state);

            assert_eq!(access.len(), 1);
            assert!(access.contains(&ComponentId::of::<Position>()));
        }

        #[test]
        fn test_ref_component_access_does_not_contain_other_components() {
            let state = ComponentId::of::<Position>();
            let access = <&Position>::component_access(&state);
            assert!(!access.contains(&ComponentId::of::<Velocity>()));
        }

        #[test]
        fn test_ref_multiple_entities_same_component() {
            let mut world = World::new();

            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });
            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });
            let e3 = world.spawn_empty();
            world.insert(e3, Position { x: 5.0, y: 6.0 });

            let state = <&Position>::init_state(&world);

            assert_eq!(
                <&Position>::fetch(&state, &world, e1).unwrap(),
                &Position { x: 1.0, y: 2.0 }
            );
            assert_eq!(
                <&Position>::fetch(&state, &world, e2).unwrap(),
                &Position { x: 3.0, y: 4.0 }
            );
            assert_eq!(
                <&Position>::fetch(&state, &world, e3).unwrap(),
                &Position { x: 5.0, y: 6.0 }
            );
        }

        #[test]
        fn test_ref_fetch_after_component_update() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <&Position>::init_state(&world);

            let pos1 = <&Position>::fetch(&state, &world, entity).unwrap();
            assert_eq!(pos1, &Position { x: 1.0, y: 2.0 });

            world.insert(entity, Position { x: 10.0, y: 20.0 });

            let pos2 = <&Position>::fetch(&state, &world, entity).unwrap();
            assert_eq!(pos2, &Position { x: 10.0, y: 20.0 });
        }

        #[test]
        fn test_ref_fetch_stale_entity() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.despawn(entity);

            let new_entity = world.spawn_empty();
            world.insert(new_entity, Position { x: 99.0, y: 99.0 });

            let state = <&Position>::init_state(&world);

            assert!(<&Position>::fetch(&state, &world, entity).is_none());

            let new_result = <&Position>::fetch(&state, &world, new_entity);
            assert_eq!(new_result.unwrap(), &Position { x: 99.0, y: 99.0 });
        }

        #[test]
        fn test_ref_implements_world_query() {
            fn requires_world_query<Q: WorldQuery>() {}
            requires_world_query::<&Position>();
            requires_world_query::<&Velocity>();
            requires_world_query::<&Player>();
        }

        #[test]
        fn test_ref_state_implements_query_state() {
            fn requires_query_state<S: QueryState>() {}
            requires_query_state::<ComponentId>();
        }
    }

    // =========================================================================
    // Mutable Component Reference (&mut T) Tests
    // =========================================================================

    mod mut_component_ref {
        use super::*;

        #[test]
        fn test_mut_ref_init_state() {
            let world = World::new();
            let state = <&mut Position>::init_state(&world);
            assert_eq!(state.component_id, ComponentId::of::<Position>());
        }

        #[test]
        fn test_mut_ref_state_is_mut_state() {
            fn check_state_type<T: crate::ecs::Component>() {
                let world = World::new();
                let state: MutState = <&mut T>::init_state(&world);
                assert_eq!(state.component_id, ComponentId::of::<T>());
            }
            check_state_type::<Position>();
            check_state_type::<Velocity>();
        }

        #[test]
        fn test_mut_ref_matches_archetype_with_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = MutState::of::<Position>();
            assert!(<&mut Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_mut_ref_does_not_match_empty_archetype() {
            let archetype = Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());
            let state = MutState::of::<Position>();
            assert!(!<&mut Position>::matches_archetype(&state, &archetype));
        }

        #[test]
        fn test_mut_ref_fetch_returns_none() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <&mut Position>::init_state(&world);
            let result = <&mut Position>::fetch(&state, &world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_mut_ref_fetch_mut_entity_with_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <&mut Position>::init_state(&world);
            let result = <&mut Position>::fetch_mut(&state, &mut world, entity);

            assert!(result.is_some());
            let pos = result.unwrap();
            assert_eq!(pos.x, 1.0);
        }

        #[test]
        fn test_mut_ref_fetch_mut_and_modify() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <&mut Position>::init_state(&world);
            if let Some(pos) = <&mut Position>::fetch_mut(&state, &mut world, entity) {
                pos.x += 10.0;
                pos.y += 20.0;
            }

            let pos = world.get::<Position>(entity).unwrap();
            assert_eq!(pos.x, 11.0);
            assert_eq!(pos.y, 22.0);
        }

        #[test]
        fn test_mut_ref_fetch_mut_entity_without_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 1.0, y: 2.0 });

            let state = <&mut Position>::init_state(&world);
            let result = <&mut Position>::fetch_mut(&state, &mut world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_mut_ref_fetch_mut_dead_entity() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.despawn(entity);

            let state = <&mut Position>::init_state(&world);
            let result = <&mut Position>::fetch_mut(&state, &mut world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_mut_ref_fetch_mut_placeholder_entity() {
            let mut world = World::new();
            let state = <&mut Position>::init_state(&world);
            let result = <&mut Position>::fetch_mut(&state, &mut world, Entity::PLACEHOLDER);
            assert!(result.is_none());
        }

        #[test]
        fn test_mut_ref_is_not_read_only() {
            fn requires_world_query<Q: WorldQuery>() {}
            requires_world_query::<&mut Position>();
        }

        #[test]
        fn test_mut_ref_component_access_contains_component_id() {
            let state = MutState::of::<Position>();
            let access = <&mut Position>::component_access(&state);

            assert_eq!(access.len(), 1);
            assert!(access.contains(&ComponentId::of::<Position>()));
        }

        #[test]
        fn test_mut_ref_fetch_mut_stale_entity() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.despawn(entity);

            let new_entity = world.spawn_empty();
            world.insert(new_entity, Position { x: 99.0, y: 99.0 });

            let state = <&mut Position>::init_state(&world);

            assert!(<&mut Position>::fetch_mut(&state, &mut world, entity).is_none());

            let new_result = <&mut Position>::fetch_mut(&state, &mut world, new_entity);
            assert_eq!(new_result.unwrap().x, 99.0);
        }
    }

    // =========================================================================
    // MutState Tests
    // =========================================================================

    mod mut_state {
        use super::*;

        #[test]
        fn test_mut_state_of() {
            let state = MutState::of::<Position>();
            assert_eq!(state.component_id, ComponentId::of::<Position>());
        }

        #[test]
        fn test_mut_state_different_types() {
            let pos_state = MutState::of::<Position>();
            let vel_state = MutState::of::<Velocity>();
            assert_ne!(pos_state.component_id, vel_state.component_id);
        }

        #[test]
        fn test_mut_state_implements_query_state() {
            fn requires_query_state<S: QueryState>() {}
            requires_query_state::<MutState>();
        }

        #[test]
        fn test_mut_state_is_send_sync() {
            fn requires_send_sync<T: Send + Sync>() {}
            requires_send_sync::<MutState>();
        }

        #[test]
        fn test_mut_state_is_clone() {
            let state = MutState::of::<Position>();
            let cloned = state.clone();
            assert_eq!(state, cloned);
        }

        #[test]
        fn test_mut_state_debug() {
            let state = MutState::of::<Position>();
            let debug_str = format!("{:?}", state);
            assert!(debug_str.contains("MutState"));
        }
    }

    // =========================================================================
    // WriteAccess Tests
    // =========================================================================

    mod write_access {
        use super::*;

        #[test]
        fn test_write_access_new() {
            let id = ComponentId::of::<Position>();
            let access = WriteAccess(id);
            assert_eq!(access.0, id);
        }

        #[test]
        fn test_write_access_equality() {
            let id1 = ComponentId::of::<Position>();
            let id2 = ComponentId::of::<Position>();
            let id3 = ComponentId::of::<Velocity>();

            assert_eq!(WriteAccess(id1), WriteAccess(id2));
            assert_ne!(WriteAccess(id1), WriteAccess(id3));
        }

        #[test]
        fn test_write_access_hash() {
            use std::collections::HashSet;

            let id1 = ComponentId::of::<Position>();
            let id2 = ComponentId::of::<Velocity>();

            let mut set = HashSet::new();
            set.insert(WriteAccess(id1));
            set.insert(WriteAccess(id2));

            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_write_access_ordering() {
            let id1 = ComponentId::of::<Position>();
            let id2 = ComponentId::of::<Velocity>();

            let mut set = BTreeSet::new();
            set.insert(WriteAccess(id1));
            set.insert(WriteAccess(id2));

            assert_eq!(set.len(), 2);
        }
    }
}
