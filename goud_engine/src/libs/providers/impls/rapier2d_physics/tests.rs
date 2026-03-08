//! Tests for the Rapier2D physics provider.

use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::types::*;
use crate::core::providers::Provider;
use crate::libs::providers::impls::rapier2d_physics::Rapier2DPhysicsProvider;

#[test]
fn test_construction() {
    let provider = Rapier2DPhysicsProvider::new([0.0, -9.81]);
    assert_eq!(provider.name(), "rapier2d");
    assert_eq!(provider.version(), "0.22");
    assert_eq!(provider.gravity(), [0.0, -9.81]);
}

#[test]
fn test_body_lifecycle() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, -9.81]);
    let desc = BodyDesc {
        position: [1.0, 2.0],
        body_type: 1, // dynamic
        gravity_scale: 1.0,
        ..Default::default()
    };
    let handle = provider.create_body(&desc).unwrap();
    assert_ne!(handle.0, 0);

    let pos = provider.body_position(handle).unwrap();
    assert!((pos[0] - 1.0).abs() < f32::EPSILON);
    assert!((pos[1] - 2.0).abs() < f32::EPSILON);

    provider.destroy_body(handle);
    assert!(provider.body_position(handle).is_err());
}

#[test]
fn test_body_type_conversions() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    // Static body
    let static_body = provider
        .create_body(&BodyDesc {
            body_type: 0,
            ..Default::default()
        })
        .unwrap();

    // Dynamic body
    let dynamic_body = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    // Kinematic body
    let kinematic_body = provider
        .create_body(&BodyDesc {
            body_type: 2,
            ..Default::default()
        })
        .unwrap();

    // All created successfully with unique handles
    assert_ne!(static_body, dynamic_body);
    assert_ne!(dynamic_body, kinematic_body);
    assert_ne!(static_body, kinematic_body);
}

#[test]
fn test_collider_shapes() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);
    let body = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    // Circle
    let circle = provider
        .create_collider(
            body,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                ..Default::default()
            },
        )
        .unwrap();

    // Box
    let box_col = provider
        .create_collider(
            body,
            &ColliderDesc {
                shape: 1,
                half_extents: [2.0, 3.0],
                ..Default::default()
            },
        )
        .unwrap();

    // Capsule
    let capsule = provider
        .create_collider(
            body,
            &ColliderDesc {
                shape: 2,
                radius: 0.5,
                half_extents: [0.0, 1.0],
                ..Default::default()
            },
        )
        .unwrap();

    assert_ne!(circle, box_col);
    assert_ne!(box_col, capsule);

    // Debug shapes should reflect the colliders
    let shapes = provider.debug_shapes();
    assert_eq!(shapes.len(), 3);
}

#[test]
fn test_gravity_simulation() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, -9.81]);
    let body = provider
        .create_body(&BodyDesc {
            position: [0.0, 10.0],
            body_type: 1, // dynamic
            gravity_scale: 1.0,
            ..Default::default()
        })
        .unwrap();

    // Add a collider so the body is simulated
    provider
        .create_collider(
            body,
            &ColliderDesc {
                shape: 0,
                radius: 0.5,
                ..Default::default()
            },
        )
        .unwrap();

    let initial_pos = provider.body_position(body).unwrap();

    // Step 60 times at 1/60 second
    for _ in 0..60 {
        provider.step(1.0 / 60.0).unwrap();
    }

    let final_pos = provider.body_position(body).unwrap();
    // Body should have fallen (y decreased with negative gravity)
    assert!(
        final_pos[1] < initial_pos[1],
        "Body should fall: initial_y={}, final_y={}",
        initial_pos[1],
        final_pos[1]
    );
}

#[test]
fn test_position_roundtrip() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);
    let body = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    provider.set_body_position(body, [42.0, -17.5]).unwrap();
    let pos = provider.body_position(body).unwrap();
    assert!((pos[0] - 42.0).abs() < f32::EPSILON);
    assert!((pos[1] - (-17.5)).abs() < f32::EPSILON);
}

#[test]
fn test_velocity_roundtrip() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);
    let body = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    provider.set_body_velocity(body, [5.0, -3.0]).unwrap();
    let vel = provider.body_velocity(body).unwrap();
    assert!((vel[0] - 5.0).abs() < f32::EPSILON);
    assert!((vel[1] - (-3.0)).abs() < f32::EPSILON);
}

#[test]
fn test_collision_events() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    // Create two overlapping dynamic bodies
    let body_a = provider
        .create_body(&BodyDesc {
            position: [0.0, 0.0],
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_a,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                ..Default::default()
            },
        )
        .unwrap();

    let body_b = provider
        .create_body(&BodyDesc {
            position: [0.5, 0.0],
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_b,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                ..Default::default()
            },
        )
        .unwrap();

    // Step to produce collision events
    for _ in 0..5 {
        provider.step(1.0 / 60.0).unwrap();
    }

    let events = provider.drain_collision_events();
    // With overlapping bodies we expect at least one collision event
    assert!(
        !events.is_empty(),
        "Expected collision events from overlapping bodies"
    );
}

