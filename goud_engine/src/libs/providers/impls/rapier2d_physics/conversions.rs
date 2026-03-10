//! Type conversion helpers between engine types and Rapier2D types.

use crate::core::error::{GoudError, GoudResult};
use rapier2d::prelude::*;

use crate::core::providers::types::{ColliderDesc, JointDesc, JointKind, JointLimits, JointMotor};

/// Convert an engine body-type integer to a Rapier `RigidBodyType`.
///
/// Mapping: 0 = Fixed, 1 = Dynamic, 2 = KinematicPositionBased.
/// Unknown values default to Dynamic.
pub fn body_type_from_u32(t: u32) -> RigidBodyType {
    match t {
        0 => RigidBodyType::Fixed,
        1 => RigidBodyType::Dynamic,
        2 => RigidBodyType::KinematicPositionBased,
        _ => RigidBodyType::Dynamic,
    }
}

/// Convert an engine `ColliderDesc` shape field to a Rapier `SharedShape`.
///
/// Mapping: 0 = circle (ball), 1 = box (cuboid), 2 = capsule.
/// Unknown values default to a ball.
pub fn shape_from_desc(desc: &ColliderDesc) -> SharedShape {
    match desc.shape {
        0 => SharedShape::ball(desc.radius),
        1 => SharedShape::cuboid(desc.half_extents[0], desc.half_extents[1]),
        2 => SharedShape::capsule_y(desc.half_extents[1], desc.radius),
        _ => SharedShape::ball(desc.radius),
    }
}

/// Convert an engine `ColliderDesc` layer/mask to Rapier interaction groups.
pub fn collision_groups_from_desc(desc: &ColliderDesc) -> InteractionGroups {
    InteractionGroups::new(Group::from(desc.layer), Group::from(desc.mask))
}

/// Build a raycast query filter that selects colliders by layer mask.
pub fn raycast_query_filter(layer_mask: u32) -> QueryFilter<'static> {
    let query_groups = InteractionGroups::new(Group::ALL, Group::from(layer_mask));
    QueryFilter::default().groups(query_groups)
}

/// Convert an engine `JointDesc` to a Rapier `GenericJoint`.
///
/// Mapping: 0 = revolute, 1 = prismatic, 2 = rope/distance.
/// Unknown values default to rope.
pub fn joint_from_desc(
    desc: &JointDesc,
    body_a: RigidBodyHandle,
    body_b: RigidBodyHandle,
) -> GoudResult<(GenericJoint, RigidBodyHandle, RigidBodyHandle)> {
    let anchor_a = point![desc.anchor_a[0], desc.anchor_a[1]];
    let anchor_b = point![desc.anchor_b[0], desc.anchor_b[1]];
    let joint: GenericJoint = match desc.kind {
        JointKind::Revolute => apply_angular_options(
            RevoluteJointBuilder::new()
                .local_anchor1(anchor_a)
                .local_anchor2(anchor_b),
            desc.limits,
            desc.motor,
        )
        .build()
        .into(),
        JointKind::Prismatic => {
            let axis = normalized_axis(desc.axis)?;
            apply_linear_options(
                PrismaticJointBuilder::new(axis)
                    .local_anchor1(anchor_a)
                    .local_anchor2(anchor_b),
                desc.limits,
                desc.motor,
            )
            .build()
            .into()
        }
        JointKind::Distance => {
            let max_dist = desc
                .limits
                .map(|limits| limits.max.max(0.01))
                .unwrap_or_else(|| {
                    let diff = vector![
                        desc.anchor_b[0] - desc.anchor_a[0],
                        desc.anchor_b[1] - desc.anchor_a[1]
                    ];
                    diff.norm().max(0.01)
                });
            apply_distance_options(
                RopeJointBuilder::new(max_dist)
                    .local_anchor1(anchor_a)
                    .local_anchor2(anchor_b),
                desc.limits,
                desc.motor,
            )
            .build()
            .into()
        }
    };
    Ok((joint, body_a, body_b))
}

fn normalized_axis(axis: [f32; 2]) -> GoudResult<UnitVector<Real>> {
    let vector = vector![axis[0], axis[1]];
    UnitVector::try_new(vector, 1.0e-6).ok_or_else(|| GoudError::ProviderError {
        subsystem: "physics",
        message: "prismatic joint axis cannot be zero".to_string(),
    })
}

fn apply_linear_options(
    mut builder: PrismaticJointBuilder,
    limits: Option<JointLimits>,
    motor: Option<JointMotor>,
) -> PrismaticJointBuilder {
    if let Some(limits) = limits {
        builder = builder.limits([limits.min, limits.max]);
    }
    if let Some(motor) = motor {
        builder = builder
            .motor_velocity(motor.target_velocity, 1.0)
            .motor_max_force(motor.max_force);
    }
    builder
}

fn apply_distance_options(
    mut builder: RopeJointBuilder,
    limits: Option<JointLimits>,
    motor: Option<JointMotor>,
) -> RopeJointBuilder {
    if let Some(limits) = limits {
        builder = builder.max_distance(limits.max.max(0.01));
    }
    if let Some(motor) = motor {
        builder = builder
            .motor_velocity(motor.target_velocity, 1.0)
            .motor_max_force(motor.max_force);
    }
    builder
}

