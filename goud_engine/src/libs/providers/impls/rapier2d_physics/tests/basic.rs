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
fn test_body_desc_defaults_ccd_disabled() {
    let desc = BodyDesc::default();
    assert!(!desc.ccd_enabled);
    assert_eq!(desc.gravity_scale, 1.0);
}

#[test]
fn test_create_body_threads_ccd_flag() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    let disabled = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    let disabled_rb = provider
        .rigid_body_set
        .get(provider.get_rapier_body(disabled).unwrap())
        .unwrap();
    assert!(!disabled_rb.is_ccd_enabled());

    let enabled = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ccd_enabled: true,
            ..Default::default()
        })
        .unwrap();
    let enabled_rb = provider
        .rigid_body_set
        .get(provider.get_rapier_body(enabled).unwrap())
        .unwrap();
    assert!(enabled_rb.is_ccd_enabled());
}

#[test]
fn test_create_and_destroy_revolute_joint() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);
    let body_a = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    let body_b = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    let joint = provider
        .create_joint(&JointDesc {
            body_a: Some(body_a),
            body_b: Some(body_b),
            kind: JointKind::Revolute,
            limits: Some(JointLimits {
                min: -0.5,
                max: 0.5,
            }),
            motor: Some(JointMotor {
                target_velocity: 1.5,
                max_force: 8.0,
            }),
            ..Default::default()
        })
        .unwrap();

    assert!(provider.joint_handles.contains_key(&joint.0));

    provider.destroy_joint(joint);
    assert!(!provider.joint_handles.contains_key(&joint.0));
}

#[test]
fn test_create_prismatic_joint_rejects_zero_axis() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);
    let body_a = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    let body_b = provider
        .create_body(&BodyDesc {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();

    let err = provider
        .create_joint(&JointDesc {
            body_a: Some(body_a),
            body_b: Some(body_b),
            kind: JointKind::Prismatic,
            axis: [0.0, 0.0],
            ..Default::default()
        })
        .unwrap_err();

    assert!(matches!(
        err,
        crate::core::error::GoudError::ProviderError { .. }
    ));
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
