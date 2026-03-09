//! Collision-event and query behavior tests for the Rapier2D provider.

use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::types::{
    BodyDesc, BodyHandle, ColliderDesc, CollisionEvent, CollisionEventKind,
};
use crate::libs::providers::impls::rapier2d_physics::Rapier2DPhysicsProvider;

fn pair_matches(event: &CollisionEvent, a: BodyHandle, b: BodyHandle) -> bool {
    (event.body_a == a && event.body_b == b) || (event.body_a == b && event.body_b == a)
}

fn pair_kind_count(
    events: &[CollisionEvent],
    a: BodyHandle,
    b: BodyHandle,
    kind: CollisionEventKind,
) -> usize {
    events
        .iter()
        .filter(|event| pair_matches(event, a, b) && event.kind == kind)
        .count()
}

#[test]
fn test_collision_events_have_enter_stay_exit_without_duplicates() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    let body_a = provider
        .create_body(&BodyDesc {
            position: [0.0, 0.0],
            body_type: 0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_a,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                ..Default::default()
            },
        )
        .unwrap();

    let body_b = provider
        .create_body(&BodyDesc {
            position: [0.5, 0.0],
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_b,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                ..Default::default()
            },
        )
        .unwrap();

    provider.step(1.0 / 60.0).unwrap();
    let first_step_events = provider.drain_collision_events();
    assert_eq!(
        pair_kind_count(
            &first_step_events,
            body_a,
            body_b,
            CollisionEventKind::Enter
        ),
        1,
        "Expected exactly one Enter event on first overlap"
    );
    assert_eq!(
        pair_kind_count(&first_step_events, body_a, body_b, CollisionEventKind::Exit),
        0
    );
    assert_eq!(
        pair_kind_count(&first_step_events, body_a, body_b, CollisionEventKind::Stay),
        0,
        "Stay should start on subsequent overlapping steps"
    );

    provider.step(1.0 / 60.0).unwrap();
    let second_step_events = provider.drain_collision_events();
    assert_eq!(
        pair_kind_count(
            &second_step_events,
            body_a,
            body_b,
            CollisionEventKind::Enter
        ),
        0
    );
    assert_eq!(
        pair_kind_count(
            &second_step_events,
            body_a,
            body_b,
            CollisionEventKind::Stay
        ),
        1,
        "Expected exactly one Stay event while overlap persists"
    );
    assert_eq!(
        pair_kind_count(
            &second_step_events,
            body_a,
            body_b,
            CollisionEventKind::Exit
        ),
        0
    );

    provider.set_body_position(body_b, [10.0, 0.0]).unwrap();
    provider.step(1.0 / 60.0).unwrap();
    let separation_events = provider.drain_collision_events();
    assert_eq!(
        pair_kind_count(
            &separation_events,
            body_a,
            body_b,
            CollisionEventKind::Enter
        ),
        0
    );
    assert_eq!(
        pair_kind_count(&separation_events, body_a, body_b, CollisionEventKind::Stay),
        0
    );
    assert_eq!(
        pair_kind_count(&separation_events, body_a, body_b, CollisionEventKind::Exit),
        1,
        "Expected exactly one Exit event when overlap ends"
    );
}

#[test]
fn test_collision_exit_waits_for_last_active_collider_pair_between_bodies() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    let body_a = provider
        .create_body(&BodyDesc {
            position: [0.0, 0.0],
            body_type: 0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_a,
            &ColliderDesc {
                shape: 0,
                radius: 3.0,
                ..Default::default()
            },
        )
        .unwrap();
    provider
        .create_collider(
            body_a,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                ..Default::default()
            },
        )
        .unwrap();

    let body_b = provider
        .create_body(&BodyDesc {
            position: [1.0, 0.0],
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_b,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                ..Default::default()
            },
        )
        .unwrap();

    provider.step(1.0 / 60.0).unwrap();
    let initial_events = provider.drain_collision_events();
    assert_eq!(
        pair_kind_count(&initial_events, body_a, body_b, CollisionEventKind::Enter),
        1,
        "first overlap should still produce exactly one Enter"
    );

    provider.set_body_position(body_b, [3.0, 0.0]).unwrap();
    provider.step(1.0 / 60.0).unwrap();
    let partial_overlap_events = provider.drain_collision_events();
    assert_eq!(
        pair_kind_count(
            &partial_overlap_events,
            body_a,
            body_b,
            CollisionEventKind::Exit
        ),
        0,
        "stopping one collider pair must not emit Exit while another pair still overlaps"
    );
    assert_eq!(
        pair_kind_count(
            &partial_overlap_events,
            body_a,
            body_b,
            CollisionEventKind::Stay
        ),
        1,
        "body pair should remain in Stay while any collider pair is still overlapping"
    );

    provider.set_body_position(body_b, [10.0, 0.0]).unwrap();
    provider.step(1.0 / 60.0).unwrap();
    let separation_events = provider.drain_collision_events();
    assert_eq!(
        pair_kind_count(&separation_events, body_a, body_b, CollisionEventKind::Exit),
        1,
        "Exit should fire once after the final overlapping collider pair ends"
    );
}

