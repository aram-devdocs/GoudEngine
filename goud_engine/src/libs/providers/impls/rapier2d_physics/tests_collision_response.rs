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
