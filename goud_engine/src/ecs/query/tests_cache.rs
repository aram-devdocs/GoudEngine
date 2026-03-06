//! Tests for query archetype caching.

use crate::ecs::Component;
use crate::ecs::World;

use super::{Query, With, Without};
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

#[derive(Debug, Clone, Copy)]
struct Health {
    value: i32,
}
impl Component for Health {}

// =========================================================================
// Cache Correctness — cached results match uncached results
// =========================================================================

mod cache_correctness {
    use super::*;

    #[test]
    fn cached_query_returns_identical_results_to_uncached() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 3.0, y: 4.0 });
        world.insert(e2, Player);

        let e3 = world.spawn_empty();
        world.insert(e3, Velocity { x: 5.0, y: 6.0 });

        let uncached: Query<&Position> = Query::new(&world);
        let cached: Query<&Position> = Query::new(&world).with_cache(&world);

        let uncached_results: Vec<_> = uncached.iter(&world).collect();
        let cached_results: Vec<_> = cached.iter(&world).collect();

        assert_eq!(uncached_results.len(), cached_results.len());
        for item in &cached_results {
            assert!(uncached_results.contains(item));
        }
    }

    #[test]
    fn cached_query_count_matches_uncached() {
        let mut world = World::new();
        for i in 0..10 {
            let e = world.spawn_empty();
            world.insert(
                e,
                Position {
                    x: i as f32,
                    y: 0.0,
                },
            );
            if i % 3 == 0 {
                world.insert(e, Player);
            }
        }

        let uncached: Query<&Position> = Query::new(&world);
        let cached: Query<&Position> = Query::new(&world).with_cache(&world);

        assert_eq!(uncached.count(&world), cached.iter(&world).count());
    }

    #[test]
    fn cached_query_empty_world() {
        let world = World::new();
        let cached: Query<&Position> = Query::new(&world).with_cache(&world);
        assert_eq!(cached.iter(&world).count(), 0);
    }
}

// =========================================================================
// Cache Update — new archetypes are picked up incrementally
// =========================================================================

mod cache_update {
    use super::*;

    #[test]
    fn cache_updates_when_new_archetype_created() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 0.0 });

        let mut query: Query<&Position> = Query::new(&world).with_cache(&world);
        assert_eq!(query.iter(&world).count(), 1);

        // Insert a new component type, creating a new archetype
        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 2.0, y: 0.0 });
        world.insert(e2, Velocity { x: 0.0, y: 0.0 });

        // Cache is stale — update it
        query.update_cache(&world);
        assert_eq!(query.iter(&world).count(), 2);
    }

    #[test]
    fn incremental_update_only_evaluates_new_archetypes() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 0.0 });

        let mut query: Query<&Position> = Query::new(&world).with_cache(&world);
        let gen_before = query.archetype_cache().unwrap().generation();

        // Add entity with same archetype — no new archetypes created
        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 2.0, y: 0.0 });

        query.update_cache(&world);
        let gen_after = query.archetype_cache().unwrap().generation();

        // Generation should not change if no new archetypes were added
        assert_eq!(gen_before, gen_after);
        // But both entities should be found
        assert_eq!(query.iter(&world).count(), 2);
    }

    #[test]
    fn cache_generation_advances_with_new_archetypes() {
        let mut world = World::new();

        let mut query: Query<&Position> = Query::new(&world).with_cache(&world);
        let gen1 = query.archetype_cache().unwrap().generation();

        // Create a new archetype by inserting Position
        let e = world.spawn_empty();
        world.insert(e, Position { x: 0.0, y: 0.0 });

        query.update_cache(&world);
        let gen2 = query.archetype_cache().unwrap().generation();

        assert!(gen2 > gen1);
    }
}

// =========================================================================
// Cache with Despawn — entity gone but archetype still valid
// =========================================================================

mod cache_despawn {
    use super::*;

    #[test]
    fn cache_handles_entity_despawn_correctly() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 0.0 });

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 2.0, y: 0.0 });

        let query: Query<&Position> = Query::new(&world).with_cache(&world);
        assert_eq!(query.iter(&world).count(), 2);

        // Despawn one entity — archetype still exists
        world.despawn(e1);

        // Cache still points to the same archetypes; iteration should skip
        // the despawned entity gracefully
        assert_eq!(query.iter(&world).count(), 1);
    }

    #[test]
    fn cached_iter_mut_handles_despawn() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 0.0 });

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 2.0, y: 0.0 });

        let query: Query<&mut Position> = Query::new(&world).with_cache(&world);

        world.despawn(e1);

        let mut count = 0;
        for pos in query.iter_mut(&mut world) {
            pos.x += 10.0;
            count += 1;
        }
        assert_eq!(count, 1);
        assert_eq!(world.get::<Position>(e2).unwrap().x, 12.0);
    }
}

