use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::{BodyDesc3D, BodyHandle, ColliderDesc3D};
use crate::libs::providers::impls::rapier3d_physics::Rapier3DPhysicsProvider;

#[test]
fn test_body_gravity_scale_get_set() {
    let mut provider = Rapier3DPhysicsProvider::new();
    let body = provider
        .create_body(&BodyDesc3D {
            body_type: 1, // dynamic
            gravity_scale: 1.0,
            ..Default::default()
        })
        .unwrap();

    // Default gravity scale should be 1.0
    let scale = provider.body_gravity_scale(body).unwrap();
    assert!((scale - 1.0).abs() < f32::EPSILON, "default scale={scale}");

    // Set to 2.0 and verify
    provider.set_body_gravity_scale(body, 2.0).unwrap();
    let scale = provider.body_gravity_scale(body).unwrap();
    assert!((scale - 2.0).abs() < f32::EPSILON, "updated scale={scale}");

    // Invalid handle should error
    assert!(provider.body_gravity_scale(BodyHandle(9999)).is_err());
    assert!(provider
        .set_body_gravity_scale(BodyHandle(9999), 1.0)
        .is_err());
}

#[test]
fn test_collider_friction_get_set() {
    let mut provider = Rapier3DPhysicsProvider::new();
    let body = provider
        .create_body(&BodyDesc3D {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    let collider = provider
        .create_collider(
            body,
            &ColliderDesc3D {
                shape: 0,
                radius: 1.0,
                friction: 0.3,
                ..Default::default()
            },
        )
        .unwrap();

    // Read back initial friction
    let friction = provider.collider_friction(collider).unwrap();
    assert!(
        (friction - 0.3).abs() < f32::EPSILON,
        "initial friction={friction}"
    );

    // Set new friction and verify
    provider.set_collider_friction(collider, 0.8).unwrap();
    let friction = provider.collider_friction(collider).unwrap();
    assert!(
        (friction - 0.8).abs() < f32::EPSILON,
        "updated friction={friction}"
    );
}

#[test]
fn test_collider_restitution_get_set() {
    let mut provider = Rapier3DPhysicsProvider::new();
    let body = provider
        .create_body(&BodyDesc3D {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    let collider = provider
        .create_collider(
            body,
            &ColliderDesc3D {
                shape: 0,
                radius: 1.0,
                restitution: 0.0,
                ..Default::default()
            },
        )
        .unwrap();

    // Read back initial restitution
    let restitution = provider.collider_restitution(collider).unwrap();
    assert!(
        restitution.abs() < f32::EPSILON,
        "initial restitution={restitution}"
    );

    // Set new restitution and verify
    provider.set_collider_restitution(collider, 0.9).unwrap();
    let restitution = provider.collider_restitution(collider).unwrap();
    assert!(
        (restitution - 0.9).abs() < f32::EPSILON,
        "updated restitution={restitution}"
    );
}
