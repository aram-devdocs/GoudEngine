//! Tests for the RigidBody component and RigidBodyType enum.

use crate::core::math::Vec2;
use crate::ecs::components::{RigidBody, RigidBodyType};
use crate::ecs::Component;

// =========================================================================
// RigidBodyType Tests
// =========================================================================

#[test]
fn test_rigidbody_type_default() {
    assert_eq!(RigidBodyType::default(), RigidBodyType::Dynamic);
}

#[test]
fn test_rigidbody_type_is_affected_by_gravity() {
    assert!(RigidBodyType::Dynamic.is_affected_by_gravity());
    assert!(!RigidBodyType::Kinematic.is_affected_by_gravity());
    assert!(!RigidBodyType::Static.is_affected_by_gravity());
}

#[test]
fn test_rigidbody_type_is_affected_by_forces() {
    assert!(RigidBodyType::Dynamic.is_affected_by_forces());
    assert!(!RigidBodyType::Kinematic.is_affected_by_forces());
    assert!(!RigidBodyType::Static.is_affected_by_forces());
}

#[test]
fn test_rigidbody_type_can_move() {
    assert!(RigidBodyType::Dynamic.can_move());
    assert!(RigidBodyType::Kinematic.can_move());
    assert!(!RigidBodyType::Static.can_move());
}

#[test]
fn test_rigidbody_type_responds_to_collisions() {
    assert!(RigidBodyType::Dynamic.responds_to_collisions());
    assert!(!RigidBodyType::Kinematic.responds_to_collisions());
    assert!(!RigidBodyType::Static.responds_to_collisions());
}

#[test]
fn test_rigidbody_type_name() {
    assert_eq!(RigidBodyType::Dynamic.name(), "Dynamic");
    assert_eq!(RigidBodyType::Kinematic.name(), "Kinematic");
    assert_eq!(RigidBodyType::Static.name(), "Static");
}

#[test]
fn test_rigidbody_type_display() {
    assert_eq!(format!("{}", RigidBodyType::Dynamic), "Dynamic");
    assert_eq!(format!("{}", RigidBodyType::Kinematic), "Kinematic");
    assert_eq!(format!("{}", RigidBodyType::Static), "Static");
}

// =========================================================================
// RigidBody Construction Tests
// =========================================================================

#[test]
fn test_rigidbody_new_dynamic() {
    let body = RigidBody::new(RigidBodyType::Dynamic);
    assert_eq!(body.body_type, RigidBodyType::Dynamic);
    assert_eq!(body.mass, 1.0);
    assert_eq!(body.inverse_mass, 1.0);
    assert!(body.can_sleep());
    assert!(!body.is_sleeping());
}

#[test]
fn test_rigidbody_new_kinematic() {
    let body = RigidBody::new(RigidBodyType::Kinematic);
    assert_eq!(body.body_type, RigidBodyType::Kinematic);
    assert_eq!(body.mass, 0.0);
    assert_eq!(body.inverse_mass, 0.0);
}

#[test]
fn test_rigidbody_new_static() {
    let body = RigidBody::new(RigidBodyType::Static);
    assert_eq!(body.body_type, RigidBodyType::Static);
    assert_eq!(body.mass, 0.0);
    assert_eq!(body.inverse_mass, 0.0);
}

#[test]
fn test_rigidbody_dynamic() {
    let body = RigidBody::dynamic();
    assert!(body.is_dynamic());
    assert!(!body.is_kinematic());
    assert!(!body.is_static());
}

#[test]
fn test_rigidbody_kinematic() {
    let body = RigidBody::kinematic();
    assert!(!body.is_dynamic());
    assert!(body.is_kinematic());
    assert!(!body.is_static());
}

#[test]
fn test_rigidbody_static_body() {
    let body = RigidBody::static_body();
    assert!(!body.is_dynamic());
    assert!(!body.is_kinematic());
    assert!(body.is_static());
}

#[test]
fn test_rigidbody_default() {
    let body = RigidBody::default();
    assert!(body.is_dynamic());
    assert_eq!(body.mass, 1.0);
}

// =========================================================================
// Builder Pattern Tests
// =========================================================================

#[test]
fn test_rigidbody_with_velocity() {
    let body = RigidBody::dynamic().with_velocity(Vec2::new(100.0, 50.0));
    assert_eq!(body.linear_velocity, Vec2::new(100.0, 50.0));
}

#[test]
fn test_rigidbody_with_angular_velocity() {
    let body = RigidBody::dynamic().with_angular_velocity(3.14);
    assert_eq!(body.angular_velocity, 3.14);
}

#[test]
fn test_rigidbody_with_mass() {
    let body = RigidBody::dynamic().with_mass(2.0);
    assert_eq!(body.mass, 2.0);
    assert_eq!(body.inverse_mass, 0.5);
}

#[test]
#[should_panic(expected = "Mass must be positive and finite")]
fn test_rigidbody_with_mass_zero() {
    let _ = RigidBody::dynamic().with_mass(0.0);
}

