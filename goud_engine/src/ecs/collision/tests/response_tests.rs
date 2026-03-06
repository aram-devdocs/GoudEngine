//! Tests for [`CollisionResponse`], [`resolve_collision`], and
//! [`compute_position_correction`].

use crate::core::math::Vec2;
use crate::ecs::collision::contact::Contact;
use crate::ecs::collision::response::{
    compute_position_correction, resolve_collision, CollisionResponse,
};

// -------------------------------------------------------------------------
// CollisionResponse Tests
// -------------------------------------------------------------------------

#[test]
fn test_collision_response_new() {
    let response = CollisionResponse::new(0.5, 0.4, 0.6, 0.02);
    assert_eq!(response.restitution, 0.5);
    assert_eq!(response.friction, 0.4);
    assert_eq!(response.position_correction, 0.6);
    assert_eq!(response.slop, 0.02);
}

#[test]
fn test_collision_response_new_clamping() {
    let response = CollisionResponse::new(1.5, -0.2, 2.0, -0.01);
    assert_eq!(response.restitution, 1.0);
    assert_eq!(response.friction, 0.0);
    assert_eq!(response.position_correction, 1.0);
    assert_eq!(response.slop, 0.0);
}

#[test]
fn test_collision_response_default() {
    let response = CollisionResponse::default();
    assert_eq!(response.restitution, 0.4);
    assert_eq!(response.friction, 0.4);
    assert_eq!(response.position_correction, 0.4);
    assert_eq!(response.slop, 0.01);
}

#[test]
fn test_collision_response_bouncy() {
    let response = CollisionResponse::bouncy();
    assert_eq!(response.restitution, 0.8);
    assert_eq!(response.friction, 0.2);
}

#[test]
fn test_collision_response_character() {
    let response = CollisionResponse::character();
    assert_eq!(response.restitution, 0.0);
    assert_eq!(response.friction, 0.8);
}

#[test]
fn test_collision_response_slippery() {
    let response = CollisionResponse::slippery();
    assert!(response.friction < 0.2);
    assert!(response.restitution > 0.0);
}

#[test]
fn test_collision_response_elastic() {
    let response = CollisionResponse::elastic();
    assert_eq!(response.restitution, 1.0);
}

#[test]
fn test_collision_response_clone() {
    let response = CollisionResponse::default();
    let cloned = response.clone();
    assert_eq!(response, cloned);
}

#[test]
fn test_collision_response_debug() {
    let response = CollisionResponse::default();
    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("CollisionResponse"));
}

// -------------------------------------------------------------------------
// Impulse Resolution Tests
// -------------------------------------------------------------------------

#[test]
fn test_resolve_collision_head_on() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.1);

    let vel_a = Vec2::new(10.0, 0.0);
    let vel_b = Vec2::new(-10.0, 0.0);

    let response = CollisionResponse::elastic();
    let (delta_a, delta_b) = resolve_collision(&contact, vel_a, vel_b, 1.0, 1.0, &response);

    let new_vel_a = vel_a + delta_a;
    let new_vel_b = vel_b + delta_b;

    assert!(new_vel_a.x < 0.0);
    assert!(new_vel_b.x > 0.0);
}

#[test]
fn test_resolve_collision_static_wall() {
    let contact = Contact::new(Vec2::new(1.0, 0.0), Vec2::new(1.0, 0.0), 0.1);

    let vel_a = Vec2::new(10.0, 0.0);
    let vel_b = Vec2::zero();

    let response = CollisionResponse::bouncy();
    let (delta_a, delta_b) = resolve_collision(&contact, vel_a, vel_b, 1.0, 0.0, &response);

    assert_eq!(delta_b, Vec2::zero());
    let new_vel_a = vel_a + delta_a;
    assert!(new_vel_a.x < vel_a.x);
}

#[test]
fn test_resolve_collision_no_bounce() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.1);

    let vel_a = Vec2::new(10.0, 0.0);
    let vel_b = Vec2::zero();

    let response = CollisionResponse::character();
    let (delta_a, _) = resolve_collision(&contact, vel_a, vel_b, 1.0, 1.0, &response);

    let new_vel_a = vel_a + delta_a;
    assert!(new_vel_a.x < vel_a.x);
}

#[test]
fn test_resolve_collision_separating() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.1);

    let vel_a = Vec2::new(-5.0, 0.0);
    let vel_b = Vec2::new(5.0, 0.0);

    let response = CollisionResponse::default();
    let (delta_a, delta_b) = resolve_collision(&contact, vel_a, vel_b, 1.0, 1.0, &response);

    assert_eq!(delta_a, Vec2::zero());
    assert_eq!(delta_b, Vec2::zero());
}

