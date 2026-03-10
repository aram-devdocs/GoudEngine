use crate::core::math::Vec2;
use crate::core::providers::types::{BodyHandle, JointKind, JointLimits, JointMotor};
use crate::ecs::components::joint::Joint;
use crate::ecs::entity::Entity;
use crate::ecs::Component;

#[test]
fn test_joint_defaults_to_placeholder_revolute() {
    let joint = Joint::default();
    assert_eq!(joint.connected_entity, Entity::PLACEHOLDER);
    assert_eq!(joint.kind, JointKind::Revolute);
    assert_eq!(joint.axis, Vec2::unit_x());
}

#[test]
fn test_joint_builders_capture_configuration() {
    let joint = Joint::prismatic(Entity::new(7, 2), Vec2::unit_y())
        .with_anchors(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0))
        .with_limits(JointLimits {
            min: -2.0,
            max: 5.0,
        })
        .with_motor(JointMotor {
            target_velocity: 6.0,
            max_force: 7.0,
        });

    assert_eq!(joint.connected_entity, Entity::new(7, 2));
    assert_eq!(joint.kind, JointKind::Prismatic);
    assert_eq!(joint.anchor_a, Vec2::new(1.0, 2.0));
    assert_eq!(joint.anchor_b, Vec2::new(3.0, 4.0));
    assert_eq!(joint.axis, Vec2::unit_y());
    assert_eq!(joint.limits.unwrap().max, 5.0);
    assert_eq!(joint.motor.unwrap().target_velocity, 6.0);
}

#[test]
fn test_joint_to_desc_maps_entities_to_handles() {
    let joint = Joint::distance(Entity::new(3, 1))
        .with_anchor_a(Vec2::new(0.5, -1.0))
        .with_anchor_b(Vec2::new(-0.25, 2.0));

    let desc = joint.to_desc(BodyHandle(11), BodyHandle(12));

    assert_eq!(desc.body_a, Some(BodyHandle(11)));
    assert_eq!(desc.body_b, Some(BodyHandle(12)));
    assert_eq!(desc.kind, JointKind::Distance);
    assert_eq!(desc.anchor_a, [0.5, -1.0]);
    assert_eq!(desc.anchor_b, [-0.25, 2.0]);
}

#[test]
fn test_joint_is_an_ecs_component() {
    fn requires_component<T: Component>() {}
    requires_component::<Joint>();
}
