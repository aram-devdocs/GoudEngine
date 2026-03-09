//! RED tests for collision event semantics, layer/mask filtering, and raycast behavior.

use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::types::{
    BodyDesc, BodyHandle, ColliderDesc, ColliderHandle, CollisionEvent, CollisionEventKind,
};
use crate::libs::providers::impls::rapier2d_physics::Rapier2DPhysicsProvider;
use rapier2d::prelude::{Group, InteractionGroups};

fn add_circle(
    provider: &mut Rapier2DPhysicsProvider,
    body_type: u32,
    position: [f32; 2],
    radius: f32,
    is_sensor: bool,
) -> (BodyHandle, ColliderHandle) {
    let body = provider
        .create_body(&BodyDesc {
            body_type,
            position,
            gravity_scale: 0.0,
            ..Default::default()
        })
        .unwrap();

    let collider = provider
        .create_collider(
            body,
            &ColliderDesc {
                shape: 0,
                radius,
                is_sensor,
                ..Default::default()
            },
        )
        .unwrap();

    (body, collider)
}

fn body_pair_matches(event: &CollisionEvent, a: BodyHandle, b: BodyHandle) -> bool {
    (event.body_a == a && event.body_b == b) || (event.body_a == b && event.body_b == a)
}

fn kind_count(
    events: &[CollisionEvent],
    a: BodyHandle,
    b: BodyHandle,
    kind: CollisionEventKind,
) -> usize {
    events
        .iter()
        .filter(|event| body_pair_matches(event, a, b) && event.kind == kind)
        .count()
}

fn set_groups(
    provider: &mut Rapier2DPhysicsProvider,
    collider: ColliderHandle,
    memberships: Group,
    filter: Group,
) {
    let rapier_handle = provider
        .get_rapier_collider(collider)
        .expect("collider handle should map to Rapier");

    let rapier_collider = provider
        .collider_set
        .get_mut(rapier_handle)
        .expect("collider should exist");
    rapier_collider.set_collision_groups(InteractionGroups::new(memberships, filter));
}

fn groups(provider: &Rapier2DPhysicsProvider, collider: ColliderHandle) -> InteractionGroups {
    let rapier_handle = provider
        .get_rapier_collider(collider)
        .expect("collider handle should map to Rapier");

    provider
        .collider_set
        .get(rapier_handle)
        .expect("collider should exist")
        .collision_groups()
}

fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() <= epsilon
}

#[test]
fn test_collision_events_should_emit_enter_stay_and_single_exit() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    let (body_a, _collider_a) = add_circle(&mut provider, 0, [0.0, 0.0], 1.0, false);
    let (body_b, _collider_b) = add_circle(&mut provider, 1, [0.5, 0.0], 1.0, false);

    provider.step(1.0 / 60.0).unwrap();
    let step_1 = provider.drain_collision_events();
    assert_eq!(
        kind_count(&step_1, body_a, body_b, CollisionEventKind::Enter),
        1,
        "expected exactly one Enter event on first overlap step"
    );
    assert_eq!(
        kind_count(&step_1, body_a, body_b, CollisionEventKind::Stay),
        0,
        "Stay should not fire on first overlap step"
    );

    provider.step(1.0 / 60.0).unwrap();
    let step_2 = provider.drain_collision_events();
    assert_eq!(
        kind_count(&step_2, body_a, body_b, CollisionEventKind::Enter),
        0,
        "Enter should fire exactly once"
    );
    assert_eq!(
        kind_count(&step_2, body_a, body_b, CollisionEventKind::Stay),
        1,
        "expected Stay event each overlapping step (step 2)"
    );

    provider.step(1.0 / 60.0).unwrap();
    let step_3 = provider.drain_collision_events();
    assert_eq!(
        kind_count(&step_3, body_a, body_b, CollisionEventKind::Stay),
        1,
        "expected Stay event each overlapping step (step 3)"
    );

    provider.set_body_position(body_b, [6.0, 0.0]).unwrap();
    provider.step(1.0 / 60.0).unwrap();
    let step_4 = provider.drain_collision_events();
    assert_eq!(
        kind_count(&step_4, body_a, body_b, CollisionEventKind::Exit),
        1,
        "expected exactly one Exit event when overlap ends"
    );

    provider.step(1.0 / 60.0).unwrap();
    let step_5 = provider.drain_collision_events();
    assert_eq!(
        step_5
            .iter()
            .filter(|event| body_pair_matches(event, body_a, body_b))
            .count(),
        0,
        "expected no duplicate Exit events after separation"
    );
}

