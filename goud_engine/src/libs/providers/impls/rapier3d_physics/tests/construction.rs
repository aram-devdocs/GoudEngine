use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::BodyDesc3D;
use crate::core::providers::{Provider, ProviderLifecycle};
use crate::libs::providers::impls::rapier3d_physics::Rapier3DPhysicsProvider;

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
fn test_body_desc_3d_defaults_ccd_disabled() {
    let desc = BodyDesc3D::default();
    assert!(!desc.ccd_enabled);
}

#[test]
fn test_create_body_threads_ccd_flag() {
    let mut provider = Rapier3DPhysicsProvider::new();

    let disabled = provider
        .create_body(&BodyDesc3D {
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    let disabled_rb = provider
        .rigid_body_set
        .get(provider.resolve_body(disabled).unwrap())
        .unwrap();
    assert!(!disabled_rb.is_ccd_enabled());

    let enabled = provider
        .create_body(&BodyDesc3D {
            body_type: 1,
            ccd_enabled: true,
            ..Default::default()
        })
        .unwrap();
    let enabled_rb = provider
        .rigid_body_set
        .get(provider.resolve_body(enabled).unwrap())
        .unwrap();
    assert!(enabled_rb.is_ccd_enabled());
}
