//! Factory functions for creating Transform2D values.

use crate::core::math::Vec2;
use crate::core::types::FfiTransform2D;
use crate::ecs::components::Transform2D;

/// Creates a default Transform2D at the origin with no rotation and unit scale.
///
/// # Returns
///
/// A Transform2D with position (0, 0), rotation 0, and scale (1, 1).
#[no_mangle]
pub extern "C" fn goud_transform2d_default() -> FfiTransform2D {
    Transform2D::default().into()
}

/// Creates a Transform2D at the specified position with default rotation and scale.
///
/// # Parameters
///
/// - `x`: X position in world space
/// - `y`: Y position in world space
///
/// # Returns
///
/// A Transform2D with the specified position, rotation 0, and scale (1, 1).
#[no_mangle]
pub extern "C" fn goud_transform2d_from_position(x: f32, y: f32) -> FfiTransform2D {
    Transform2D::from_position(Vec2::new(x, y)).into()
}

/// Creates a Transform2D at the origin with the specified rotation.
///
/// # Parameters
///
/// - `rotation`: Rotation angle in radians (counter-clockwise)
///
/// # Returns
///
/// A Transform2D with position (0, 0), the specified rotation, and scale (1, 1).
#[no_mangle]
pub extern "C" fn goud_transform2d_from_rotation(rotation: f32) -> FfiTransform2D {
    Transform2D::from_rotation(rotation).into()
}

/// Creates a Transform2D at the origin with the specified rotation in degrees.
///
/// # Parameters
///
/// - `degrees`: Rotation angle in degrees (counter-clockwise)
///
/// # Returns
///
/// A Transform2D with position (0, 0), the specified rotation, and scale (1, 1).
#[no_mangle]
pub extern "C" fn goud_transform2d_from_rotation_degrees(degrees: f32) -> FfiTransform2D {
    Transform2D::from_rotation_degrees(degrees).into()
}

/// Creates a Transform2D at the origin with the specified scale.
///
/// # Parameters
///
/// - `scale_x`: Scale along X axis
/// - `scale_y`: Scale along Y axis
///
/// # Returns
///
/// A Transform2D with position (0, 0), rotation 0, and the specified scale.
#[no_mangle]
pub extern "C" fn goud_transform2d_from_scale(scale_x: f32, scale_y: f32) -> FfiTransform2D {
    Transform2D::from_scale(Vec2::new(scale_x, scale_y)).into()
}

/// Creates a Transform2D with uniform scale.
///
/// # Parameters
///
/// - `scale`: Uniform scale factor for both axes
///
/// # Returns
///
/// A Transform2D with position (0, 0), rotation 0, and uniform scale.
#[no_mangle]
pub extern "C" fn goud_transform2d_from_scale_uniform(scale: f32) -> FfiTransform2D {
    Transform2D::from_scale_uniform(scale).into()
}

/// Creates a Transform2D with position and rotation.
///
/// # Parameters
///
/// - `x`: X position in world space
/// - `y`: Y position in world space
/// - `rotation`: Rotation angle in radians
///
/// # Returns
///
/// A Transform2D with the specified position and rotation, with scale (1, 1).
#[no_mangle]
pub extern "C" fn goud_transform2d_from_position_rotation(
    x: f32,
    y: f32,
    rotation: f32,
) -> FfiTransform2D {
    Transform2D::from_position_rotation(Vec2::new(x, y), rotation).into()
}

/// Creates a Transform2D with all components specified.
///
/// # Parameters
///
/// - `pos_x`: X position in world space
/// - `pos_y`: Y position in world space
/// - `rotation`: Rotation angle in radians
/// - `scale_x`: Scale along X axis
/// - `scale_y`: Scale along Y axis
///
/// # Returns
///
/// A fully specified Transform2D.
#[no_mangle]
pub extern "C" fn goud_transform2d_new(
    pos_x: f32,
    pos_y: f32,
    rotation: f32,
    scale_x: f32,
    scale_y: f32,
) -> FfiTransform2D {
    Transform2D::new(
        Vec2::new(pos_x, pos_y),
        rotation,
        Vec2::new(scale_x, scale_y),
    )
    .into()
}

/// Creates a Transform2D looking at a target position.
///
/// The transform's forward direction (positive X after rotation)
/// will point towards the target.
///
/// # Parameters
///
/// - `pos_x`: X position of the transform
/// - `pos_y`: Y position of the transform
/// - `target_x`: X position of the target to look at
/// - `target_y`: Y position of the target to look at
///
/// # Returns
///
/// A Transform2D positioned at (pos_x, pos_y) looking towards (target_x, target_y).
#[no_mangle]
pub extern "C" fn goud_transform2d_look_at(
    pos_x: f32,
    pos_y: f32,
    target_x: f32,
    target_y: f32,
) -> FfiTransform2D {
    Transform2D::look_at(Vec2::new(pos_x, pos_y), Vec2::new(target_x, target_y)).into()
}
