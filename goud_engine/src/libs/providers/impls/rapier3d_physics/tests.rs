//! Unit tests for the Rapier3D physics provider.

use super::Rapier3DPhysicsProvider;
use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::{BodyDesc3D, ColliderDesc3D};
use crate::core::providers::{Provider, ProviderLifecycle};

#[test]
fn test_construction() {
    let provider = Rapier3DPhysicsProvider::new();
    assert_eq!(provider.name(), "rapier3d");
    assert_eq!(provider.version(), "0.22");

    let caps = provider.physics_capabilities();
    assert!(caps.supports_continuous_collision);
    assert!(caps.supports_joints);
    assert_eq!(caps.max_bodies, u32::MAX);
}

#[test]
fn test_body_lifecycle() {
    let mut provider = Rapier3DPhysicsProvider::new();
    provider.init().unwrap();

    let body = provider
        .create_body(&BodyDesc3D {
            position: [1.0, 2.0, 3.0],
            body_type: 1, // dynamic
            ..Default::default()
        })
        .unwrap();

    let pos = provider.body_position(body).unwrap();
    assert!((pos[0] - 1.0).abs() < 1e-5);
    assert!((pos[1] - 2.0).abs() < 1e-5);
    assert!((pos[2] - 3.0).abs() < 1e-5);

    provider.destroy_body(body);

    // After destruction, the handle should be invalid.
    assert!(provider.body_position(body).is_err());
}

