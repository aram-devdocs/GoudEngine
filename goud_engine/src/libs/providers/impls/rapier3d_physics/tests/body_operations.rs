use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::BodyDesc3D;
use crate::libs::providers::impls::rapier3d_physics::Rapier3DPhysicsProvider;

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
