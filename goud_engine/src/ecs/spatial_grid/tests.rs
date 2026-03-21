//! Unit tests for SpatialGrid.

use super::SpatialGrid;
use crate::core::math::Vec2;
use crate::ecs::Entity;

#[test]
fn test_new_grid_is_empty() {
    let grid = SpatialGrid::new(32.0);
    assert!(grid.is_empty());
    assert_eq!(grid.entity_count(), 0);
    assert_eq!(grid.cell_size(), 32.0);
}

#[test]
fn test_with_capacity() {
    let grid = SpatialGrid::with_capacity(64.0, 1000);
    assert!(grid.is_empty());
    assert_eq!(grid.cell_size(), 64.0);
}

#[test]
#[should_panic(expected = "Cell size must be positive and finite")]
fn test_zero_cell_size_panics() {
    SpatialGrid::new(0.0);
}

#[test]
#[should_panic(expected = "Cell size must be positive and finite")]
fn test_negative_cell_size_panics() {
    SpatialGrid::new(-10.0);
}

#[test]
#[should_panic(expected = "Cell size must be positive and finite")]
fn test_infinite_cell_size_panics() {
    SpatialGrid::new(f32::INFINITY);
}

#[test]
fn test_insert_and_contains() {
    let mut grid = SpatialGrid::new(32.0);
    let e = Entity::new(0, 0);

    assert!(!grid.contains(e));
    grid.insert(e, Vec2::new(10.0, 20.0));
    assert!(grid.contains(e));
    assert_eq!(grid.entity_count(), 1);
}

#[test]
fn test_insert_multiple() {
    let mut grid = SpatialGrid::new(32.0);
    let e1 = Entity::new(0, 0);
    let e2 = Entity::new(1, 0);
    let e3 = Entity::new(2, 0);

    grid.insert(e1, Vec2::new(0.0, 0.0));
    grid.insert(e2, Vec2::new(100.0, 100.0));
    grid.insert(e3, Vec2::new(200.0, 200.0));

    assert_eq!(grid.entity_count(), 3);
    assert!(grid.contains(e1));
    assert!(grid.contains(e2));
    assert!(grid.contains(e3));
}

#[test]
fn test_insert_replaces_existing() {
    let mut grid = SpatialGrid::new(32.0);
    let e = Entity::new(0, 0);

    grid.insert(e, Vec2::new(10.0, 20.0));
    grid.insert(e, Vec2::new(500.0, 500.0));

    assert_eq!(grid.entity_count(), 1);
    assert_eq!(grid.get_position(e), Some(Vec2::new(500.0, 500.0)));
}

#[test]
fn test_remove() {
    let mut grid = SpatialGrid::new(32.0);
    let e = Entity::new(0, 0);

    grid.insert(e, Vec2::new(10.0, 20.0));
    assert!(grid.remove(e));
    assert!(!grid.contains(e));
    assert_eq!(grid.entity_count(), 0);
}

#[test]
fn test_remove_nonexistent() {
    let mut grid = SpatialGrid::new(32.0);
    let e = Entity::new(0, 0);
    assert!(!grid.remove(e));
}

#[test]
fn test_update_same_cell() {
    let mut grid = SpatialGrid::new(64.0);
    let e = Entity::new(0, 0);

    grid.insert(e, Vec2::new(10.0, 10.0));
    assert!(grid.update(e, Vec2::new(15.0, 15.0)));
    assert_eq!(grid.get_position(e), Some(Vec2::new(15.0, 15.0)));
    assert_eq!(grid.entity_count(), 1);
}

#[test]
fn test_update_cross_cell() {
    let mut grid = SpatialGrid::new(32.0);
    let e = Entity::new(0, 0);

    grid.insert(e, Vec2::new(10.0, 10.0));
    assert!(grid.update(e, Vec2::new(500.0, 500.0)));
    assert_eq!(grid.get_position(e), Some(Vec2::new(500.0, 500.0)));
    assert_eq!(grid.entity_count(), 1);
}

#[test]
fn test_update_nonexistent() {
    let mut grid = SpatialGrid::new(32.0);
    let e = Entity::new(0, 0);
    assert!(!grid.update(e, Vec2::new(10.0, 10.0)));
}