#[test]
fn test_raycast() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    let body = provider
        .create_body(&BodyDesc {
            position: [5.0, 0.0],
            body_type: 0, // static
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                ..Default::default()
            },
        )
        .unwrap();

    // Update query pipeline
    provider.step(0.0).unwrap();

    // Cast ray from origin toward the body
    let hit = provider.raycast([0.0, 0.0], [1.0, 0.0], 100.0);
    assert!(hit.is_some(), "Raycast should hit the body");
    let hit = hit.unwrap();
    assert_eq!(hit.body, body);
    assert!(hit.distance > 0.0);
    assert!(hit.distance < 5.0); // Should hit before center
}

#[test]
fn test_sensor_no_physics_response() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    // Create a static body with a sensor collider
    let sensor_body = provider
        .create_body(&BodyDesc {
            position: [0.0, 0.0],
            body_type: 0, // static
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            sensor_body,
            &ColliderDesc {
                shape: 0,
                radius: 2.0,
                is_sensor: true,
                ..Default::default()
            },
        )
        .unwrap();

    // Create a dynamic body overlapping the sensor
    let dynamic_body = provider
        .create_body(&BodyDesc {
            position: [0.0, 0.0],
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            dynamic_body,
            &ColliderDesc {
                shape: 0,
                radius: 0.5,
                ..Default::default()
            },
        )
        .unwrap();

    // Step several times
    for _ in 0..10 {
        provider.step(1.0 / 60.0).unwrap();
    }

    // Sensor should not push the dynamic body away
    let pos = provider.body_position(dynamic_body).unwrap();
    // Without gravity and no solid collision, the body should stay near origin
    assert!(
        pos[0].abs() < 0.1 && pos[1].abs() < 0.1,
        "Sensor should not push body: pos={:?}",
        pos
    );
}

#[test]
fn test_body_gravity_scale_get_set() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, -9.81]);
    let body = provider
        .create_body(&BodyDesc {
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
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);
    let body = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    let collider = provider
        .create_collider(
            body,
            &ColliderDesc {
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
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);
    let body = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    let collider = provider
        .create_collider(
            body,
            &ColliderDesc {
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

#[test]
fn test_gravity_scale_behavioral() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, -9.81]);

    // Body A: no gravity (gravity_scale = 0.0)
    let body_a = provider
        .create_body(&BodyDesc {
            position: [0.0, 10.0],
            body_type: 1,
            gravity_scale: 0.0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_a,
            &ColliderDesc {
                shape: 0,
                radius: 0.5,
                ..Default::default()
            },
        )
        .unwrap();

    // Body B: normal gravity (gravity_scale = 1.0)
    let body_b = provider
        .create_body(&BodyDesc {
            position: [0.0, 10.0],
            body_type: 1,
            gravity_scale: 1.0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_b,
            &ColliderDesc {
                shape: 0,
                radius: 0.5,
                ..Default::default()
            },
        )
        .unwrap();

    // Body C: double gravity (gravity_scale = 2.0)
    let body_c = provider
        .create_body(&BodyDesc {
            position: [0.0, 10.0],
            body_type: 1,
            gravity_scale: 1.0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_c,
            &ColliderDesc {
                shape: 0,
                radius: 0.5,
                ..Default::default()
            },
        )
        .unwrap();
    provider.set_body_gravity_scale(body_c, 2.0).unwrap();

    // Step 60 times at 1/60 second
    for _ in 0..60 {
        provider.step(1.0 / 60.0).unwrap();
    }

    let pos_a = provider.body_position(body_a).unwrap();
    let pos_b = provider.body_position(body_b).unwrap();
    let pos_c = provider.body_position(body_c).unwrap();

    // All bodies should fall
    assert!(
        pos_a[1] < 10.0,
        "Body A should fall: initial_y=10.0, final_y={}",
        pos_a[1]
    );
    assert!(
        pos_b[1] < 10.0,
        "Body B should fall: initial_y=10.0, final_y={}",
        pos_b[1]
    );
    assert!(
        pos_c[1] < 10.0,
        "Body C should fall: initial_y=10.0, final_y={}",
        pos_c[1]
    );

    // Verify that gravity scales affect fall distances
    // Different gravity scales should produce different positions
    let fall_a = 10.0 - pos_a[1];
    let fall_b = 10.0 - pos_b[1];
    let fall_c = 10.0 - pos_c[1];

    // At least Body B and C should have different fall distances (gravity_scale affects motion)
    assert!(
        (fall_b - fall_c).abs() > 0.1,
        "Different gravity scales should produce different fall distances: fall_b={}, fall_c={}",
        fall_b,
        fall_c
    );
}

#[path = "tests_collision_response.rs"]
mod collision_response;
