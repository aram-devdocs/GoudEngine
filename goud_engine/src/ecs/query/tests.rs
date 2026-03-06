//! Tests for the query module — basic query behaviour (struct, get, iter).

use crate::ecs::component::ComponentId;
use crate::ecs::Component;
use crate::ecs::World;

use super::{Query, With, Without, WorldQuery};
use crate::ecs::entity::Entity;

// Test components shared across all sub-modules.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Position {
    pub x: f32,
    pub y: f32,
}
impl Component for Position {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Velocity {
    pub x: f32,
    pub y: f32,
}
impl Component for Velocity {}

#[derive(Debug, Clone, Copy)]
pub(super) struct Player;
impl Component for Player {}

#[derive(Debug, Clone, Copy)]
pub(super) struct Enemy;
impl Component for Enemy {}

// =========================================================================
// Query Structure Tests
// =========================================================================

mod query_struct {
    use super::*;

    #[test]
    fn test_query_new() {
        let world = World::new();
        let query: Query<&Position> = Query::new(&world);
        assert!(query.is_empty(&world));
    }

    #[test]
    fn test_query_with_filter() {
        let world = World::new();
        let query: Query<&Position, With<Player>> = Query::new(&world);
        assert!(query.is_empty(&world));
    }

    #[test]
    fn test_query_from_state() {
        let world = World::new();
        let query_state = <&Position>::init_state(&world);
        let filter_state = ();
        let query: Query<&Position> = Query::from_state(query_state, filter_state);
        assert!(query.is_empty(&world));
    }

    #[test]
    fn test_query_debug() {
        let world = World::new();
        let query: Query<&Position> = Query::new(&world);
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("Query"));
    }

    #[test]
    fn test_query_component_access() {
        let world = World::new();
        let query: Query<&Position> = Query::new(&world);
        let access = query.component_access();
        assert!(access
            .reads()
            .any(|&id| id == ComponentId::of::<Position>()));
    }
}

// =========================================================================
// Query Get Tests
// =========================================================================

mod query_get {
    use super::*;

    #[test]
    fn test_query_get_existing() {
        let mut world = World::new();
        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });

        let query: Query<&Position> = Query::new(&world);
        let result = query.get(&world, entity);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), &Position { x: 1.0, y: 2.0 });
    }

    #[test]
    fn test_query_get_missing_component() {
        let mut world = World::new();
        let entity = world.spawn_empty();
        world.insert(entity, Velocity { x: 1.0, y: 2.0 });

        let query: Query<&Position> = Query::new(&world);
        assert!(query.get(&world, entity).is_none());
    }

    #[test]
    fn test_query_get_dead_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });
        world.despawn(entity);

        let query: Query<&Position> = Query::new(&world);
        assert!(query.get(&world, entity).is_none());
    }

    #[test]
    fn test_query_get_with_filter_passing() {
        let mut world = World::new();
        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });
        world.insert(entity, Player);

        let query: Query<&Position, With<Player>> = Query::new(&world);
        assert!(query.get(&world, entity).is_some());
    }

    #[test]
    fn test_query_get_with_filter_failing() {
        let mut world = World::new();
        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });

        let query: Query<&Position, With<Player>> = Query::new(&world);
        assert!(query.get(&world, entity).is_none());
    }

    #[test]
    fn test_query_get_with_without_filter() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });
        world.insert(e1, Enemy);

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 3.0, y: 4.0 });

        let query: Query<&Position, Without<Enemy>> = Query::new(&world);
        assert!(query.get(&world, e1).is_none());
        assert!(query.get(&world, e2).is_some());
    }
}

// =========================================================================
// Query Iteration Tests
// =========================================================================

mod query_iter {
    use super::*;

    #[test]
    fn test_query_iter_empty_world() {
        let world = World::new();
        let query: Query<&Position> = Query::new(&world);
        assert_eq!(query.iter(&world).count(), 0);
    }

    #[test]
    fn test_query_iter_single_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty();
        world.insert(entity, Position { x: 1.0, y: 2.0 });

        let query: Query<&Position> = Query::new(&world);
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], &Position { x: 1.0, y: 2.0 });
    }

    #[test]
    fn test_query_iter_multiple_entities() {
        let mut world = World::new();
        for i in 0..3 {
            let e = world.spawn_empty();
            world.insert(
                e,
                Position {
                    x: i as f32,
                    y: 0.0,
                },
            );
        }

        let query: Query<&Position> = Query::new(&world);
        assert_eq!(query.iter(&world).count(), 3);
    }

    #[test]
    fn test_query_iter_with_filter() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });
        world.insert(e1, Player);

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 3.0, y: 4.0 });

        let e3 = world.spawn_empty();
        world.insert(e3, Position { x: 5.0, y: 6.0 });
        world.insert(e3, Player);

        let query: Query<&Position, With<Player>> = Query::new(&world);
        assert_eq!(query.iter(&world).count(), 2);
    }

    #[test]
    fn test_query_iter_skips_non_matching() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });

        let e2 = world.spawn_empty();
        world.insert(e2, Velocity { x: 3.0, y: 4.0 });

        let query: Query<&Position> = Query::new(&world);
        assert_eq!(query.iter(&world).count(), 1);
    }

    #[test]
    fn test_query_iter_entity() {
        let mut world = World::new();
        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();

        let query: Query<Entity> = Query::new(&world);
        let entities: Vec<_> = query.iter(&world).collect();

        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
    }
}
