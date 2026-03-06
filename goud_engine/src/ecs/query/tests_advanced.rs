//! Tests for the query module — mutable iteration, count, system param, access, integration.

use crate::ecs::component::ComponentId;
use crate::ecs::system::{ReadOnlySystemParam, SystemParam, SystemParamState};
use crate::ecs::Component;
use crate::ecs::World;

use super::{Access, Query, QuerySystemParamState, With, Without};
use crate::ecs::entity::Entity;

// Test components — redeclared locally so this file is self-contained.
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

#[derive(Debug, Clone, Copy)]
struct Enemy;
impl Component for Enemy {}

// =========================================================================
// Query Iter Mut Tests
// =========================================================================

mod query_iter_mut {
    use super::*;

    #[test]
    fn test_query_iter_mut_modify() {
        let mut world = World::new();
        let e = world.spawn_empty();
        world.insert(e, Position { x: 1.0, y: 2.0 });

        {
            let query: Query<&mut Position> = Query::new(&world);
            for pos in query.iter_mut(&mut world) {
                pos.x += 10.0;
                pos.y += 20.0;
            }
        }

        let pos = world.get::<Position>(e).unwrap();
        assert_eq!(pos.x, 11.0);
        assert_eq!(pos.y, 22.0);
    }

    #[test]
    fn test_query_iter_mut_multiple() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 3.0, y: 4.0 });

        {
            let query: Query<&mut Position> = Query::new(&world);
            for pos in query.iter_mut(&mut world) {
                pos.x *= 2.0;
            }
        }

        assert_eq!(world.get::<Position>(e1).unwrap().x, 2.0);
        assert_eq!(world.get::<Position>(e2).unwrap().x, 6.0);
    }

    #[test]
    fn test_query_iter_mut_with_filter() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });
        world.insert(e1, Player);

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 3.0, y: 4.0 });

        {
            let query: Query<&mut Position, With<Player>> = Query::new(&world);
            for pos in query.iter_mut(&mut world) {
                pos.x = 100.0;
            }
        }

        assert_eq!(world.get::<Position>(e1).unwrap().x, 100.0);
        assert_eq!(world.get::<Position>(e2).unwrap().x, 3.0);
    }
}

// =========================================================================
// Query Count and Single Tests
// =========================================================================

mod query_count {
    use super::*;

    #[test]
    fn test_query_count_empty() {
        let world = World::new();
        let query: Query<&Position> = Query::new(&world);
        assert_eq!(query.count(&world), 0);
    }

    #[test]
    fn test_query_count_multiple() {
        let mut world = World::new();
        for _ in 0..5 {
            let e = world.spawn_empty();
            world.insert(e, Position { x: 0.0, y: 0.0 });
        }

        let query: Query<&Position> = Query::new(&world);
        assert_eq!(query.count(&world), 5);
    }

    #[test]
    fn test_query_is_empty() {
        let world = World::new();
        let query: Query<&Position> = Query::new(&world);
        assert!(query.is_empty(&world));
    }

    #[test]
    fn test_query_is_not_empty() {
        let mut world = World::new();
        let e = world.spawn_empty();
        world.insert(e, Position { x: 0.0, y: 0.0 });

        let query: Query<&Position> = Query::new(&world);
        assert!(!query.is_empty(&world));
    }

    #[test]
    fn test_query_single_one_entity() {
        let mut world = World::new();
        let e = world.spawn_empty();
        world.insert(e, Position { x: 1.0, y: 2.0 });

        let query: Query<&Position> = Query::new(&world);
        let result = query.single(&world);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), &Position { x: 1.0, y: 2.0 });
    }

    #[test]
    fn test_query_single_no_entities() {
        let world = World::new();
        let query: Query<&Position> = Query::new(&world);
        assert!(query.single(&world).is_none());
    }

    #[test]
    fn test_query_single_multiple_entities() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 3.0, y: 4.0 });

        let query: Query<&Position> = Query::new(&world);
        assert!(query.single(&world).is_none());
    }
}

// =========================================================================
// Query SystemParam Tests
// =========================================================================

mod query_system_param {
    use super::*;

    #[test]
    fn test_query_state_init() {
        let mut world = World::new();
        let state: QuerySystemParamState<&Position, ()> = QuerySystemParamState::init(&mut world);
        assert_eq!(state.query_state, ComponentId::of::<Position>());
    }

    #[test]
    fn test_query_state_with_filter() {
        let mut world = World::new();
        let state: QuerySystemParamState<&Position, With<Player>> =
            QuerySystemParamState::init(&mut world);
        assert_eq!(state.query_state, ComponentId::of::<Position>());
        assert_eq!(state.filter_state, ComponentId::of::<Player>());
    }

    #[test]
    fn test_query_update_access() {
        let mut world = World::new();
        let state: QuerySystemParamState<&Position, ()> = QuerySystemParamState::init(&mut world);

        let mut access = Access::new();
        Query::<&Position>::update_access(&state, &mut access);

        assert!(access
            .reads()
            .any(|&id| id == ComponentId::of::<Position>()));
        assert!(access.is_read_only());
    }

