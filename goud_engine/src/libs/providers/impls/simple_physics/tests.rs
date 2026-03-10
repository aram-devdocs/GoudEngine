use super::*;

fn dynamic_body(position: [f32; 2]) -> BodyDesc {
    BodyDesc {
        position,
        body_type: 1,
        ..Default::default()
    }
}

fn box_collider(half_extents: [f32; 2]) -> ColliderDesc {
    ColliderDesc {
        shape: 1,
        half_extents,
        ..Default::default()
    }
}

#[test]
fn step_applies_gravity_to_dynamic_bodies() {
    let mut provider = SimplePhysicsProvider::new([0.0, -10.0]);
    let body = provider.create_body(&dynamic_body([0.0, 5.0])).unwrap();

    provider.step(0.5).unwrap();

    let position = provider.body_position(body).unwrap();
    assert!(position[1] < 5.0);
}

#[test]
fn overlapping_dynamic_and_static_boxes_are_separated() {
    let mut provider = SimplePhysicsProvider::default();
    let dynamic = provider.create_body(&dynamic_body([0.0, 0.0])).unwrap();
    let static_body = provider
        .create_body(&BodyDesc {
            position: [0.5, 0.0],
            body_type: 0,
            ..Default::default()
        })
        .unwrap();

    provider
        .create_collider(dynamic, &box_collider([0.5, 0.5]))
        .unwrap();
    provider
        .create_collider(static_body, &box_collider([0.5, 0.5]))
        .unwrap();

    provider.step(0.016).unwrap();

    let dynamic_pos = provider.body_position(dynamic).unwrap();
    assert!(dynamic_pos[0] <= 0.0);
    assert!(!provider.contact_pairs().is_empty());
}

#[test]
fn raycast_hits_nearest_box() {
    let mut provider = SimplePhysicsProvider::default();
    let body = provider.create_body(&dynamic_body([2.0, 0.0])).unwrap();
    let collider = provider
        .create_collider(body, &box_collider([0.5, 0.5]))
        .unwrap();

    let hit = provider
        .raycast([0.0, 0.0], [1.0, 0.0], 10.0)
        .expect("expected hit");

    assert_eq!(hit.body, body);
    assert_eq!(hit.collider, collider);
    assert!(hit.distance >= 1.0);
}

#[test]
fn overlap_circle_respects_geometry() {
    let mut provider = SimplePhysicsProvider::default();
    let body = provider.create_body(&dynamic_body([1.0, 1.0])).unwrap();
    provider
        .create_collider(body, &box_collider([0.5, 0.5]))
        .unwrap();

    let overlaps = provider.overlap_circle([1.0, 1.0], 1.0);
    assert_eq!(overlaps, vec![body]);
}

#[test]
fn layer_mask_filters_raycast_candidates() {
    let mut provider = SimplePhysicsProvider::default();
    let body = provider.create_body(&dynamic_body([2.0, 0.0])).unwrap();
    provider
        .create_collider(
            body,
            &ColliderDesc {
                shape: 1,
                half_extents: [0.5, 0.5],
                layer: 0b0010,
                ..Default::default()
            },
        )
        .unwrap();

    assert!(provider
        .raycast_with_mask([0.0, 0.0], [1.0, 0.0], 10.0, 0b0001)
        .is_none());
    assert!(provider
        .raycast_with_mask([0.0, 0.0], [1.0, 0.0], 10.0, 0b0010)
        .is_some());
}

#[test]
fn joints_are_rejected_explicitly() {
    let mut provider = SimplePhysicsProvider::default();

    let err = provider.create_joint(&JointDesc::default()).unwrap_err();

    assert!(matches!(err, GoudError::ProviderError { .. }));
}

#[test]
fn debug_shapes_use_static_dynamic_and_sensor_colors() {
    let mut provider = SimplePhysicsProvider::default();
    let dynamic = provider.create_body(&dynamic_body([0.0, 0.0])).unwrap();
    let static_body = provider
        .create_body(&BodyDesc {
            position: [2.0, 0.0],
            body_type: 0,
            ..Default::default()
        })
        .unwrap();
    let sensor = provider.create_body(&dynamic_body([4.0, 0.0])).unwrap();

    provider
        .create_collider(dynamic, &box_collider([0.5, 0.5]))
        .unwrap();
    provider
        .create_collider(static_body, &box_collider([0.5, 0.5]))
        .unwrap();
    provider
        .create_collider(
            sensor,
            &ColliderDesc {
                shape: 1,
                half_extents: [0.5, 0.5],
                is_sensor: true,
                ..Default::default()
            },
        )
        .unwrap();

    let shapes = provider.debug_shapes();

    assert!(shapes
        .iter()
        .any(|shape| shape.color == [0.0, 0.0, 1.0, 0.5]));
    assert!(shapes
        .iter()
        .any(|shape| shape.color == [0.0, 1.0, 0.0, 0.5]));
    assert!(shapes
        .iter()
        .any(|shape| shape.color == [1.0, 1.0, 0.0, 0.5]));
}