#[test]
fn test_sensor_emits_enter_stay_exit_and_has_no_push_response() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    let sensor_body = provider
        .create_body(&BodyDesc {
            position: [0.0, 0.0],
            body_type: 0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            sensor_body,
            &ColliderDesc {
                shape: 0,
                radius: 2.0,
                is_sensor: true,
                ..Default::default()
            },
        )
        .unwrap();

    let dynamic_body = provider
        .create_body(&BodyDesc {
            position: [0.0, 0.0],
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            dynamic_body,
            &ColliderDesc {
                shape: 0,
                radius: 0.5,
                ..Default::default()
            },
        )
        .unwrap();

    provider.step(1.0 / 60.0).unwrap();
    let enter_events = provider.drain_collision_events();
    assert_eq!(
        pair_kind_count(
            &enter_events,
            sensor_body,
            dynamic_body,
            CollisionEventKind::Enter
        ),
        1
    );

    provider.step(1.0 / 60.0).unwrap();
    let stay_events = provider.drain_collision_events();
    assert_eq!(
        pair_kind_count(
            &stay_events,
            sensor_body,
            dynamic_body,
            CollisionEventKind::Stay
        ),
        1
    );

    let pos = provider.body_position(dynamic_body).unwrap();
    assert!(
        pos[0].abs() < 0.1 && pos[1].abs() < 0.1,
        "Sensor should not push dynamic body: pos={pos:?}"
    );

    provider
        .set_body_position(dynamic_body, [10.0, 0.0])
        .unwrap();
    provider.step(1.0 / 60.0).unwrap();
    let exit_events = provider.drain_collision_events();
    assert_eq!(
        pair_kind_count(
            &exit_events,
            sensor_body,
            dynamic_body,
            CollisionEventKind::Exit
        ),
        1
    );
}

#[test]
fn test_collider_layer_and_mask_filter_collisions() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    let body_a = provider
        .create_body(&BodyDesc {
            position: [0.0, 0.0],
            body_type: 0,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_a,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                layer: 0b0001,
                mask: 0b0010,
                ..Default::default()
            },
        )
        .unwrap();

    let body_b = provider
        .create_body(&BodyDesc {
            position: [0.25, 0.0],
            body_type: 1,
            ..Default::default()
        })
        .unwrap();
    provider
        .create_collider(
            body_b,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                layer: 0b0100,
                mask: 0b1000,
                ..Default::default()
            },
        )
        .unwrap();

    for _ in 0..3 {
        provider.step(1.0 / 60.0).unwrap();
    }

    let events = provider.drain_collision_events();
    assert_eq!(
        pair_kind_count(&events, body_a, body_b, CollisionEventKind::Enter),
        0,
        "Incompatible layers/masks should block collision events"
    );
    assert!(
        provider.contact_pairs().is_empty(),
        "Incompatible layers/masks should prevent contacts"
    );
}

#[test]
fn test_raycast_with_mask_returns_filtered_full_hit_payload() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    let body_a = provider
        .create_body(&BodyDesc {
            position: [5.0, 0.0],
            body_type: 0,
            ..Default::default()
        })
        .unwrap();
    let collider_a = provider
        .create_collider(
            body_a,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                layer: 0b0001,
                ..Default::default()
            },
        )
        .unwrap();

    let body_b = provider
        .create_body(&BodyDesc {
            position: [10.0, 0.0],
            body_type: 0,
            ..Default::default()
        })
        .unwrap();
    let collider_b = provider
        .create_collider(
            body_b,
            &ColliderDesc {
                shape: 0,
                radius: 1.0,
                layer: 0b0010,
                ..Default::default()
            },
        )
        .unwrap();

    provider.step(0.0).unwrap();

    let hit_layer_1 = provider
        .raycast_with_mask([0.0, 0.0], [1.0, 0.0], 100.0, 0b0001)
        .expect("Mask 0b0001 should hit body_a");
    assert_eq!(hit_layer_1.body, body_a);
    assert_eq!(hit_layer_1.collider, collider_a);
    assert!(hit_layer_1.distance > 0.0);
    assert!(hit_layer_1.point[0] > 0.0);
    assert_ne!(hit_layer_1.normal, [0.0, 0.0]);

    let hit_layer_2 = provider
        .raycast_with_mask([0.0, 0.0], [1.0, 0.0], 100.0, 0b0010)
        .expect("Mask 0b0010 should hit body_b");
    assert_eq!(hit_layer_2.body, body_b);
    assert_eq!(hit_layer_2.collider, collider_b);

    let miss = provider.raycast_with_mask([0.0, 0.0], [1.0, 0.0], 100.0, 0b0100);
    assert!(
        miss.is_none(),
        "Mask 0b0100 should ignore colliders on layers 0b0001 and 0b0010"
    );
}

#[test]
fn test_collider_desc_default_layer_and_mask() {
    let desc = ColliderDesc::default();
    assert_eq!(desc.layer, 1);
    assert_eq!(desc.mask, u32::MAX);
}