    #[test]
    fn test_query_get_param() {
        let mut world = World::new();
        let e = world.spawn_empty();
        world.insert(e, Position { x: 1.0, y: 2.0 });

        let mut state: QuerySystemParamState<&Position, ()> =
            QuerySystemParamState::init(&mut world);

        let query: Query<&Position> = Query::get_param(&mut state, &world);
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], &Position { x: 1.0, y: 2.0 });
    }

    #[test]
    fn test_query_get_param_mut() {
        let mut world = World::new();
        let e = world.spawn_empty();
        world.insert(e, Position { x: 1.0, y: 2.0 });

        let mut state: QuerySystemParamState<&mut Position, ()> =
            QuerySystemParamState::init(&mut world);

        let query: Query<&mut Position> = Query::get_param_mut(&mut state, &mut world);

        for pos in query.iter_mut(&mut world) {
            pos.x += 10.0;
        }

        assert_eq!(world.get::<Position>(e).unwrap().x, 11.0);
    }

    #[test]
    fn test_query_implements_system_param() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<Query<&Position>>();
        requires_system_param::<Query<&Position, With<Player>>>();
        requires_system_param::<Query<&mut Position>>();
    }

    #[test]
    fn test_read_only_query_implements_read_only_param() {
        fn requires_read_only<T: ReadOnlySystemParam>() {}
        requires_read_only::<Query<&Position>>();
        requires_read_only::<Query<&Position, With<Player>>>();
        requires_read_only::<Query<Entity>>();
    }

    #[test]
    fn test_query_state_is_clone() {
        let mut world = World::new();
        let state: QuerySystemParamState<&Position, ()> = QuerySystemParamState::init(&mut world);
        let _cloned = state.clone();
    }

    #[test]
    fn test_query_state_is_send_sync() {
        fn requires_send_sync<T: Send + Sync>() {}
        requires_send_sync::<QuerySystemParamState<&Position, ()>>();
        requires_send_sync::<QuerySystemParamState<&Position, With<Player>>>();
    }
}

// =========================================================================
// Query Access Conflict Tests
// =========================================================================

mod query_access {
    use super::*;

    #[test]
    fn test_read_queries_no_conflict() {
        let mut world = World::new();

        let state1: QuerySystemParamState<&Position, ()> = QuerySystemParamState::init(&mut world);
        let state2: QuerySystemParamState<&Position, ()> = QuerySystemParamState::init(&mut world);

        let mut access1 = Access::new();
        Query::<&Position>::update_access(&state1, &mut access1);

        let mut access2 = Access::new();
        Query::<&Position>::update_access(&state2, &mut access2);

        assert!(!access1.conflicts_with(&access2));
    }

    #[test]
    fn test_different_component_queries_no_conflict() {
        let mut world = World::new();

        let state1: QuerySystemParamState<&Position, ()> = QuerySystemParamState::init(&mut world);
        let state2: QuerySystemParamState<&Velocity, ()> = QuerySystemParamState::init(&mut world);

        let mut access1 = Access::new();
        Query::<&Position>::update_access(&state1, &mut access1);

        let mut access2 = Access::new();
        Query::<&Velocity>::update_access(&state2, &mut access2);

        assert!(!access1.conflicts_with(&access2));
    }

    #[test]
    fn test_query_with_filter_access() {
        let mut world = World::new();

        let state: QuerySystemParamState<&Position, With<Player>> =
            QuerySystemParamState::init(&mut world);

        let mut access = Access::new();
        Query::<&Position, With<Player>>::update_access(&state, &mut access);

        assert!(access
            .reads()
            .any(|&id| id == ComponentId::of::<Position>()));
    }
}

// =========================================================================
// Integration Tests
// =========================================================================

mod integration {
    use super::*;

    #[test]
    fn test_query_with_entity_and_component() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 3.0, y: 4.0 });

        let entity_query: Query<Entity> = Query::new(&world);
        let pos_query: Query<&Position> = Query::new(&world);

        assert_eq!(entity_query.iter(&world).count(), 2);
        assert_eq!(pos_query.iter(&world).count(), 2);
    }

    #[test]
    fn test_complex_filter_chain() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });
        world.insert(e1, Player);

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 3.0, y: 4.0 });
        world.insert(e2, Enemy);

        let e3 = world.spawn_empty();
        world.insert(e3, Position { x: 5.0, y: 6.0 });

        let player_query: Query<&Position, With<Player>> = Query::new(&world);
        assert_eq!(player_query.count(&world), 1);

        let non_enemy_query: Query<&Position, Without<Enemy>> = Query::new(&world);
        assert_eq!(non_enemy_query.count(&world), 2);
    }

    #[test]
    fn test_query_after_despawn() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 3.0, y: 4.0 });

        let query: Query<&Position> = Query::new(&world);
        assert_eq!(query.count(&world), 2);

        world.despawn(e1);

        let query: Query<&Position> = Query::new(&world);
        assert_eq!(query.count(&world), 1);
    }

    #[test]
    fn test_query_stress_test() {
        let mut world = World::new();

        for i in 0..1000 {
            let e = world.spawn_empty();
            world.insert(
                e,
                Position {
                    x: i as f32,
                    y: 0.0,
                },
            );
            if i % 2 == 0 {
                world.insert(e, Player);
            }
        }

        let query: Query<&Position> = Query::new(&world);
        assert_eq!(query.count(&world), 1000);

        let player_query: Query<&Position, With<Player>> = Query::new(&world);
        assert_eq!(player_query.count(&world), 500);
    }
}
