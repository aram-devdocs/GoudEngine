//! Tests for circle-based collision detection.

use crate::core::math::Vec2;
use crate::ecs::collision::detection_circle::{
    circle_aabb_collision, circle_circle_collision, circle_obb_collision,
};

// -------------------------------------------------------------------------
// Circle-Circle Collision Tests
// -------------------------------------------------------------------------

#[test]
fn test_circle_circle_collision_overlapping() {
    let contact = circle_circle_collision(Vec2::new(0.0, 0.0), 1.0, Vec2::new(1.5, 0.0), 1.0);
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
    assert_eq!(contact.normal, Vec2::new(1.0, 0.0));
}

#[test]
fn test_circle_circle_collision_separated() {
    let contact = circle_circle_collision(Vec2::new(0.0, 0.0), 1.0, Vec2::new(5.0, 0.0), 1.0);
    assert!(contact.is_none());
}

#[test]
fn test_circle_circle_collision_touching() {
    let contact = circle_circle_collision(Vec2::new(0.0, 0.0), 1.0, Vec2::new(2.0, 0.0), 1.0);
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!((contact.penetration).abs() < 1e-5);
}

#[test]
fn test_circle_circle_collision_same_position() {
    let contact = circle_circle_collision(Vec2::new(0.0, 0.0), 1.0, Vec2::new(0.0, 0.0), 1.0);
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert_eq!(contact.penetration, 2.0);
    assert_eq!(contact.point, Vec2::zero());
}

#[test]
fn test_circle_circle_collision_diagonal() {
    let contact = circle_circle_collision(Vec2::new(0.0, 0.0), 1.0, Vec2::new(1.0, 1.0), 1.0);
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);

    let expected_normal = Vec2::new(1.0, 1.0).normalize();
    assert!((contact.normal.x - expected_normal.x).abs() < 1e-5);
    assert!((contact.normal.y - expected_normal.y).abs() < 1e-5);
}

#[test]
fn test_circle_circle_collision_different_radii() {
    let contact = circle_circle_collision(Vec2::new(0.0, 0.0), 2.0, Vec2::new(3.0, 0.0), 1.5);
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
    assert!((contact.penetration - 0.5).abs() < 1e-5);
}

#[test]
fn test_circle_circle_collision_negative_coordinates() {
    let contact =
        circle_circle_collision(Vec2::new(-10.0, -10.0), 1.0, Vec2::new(-9.0, -10.0), 1.0);
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
    assert_eq!(contact.normal, Vec2::new(1.0, 0.0));
}

#[test]
fn test_circle_circle_collision_contact_point() {
    let contact =
        circle_circle_collision(Vec2::new(0.0, 0.0), 1.0, Vec2::new(1.5, 0.0), 1.0).unwrap();

    assert!(contact.point.x > 0.0 && contact.point.x < 1.5);
    assert_eq!(contact.point.y, 0.0);
}

#[test]
fn test_circle_circle_collision_symmetry() {
    let contact_ab = circle_circle_collision(Vec2::new(0.0, 0.0), 1.0, Vec2::new(1.5, 0.0), 1.0);
    let contact_ba = circle_circle_collision(Vec2::new(1.5, 0.0), 1.0, Vec2::new(0.0, 0.0), 1.0);

    assert!(contact_ab.is_some());
    assert!(contact_ba.is_some());

    let contact_ab = contact_ab.unwrap();
    let contact_ba = contact_ba.unwrap();

    assert_eq!(contact_ab.penetration, contact_ba.penetration);
    assert_eq!(contact_ab.normal, contact_ba.normal * -1.0);
}

#[test]
fn test_circle_circle_collision_large_circles() {
    let contact = circle_circle_collision(Vec2::new(0.0, 0.0), 100.0, Vec2::new(150.0, 0.0), 100.0);
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!((contact.penetration - 50.0).abs() < 1e-3);
}

