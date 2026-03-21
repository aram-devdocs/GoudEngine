use super::*;
use crate::ecs::components::PoolMember;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Health(i32);
impl Component for Health {}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Speed(f32);
impl Component for Speed {}

#[test]
fn test_create_pool() {
    let mut world = World::new();
    let pool = world.create_entity_pool(10);

    let stats = world.pool_stats(pool).unwrap();
    assert_eq!(stats.capacity, 10);
    assert_eq!(stats.available, 10);
    assert_eq!(stats.active, 0);
    assert_eq!(world.entity_count(), 10);
}

#[test]
fn test_create_pool_zero_capacity() {
    let mut world = World::new();
    let pool = world.create_entity_pool(0);

    let stats = world.pool_stats(pool).unwrap();
    assert_eq!(stats.capacity, 0);
    assert_eq!(stats.available, 0);
    assert_eq!(stats.active, 0);
}

#[test]
fn test_acquire_from_pool() {
    let mut world = World::new();
    let pool = world.create_entity_pool(5);

    let entity = world.acquire_from_pool(pool).unwrap();
    assert!(world.is_alive(entity));
    assert!(world.has::<PoolMember>(entity));

    let stats = world.pool_stats(pool).unwrap();
    assert_eq!(stats.active, 1);
    assert_eq!(stats.available, 4);
}

#[test]
fn test_acquire_then_add_components() {
    let mut world = World::new();
    let pool = world.create_entity_pool(3);

    let entity = world.acquire_from_pool(pool).unwrap();
    world.insert(entity, Health(100));
    world.insert(entity, Speed(5.0));

    assert_eq!(world.get::<Health>(entity), Some(&Health(100)));
    assert_eq!(world.get::<Speed>(entity), Some(&Speed(5.0)));
}

#[test]
fn test_release_to_pool() {
    let mut world = World::new();
    let pool = world.create_entity_pool(3);

    let entity = world.acquire_from_pool(pool).unwrap();
    world.insert(entity, Health(100));
    world.insert(entity, Speed(5.0));

    assert!(world.release_to_pool(pool, entity));

    assert!(world.is_alive(entity));
    assert!(!world.has::<Health>(entity));
    assert!(!world.has::<Speed>(entity));
    assert!(!world.has::<PoolMember>(entity));

    let stats = world.pool_stats(pool).unwrap();
    assert_eq!(stats.active, 0);
    assert_eq!(stats.available, 3);
}

#[test]
fn test_acquire_release_reacquire() {
    let mut world = World::new();
    let pool = world.create_entity_pool(1);

    let entity1 = world.acquire_from_pool(pool).unwrap();
    world.insert(entity1, Health(100));

    assert!(world.acquire_from_pool(pool).is_none());
    assert!(world.release_to_pool(pool, entity1));

    let entity2 = world.acquire_from_pool(pool).unwrap();
    assert_eq!(entity1, entity2);
    assert!(!world.has::<Health>(entity2));
    assert!(world.has::<PoolMember>(entity2));
}

#[test]
fn test_pool_exhaustion() {
    let mut world = World::new();
    let pool = world.create_entity_pool(2);

    let _e1 = world.acquire_from_pool(pool).unwrap();
    let _e2 = world.acquire_from_pool(pool).unwrap();

    assert!(world.acquire_from_pool(pool).is_none());

    let stats = world.pool_stats(pool).unwrap();
    assert_eq!(stats.active, 2);
    assert_eq!(stats.available, 0);
}

#[test]
fn test_reset_entity() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    world.insert(entity, Health(50));
    world.insert(entity, Speed(3.0));

    assert!(world.reset_entity(entity));
    assert!(world.is_alive(entity));
    assert!(!world.has::<Health>(entity));
    assert!(!world.has::<Speed>(entity));
}

#[test]
fn test_reset_entity_already_empty() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    assert!(world.reset_entity(entity));
    assert!(world.is_alive(entity));
}

#[test]
fn test_reset_entity_dead() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.despawn(entity);
    assert!(!world.reset_entity(entity));
}

#[test]
fn test_destroy_pool_with_despawn() {
    let mut world = World::new();
    let pool = world.create_entity_pool(5);
    assert_eq!(world.entity_count(), 5);

    assert!(world.destroy_entity_pool(pool, true));
    assert_eq!(world.entity_count(), 0);
    assert!(world.pool_stats(pool).is_none());
}

#[test]
fn test_destroy_pool_without_despawn() {
    let mut world = World::new();
    let pool = world.create_entity_pool(5);

    assert!(world.destroy_entity_pool(pool, false));
    assert_eq!(world.entity_count(), 5);
    assert!(world.pool_stats(pool).is_none());
}

#[test]
fn test_destroy_invalid_pool() {
    let mut world = World::new();
    assert!(!world.destroy_entity_pool(999, true));
}

#[test]
fn test_pool_stats_invalid_handle() {
    let world = World::new();
    assert!(world.pool_stats(42).is_none());
}

#[test]
fn test_release_wrong_pool() {
    let mut world = World::new();
    let pool_a = world.create_entity_pool(3);
    let pool_b = world.create_entity_pool(3);

    let entity = world.acquire_from_pool(pool_a).unwrap();
    assert!(!world.release_to_pool(pool_b, entity));
}

#[test]
fn test_release_non_pooled_entity() {
    let mut world = World::new();
    let pool = world.create_entity_pool(3);
    let free_entity = world.spawn_empty();
    assert!(!world.release_to_pool(pool, free_entity));
}

#[test]
fn test_multiple_pools() {
    let mut world = World::new();
    let pool_a = world.create_entity_pool(3);
    let pool_b = world.create_entity_pool(5);

    let stats_a = world.pool_stats(pool_a).unwrap();
    let stats_b = world.pool_stats(pool_b).unwrap();
    assert_eq!(stats_a.capacity, 3);
    assert_eq!(stats_b.capacity, 5);

    let ea = world.acquire_from_pool(pool_a).unwrap();
    let eb = world.acquire_from_pool(pool_b).unwrap();
    assert_ne!(ea, eb);

    let stats_a = world.pool_stats(pool_a).unwrap();
    let stats_b = world.pool_stats(pool_b).unwrap();
    assert_eq!(stats_a.active, 1);
    assert_eq!(stats_b.active, 1);
}

#[test]
fn test_acquire_from_invalid_pool() {
    let mut world = World::new();
    assert!(world.acquire_from_pool(999).is_none());
}