#[test]
fn test_resolve_collision_two_static() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.1);

    let response = CollisionResponse::default();
    let (delta_a, delta_b) =
        resolve_collision(&contact, Vec2::zero(), Vec2::zero(), 0.0, 0.0, &response);

    assert_eq!(delta_a, Vec2::zero());
    assert_eq!(delta_b, Vec2::zero());
}

#[test]
fn test_resolve_collision_with_friction() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, -1.0), 0.1);

    let vel_a = Vec2::new(10.0, -5.0);
    let vel_b = Vec2::zero();

    let response = CollisionResponse::character();
    let (delta_a, _) = resolve_collision(&contact, vel_a, vel_b, 1.0, 0.0, &response);

    assert!(delta_a.length() > 0.0);
    let new_vel_a = vel_a + delta_a;
    assert!(new_vel_a.x < vel_a.x);
}

#[test]
fn test_resolve_collision_diagonal() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(0.707, 0.707), 0.1);

    let vel_a = Vec2::new(10.0, 10.0);
    let vel_b = Vec2::zero();

    let response = CollisionResponse::default();
    let (delta_a, _) = resolve_collision(&contact, vel_a, vel_b, 1.0, 0.0, &response);

    let new_vel_a = vel_a + delta_a;
    assert!(new_vel_a.length() < vel_a.length());
}

#[test]
fn test_resolve_collision_mass_ratio() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.1);

    let vel_a = Vec2::new(10.0, 0.0);
    let vel_b = Vec2::zero();

    let response = CollisionResponse::default();
    let (delta_a, delta_b) = resolve_collision(&contact, vel_a, vel_b, 0.1, 1.0, &response);

    assert!(delta_b.length() > delta_a.length());
}

// -------------------------------------------------------------------------
// Position Correction Tests
// -------------------------------------------------------------------------

#[test]
fn test_position_correction_basic() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.1);
    let response = CollisionResponse::default();

    let (corr_a, corr_b) = compute_position_correction(&contact, 1.0, 1.0, &response);

    assert!(corr_a.length() > 0.0);
    assert!(corr_b.length() > 0.0);
    assert!(corr_a.x < 0.0);
    assert!(corr_b.x > 0.0);
}

#[test]
fn test_position_correction_below_slop() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.005);
    let response = CollisionResponse::default();

    let (corr_a, corr_b) = compute_position_correction(&contact, 1.0, 1.0, &response);

    assert_eq!(corr_a, Vec2::zero());
    assert_eq!(corr_b, Vec2::zero());
}

#[test]
fn test_position_correction_static_wall() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.1);
    let response = CollisionResponse::default();

    let (corr_a, corr_b) = compute_position_correction(&contact, 1.0, 0.0, &response);

    assert!(corr_a.length() > 0.0);
    assert_eq!(corr_b, Vec2::zero());
}

#[test]
fn test_position_correction_two_static() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.1);
    let response = CollisionResponse::default();

    let (corr_a, corr_b) = compute_position_correction(&contact, 0.0, 0.0, &response);

    assert_eq!(corr_a, Vec2::zero());
    assert_eq!(corr_b, Vec2::zero());
}

#[test]
fn test_position_correction_zero_percent() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.1);
    let response = CollisionResponse::new(0.5, 0.5, 0.0, 0.01);

    let (corr_a, corr_b) = compute_position_correction(&contact, 1.0, 1.0, &response);

    assert_eq!(corr_a, Vec2::zero());
    assert_eq!(corr_b, Vec2::zero());
}

#[test]
fn test_position_correction_full_percent() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.1);
    let response = CollisionResponse::new(0.5, 0.5, 1.0, 0.01);

    let (corr_a, corr_b) = compute_position_correction(&contact, 1.0, 1.0, &response);

    assert!(corr_a.length() > 0.0);
    assert!(corr_b.length() > 0.0);
}

#[test]
fn test_position_correction_mass_ratio() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 0.1);
    let response = CollisionResponse::default();

    let (corr_a, corr_b) = compute_position_correction(&contact, 0.1, 1.0, &response);

    assert!(corr_b.length() > corr_a.length());
}

#[test]
fn test_position_correction_direction() {
    let contact = Contact::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0), 0.1);
    let response = CollisionResponse::default();

    let (corr_a, corr_b) = compute_position_correction(&contact, 1.0, 1.0, &response);

    assert!(corr_a.y < 0.0);
    assert!(corr_b.y > 0.0);
    assert!(corr_a.x.abs() < 1e-6);
    assert!(corr_b.x.abs() < 1e-6);
}
