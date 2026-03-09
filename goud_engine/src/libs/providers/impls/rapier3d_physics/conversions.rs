//! 3D type conversion helpers between engine types and rapier3d types.

use crate::core::providers::types::{ColliderDesc3D, JointDesc3D};
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
) -> GenericJoint {
    match desc.joint_type {
        0 => RevoluteJointBuilder::new(Vector::new(0.0, 1.0, 0.0))
            .local_anchor1(Vector::new(
                desc.anchor_a[0],
                desc.anchor_a[1],
                desc.anchor_a[2],
            ))
            .local_anchor2(Vector::new(
                desc.anchor_b[0],
                desc.anchor_b[1],
                desc.anchor_b[2],
            ))
            .build()
            .into(),
        1 => PrismaticJointBuilder::new(Vector::new(1.0, 0.0, 0.0))
            .local_anchor1(Vector::new(
                desc.anchor_a[0],
                desc.anchor_a[1],
                desc.anchor_a[2],
            ))
            .local_anchor2(Vector::new(
                desc.anchor_b[0],
                desc.anchor_b[1],
                desc.anchor_b[2],
            ))
            .build()
            .into(),
        _ => {
            let dist = Vector::new(
                desc.anchor_b[0] - desc.anchor_a[0],
                desc.anchor_b[1] - desc.anchor_a[1],
                desc.anchor_b[2] - desc.anchor_a[2],
            )
            .length()
            .max(0.01);
            RopeJointBuilder::new(dist).build().into()
        }
    }
}
