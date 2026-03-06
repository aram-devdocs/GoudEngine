//! Tests for tuple WorldQuery implementations.

#[cfg(test)]
mod tests {
    use crate::ecs::component::ComponentId;
    use crate::ecs::entity::Entity;
    use crate::ecs::query::fetch::{ReadOnlyWorldQuery, With, WorldQuery};
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
    struct Health(f32);
    impl Component for Health {}

    mod tuple_queries {
        use super::*;

        #[test]
        fn test_tuple_2_init_state() {
            let world = World::new();
            let state = <(&Position, &Velocity)>::init_state(&world);
            let (pos_id, vel_id) = state;
            assert_eq!(pos_id, ComponentId::of::<Position>());
            assert_eq!(vel_id, ComponentId::of::<Velocity>());
        }

        #[test]
        fn test_tuple_2_component_access() {
            let world = World::new();
            let state = <(&Position, &Velocity)>::init_state(&world);
            let access = <(&Position, &Velocity)>::component_access(&state);

            assert_eq!(access.len(), 2);
            assert!(access.contains(&ComponentId::of::<Position>()));
            assert!(access.contains(&ComponentId::of::<Velocity>()));
        }

        #[test]
        fn test_tuple_2_fetch() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.insert(entity, Velocity { x: 3.0, y: 4.0 });

            let state = <(&Position, &Velocity)>::init_state(&world);
            let result = <(&Position, &Velocity)>::fetch(&state, &world, entity);

            assert!(result.is_some());
            let (pos, vel) = result.unwrap();
            assert_eq!(pos.x, 1.0);
            assert_eq!(vel.x, 3.0);
        }

        #[test]
        fn test_tuple_2_fetch_missing_component() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <(&Position, &Velocity)>::init_state(&world);
            let result = <(&Position, &Velocity)>::fetch(&state, &world, entity);

            assert!(result.is_none());
        }

        #[test]
        fn test_tuple_3_fetch() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.insert(entity, Velocity { x: 3.0, y: 4.0 });
            world.insert(entity, Health(100.0));

            let state = <(&Position, &Velocity, &Health)>::init_state(&world);
            let result = <(&Position, &Velocity, &Health)>::fetch(&state, &world, entity);

            assert!(result.is_some());
            let (pos, vel, health) = result.unwrap();
            assert_eq!(pos.x, 1.0);
            assert_eq!(vel.x, 3.0);
            assert_eq!(health.0, 100.0);
        }

        #[test]
        fn test_tuple_with_entity() {
            let mut world = World::new();
            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });

            let state = <(Entity, &Position)>::init_state(&world);
            let result = <(Entity, &Position)>::fetch(&state, &world, entity);

            assert!(result.is_some());
            let (e, pos) = result.unwrap();
            assert_eq!(e, entity);
            assert_eq!(pos.x, 1.0);
        }

        #[test]
        fn test_tuple_is_read_only() {
            fn assert_read_only<T: ReadOnlyWorldQuery>() {}
            assert_read_only::<(&Position, &Velocity)>();
            assert_read_only::<(Entity, &Position)>();
        }

        #[test]
        fn test_tuple_matches_archetype() {
            let mut world = World::new();

            let entity = world.spawn_empty();
            world.insert(entity, Position { x: 1.0, y: 2.0 });
            world.insert(entity, Velocity { x: 3.0, y: 4.0 });

            let archetype_id = world.entity_archetype(entity).unwrap();
            let archetype = world.archetypes().get(archetype_id).unwrap();

            let state = <(&Position, &Velocity)>::init_state(&world);
            assert!(<(&Position, &Velocity)>::matches_archetype(
                &state, archetype
            ));

            let entity2 = world.spawn_empty();
            world.insert(entity2, Position { x: 5.0, y: 6.0 });

            let archetype_id2 = world.entity_archetype(entity2).unwrap();
            let archetype2 = world.archetypes().get(archetype_id2).unwrap();

            assert!(!<(&Position, &Velocity)>::matches_archetype(
                &state, archetype2
            ));
        }

        #[test]
        fn test_tuple_4_elements() {
            let world = World::new();
            let state = <(&Position, &Velocity, &Health, Entity)>::init_state(&world);
            let access = <(&Position, &Velocity, &Health, Entity)>::component_access(&state);

            // Entity doesn't contribute to component access
            assert_eq!(access.len(), 3);
        }

        #[test]
        fn test_tuple_with_filters() {
            let world = World::new();
            let state = <(&Position, With<Velocity>)>::init_state(&world);
            let access = <(&Position, With<Velocity>)>::component_access(&state);

            // With<T> filter doesn't add to access
            assert_eq!(access.len(), 1);
            assert!(access.contains(&ComponentId::of::<Position>()));
        }
    }
}
