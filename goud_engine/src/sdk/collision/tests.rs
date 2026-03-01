use super::*;

#[test]
fn test_aabb_aabb_overlapping() {
    let contact = aabb_aabb(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.5, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());
    let c = contact.unwrap();
    assert!(c.penetration > 0.0);
}

#[test]
fn test_aabb_aabb_separated() {
    let contact = aabb_aabb(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(5.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_none());
}

#[test]
fn test_aabb_aabb_exact_touch() {
    let contact = aabb_aabb(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    if let Some(c) = contact {
        assert!(c.penetration >= 0.0);
    }
}

#[test]
fn test_circle_circle_overlapping() {
    let contact = circle_circle(Vec2::new(0.0, 0.0), 1.0, Vec2::new(1.5, 0.0), 1.0);
    assert!(contact.is_some());
    let c = contact.unwrap();
    assert!((c.penetration - 0.5).abs() < 0.01);
}

#[test]
fn test_circle_circle_separated() {
    let contact = circle_circle(Vec2::new(0.0, 0.0), 1.0, Vec2::new(5.0, 0.0), 1.0);
    assert!(contact.is_none());
}

#[test]
fn test_circle_circle_same_center() {
    let contact = circle_circle(Vec2::new(0.0, 0.0), 1.0, Vec2::new(0.0, 0.0), 1.0);
    assert!(contact.is_some());
    let c = contact.unwrap();
    assert!((c.penetration - 2.0).abs() < 0.01);
}

#[test]
fn test_circle_aabb_overlapping() {
    let contact = circle_aabb(
        Vec2::new(0.0, 0.0),
        1.0,
        Vec2::new(1.5, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_some());
}

#[test]
fn test_circle_aabb_separated() {
    let contact = circle_aabb(
        Vec2::new(0.0, 0.0),
        1.0,
        Vec2::new(5.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    assert!(contact.is_none());
}

#[test]
fn test_point_in_rect_inside() {
    assert!(point_in_rect(
        Vec2::new(5.0, 5.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));
}

#[test]
fn test_point_in_rect_outside() {
    assert!(!point_in_rect(
        Vec2::new(15.0, 5.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));
}

#[test]
fn test_point_in_rect_on_edge() {
    assert!(point_in_rect(
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));
    assert!(point_in_rect(
        Vec2::new(10.0, 10.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));
}

#[test]
fn test_point_in_circle_inside() {
    assert!(point_in_circle(
        Vec2::new(0.5, 0.0),
        Vec2::new(0.0, 0.0),
        1.0,
    ));
}

#[test]
fn test_point_in_circle_outside() {
    assert!(!point_in_circle(
        Vec2::new(2.0, 0.0),
        Vec2::new(0.0, 0.0),
        1.0,
    ));
}

#[test]
fn test_point_in_circle_on_edge() {
    assert!(point_in_circle(
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 0.0),
        1.0,
    ));
}

#[test]
fn test_aabb_overlap_true() {
    assert!(aabb_overlap(
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 2.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(3.0, 3.0),
    ));
}

#[test]
fn test_aabb_overlap_false() {
    assert!(!aabb_overlap(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 2.0),
        Vec2::new(3.0, 3.0),
    ));
}

#[test]
fn test_circle_overlap_true() {
    assert!(circle_overlap(
        Vec2::new(0.0, 0.0),
        1.0,
        Vec2::new(1.5, 0.0),
        1.0,
    ));
}

#[test]
fn test_circle_overlap_false() {
    assert!(!circle_overlap(
        Vec2::new(0.0, 0.0),
        1.0,
        Vec2::new(5.0, 0.0),
        1.0,
    ));
}

#[test]
fn test_distance_pythagorean() {
    let d = distance(Vec2::new(0.0, 0.0), Vec2::new(3.0, 4.0));
    assert!((d - 5.0).abs() < 0.001);
}

#[test]
fn test_distance_zero() {
    let d = distance(Vec2::new(1.0, 1.0), Vec2::new(1.0, 1.0));
    assert!((d - 0.0).abs() < 0.001);
}

#[test]
fn test_distance_squared_pythagorean() {
    let d2 = distance_squared(Vec2::new(0.0, 0.0), Vec2::new(3.0, 4.0));
    assert!((d2 - 25.0).abs() < 0.001);
}

#[test]
fn test_distance_squared_zero() {
    let d2 = distance_squared(Vec2::new(1.0, 1.0), Vec2::new(1.0, 1.0));
    assert!((d2 - 0.0).abs() < 0.001);
}

#[test]
fn test_contact_reexported() {
    let contact = Contact::new(Vec2::zero(), Vec2::unit_x(), 0.5);
    assert!(contact.is_colliding());
    assert_eq!(contact.separation_distance(), 0.5);
}
