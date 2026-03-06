//! Tests for `Option<Q>` optional query support.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::ecs::archetype::{Archetype, ArchetypeId};
    use crate::ecs::component::ComponentId;
    use crate::ecs::query::fetch::{ReadOnlyWorldQuery, WorldQuery};
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
    // Option<&T> Immutable Tests
    // =========================================================================

    mod option_ref {
        use super::*;

        #[test]
        fn test_option_ref_returns_some_when_component_present() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <Option<&Position>>::init_state(&world);
            let result = <Option<&Position>>::fetch(&state, &world, entity);

            // fetch always returns Some for optional queries
            assert!(result.is_some());
            // The inner value should be Some(&Position)
            let inner = result.unwrap();
            assert_eq!(inner, Some(&Position { x: 1.0, y: 2.0 }));
        }

        #[test]
        fn test_option_ref_returns_none_when_component_absent() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 1.0, y: 2.0 });

            let state = <Option<&Position>>::init_state(&world);
            let result = <Option<&Position>>::fetch(&state, &world, entity);

            // fetch always returns Some for optional queries
            assert!(result.is_some());
            // The inner value should be None since entity lacks Position
            let inner = result.unwrap();
            assert_eq!(inner, None);
        }

        #[test]
        fn test_option_ref_is_read_only() {
            fn requires_read_only<Q: ReadOnlyWorldQuery>() {}
            requires_read_only::<Option<&Position>>();
        }
    }

    // =========================================================================
    // Option<&mut T> Mutable Tests
    // =========================================================================

    mod option_mut_ref {
        use super::*;

        #[test]
        fn test_option_mut_ref_fetch_mut_with_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <Option<&mut Position>>::init_state(&world);
            let result =
                <Option<&mut Position>>::fetch_mut(&state, &mut world, entity);

            assert!(result.is_some());
            let inner = result.unwrap();
            assert!(inner.is_some());
            let pos = inner.unwrap();
            assert_eq!(pos.x, 1.0);
            assert_eq!(pos.y, 2.0);
        }

        #[test]
        fn test_option_mut_ref_fetch_mut_without_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 1.0, y: 2.0 });

            let state = <Option<&mut Position>>::init_state(&world);
            let result =
                <Option<&mut Position>>::fetch_mut(&state, &mut world, entity);

            assert!(result.is_some());
            let inner = result.unwrap();
            assert!(inner.is_none());
        }

        #[test]
        fn test_option_mut_ref_modify_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <Option<&mut Position>>::init_state(&world);
            if let Some(Some(pos)) =
                <Option<&mut Position>>::fetch_mut(&state, &mut world, entity)
            {
                pos.x += 10.0;
                pos.y += 20.0;
            }

            let pos = world.get::<Position>(entity).unwrap();
            assert_eq!(pos.x, 11.0);
            assert_eq!(pos.y, 22.0);
        }
    }

    // =========================================================================
    // Archetype Matching Tests
    // =========================================================================

    mod archetype_matching {
        use super::*;

        #[test]
        fn test_option_matches_archetype_always_true() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Position>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = <Option<&Position>>::init_state(&World::new());
            assert!(<Option<&Position>>::matches_archetype(
                &state, &archetype
            ));
        }

        #[test]
        fn test_option_matches_empty_archetype() {
            let archetype =
                Archetype::new(ArchetypeId::EMPTY, BTreeSet::new());

            let state = <Option<&Position>>::init_state(&World::new());
            assert!(<Option<&Position>>::matches_archetype(
                &state, &archetype
            ));
        }

        #[test]
        fn test_option_matches_archetype_without_target_component() {
            let mut components = BTreeSet::new();
            components.insert(ComponentId::of::<Velocity>());
            let archetype = Archetype::new(ArchetypeId::new(1), components);

            let state = <Option<&Position>>::init_state(&World::new());
            assert!(<Option<&Position>>::matches_archetype(
                &state, &archetype
            ));
        }
    }

    // =========================================================================
    // Component Access Tests
    // =========================================================================

    mod component_access {
        use super::*;

        #[test]
        fn test_option_component_access_is_empty() {
            let state = <Option<&Position>>::init_state(&World::new());
            let access = <Option<&Position>>::component_access(&state);
            assert!(access.is_empty());
        }

        #[test]
        fn test_option_mut_component_access_is_empty() {
            let state = <Option<&mut Position>>::init_state(&World::new());
            let access = <Option<&mut Position>>::component_access(&state);
            assert!(access.is_empty());
        }
    }

    // =========================================================================
    // Tuple Query Integration Tests
    // =========================================================================

    mod tuple_queries {
        use super::*;

        #[test]
        fn test_tuple_with_optional_both_present() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.insert(entity, Velocity { x: 3.0, y: 4.0 });

            let state =
                <(&Position, Option<&Velocity>)>::init_state(&world);
            let result =
                <(&Position, Option<&Velocity>)>::fetch(&state, &world, entity);

            assert!(result.is_some());
            let (pos, vel) = result.unwrap();
            assert_eq!(pos, &Position { x: 1.0, y: 2.0 });
            assert_eq!(vel, Some(&Velocity { x: 3.0, y: 4.0 }));
        }

        #[test]
        fn test_tuple_with_optional_only_required_present() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            // No Velocity component

            let state =
                <(&Position, Option<&Velocity>)>::init_state(&world);
            let result =
                <(&Position, Option<&Velocity>)>::fetch(&state, &world, entity);

            assert!(result.is_some());
            let (pos, vel) = result.unwrap();
            assert_eq!(pos, &Position { x: 1.0, y: 2.0 });
            assert_eq!(vel, None);
        }

        #[test]
        fn test_tuple_with_optional_missing_required_returns_none() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Velocity { x: 3.0, y: 4.0 });
            // No Position component (required)

            let state =
                <(&Position, Option<&Velocity>)>::init_state(&world);
            let result =
                <(&Position, Option<&Velocity>)>::fetch(&state, &world, entity);

            // Should be None because Position is required and missing
            assert!(result.is_none());
        }

        #[test]
        fn test_entities_without_optional_still_yielded() {
            let mut world = World::new();
            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });
            world.insert(e1, Velocity { x: 10.0, y: 20.0 });

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });
            // e2 has no Velocity

            let state =
                <(&Position, Option<&Velocity>)>::init_state(&world);

            // Both entities should be fetchable
            let r1 =
                <(&Position, Option<&Velocity>)>::fetch(&state, &world, e1);
            assert!(r1.is_some());
            let (pos1, vel1) = r1.unwrap();
            assert_eq!(pos1, &Position { x: 1.0, y: 2.0 });
            assert_eq!(vel1, Some(&Velocity { x: 10.0, y: 20.0 }));

            let r2 =
                <(&Position, Option<&Velocity>)>::fetch(&state, &world, e2);
            assert!(r2.is_some());
            let (pos2, vel2) = r2.unwrap();
            assert_eq!(pos2, &Position { x: 3.0, y: 4.0 });
            assert_eq!(vel2, None);
        }
    }
}
