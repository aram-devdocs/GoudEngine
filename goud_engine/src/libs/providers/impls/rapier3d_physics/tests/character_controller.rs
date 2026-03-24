use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::{
    BodyDesc3D, CharacterControllerDesc3D, CharacterControllerHandle, ColliderDesc3D,
};
use crate::core::providers::ProviderLifecycle;
use crate::libs::providers::impls::rapier3d_physics::Rapier3DPhysicsProvider;

fn create_provider() -> Rapier3DPhysicsProvider {
    let mut p = Rapier3DPhysicsProvider::new();
    p.init().unwrap();
    p
}

#[test]
fn test_create_and_destroy_character_controller() {
    let mut p = create_provider();
    let handle = p
        .create_character_controller(&CharacterControllerDesc3D {
            radius: 0.3,
            half_height: 0.5,
            position: [0.0, 1.0, 0.0],
            ..Default::default()
        })
        .unwrap();

    let pos = p.character_position(handle).unwrap();
    assert!((pos[0] - 0.0).abs() < 0.01);
    assert!((pos[1] - 1.0).abs() < 0.01);
    assert!((pos[2] - 0.0).abs() < 0.01);

    p.destroy_character_controller(handle);

    // After destroy, queries should fail.
    assert!(p.character_position(handle).is_err());
}

#[test]
fn test_character_controller_grounded_detection() {
    let mut p = create_provider();

    // Create a static floor at y=0.
    let floor_body = p
        .create_body(&BodyDesc3D {
            position: [0.0, -0.5, 0.0],
            body_type: 0, // static
            ..Default::default()
        })
        .unwrap();
    p.create_collider(
        floor_body,
        &ColliderDesc3D {
            shape: 1, // box
            half_extents: [50.0, 0.5, 50.0],
            ..Default::default()
        },
    )
    .unwrap();

    // Step once so the query pipeline is up to date.
    p.step(1.0 / 60.0).unwrap();

    // Create a character controller just above the floor.
    let handle = p
        .create_character_controller(&CharacterControllerDesc3D {
            radius: 0.3,
            half_height: 0.5,
            position: [0.0, 1.0, 0.0],
            ..Default::default()
        })
        .unwrap();

    // Move downward to land on the floor.
    let result = p
        .move_character(handle, [0.0, -2.0, 0.0], 1.0 / 60.0)
        .unwrap();
    assert!(
        result.grounded,
        "character should be grounded after landing"
    );

    // Position should be above or at the floor, not below it.
    assert!(
        result.position[1] >= -0.1,
        "character y={} should be above the floor",
        result.position[1]
    );
}

#[test]
fn test_character_controller_move_horizontal() {
    let mut p = create_provider();

    // Create a static floor.
    let floor_body = p
        .create_body(&BodyDesc3D {
            position: [0.0, -0.5, 0.0],
            body_type: 0,
            ..Default::default()
        })
        .unwrap();
    p.create_collider(
        floor_body,
        &ColliderDesc3D {
            shape: 1,
            half_extents: [50.0, 0.5, 50.0],
            ..Default::default()
        },
    )
    .unwrap();

    p.step(1.0 / 60.0).unwrap();

    let handle = p
        .create_character_controller(&CharacterControllerDesc3D {
            radius: 0.3,
            half_height: 0.5,
            position: [0.0, 1.0, 0.0],
            ..Default::default()
        })
        .unwrap();

    // Move horizontally.
    let result = p
        .move_character(handle, [5.0, 0.0, 0.0], 1.0 / 60.0)
        .unwrap();
    assert!(
        result.position[0] > 0.0,
        "character should have moved in +X direction"
    );
}

#[test]
fn test_character_controller_invalid_handle() {
    let p = create_provider();
    let bad_handle = CharacterControllerHandle(9999);
    assert!(p.character_position(bad_handle).is_err());
    assert!(p.is_character_grounded(bad_handle).is_err());
}

#[test]
fn test_character_controller_gravity_when_airborne() {
    let mut p = create_provider();

    // No floor -- character is in the air.
    let handle = p
        .create_character_controller(&CharacterControllerDesc3D {
            radius: 0.3,
            half_height: 0.5,
            position: [0.0, 10.0, 0.0],
            ..Default::default()
        })
        .unwrap();

    // Not grounded initially (we check after a move with no displacement).
    let result = p
        .move_character(handle, [0.0, 0.0, 0.0], 1.0 / 60.0)
        .unwrap();
    // Should have moved downward due to gravity.
    assert!(
        result.position[1] < 10.0,
        "character at y={} should have fallen from y=10",
        result.position[1]
    );
}