#[test]
#[should_panic(expected = "Mass must be positive and finite")]
fn test_rigidbody_with_mass_negative() {
    let _ = RigidBody::dynamic().with_mass(-1.0);
}

#[test]
fn test_rigidbody_with_damping() {
    let body = RigidBody::dynamic()
        .with_linear_damping(0.5)
        .with_angular_damping(0.3);
    assert_eq!(body.linear_damping, 0.5);
    assert_eq!(body.angular_damping, 0.3);
}

#[test]
fn test_rigidbody_with_restitution() {
    let body = RigidBody::dynamic().with_restitution(0.8);
    assert_eq!(body.restitution, 0.8);
}

#[test]
fn test_rigidbody_with_friction() {
    let body = RigidBody::dynamic().with_friction(0.7);
    assert_eq!(body.friction, 0.7);
}

#[test]
fn test_rigidbody_with_gravity_scale() {
    let body = RigidBody::dynamic().with_gravity_scale(2.0);
    assert_eq!(body.gravity_scale, 2.0);
}

#[test]
fn test_rigidbody_with_can_sleep() {
    let body1 = RigidBody::dynamic().with_can_sleep(true);
    assert!(body1.can_sleep());

    let body2 = RigidBody::dynamic().with_can_sleep(false);
    assert!(!body2.can_sleep());
}

#[test]
fn test_rigidbody_with_continuous_cd() {
    let body1 = RigidBody::dynamic().with_continuous_cd(true);
    assert!(body1.has_continuous_cd());

    let body2 = RigidBody::dynamic().with_continuous_cd(false);
    assert!(!body2.has_continuous_cd());
}

#[test]
fn test_rigidbody_with_fixed_rotation() {
    let body = RigidBody::dynamic()
        .with_angular_velocity(5.0)
        .with_fixed_rotation(true);

    assert!(body.has_fixed_rotation());
    assert_eq!(body.angular_velocity, 0.0);
    assert_eq!(body.inertia, 0.0);
    assert_eq!(body.inverse_inertia, 0.0);
}

#[test]
fn test_rigidbody_builder_chaining() {
    let body = RigidBody::dynamic()
        .with_velocity(Vec2::new(100.0, 50.0))
        .with_mass(2.0)
        .with_restitution(0.8)
        .with_friction(0.5)
        .with_gravity_scale(1.5);

    assert_eq!(body.linear_velocity, Vec2::new(100.0, 50.0));
    assert_eq!(body.mass, 2.0);
    assert_eq!(body.restitution, 0.8);
    assert_eq!(body.friction, 0.5);
    assert_eq!(body.gravity_scale, 1.5);
}

// =========================================================================
// Accessor Tests
// =========================================================================

#[test]
fn test_rigidbody_linear_speed() {
    let body = RigidBody::dynamic().with_velocity(Vec2::new(3.0, 4.0));
    assert_eq!(body.linear_speed(), 5.0);
}

#[test]
fn test_rigidbody_linear_speed_squared() {
    let body = RigidBody::dynamic().with_velocity(Vec2::new(3.0, 4.0));
    assert_eq!(body.linear_speed_squared(), 25.0);
}

#[test]
fn test_rigidbody_kinetic_energy() {
    let body = RigidBody::dynamic()
        .with_mass(2.0)
        .with_velocity(Vec2::new(3.0, 4.0));
    // KE = 0.5 * m * v² = 0.5 * 2.0 * 25.0 = 25.0
    assert_eq!(body.kinetic_energy(), 25.0);
}

// =========================================================================
// Mutator Tests
// =========================================================================

#[test]
fn test_rigidbody_set_velocity() {
    let mut body = RigidBody::dynamic();
    body.set_velocity(Vec2::new(100.0, 50.0));
    assert_eq!(body.linear_velocity, Vec2::new(100.0, 50.0));
    assert!(!body.is_sleeping()); // Should wake
}

#[test]
fn test_rigidbody_set_angular_velocity() {
    let mut body = RigidBody::dynamic();
    body.set_angular_velocity(3.14);
    assert_eq!(body.angular_velocity, 3.14);
    assert!(!body.is_sleeping()); // Should wake
}

#[test]
fn test_rigidbody_set_mass() {
    let mut body = RigidBody::dynamic();
    body.set_mass(2.0);
    assert_eq!(body.mass, 2.0);
    assert_eq!(body.inverse_mass, 0.5);
}

#[test]
#[should_panic(expected = "Cannot set mass on non-dynamic body")]
fn test_rigidbody_set_mass_kinematic() {
    let mut body = RigidBody::kinematic();
    body.set_mass(2.0);
}

#[test]
fn test_rigidbody_set_body_type() {
    let mut body = RigidBody::dynamic();
    body.set_body_type(RigidBodyType::Kinematic);

    assert!(body.is_kinematic());
    assert_eq!(body.inverse_mass, 0.0);
    assert_eq!(body.inverse_inertia, 0.0);
}

// =========================================================================
// Physics Operations Tests
// =========================================================================

