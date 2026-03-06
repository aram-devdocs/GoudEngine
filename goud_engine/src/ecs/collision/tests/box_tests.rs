//! Tests for box-based collision detection (AABB and OBB).

use crate::core::math::Vec2;
use crate::ecs::collision::detection_box::{aabb_aabb_collision, box_box_collision};

// -------------------------------------------------------------------------
// Box-Box Collision (SAT) Tests
// -------------------------------------------------------------------------

#[test]
fn test_box_box_collision_axis_aligned_overlapping() {
    let contact = box_box_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
        Vec2::new(1.5, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
    assert!((contact.penetration - 0.5).abs() < 1e-5);
    assert!((contact.normal.x.abs() - 1.0).abs() < 1e-5);
    assert!(contact.normal.y.abs() < 1e-5);
}

#[test]
fn test_box_box_collision_axis_aligned_separated() {
    let contact = box_box_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
        Vec2::new(5.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
    );
    assert!(contact.is_none());
}

#[test]
fn test_box_box_collision_rotated_overlapping() {
    use std::f32::consts::PI;

    let contact = box_box_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 4.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

#[test]
fn test_box_box_collision_rotated_separated() {
    use std::f32::consts::PI;

    let contact = box_box_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 6.0,
        Vec2::new(5.0, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 4.0,
    );
    assert!(contact.is_none());
}

#[test]
fn test_box_box_collision_both_rotated() {
    use std::f32::consts::PI;

    let contact = box_box_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 6.0,
        Vec2::new(1.2, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 3.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

#[test]
fn test_box_box_collision_same_position() {
    let contact = box_box_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

#[test]
fn test_box_box_collision_touching() {
    let contact = box_box_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
        Vec2::new(2.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
    );

    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration.abs() < 1e-3);
}

#[test]
fn test_box_box_collision_different_sizes() {
    let contact = box_box_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 1.0),
        0.0,
        Vec2::new(2.5, 0.0),
        Vec2::new(1.0, 2.0),
        0.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

#[test]
fn test_box_box_collision_normal_direction() {
    let contact = box_box_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
        Vec2::new(1.5, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
    )
    .unwrap();

    assert!(contact.normal.x > 0.0);
    assert!(contact.normal.y.abs() < 1e-5);
}

#[test]
fn test_box_box_collision_symmetry() {
    use std::f32::consts::PI;

    let contact_ab = box_box_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
        Vec2::new(1.5, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 6.0,
    );
    let contact_ba = box_box_collision(
        Vec2::new(1.5, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 6.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
    );

    assert!(contact_ab.is_some());
    assert!(contact_ba.is_some());

    let contact_ab = contact_ab.unwrap();
    let contact_ba = contact_ba.unwrap();

    assert!((contact_ab.penetration - contact_ba.penetration).abs() < 1e-5);
    assert!((contact_ab.normal.x + contact_ba.normal.x).abs() < 1e-5);
    assert!((contact_ab.normal.y + contact_ba.normal.y).abs() < 1e-5);
}

#[test]
fn test_box_box_collision_90_degree_rotation() {
    use std::f32::consts::PI;

    let contact = box_box_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 1.0),
        0.0,
        Vec2::new(1.5, 0.0),
        Vec2::new(2.0, 1.0),
        PI / 2.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

// -------------------------------------------------------------------------
// AABB-AABB Collision Tests
// -------------------------------------------------------------------------

#[test]
fn test_aabb_aabb_collision_overlapping() {
    let contact = aabb_aabb_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.5, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
    assert!((contact.penetration - 0.5).abs() < 1e-5);
    assert_eq!(contact.normal, Vec2::unit_x());
}

#[test]
fn test_aabb_aabb_collision_separated() {
    let contact = aabb_aabb_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(5.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_none());
}

#[test]
fn test_aabb_aabb_collision_touching() {
    let contact = aabb_aabb_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(1.0, 1.0),
    );

    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration.abs() < 1e-5);
}

#[test]
fn test_aabb_aabb_collision_vertical_overlap() {
    let contact = aabb_aabb_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.5),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
    assert!((contact.penetration - 0.5).abs() < 1e-5);
    assert_eq!(contact.normal, Vec2::unit_y());
}

#[test]
fn test_aabb_aabb_collision_different_sizes() {
    let contact = aabb_aabb_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 1.0),
        Vec2::new(2.5, 0.0),
        Vec2::new(1.0, 2.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

#[test]
fn test_aabb_aabb_collision_same_position() {
    let contact = aabb_aabb_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
    assert!((contact.penetration - 2.0).abs() < 1e-5);
}

#[test]
fn test_aabb_aabb_collision_contact_point() {
    let contact = aabb_aabb_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.5, 0.0),
        Vec2::new(1.0, 1.0),
    )
    .unwrap();

    assert!(contact.point.x > 0.0 && contact.point.x < 2.0);
    assert!(contact.point.y >= -1.0 && contact.point.y <= 1.0);
}

#[test]
fn test_aabb_aabb_collision_normal_direction() {
    let contact = aabb_aabb_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.5, 0.0),
        Vec2::new(1.0, 1.0),
    )
    .unwrap();

    assert_eq!(contact.normal, Vec2::unit_x());

    let contact_vertical = aabb_aabb_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.5),
        Vec2::new(1.0, 1.0),
    )
    .unwrap();

    assert_eq!(contact_vertical.normal, Vec2::unit_y());
}

#[test]
fn test_aabb_aabb_collision_symmetry() {
    let contact_ab = aabb_aabb_collision(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.5, 0.0),
        Vec2::new(1.0, 1.0),
    );
    let contact_ba = aabb_aabb_collision(
        Vec2::new(1.5, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );

    assert!(contact_ab.is_some());
    assert!(contact_ba.is_some());

    let contact_ab = contact_ab.unwrap();
    let contact_ba = contact_ba.unwrap();

    assert_eq!(contact_ab.penetration, contact_ba.penetration);
    assert_eq!(contact_ab.normal, contact_ba.normal * -1.0);
}

#[test]
fn test_aabb_aabb_collision_negative_coordinates() {
    let contact = aabb_aabb_collision(
        Vec2::new(-10.0, -10.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(-9.0, -10.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
    assert_eq!(contact.normal, Vec2::unit_x());
}
