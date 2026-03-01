//! Factory functions for creating Transform2D values by copy.

use crate::core::math::Vec2;
use crate::core::types::FfiTransform2D;
use crate::ecs::components::Transform2D;
use std::f32::consts::PI;

/// Zero-sized type hosting Transform2D factory operations.
pub struct Transform2DOps;

// NOTE: FFI wrappers are hand-written in ffi/component_transform2d.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl Transform2DOps {
    /// Creates a default Transform2D (origin, no rotation, unit scale).
    pub fn new_default() -> FfiTransform2D {
        Transform2D::default().into()
    }

    /// Creates a Transform2D at the specified position.
    pub fn from_position(x: f32, y: f32) -> FfiTransform2D {
        Transform2D::from_position(Vec2::new(x, y)).into()
    }

    /// Creates a Transform2D with the specified rotation (radians).
    pub fn from_rotation(rotation: f32) -> FfiTransform2D {
        Transform2D::from_rotation(rotation).into()
    }

    /// Creates a Transform2D with the specified rotation (degrees).
    pub fn from_rotation_degrees(degrees: f32) -> FfiTransform2D {
        Transform2D::from_rotation_degrees(degrees).into()
    }

    /// Creates a Transform2D with the specified scale.
    pub fn from_scale(scale_x: f32, scale_y: f32) -> FfiTransform2D {
        Transform2D::from_scale(Vec2::new(scale_x, scale_y)).into()
    }

    /// Creates a Transform2D with uniform scale.
    pub fn from_scale_uniform(scale: f32) -> FfiTransform2D {
        Transform2D::from_scale_uniform(scale).into()
    }

    /// Creates a Transform2D with position and rotation.
    pub fn from_position_rotation(x: f32, y: f32, rotation: f32) -> FfiTransform2D {
        Transform2D::from_position_rotation(Vec2::new(x, y), rotation).into()
    }

    /// Creates a fully specified Transform2D.
    pub fn new_full(
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
    pub fn look_at(pos_x: f32, pos_y: f32, target_x: f32, target_y: f32) -> FfiTransform2D {
        Transform2D::look_at(Vec2::new(pos_x, pos_y), Vec2::new(target_x, target_y)).into()
    }

    /// Linearly interpolates between two transforms.
    pub fn lerp(from: FfiTransform2D, to: FfiTransform2D, t: f32) -> FfiTransform2D {
        let from_t: Transform2D = from.into();
        let to_t: Transform2D = to.into();
        from_t.lerp(to_t, t).into()
    }

    /// Normalizes an angle to [-PI, PI).
    pub fn normalize_angle(angle: f32) -> f32 {
        let mut result = angle % (2.0 * PI);
        if result >= PI {
            result -= 2.0 * PI;
        } else if result < -PI {
            result += 2.0 * PI;
        }
        result
    }
}
