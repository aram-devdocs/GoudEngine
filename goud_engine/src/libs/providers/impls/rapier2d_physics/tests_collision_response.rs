//! Collision response tests for the Rapier2D physics provider.

use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::types::*;
use crate::libs::providers::impls::rapier2d_physics::Rapier2DPhysicsProvider;

#[test]
fn test_collision_response_restitution_range() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, -9.81]);

    // Static floor at y=0
    let floor = provider
        .create_body(&BodyDesc {
            position: [0.0, 0.0],
            body_type: 0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            floor,
            &ColliderDesc {
                shape: 1,
                half_extents: [50.0, 0.5],
                ..Default::default()
            },
        )
        .unwrap();

    let restitutions = [0.0_f32, 0.5, 1.0];
    let mut ball_handles = Vec::new();

    for (i, &rest) in restitutions.iter().enumerate() {
        let ball = provider
            .create_body(&BodyDesc {
                position: [i as f32 * 5.0, 5.0],
                body_type: 1,
                gravity_scale: 1.0,
                ..Default::default()
            })
            .unwrap();
        provider
            .create_collider(
                ball,
                &ColliderDesc {
                    shape: 0,
                    radius: 0.5,
                    restitution: rest,
                    ..Default::default()
                },
            )
            .unwrap();
        ball_handles.push(ball);
    }

    for _ in 0..120 {
        provider.step(1.0 / 60.0).unwrap();
    }

    let pos_0 = provider.body_position(ball_handles[0]).unwrap();
    let pos_1 = provider.body_position(ball_handles[2]).unwrap();

    assert!(
        pos_1[1] > pos_0[1],
        "Restitution 1.0 ball (y={}) should be higher than 0.0 ball (y={})",
        pos_1[1],
        pos_0[1]
    );
}

#[test]
fn test_friction_effect() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, -9.81]);

    let floor = provider
        .create_body(&BodyDesc {
            position: [0.0, 0.0],
            body_type: 0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            floor,
            &ColliderDesc {
                shape: 1,
                half_extents: [100.0, 0.5],
                friction: 0.5,
                ..Default::default()
            },
        )
        .unwrap();

    let low_friction_body = provider
        .create_body(&BodyDesc {
            position: [-10.0, 2.0],
            body_type: 1,
            gravity_scale: 1.0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            low_friction_body,
            &ColliderDesc {
                shape: 1,
                half_extents: [0.5, 0.5],
                friction: 0.0,
                ..Default::default()
            },
        )
        .unwrap();

    let high_friction_body = provider
        .create_body(&BodyDesc {
            position: [10.0, 2.0],
            body_type: 1,
            gravity_scale: 1.0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            high_friction_body,
            &ColliderDesc {
                shape: 1,
                half_extents: [0.5, 0.5],
                friction: 1.0,
                ..Default::default()
            },
        )
        .unwrap();

    for _ in 0..60 {
        provider.step(1.0 / 60.0).unwrap();
    }

    provider
        .set_body_velocity(low_friction_body, [10.0, 0.0])
        .unwrap();
    provider
        .set_body_velocity(high_friction_body, [10.0, 0.0])
        .unwrap();

    for _ in 0..60 {
        provider.step(1.0 / 60.0).unwrap();
    }

    let vel_low = provider.body_velocity(low_friction_body).unwrap();
    let vel_high = provider.body_velocity(high_friction_body).unwrap();

    assert!(
        vel_low[0].abs() > vel_high[0].abs(),
        "Low friction x-vel ({}) should exceed high friction x-vel ({})",
        vel_low[0].abs(),
        vel_high[0].abs()
    );
}

#[test]
fn test_dynamic_dynamic_collision_response() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    // Body A: moving right
    let body_a = provider
        .create_body(&BodyDesc {
            position: [-5.0, 0.0],
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
                radius: 1.0,
                restitution: 1.0,
                ..Default::default()
            },
        )
        .unwrap();
    provider.set_body_velocity(body_a, [10.0, 0.0]).unwrap();

    // Body B: moving left
    let body_b = provider
        .create_body(&BodyDesc {
            position: [5.0, 0.0],
            body_type: 1,
            gravity_scale: 0.0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_b,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                restitution: 1.0,
                ..Default::default()
            },
        )
        .unwrap();
    provider.set_body_velocity(body_b, [-10.0, 0.0]).unwrap();

    // Step 120 times at 1/60 second
    for _ in 0..120 {
        provider.step(1.0 / 60.0).unwrap();
    }

    let pos_a = provider.body_position(body_a).unwrap();
    let pos_b = provider.body_position(body_b).unwrap();

    // After collision, Body A should bounce back to the left
    assert!(
        pos_a[0] < 0.0,
        "Body A should bounce back left: x={}",
        pos_a[0]
    );

    // After collision, Body B should bounce back to the right
    assert!(
        pos_b[0] > 0.0,
        "Body B should bounce back right: x={}",
        pos_b[0]
    );
}
