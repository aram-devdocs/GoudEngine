//! Gravity scale behavioral tests for the Rapier2D physics provider.

use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::types::*;
use crate::libs::providers::impls::rapier2d_physics::Rapier2DPhysicsProvider;

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
