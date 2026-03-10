//! 3D type conversion helpers between engine types and rapier3d types.

use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::types::{
    ColliderDesc3D, JointDesc3D, JointKind, JointLimits, JointMotor,
};
use rapier3d::prelude::*;

/// Convert an engine body-type integer to a rapier `RigidBodyType`.
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

/// Build a `SharedShape` from a 3D collider descriptor.
///
/// Shape mapping: 0 = sphere (ball), 1 = box (cuboid), 2 = capsule (Y-axis).
/// Unknown values default to sphere.
pub fn shape_from_desc(desc: &ColliderDesc3D) -> SharedShape {
    match desc.shape {
        0 => SharedShape::ball(desc.radius),
        1 => SharedShape::cuboid(
            desc.half_extents[0],
            desc.half_extents[1],
            desc.half_extents[2],
        ),
        2 => SharedShape::capsule_y(desc.half_height, desc.radius),
        _ => SharedShape::ball(desc.radius),
    }
}

/// Build a `GenericJoint` from a 3D joint descriptor and body handles.
///
/// Joint mapping: 0 = revolute (Y-axis), 1 = prismatic (X-axis), 2+ = rope.
pub fn joint_from_desc(
    desc: &JointDesc3D,
    _body_a: RigidBodyHandle,
    _body_b: RigidBodyHandle,
) -> GoudResult<GenericJoint> {
    let anchor_a = point![desc.anchor_a[0], desc.anchor_a[1], desc.anchor_a[2]];
    let anchor_b = point![desc.anchor_b[0], desc.anchor_b[1], desc.anchor_b[2]];
    let joint = match desc.kind {
        JointKind::Revolute => {
            let axis = normalized_axis(desc.axis, vector![0.0, 1.0, 0.0])?;
            apply_angular_options(
                RevoluteJointBuilder::new(axis)
                    .local_anchor1(anchor_a)
                    .local_anchor2(anchor_b),
                desc.limits,
                desc.motor,
            )
            .build()
            .into()
        }
        JointKind::Prismatic => {
            let axis = normalized_axis(desc.axis, vector![1.0, 0.0, 0.0])?;
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
                    (vector![
                        desc.anchor_b[0] - desc.anchor_a[0],
                        desc.anchor_b[1] - desc.anchor_a[1],
                        desc.anchor_b[2] - desc.anchor_a[2]
                    ])
                    .norm()
                    .max(0.01)
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
    Ok(joint)
}

fn normalized_axis(axis: [f32; 3], fallback: Vector<Real>) -> GoudResult<UnitVector<Real>> {
    let requested = vector![axis[0], axis[1], axis[2]];
    UnitVector::try_new(requested, 1.0e-6)
        .or_else(|| UnitVector::try_new(fallback, 1.0e-6))
        .ok_or_else(|| GoudError::ProviderError {
            subsystem: "physics",
            message: "joint axis cannot be zero".to_string(),
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
    fn test_joint_from_desc_revolute_uses_axis_limits_and_motor() {
        let joint = joint_from_desc(
            &JointDesc3D {
                kind: JointKind::Revolute,
                axis: [0.0, 0.0, 2.0],
                limits: Some(JointLimits {
                    min: -1.0,
                    max: 1.0,
                }),
                motor: Some(JointMotor {
                    target_velocity: 3.0,
                    max_force: 7.5,
                }),
                ..Default::default()
            },
            RigidBodyHandle::invalid(),
            RigidBodyHandle::invalid(),
        )
        .unwrap();

        assert!((joint.local_axis1().x - 0.0).abs() < f32::EPSILON);
        assert!((joint.local_axis1().y - 0.0).abs() < f32::EPSILON);
        assert!((joint.local_axis1().z - 1.0).abs() < f32::EPSILON);
        assert_eq!(
            joint.limits(JointAxis::AngX).map(|limits| limits.max),
            Some(1.0)
        );
        assert_eq!(
            joint.motor(JointAxis::AngX).map(|motor| motor.max_force),
            Some(7.5)
        );
    }

    #[test]
    fn test_joint_from_desc_prismatic_uses_axis_limits_and_motor() {
        let joint = joint_from_desc(
            &JointDesc3D {
                kind: JointKind::Prismatic,
                axis: [0.0, 4.0, 0.0],
                limits: Some(JointLimits { min: 0.5, max: 2.5 }),
                motor: Some(JointMotor {
                    target_velocity: 1.25,
                    max_force: 4.5,
                }),
                ..Default::default()
            },
            RigidBodyHandle::invalid(),
            RigidBodyHandle::invalid(),
        )
        .unwrap();

        assert!((joint.local_axis1().x - 0.0).abs() < f32::EPSILON);
        assert!((joint.local_axis1().y - 1.0).abs() < f32::EPSILON);
        assert!((joint.local_axis1().z - 0.0).abs() < f32::EPSILON);
        assert_eq!(
            joint.limits(JointAxis::LinX).map(|limits| limits.min),
            Some(0.5)
        );
        assert_eq!(
            joint.motor(JointAxis::LinX).map(|motor| motor.target_vel),
            Some(1.25)
        );
    }

    #[test]
    fn test_joint_from_desc_distance_uses_explicit_limit_range() {
        let joint = joint_from_desc(
            &JointDesc3D {
                kind: JointKind::Distance,
                limits: Some(JointLimits { min: 2.0, max: 6.0 }),
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
            Some((0.0, 6.0))
        );
    }
}
