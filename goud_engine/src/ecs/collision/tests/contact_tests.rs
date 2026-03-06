//! Tests for the [`Contact`] type.

use crate::core::math::Vec2;
use crate::ecs::collision::contact::Contact;

#[test]
fn test_contact_new() {
    let contact = Contact::new(Vec2::new(1.0, 2.0), Vec2::new(1.0, 0.0), 0.5);
    assert_eq!(contact.point, Vec2::new(1.0, 2.0));
    assert_eq!(contact.normal, Vec2::new(1.0, 0.0));
    assert_eq!(contact.penetration, 0.5);
}

#[test]
fn test_contact_default() {
    let contact = Contact::default();
    assert_eq!(contact.point, Vec2::zero());
    assert_eq!(contact.normal, Vec2::unit_x());
    assert_eq!(contact.penetration, 0.0);
}

#[test]
fn test_contact_is_colliding() {
    let colliding = Contact::new(Vec2::zero(), Vec2::unit_x(), 0.5);
    let touching = Contact::new(Vec2::zero(), Vec2::unit_x(), 0.0);
    let separated = Contact::new(Vec2::zero(), Vec2::unit_x(), -0.1);

    assert!(colliding.is_colliding());
    assert!(!touching.is_colliding());
    assert!(!separated.is_colliding());
}

#[test]
fn test_contact_separation_distance() {
    let contact = Contact::new(Vec2::zero(), Vec2::unit_x(), 0.5);
    assert_eq!(contact.separation_distance(), 0.5);

    let negative = Contact::new(Vec2::zero(), Vec2::unit_x(), -0.3);
    assert_eq!(negative.separation_distance(), 0.3);
}

#[test]
fn test_contact_separation_vector() {
    let contact = Contact::new(Vec2::zero(), Vec2::new(1.0, 0.0), 0.5);
    assert_eq!(contact.separation_vector(), Vec2::new(0.5, 0.0));

    let diagonal = Contact::new(
        Vec2::zero(),
        Vec2::new(0.6, 0.8), // Normalized (approximately)
        2.0,
    );
    let sep = diagonal.separation_vector();
    assert!((sep.x - 1.2).abs() < 1e-5);
    assert!((sep.y - 1.6).abs() < 1e-5);
}

#[test]
fn test_contact_reversed() {
    let contact = Contact::new(Vec2::new(1.0, 2.0), Vec2::new(1.0, 0.0), 0.5);
    let reversed = contact.reversed();

    assert_eq!(reversed.point, contact.point);
    assert_eq!(reversed.normal, Vec2::new(-1.0, 0.0));
    assert_eq!(reversed.penetration, contact.penetration);
}

#[test]
fn test_contact_clone() {
    let contact = Contact::new(Vec2::new(1.0, 2.0), Vec2::unit_x(), 0.5);
    let cloned = contact.clone();
    assert_eq!(contact, cloned);
}

#[test]
fn test_contact_debug() {
    let contact = Contact::new(Vec2::new(1.0, 2.0), Vec2::unit_x(), 0.5);
    let debug_str = format!("{:?}", contact);
    assert!(debug_str.contains("Contact"));
    assert!(debug_str.contains("point"));
    assert!(debug_str.contains("normal"));
    assert!(debug_str.contains("penetration"));
}
