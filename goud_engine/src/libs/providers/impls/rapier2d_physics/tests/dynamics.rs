use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::types::*;
use crate::libs::providers::impls::rapier2d_physics::Rapier2DPhysicsProvider;

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