// =========================================================================
// Cache with Filters — With<T>, Without<T>
// =========================================================================

mod cache_filters {
    use super::*;

    #[test]
    fn cache_with_with_filter() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 0.0 });
        world.insert(e1, Player);

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 2.0, y: 0.0 });

        let uncached: Query<&Position, With<Player>> = Query::new(&world);
        let cached: Query<&Position, With<Player>> =
            Query::new(&world).with_cache(&world);

        assert_eq!(uncached.iter(&world).count(), cached.iter(&world).count());
        assert_eq!(cached.iter(&world).count(), 1);
    }

    #[test]
    fn cache_with_without_filter() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 0.0 });
        world.insert(e1, Enemy);

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 2.0, y: 0.0 });

        let uncached: Query<&Position, Without<Enemy>> = Query::new(&world);
        let cached: Query<&Position, Without<Enemy>> =
            Query::new(&world).with_cache(&world);

        assert_eq!(uncached.iter(&world).count(), cached.iter(&world).count());
        assert_eq!(cached.iter(&world).count(), 1);
    }

    #[test]
    fn cache_filter_updated_after_new_matching_archetype() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 0.0 });
        world.insert(e1, Player);

        let mut query: Query<&Position, With<Player>> =
            Query::new(&world).with_cache(&world);
        assert_eq!(query.iter(&world).count(), 1);

        // New archetype: Position + Player + Health
        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 2.0, y: 0.0 });
        world.insert(e2, Player);
        world.insert(e2, Health { value: 100 });

        query.update_cache(&world);
        assert_eq!(query.iter(&world).count(), 2);
    }
}

// =========================================================================
// Cache with Entity query
// =========================================================================

mod cache_entity_query {
    use super::*;

    #[test]
    fn cached_entity_query_returns_all_entities() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 0.0, y: 0.0 });

        let uncached: Query<Entity> = Query::new(&world);
        let cached: Query<Entity> = Query::new(&world).with_cache(&world);

        let uncached_entities: Vec<_> = uncached.iter(&world).collect();
        let cached_entities: Vec<_> = cached.iter(&world).collect();

        assert_eq!(uncached_entities.len(), cached_entities.len());
        assert!(cached_entities.contains(&e1));
        assert!(cached_entities.contains(&e2));
    }
}

// =========================================================================
// Cache Invalidation
// =========================================================================

mod cache_invalidation {
    use super::*;

    #[test]
    fn invalidate_cache_clears_and_forces_rebuild() {
        let mut world = World::new();

        let e = world.spawn_empty();
        world.insert(e, Position { x: 1.0, y: 0.0 });

        let mut query: Query<&Position> = Query::new(&world).with_cache(&world);
        assert!(query.archetype_cache().is_some());

        query.invalidate_cache();
        assert!(query.archetype_cache().is_none());

        // After invalidation, iter falls back to full scan
        assert_eq!(query.iter(&world).count(), 1);

        // Rebuild cache
        query.update_cache(&world);
        assert!(query.archetype_cache().is_some());
        assert_eq!(query.iter(&world).count(), 1);
    }
}

// =========================================================================
// Cache with Mutable Iteration
// =========================================================================

mod cache_iter_mut {
    use super::*;

    #[test]
    fn cached_iter_mut_modifies_correctly() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 3.0, y: 4.0 });

        let query: Query<&mut Position> = Query::new(&world).with_cache(&world);

        for pos in query.iter_mut(&mut world) {
            pos.x *= 10.0;
        }

        assert_eq!(world.get::<Position>(e1).unwrap().x, 10.0);
        assert_eq!(world.get::<Position>(e2).unwrap().x, 30.0);
    }

    #[test]
    fn cached_iter_mut_with_filter() {
        let mut world = World::new();

        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 0.0 });
        world.insert(e1, Player);

        let e2 = world.spawn_empty();
        world.insert(e2, Position { x: 2.0, y: 0.0 });

        let query: Query<&mut Position, With<Player>> =
            Query::new(&world).with_cache(&world);

        for pos in query.iter_mut(&mut world) {
            pos.x = 99.0;
        }

        assert_eq!(world.get::<Position>(e1).unwrap().x, 99.0);
        assert_eq!(world.get::<Position>(e2).unwrap().x, 2.0);
    }
}