#[test]
fn test_rigidbody_apply_impulse() {
    let mut body = RigidBody::dynamic().with_mass(2.0);

    body.apply_impulse(Vec2::new(10.0, 0.0));
    // Δv = impulse / mass = 10.0 / 2.0 = 5.0
    assert_eq!(body.linear_velocity, Vec2::new(5.0, 0.0));
}

#[test]
fn test_rigidbody_apply_impulse_kinematic() {
    let mut body = RigidBody::kinematic();
    let initial_velocity = body.linear_velocity;

    body.apply_impulse(Vec2::new(10.0, 0.0));
    // Should not affect kinematic body
    assert_eq!(body.linear_velocity, initial_velocity);
}

#[test]
fn test_rigidbody_apply_angular_impulse() {
    let mut body = RigidBody::dynamic();
    body.apply_angular_impulse(5.0);
    // Δω = impulse / inertia = 5.0 / 1.0 = 5.0
    assert_eq!(body.angular_velocity, 5.0);
}

#[test]
fn test_rigidbody_apply_angular_impulse_fixed_rotation() {
    let mut body = RigidBody::dynamic().with_fixed_rotation(true);

    body.apply_angular_impulse(5.0);
    // Should not affect body with fixed rotation
    assert_eq!(body.angular_velocity, 0.0);
}

#[test]
fn test_rigidbody_apply_damping() {
    let mut body = RigidBody::dynamic()
        .with_velocity(Vec2::new(100.0, 0.0))
        .with_angular_velocity(10.0)
        .with_linear_damping(0.1)
        .with_angular_damping(0.1);

    body.apply_damping(0.1); // 0.1 seconds

    // Velocity should decrease
    assert!(body.linear_velocity.x < 100.0);
    assert!(body.angular_velocity < 10.0);
}

// =========================================================================
// Sleep Management Tests
// =========================================================================

#[test]
fn test_rigidbody_sleep() {
    let mut body = RigidBody::dynamic()
        .with_velocity(Vec2::new(100.0, 50.0))
        .with_angular_velocity(5.0);

    body.sleep();

    assert!(body.is_sleeping());
    assert_eq!(body.linear_velocity, Vec2::zero());
    assert_eq!(body.angular_velocity, 0.0);
    assert_eq!(body.sleep_time(), 0.0);
}

#[test]
fn test_rigidbody_wake() {
    let mut body = RigidBody::dynamic();
    body.sleep();
    assert!(body.is_sleeping());

    body.wake();
    assert!(!body.is_sleeping());
    assert_eq!(body.sleep_time(), 0.0);
}

#[test]
fn test_rigidbody_update_sleep_time_below_threshold() {
    let mut body = RigidBody::dynamic().with_velocity(Vec2::new(1.0, 1.0));

    let should_sleep = body.update_sleep_time(0.1, 5.0, 0.1);
    assert!(should_sleep);
    assert!(body.sleep_time() > 0.0);
}

#[test]
fn test_rigidbody_update_sleep_time_above_threshold() {
    let mut body = RigidBody::dynamic().with_velocity(Vec2::new(100.0, 0.0));

    let should_sleep = body.update_sleep_time(0.1, 5.0, 0.1);
    assert!(!should_sleep);
    assert_eq!(body.sleep_time(), 0.0);
}

#[test]
fn test_rigidbody_update_sleep_time_cannot_sleep() {
    let mut body = RigidBody::dynamic()
        .with_can_sleep(false)
        .with_velocity(Vec2::new(1.0, 1.0));

    let should_sleep = body.update_sleep_time(0.1, 5.0, 0.1);
    assert!(!should_sleep);
    assert_eq!(body.sleep_time(), 0.0);
}

// =========================================================================
// Component and Display Tests
// =========================================================================

#[test]
fn test_rigidbody_is_component() {
    fn requires_component<T: Component>() {}
    requires_component::<RigidBody>();
}

#[test]
fn test_rigidbody_display() {
    let body = RigidBody::dynamic().with_velocity(Vec2::new(100.0, 50.0));

    let display = format!("{}", body);
    assert!(display.contains("RigidBody"));
    assert!(display.contains("Dynamic"));
    assert!(display.contains("vel"));
    assert!(display.contains("mass"));
}

#[test]
fn test_rigidbody_debug() {
    let body = RigidBody::dynamic();
    let debug = format!("{:?}", body);
    assert!(debug.contains("RigidBody"));
}

#[test]
fn test_rigidbody_clone() {
    let body1 = RigidBody::dynamic()
        .with_velocity(Vec2::new(100.0, 50.0))
        .with_mass(2.0);

    let body2 = body1;
    assert_eq!(body1.linear_velocity, body2.linear_velocity);
    assert_eq!(body1.mass, body2.mass);
}

// =========================================================================
// Thread Safety Tests
// =========================================================================

#[test]
fn test_rigidbody_is_send() {
    fn requires_send<T: Send>() {}
    requires_send::<RigidBody>();
}

#[test]
fn test_rigidbody_is_sync() {
    fn requires_sync<T: Sync>() {}
    requires_sync::<RigidBody>();
}