#[test]
fn test_clear() {
    let mut grid = SpatialGrid::new(32.0);
    for i in 0..10 {
        grid.insert(Entity::new(i, 0), Vec2::new(i as f32 * 50.0, 0.0));
    }
    assert_eq!(grid.entity_count(), 10);

    grid.clear();
    assert!(grid.is_empty());
    assert_eq!(grid.entity_count(), 0);
}

#[test]
fn test_query_radius_finds_nearby() {
    let mut grid = SpatialGrid::new(32.0);
    let e1 = Entity::new(0, 0);
    let e2 = Entity::new(1, 0);

    grid.insert(e1, Vec2::new(10.0, 10.0));
    grid.insert(e2, Vec2::new(20.0, 10.0));

    let results = grid.query_radius(Vec2::new(10.0, 10.0), 15.0);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_query_radius_excludes_distant() {
    let mut grid = SpatialGrid::new(32.0);
    let near = Entity::new(0, 0);
    let far = Entity::new(1, 0);

    grid.insert(near, Vec2::new(10.0, 10.0));
    grid.insert(far, Vec2::new(500.0, 500.0));

    let results = grid.query_radius(Vec2::new(10.0, 10.0), 20.0);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], near);
}

#[test]
fn test_query_radius_exact_boundary() {
    let mut grid = SpatialGrid::new(32.0);
    let e = Entity::new(0, 0);

    grid.insert(e, Vec2::new(10.0, 0.0));

    // Entity is exactly at radius distance
    let results = grid.query_radius(Vec2::new(0.0, 0.0), 10.0);
    assert_eq!(results.len(), 1);

    // Entity is just outside radius
    let results = grid.query_radius(Vec2::new(0.0, 0.0), 9.99);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_query_radius_empty_grid() {
    let grid = SpatialGrid::new(32.0);
    let results = grid.query_radius(Vec2::new(0.0, 0.0), 100.0);
    assert!(results.is_empty());
}

#[test]
fn test_query_point_same_cell() {
    let mut grid = SpatialGrid::new(64.0);
    let e1 = Entity::new(0, 0);
    let e2 = Entity::new(1, 0);

    // Both in the same cell (0..64 range)
    grid.insert(e1, Vec2::new(10.0, 10.0));
    grid.insert(e2, Vec2::new(50.0, 50.0));

    let results = grid.query_point(Vec2::new(30.0, 30.0));
    assert_eq!(results.len(), 2);
}

#[test]
fn test_query_point_different_cell() {
    let mut grid = SpatialGrid::new(32.0);
    let e1 = Entity::new(0, 0);
    let e2 = Entity::new(1, 0);

    grid.insert(e1, Vec2::new(10.0, 10.0));
    grid.insert(e2, Vec2::new(100.0, 100.0));

    let results = grid.query_point(Vec2::new(10.0, 10.0));
    assert_eq!(results.len(), 1);
}

#[test]
fn test_negative_coordinates() {
    let mut grid = SpatialGrid::new(32.0);
    let e = Entity::new(0, 0);

    grid.insert(e, Vec2::new(-100.0, -200.0));
    assert!(grid.contains(e));

    let results = grid.query_radius(Vec2::new(-100.0, -200.0), 10.0);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_display() {
    let mut grid = SpatialGrid::new(32.0);
    grid.insert(Entity::new(0, 0), Vec2::new(10.0, 10.0));
    let display = format!("{}", grid);
    assert!(display.contains("SpatialGrid"));
    assert!(display.contains("32.0"));
    assert!(display.contains("entities: 1"));
}

#[test]
fn test_get_position() {
    let mut grid = SpatialGrid::new(32.0);
    let e = Entity::new(0, 0);

    assert_eq!(grid.get_position(e), None);
    grid.insert(e, Vec2::new(42.0, 84.0));
    assert_eq!(grid.get_position(e), Some(Vec2::new(42.0, 84.0)));
}

#[test]
fn test_many_entities_same_cell() {
    let mut grid = SpatialGrid::new(1000.0);
    for i in 0..100 {
        grid.insert(Entity::new(i, 0), Vec2::new(i as f32, i as f32));
    }
    assert_eq!(grid.entity_count(), 100);

    // All in same cell, all within radius
    let results = grid.query_radius(Vec2::new(50.0, 50.0), 200.0);
    assert_eq!(results.len(), 100);
}