fn apply_angular_options(
    mut builder: RevoluteJointBuilder,
    limits: Option<JointLimits>,
    motor: Option<JointMotor>,
) -> RevoluteJointBuilder {
    if let Some(limits) = limits {
        builder = builder.limits([limits.min, limits.max]);
    }
    if let Some(motor) = motor {
        builder = builder
            .motor_velocity(motor.target_velocity, 1.0)
            .motor_max_force(motor.max_force);
    }
    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_type_conversions() {
        assert_eq!(body_type_from_u32(0), RigidBodyType::Fixed);
        assert_eq!(body_type_from_u32(1), RigidBodyType::Dynamic);
        assert_eq!(body_type_from_u32(2), RigidBodyType::KinematicPositionBased);
        // Unknown defaults to Dynamic
        assert_eq!(body_type_from_u32(99), RigidBodyType::Dynamic);
    }

    #[test]
    fn test_shape_from_desc_circle() {
        let desc = ColliderDesc {
            shape: 0,
            radius: 1.5,
            ..Default::default()
        };
        let shape = shape_from_desc(&desc);
        assert!(shape.as_ball().is_some());
        assert!((shape.as_ball().unwrap().radius - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_shape_from_desc_box() {
        let desc = ColliderDesc {
            shape: 1,
            half_extents: [2.0, 3.0],
            ..Default::default()
        };
        let shape = shape_from_desc(&desc);
        assert!(shape.as_cuboid().is_some());
        let cuboid = shape.as_cuboid().unwrap();
        assert!((cuboid.half_extents.x - 2.0).abs() < f32::EPSILON);
        assert!((cuboid.half_extents.y - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_shape_from_desc_capsule() {
        let desc = ColliderDesc {
            shape: 2,
            radius: 0.5,
            half_extents: [0.0, 1.0],
            ..Default::default()
        };
        let shape = shape_from_desc(&desc);
        assert!(shape.as_capsule().is_some());
    }

    #[test]
    fn test_shape_from_desc_unknown_defaults_to_ball() {
        let desc = ColliderDesc {
            shape: 42,
            radius: 1.0,
            ..Default::default()
        };
        let shape = shape_from_desc(&desc);
        assert!(shape.as_ball().is_some());
    }

    #[test]
    fn test_collision_groups_from_desc_maps_layer_and_mask() {
        let desc = ColliderDesc {
            layer: 0b0010,
            mask: 0b1010,
            ..Default::default()
        };
        let groups = collision_groups_from_desc(&desc);
        assert_eq!(groups.memberships.bits(), 0b0010);
        assert_eq!(groups.filter.bits(), 0b1010);
    }

    #[test]
    fn test_joint_from_desc_revolute_uses_limits_and_motor() {
        let (joint, _, _) = joint_from_desc(
            &JointDesc {
                kind: JointKind::Revolute,
                anchor_a: [1.0, 2.0],
                anchor_b: [3.0, 4.0],
                limits: Some(JointLimits {
                    min: -0.5,
                    max: 0.75,
                }),
                motor: Some(JointMotor {
                    target_velocity: 2.5,
                    max_force: 9.0,
                }),
                ..Default::default()
            },
            RigidBodyHandle::invalid(),
            RigidBodyHandle::invalid(),
        )
        .unwrap();

        assert_eq!(joint.local_anchor1(), point![1.0, 2.0]);
        assert_eq!(joint.local_anchor2(), point![3.0, 4.0]);
        assert_eq!(
            joint.limits(JointAxis::AngX).map(|limits| limits.min),
            Some(-0.5)
        );
        assert_eq!(
            joint.motor(JointAxis::AngX).map(|motor| motor.target_vel),
            Some(2.5)
        );
    }

    #[test]
    fn test_joint_from_desc_prismatic_uses_axis_limits_and_motor() {
        let (joint, _, _) = joint_from_desc(
            &JointDesc {
                kind: JointKind::Prismatic,
                axis: [0.0, 2.0],
                limits: Some(JointLimits {
                    min: -1.0,
                    max: 4.0,
                }),
                motor: Some(JointMotor {
                    target_velocity: 1.5,
                    max_force: 6.0,
                }),
                ..Default::default()
            },
            RigidBodyHandle::invalid(),
            RigidBodyHandle::invalid(),
        )
        .unwrap();

        assert!((joint.local_axis1().x - 0.0).abs() < f32::EPSILON);
        assert!((joint.local_axis1().y - 1.0).abs() < f32::EPSILON);
        assert_eq!(
            joint.limits(JointAxis::LinX).map(|limits| limits.max),
            Some(4.0)
        );
        assert_eq!(
            joint.motor(JointAxis::LinX).map(|motor| motor.max_force),
            Some(6.0)
        );
    }

    #[test]
    fn test_joint_from_desc_distance_uses_explicit_limit_range() {
        let (joint, _, _) = joint_from_desc(
            &JointDesc {
                kind: JointKind::Distance,
                limits: Some(JointLimits { min: 1.0, max: 3.5 }),
                ..Default::default()
            },
            RigidBodyHandle::invalid(),
            RigidBodyHandle::invalid(),
        )
        .unwrap();

        assert_eq!(
            joint
                .limits(JointAxis::LinX)
                .map(|limits| (limits.min, limits.max)),
            Some((0.0, 3.5))
        );
    }

    #[test]
    fn test_joint_from_desc_prismatic_rejects_zero_axis() {
        let err = joint_from_desc(
            &JointDesc {
                kind: JointKind::Prismatic,
                axis: [0.0, 0.0],
                ..Default::default()
            },
            RigidBodyHandle::invalid(),
            RigidBodyHandle::invalid(),
        )
        .unwrap_err();

        assert!(matches!(err, GoudError::ProviderError { .. }));
    }
}
