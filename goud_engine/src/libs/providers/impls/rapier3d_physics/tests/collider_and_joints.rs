use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::{
    BodyDesc3D, ColliderDesc3D, JointDesc3D, JointKind, JointLimits, JointMotor,
};
use crate::libs::providers::impls::rapier3d_physics::Rapier3DPhysicsProvider;

#[test]
fn test_create_and_destroy_prismatic_joint() {
    let mut provider = Rapier3DPhysicsProvider::new();
    let body_a = provider
        .create_body(&BodyDesc3D {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    let body_b = provider
        .create_body(&BodyDesc3D {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    let joint = provider
        .create_joint(&JointDesc3D {
            body_a: Some(body_a),
            body_b: Some(body_b),
            kind: JointKind::Prismatic,
            axis: [0.0, 1.0, 0.0],
            limits: Some(JointLimits {
                min: -1.0,
                max: 2.0,
            }),
            motor: Some(JointMotor {
                target_velocity: 2.0,
                max_force: 5.0,
            }),
            ..Default::default()
        })
        .unwrap();

    assert!(provider.joint_map.contains_key(&joint.0));

    provider.destroy_joint(joint);
    assert!(!provider.joint_map.contains_key(&joint.0));
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