#[test]
fn test_circle_circle_collision_tiny_circles() {
    let contact = circle_circle_collision(Vec2::new(0.0, 0.0), 0.01, Vec2::new(0.015, 0.0), 0.01);
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

// -------------------------------------------------------------------------
// Circle-AABB Collision Tests
// -------------------------------------------------------------------------

#[test]
fn test_circle_aabb_collision_overlapping() {
    let contact = circle_aabb_collision(
        Vec2::new(1.5, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
    assert!((contact.penetration - 0.5).abs() < 1e-5);
    assert_eq!(contact.normal, Vec2::unit_x());
}

#[test]
fn test_circle_aabb_collision_separated() {
    let contact = circle_aabb_collision(
        Vec2::new(5.0, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_none());
}

#[test]
fn test_circle_aabb_collision_corner() {
    let contact = circle_aabb_collision(
        Vec2::new(1.5, 1.5),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);

    let expected_normal = Vec2::new(0.5, 0.5).normalize();
    assert!((contact.normal.x - expected_normal.x).abs() < 1e-5);
    assert!((contact.normal.y - expected_normal.y).abs() < 1e-5);
}

#[test]
fn test_circle_aabb_collision_edge() {
    let contact = circle_aabb_collision(
        Vec2::new(1.8, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!((contact.penetration - 0.2).abs() < 1e-5);
    assert_eq!(contact.normal, Vec2::unit_x());
}

#[test]
fn test_circle_aabb_collision_inside() {
    let contact = circle_aabb_collision(
        Vec2::new(0.5, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 1.0);
    assert!(contact.normal.x.abs() > 0.9 || contact.normal.y.abs() > 0.9);
}

#[test]
fn test_circle_aabb_collision_center_coincident() {
    let contact = circle_aabb_collision(
        Vec2::new(0.0, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 1.0);
}

#[test]
fn test_circle_aabb_collision_touching() {
    let contact = circle_aabb_collision(
        Vec2::new(2.0, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration.abs() < 1e-5);
}

#[test]
fn test_circle_aabb_collision_different_sizes() {
    let contact = circle_aabb_collision(
        Vec2::new(2.0, 0.0),
        2.5,
        Vec2::new(0.0, 0.0),
        Vec2::new(0.5, 0.5),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
    assert!((contact.penetration - 1.0).abs() < 1e-5);
}

#[test]
fn test_circle_aabb_collision_vertical() {
    let contact = circle_aabb_collision(
        Vec2::new(0.0, 1.8),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!((contact.penetration - 0.2).abs() < 1e-5);
    assert_eq!(contact.normal, Vec2::unit_y());
}

#[test]
fn test_circle_aabb_collision_contact_point() {
    let contact = circle_aabb_collision(
        Vec2::new(1.8, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    )
    .unwrap();

    assert!((contact.point.x - 1.0).abs() < 1e-5);
    assert!((contact.point.y - 0.0).abs() < 1e-5);
}

#[test]
fn test_circle_aabb_collision_symmetry() {
    let contact1 = circle_aabb_collision(
        Vec2::new(1.8, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    let contact2 = circle_aabb_collision(
        Vec2::new(-1.8, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );

    assert!(contact1.is_some());
    assert!(contact2.is_some());

    let c1 = contact1.unwrap();
    let c2 = contact2.unwrap();

    assert_eq!(c1.penetration, c2.penetration);
    assert_eq!(c1.normal.x, -c2.normal.x);
}

// -------------------------------------------------------------------------
// Circle-OBB Collision Tests
// -------------------------------------------------------------------------

#[test]
fn test_circle_obb_collision_no_rotation() {
    let contact_obb = circle_obb_collision(
        Vec2::new(1.8, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        0.0,
    );
    let contact_aabb = circle_aabb_collision(
        Vec2::new(1.8, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );

    assert!(contact_obb.is_some());
    assert!(contact_aabb.is_some());

    let c_obb = contact_obb.unwrap();
    let c_aabb = contact_aabb.unwrap();

    assert!((c_obb.penetration - c_aabb.penetration).abs() < 1e-5);
}

#[test]
fn test_circle_obb_collision_45_degree_rotation() {
    use std::f32::consts::PI;

    let contact = circle_obb_collision(
        Vec2::new(1.8, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 4.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

#[test]
fn test_circle_obb_collision_90_degree_rotation() {
    use std::f32::consts::PI;

    let contact = circle_obb_collision(
        Vec2::new(1.8, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 2.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

#[test]
fn test_circle_obb_collision_rotated_rectangle() {
    use std::f32::consts::PI;

    let contact = circle_obb_collision(
        Vec2::new(2.5, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 0.5),
        PI / 6.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

#[test]
fn test_circle_obb_collision_separated_rotated() {
    use std::f32::consts::PI;

    let contact = circle_obb_collision(
        Vec2::new(5.0, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 4.0,
    );
    assert!(contact.is_none());
}

#[test]
fn test_circle_obb_collision_inside_rotated() {
    use std::f32::consts::PI;

    let contact = circle_obb_collision(
        Vec2::new(0.5, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 2.0),
        PI / 4.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

#[test]
fn test_circle_obb_collision_corner_rotated() {
    use std::f32::consts::PI;

    let contact = circle_obb_collision(
        Vec2::new(1.2, 1.2),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 4.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

#[test]
fn test_circle_obb_collision_touching_rotated() {
    use std::f32::consts::PI;

    let contact = circle_obb_collision(
        Vec2::new(1.8, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        PI / 4.0,
    );

    if let Some(c) = contact {
        assert!(c.penetration >= 0.0);
    }
}

#[test]
fn test_circle_obb_collision_large_rotation() {
    use std::f32::consts::PI;

    let contact = circle_obb_collision(
        Vec2::new(1.5, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        3.0 * PI / 4.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}

#[test]
fn test_circle_obb_collision_negative_rotation() {
    use std::f32::consts::PI;

    let contact = circle_obb_collision(
        Vec2::new(1.8, 0.0),
        1.0,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        -PI / 4.0,
    );
    assert!(contact.is_some());

    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
}