#[test]
fn test_body_type_conversions() {
    let mut provider = Rapier3DPhysicsProvider::new();

    // Static body (type 0)
    let static_body = provider
        .create_body(&BodyDesc3D {
            body_type: 0,
            ..Default::default()
        })
        .unwrap();

    // Dynamic body (type 1)
    let dynamic_body = provider
        .create_body(&BodyDesc3D {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    // Kinematic body (type 2)
    let kinematic_body = provider
        .create_body(&BodyDesc3D {
            body_type: 2,
            ..Default::default()
        })
        .unwrap();

    // All should be valid
    assert!(provider.body_position(static_body).is_ok());
    assert!(provider.body_position(dynamic_body).is_ok());
    assert!(provider.body_position(kinematic_body).is_ok());
}

#[test]
fn test_collider_shapes() {
    let mut provider = Rapier3DPhysicsProvider::new();
    let body = provider
        .create_body(&BodyDesc3D {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    // Sphere collider (shape 0)
    let sphere = provider
        .create_collider(
            body,
            &ColliderDesc3D {
                shape: 0,
                radius: 0.5,
                ..Default::default()
            },
        )
        .unwrap();

    // Box collider (shape 1)
    let _box_col = provider
        .create_collider(
            body,
            &ColliderDesc3D {
                shape: 1,
                half_extents: [1.0, 1.0, 1.0],
                ..Default::default()
            },
        )
        .unwrap();

    // Capsule collider (shape 2)
    let _capsule = provider
        .create_collider(
            body,
            &ColliderDesc3D {
                shape: 2,
                radius: 0.3,
                half_height: 0.5,
                ..Default::default()
            },
        )
        .unwrap();

    // Debug shapes should contain all three colliders
    let shapes = provider.debug_shapes();
    assert_eq!(shapes.len(), 3);

    // Cleanup
    provider.destroy_collider(sphere);
}

#[test]
fn test_gravity_simulation() {
    let mut provider = Rapier3DPhysicsProvider::new();
    provider.set_gravity([0.0, -9.81, 0.0]);
    assert_eq!(provider.gravity(), [0.0, -9.81, 0.0]);

    let body = provider
        .create_body(&BodyDesc3D {
            position: [0.0, 10.0, 0.0],
            body_type: 1, // dynamic
            ..Default::default()
        })
        .unwrap();

    provider
        .create_collider(
            body,
            &ColliderDesc3D {
                shape: 0,
                radius: 0.5,
                ..Default::default()
            },
        )
        .unwrap();

    // Step simulation several times
    for _ in 0..60 {
        provider.step(1.0 / 60.0).unwrap();
    }

    let pos = provider.body_position(body).unwrap();
    // Body should have fallen below initial Y=10
    assert!(
        pos[1] < 10.0,
        "Body should fall under gravity, y={}",
        pos[1]
    );
}

#[test]
fn test_position_roundtrip() {
    let mut provider = Rapier3DPhysicsProvider::new();
    let body = provider
        .create_body(&BodyDesc3D {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    provider.set_body_position(body, [5.0, 10.0, 15.0]).unwrap();
    let pos = provider.body_position(body).unwrap();
    assert!((pos[0] - 5.0).abs() < 1e-5);
    assert!((pos[1] - 10.0).abs() < 1e-5);
    assert!((pos[2] - 15.0).abs() < 1e-5);
}

#[test]
fn test_rotation_roundtrip() {
    let mut provider = Rapier3DPhysicsProvider::new();
    let body = provider
        .create_body(&BodyDesc3D {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    // 90-degree rotation around Y axis: [0, sin(45), 0, cos(45)]
    let input_rot = [0.0, 0.707_107, 0.0, 0.707_107];
    provider.set_body_rotation(body, input_rot).unwrap();
    let rot = provider.body_rotation(body).unwrap();

    assert!(
        (rot[0] - input_rot[0]).abs() < 1e-3,
        "x: {} vs {}",
        rot[0],
        input_rot[0]
    );
    assert!(
        (rot[1] - input_rot[1]).abs() < 1e-3,
        "y: {} vs {}",
        rot[1],
        input_rot[1]
    );
    assert!(
        (rot[2] - input_rot[2]).abs() < 1e-3,
        "z: {} vs {}",
        rot[2],
        input_rot[2]
    );
    assert!(
        (rot[3] - input_rot[3]).abs() < 1e-3,
        "w: {} vs {}",
        rot[3],
        input_rot[3]
    );
}

#[test]
fn test_velocity_roundtrip() {
    let mut provider = Rapier3DPhysicsProvider::new();
    let body = provider
        .create_body(&BodyDesc3D {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    provider.set_body_velocity(body, [3.0, -1.0, 2.5]).unwrap();
    let vel = provider.body_velocity(body).unwrap();
    assert!((vel[0] - 3.0).abs() < 1e-5);
    assert!((vel[1] - (-1.0)).abs() < 1e-5);
    assert!((vel[2] - 2.5).abs() < 1e-5);
}

#[test]
fn test_collision_events() {
    let mut provider = Rapier3DPhysicsProvider::new();
    provider.set_gravity([0.0, 0.0, 0.0]); // no gravity

    // Create two overlapping dynamic bodies at the same position
    let body_a = provider
        .create_body(&BodyDesc3D {
            position: [0.0, 0.0, 0.0],
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_a,
            &ColliderDesc3D {
                shape: 0,
                radius: 1.0,
                ..Default::default()
            },
        )
        .unwrap();

    let body_b = provider
        .create_body(&BodyDesc3D {
            position: [0.5, 0.0, 0.0],
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_b,
            &ColliderDesc3D {
                shape: 0,
                radius: 1.0,
                ..Default::default()
            },
        )
        .unwrap();

    // Step to generate collision events
    for _ in 0..5 {
        provider.step(1.0 / 60.0).unwrap();
    }

    let events = provider.drain_collision_events();
    // With overlapping bodies, we should get at least one collision event
    assert!(
        !events.is_empty(),
        "Expected collision events from overlapping bodies"
    );

    // Second drain should be empty
    let events2 = provider.drain_collision_events();
    assert!(events2.is_empty(), "Events should be drained");
}

#[test]
fn test_raycast() {
    let mut provider = Rapier3DPhysicsProvider::new();
    provider.set_gravity([0.0, 0.0, 0.0]);

    // Create a body with a box collider at x=5
    let body = provider
        .create_body(&BodyDesc3D {
            position: [5.0, 0.0, 0.0],
            body_type: 0, // static
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body,
            &ColliderDesc3D {
                shape: 1,
                half_extents: [1.0, 1.0, 1.0],
                ..Default::default()
            },
        )
        .unwrap();

    // Step once to update query pipeline
    provider.step(0.0).unwrap();

    // Cast a ray from origin along +X axis
    let hit = provider.raycast([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 100.0);
    assert!(hit.is_some(), "Raycast should hit the box at x=5");

    let hit = hit.unwrap();
    assert_eq!(hit.body, body);
    // Hit point should be around x=4 (box from x=4..6)
    assert!(
        (hit.point[0] - 4.0).abs() < 0.1,
        "Hit x={}, expected ~4.0",
        hit.point[0]
    );
    assert!(
        hit.distance > 3.5 && hit.distance < 4.5,
        "Distance={}, expected ~4.0",
        hit.distance
    );

    // Cast ray in the wrong direction -- should miss
    let miss = provider.raycast([0.0, 0.0, 0.0], [-1.0, 0.0, 0.0], 100.0);
    assert!(miss.is_none(), "Raycast in -X should miss the box at +X");
}

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
    assert!(provider
        .body_gravity_scale(BodyHandle(9999))
        .is_err());
    assert!(provider
        .set_body_gravity_scale(BodyHandle(9999), 1.0)
        .is_err());
}

#[test]
fn test_collider_friction_get_set() {
    use crate::core::providers::types::ColliderDesc3D;

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
    use crate::core::providers::types::ColliderDesc3D;

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
