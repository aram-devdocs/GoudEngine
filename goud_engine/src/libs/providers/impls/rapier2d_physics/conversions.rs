//! Type conversion helpers between engine types and Rapier2D types.

#![cfg(feature = "rapier2d")]

use rapier2d::prelude::*;

use crate::core::providers::types::ColliderDesc;
use crate::core::providers::types::JointDesc;

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

/// Convert an engine `JointDesc` to a Rapier `GenericJoint`.
///
/// Mapping: 0 = revolute, 1 = prismatic, 2 = rope/distance.
/// Unknown values default to rope.
pub fn joint_from_desc(
    desc: &JointDesc,
    body_a: RigidBodyHandle,
    body_b: RigidBodyHandle,
) -> (GenericJoint, RigidBodyHandle, RigidBodyHandle) {
    let joint: GenericJoint = match desc.joint_type {
        0 => RevoluteJointBuilder::new()
            .local_anchor1(point![desc.anchor_a[0], desc.anchor_a[1]])
            .local_anchor2(point![desc.anchor_b[0], desc.anchor_b[1]])
            .build()
            .into(),
        1 => PrismaticJointBuilder::new(UnitVector::new_normalize(vector![1.0, 0.0]))
            .local_anchor1(point![desc.anchor_a[0], desc.anchor_a[1]])
            .local_anchor2(point![desc.anchor_b[0], desc.anchor_b[1]])
            .build()
            .into(),
        _ => {
            let diff = vector![
                desc.anchor_b[0] - desc.anchor_a[0],
                desc.anchor_b[1] - desc.anchor_a[1]
            ];
            let max_dist = diff.norm().max(0.01);
            RopeJointBuilder::new(max_dist).build().into()
        }
    };
    (joint, body_a, body_b)
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
}
