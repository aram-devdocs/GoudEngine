use super::*;

// =========================================================================
// insert_batch() Tests
// =========================================================================

mod insert_batch {
    use super::super::super::super::entity::Entity;
    use super::*;

    #[test]
    fn test_insert_batch_empty() {
        let mut world = World::new();

        let count = world.insert_batch::<Position>(std::iter::empty());
        assert_eq!(count, 0);
    }

    #[test]
    fn test_insert_batch_single() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        let count = world.insert_batch(vec![(entity, Position { x: 1.0, y: 2.0 })]);

        assert_eq!(count, 1);
        assert_eq!(
            world.get::<Position>(entity),
            Some(&Position { x: 1.0, y: 2.0 })
        );
    }

    #[test]
    fn test_insert_batch_multiple() {
        let mut world = World::new();
        let entities = world.spawn_batch(5);

        let batch: Vec<_> = entities
            .iter()
            .enumerate()
            .map(|(i, &e)| {
                (
                    e,
                    Position {
                        x: i as f32,
                        y: (i * 2) as f32,
                    },
                )
            })
            .collect();

        let count = world.insert_batch(batch);

        assert_eq!(count, 5);
        for (i, entity) in entities.iter().enumerate() {
            assert_eq!(
                world.get::<Position>(*entity),
                Some(&Position {
                    x: i as f32,
                    y: (i * 2) as f32
                })
            );
        }
    }

    #[test]
    fn test_insert_batch_skips_dead_entities() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();
        let e3 = world.spawn_empty();

        // Despawn e2
        world.despawn(e2);

        let batch = vec![
            (e1, Position { x: 1.0, y: 1.0 }),
            (e2, Position { x: 2.0, y: 2.0 }), // dead
            (e3, Position { x: 3.0, y: 3.0 }),
        ];

        let count = world.insert_batch(batch);

        assert_eq!(count, 2);
        assert!(world.has::<Position>(e1));
        assert!(!world.has::<Position>(e2)); // not inserted (dead)
        assert!(world.has::<Position>(e3));
    }

    #[test]
    fn test_insert_batch_with_placeholder() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        let batch = vec![
            (entity, Position { x: 1.0, y: 1.0 }),
            (Entity::PLACEHOLDER, Position { x: 0.0, y: 0.0 }),
        ];

        let count = world.insert_batch(batch);

        assert_eq!(count, 1);
        assert!(world.has::<Position>(entity));
    }
}

// =========================================================================
// EntityWorldMut::insert() Tests
// =========================================================================

mod entity_builder {
    use super::*;

    #[test]
    fn test_entity_builder_insert_single() {
        let mut world = World::new();

        let entity = world.spawn().insert(Position { x: 1.0, y: 2.0 }).id();

        assert!(world.has::<Position>(entity));
        assert_eq!(
            world.get::<Position>(entity),
            Some(&Position { x: 1.0, y: 2.0 })
        );
    }

    #[test]
    fn test_entity_builder_insert_multiple() {
        let mut world = World::new();

        let entity = world
            .spawn()
            .insert(Position { x: 1.0, y: 2.0 })
            .insert(Velocity { x: 3.0, y: 4.0 })
            .insert(Player)
            .id();

        assert!(world.has::<Position>(entity));
        assert!(world.has::<Velocity>(entity));
        assert!(world.has::<Player>(entity));

        assert_eq!(
            world.get::<Position>(entity),
            Some(&Position { x: 1.0, y: 2.0 })
        );
        assert_eq!(
            world.get::<Velocity>(entity),
            Some(&Velocity { x: 3.0, y: 4.0 })
        );
        assert_eq!(world.get::<Player>(entity), Some(&Player));
    }

    #[test]
    fn test_entity_builder_insert_replace() {
        let mut world = World::new();

        let entity = world
            .spawn()
            .insert(Position { x: 1.0, y: 2.0 })
            .insert(Position { x: 10.0, y: 20.0 }) // Replace
            .id();

        // Should have the second value
        assert_eq!(
            world.get::<Position>(entity),
            Some(&Position { x: 10.0, y: 20.0 })
        );
    }

    #[test]
    fn test_entity_builder_chaining_returns_self() {
        let mut world = World::new();

        // Verify chaining works by using mutable borrows
        let mut builder = world.spawn();
        let entity_id = builder.id();

        // Chain returns &mut Self for fluent API
        builder
            .insert(Position { x: 0.0, y: 0.0 })
            .insert(Velocity { x: 1.0, y: 1.0 });

        // Need to drop builder to access world again
        drop(builder);

        assert!(world.has::<Position>(entity_id));
        assert!(world.has::<Velocity>(entity_id));
    }
}