#[test]
fn test_sensor_should_emit_collision_events_without_physical_push() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    let (sensor_body, _sensor_collider) = add_circle(&mut provider, 0, [0.0, 0.0], 2.0, true);
    let (dynamic_body, _dynamic_collider) = add_circle(&mut provider, 1, [0.0, 0.0], 0.5, false);

    provider.step(1.0 / 60.0).unwrap();
    let step_1 = provider.drain_collision_events();
    assert_eq!(
        kind_count(
            &step_1,
            sensor_body,
            dynamic_body,
            CollisionEventKind::Enter
        ),
        1,
        "expected one Enter event when body first overlaps sensor"
    );

    provider.step(1.0 / 60.0).unwrap();
    let step_2 = provider.drain_collision_events();
    assert_eq!(
        kind_count(&step_2, sensor_body, dynamic_body, CollisionEventKind::Stay),
        1,
        "expected Stay event each step while body remains inside sensor"
    );

    let pos = provider.body_position(dynamic_body).unwrap();
    assert!(
        pos[0].abs() < 0.2 && pos[1].abs() < 0.2,
        "sensor should not physically push dynamic body, got position={pos:?}"
    );

    provider
        .set_body_position(dynamic_body, [8.0, 0.0])
        .unwrap();
    provider.step(1.0 / 60.0).unwrap();
    let step_3 = provider.drain_collision_events();
    assert_eq!(
        kind_count(&step_3, sensor_body, dynamic_body, CollisionEventKind::Exit),
        1,
        "expected one Exit event when leaving sensor overlap"
    );
}

#[test]
fn test_default_collision_groups_should_be_layer_1_and_all_mask() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);
    let (_body, collider) = add_circle(&mut provider, 1, [0.0, 0.0], 1.0, false);

    let groups = groups(&provider, collider);
    assert_eq!(
        groups.memberships.bits(),
        0b0001,
        "default collision layer should be 1 (GROUP_1)"
    );
    assert_eq!(
        groups.filter.bits(),
        0xFFFF_FFFF,
        "default collision mask should collide with all layers"
    );
}

#[test]
fn test_default_layer_should_not_collide_with_group_2_only_collider() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    let (body_group2_only, collider_group2_only) =
        add_circle(&mut provider, 0, [0.0, 0.0], 1.0, false);
    let (body_default, _default_collider) = add_circle(&mut provider, 2, [0.5, 0.0], 1.0, false);

    set_groups(
        &mut provider,
        collider_group2_only,
        Group::GROUP_2,
        Group::GROUP_2,
    );

    provider.step(1.0 / 60.0).unwrap();
    let events = provider.drain_collision_events();

    let pair_events = events
        .iter()
        .filter(|event| body_pair_matches(event, body_group2_only, body_default))
        .count();

    assert_eq!(
        pair_events, 0,
        "a GROUP_2-only collider should not interact with default layer=1 collider"
    );
}

#[test]
fn test_raycast_hit_payload_should_include_surface_point_normal_and_distance() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);
    let (target_body, _target_collider) = add_circle(&mut provider, 0, [5.0, 0.0], 1.0, false);

    provider.step(0.0).unwrap();

    let hit = provider
        .raycast([0.0, 0.0], [1.0, 0.0], 100.0)
        .expect("raycast should hit the target collider");

    assert_eq!(hit.body, target_body, "raycast should return the hit body");
    assert!(
        approx_eq(hit.point[0], 4.0, 0.02) && approx_eq(hit.point[1], 0.0, 0.02),
        "expected hit point near [4, 0], got {:?}",
        hit.point
    );
    assert!(
        approx_eq(hit.normal[0], -1.0, 0.02) && approx_eq(hit.normal[1], 0.0, 0.02),
        "expected outward surface normal near [-1, 0], got {:?}",
        hit.normal
    );
    assert!(
        approx_eq(hit.distance, 4.0, 0.02),
        "expected hit distance near 4.0, got {}",
        hit.distance
    );
}

#[test]
fn test_raycast_should_honor_layer_mask_filtering() {
    let mut provider = Rapier2DPhysicsProvider::new([0.0, 0.0]);

    let (near_body, near_collider) = add_circle(&mut provider, 0, [2.0, 0.0], 0.75, false);
    let (far_body, far_collider) = add_circle(&mut provider, 0, [6.0, 0.0], 0.75, false);

    set_groups(&mut provider, near_collider, Group::GROUP_2, Group::ALL);
    set_groups(&mut provider, far_collider, Group::GROUP_1, Group::ALL);

    provider.step(0.0).unwrap();

    let hit_group_1 = provider
        .raycast_with_mask([0.0, 0.0], [1.0, 0.0], 100.0, Group::GROUP_1.bits())
        .expect("GROUP_1 mask should hit the GROUP_1 collider");
    assert_eq!(hit_group_1.body, far_body);
    assert_eq!(hit_group_1.collider, far_collider);

    let hit_group_2 = provider
        .raycast_with_mask([0.0, 0.0], [1.0, 0.0], 100.0, Group::GROUP_2.bits())
        .expect("GROUP_2 mask should hit the GROUP_2 collider");
    assert_eq!(hit_group_2.body, near_body);
    assert_eq!(hit_group_2.collider, near_collider);

    let miss = provider.raycast_with_mask([0.0, 0.0], [1.0, 0.0], 100.0, Group::GROUP_3.bits());
    assert_eq!(
        miss, None,
        "GROUP_3 mask should filter out both GROUP_1 and GROUP_2 colliders"
    );
}
