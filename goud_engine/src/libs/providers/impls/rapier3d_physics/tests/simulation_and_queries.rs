use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::{BodyDesc3D, ColliderDesc3D};
use crate::libs::providers::impls::rapier3d_physics::Rapier3DPhysicsProvider;

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
