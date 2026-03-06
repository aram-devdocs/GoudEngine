//! Core `Transform2D` struct definition and constructors.

use crate::core::math::Vec2;
use crate::ecs::Component;

/// A 2D spatial transformation component.
///
/// Represents position, rotation, and scale in 2D space. This is the primary
/// component for positioning entities in 2D games.
///
/// # Memory Layout
///
/// The component is laid out as:
/// - `position`: 2 x f32 (8 bytes)
/// - `rotation`: f32 (4 bytes)
/// - `scale`: 2 x f32 (8 bytes)
/// - Total: 20 bytes
///
/// # Example
///
/// ```
/// use goud_engine::ecs::components::Transform2D;
/// use goud_engine::core::math::Vec2;
///
/// // Create at origin with default rotation and scale
/// let mut transform = Transform2D::default();
///
/// // Or create with specific position
/// let transform = Transform2D::from_position(Vec2::new(100.0, 50.0));
///
/// // Or with full control
/// let transform = Transform2D::new(
///     Vec2::new(100.0, 50.0),  // position
///     0.0,                      // rotation (radians)
///     Vec2::one(),              // scale
/// );
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    /// Position in world space (or local space if entity has a parent).
    pub position: Vec2,
    /// Rotation angle in radians (counter-clockwise).
    pub rotation: f32,
    /// Scale along each axis.
    ///
    /// A scale of (1, 1) means no scaling. Negative values flip the object
    /// along that axis.
    pub scale: Vec2,
}

impl Transform2D {
    /// Creates a new Transform2D with the specified position, rotation, and scale.
    #[inline]
    pub const fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Creates a Transform2D at the specified position with default rotation and scale.
    #[inline]
    pub fn from_position(position: Vec2) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vec2::one(),
        }
    }

    /// Creates a Transform2D at the origin with the specified rotation.
    #[inline]
    pub fn from_rotation(rotation: f32) -> Self {
        Self {
            position: Vec2::zero(),
            rotation,
            scale: Vec2::one(),
        }
    }

    /// Creates a Transform2D with specified rotation in degrees.
    #[inline]
    pub fn from_rotation_degrees(degrees: f32) -> Self {
        Self::from_rotation(degrees.to_radians())
    }

    /// Creates a Transform2D at the origin with the specified scale.
    #[inline]
    pub fn from_scale(scale: Vec2) -> Self {
        Self {
            position: Vec2::zero(),
            rotation: 0.0,
            scale,
        }
    }

    /// Creates a Transform2D with uniform scale.
    #[inline]
    pub fn from_scale_uniform(scale: f32) -> Self {
        Self::from_scale(Vec2::new(scale, scale))
    }

    /// Creates a Transform2D with position and rotation.
    #[inline]
    pub fn from_position_rotation(position: Vec2, rotation: f32) -> Self {
        Self {
            position,
            rotation,
            scale: Vec2::one(),
        }
    }

    /// Creates a Transform2D looking at a target position.
    ///
    /// The transform's forward direction (positive X after rotation)
    /// will point towards the target.
    #[inline]
    pub fn look_at(position: Vec2, target: Vec2) -> Self {
        let direction = target - position;
        let rotation = direction.y.atan2(direction.x);
        Self {
            position,
            rotation,
            scale: Vec2::one(),
        }
    }
}

impl Default for Transform2D {
    /// Returns a Transform2D at the origin with no rotation and unit scale.
    #[inline]
    fn default() -> Self {
        Self {
            position: Vec2::zero(),
            rotation: 0.0,
            scale: Vec2::one(),
        }
    }
}

// Implement Component trait for Transform2D
impl Component for Transform2D {}

// FFI conversion impls — kept here (Services layer) to avoid Foundation→Services dependency.

impl From<Transform2D> for crate::core::types::FfiTransform2D {
    fn from(t: Transform2D) -> Self {
        Self {
            position_x: t.position.x,
            position_y: t.position.y,
            rotation: t.rotation,
            scale_x: t.scale.x,
            scale_y: t.scale.y,
        }
    }
}

impl From<crate::core::types::FfiTransform2D> for Transform2D {
    fn from(t: crate::core::types::FfiTransform2D) -> Self {
        Self {
            position: Vec2::new(t.position_x, t.position_y),
            rotation: t.rotation,
            scale: Vec2::new(t.scale_x, t.scale_y),
        }
    }
}

impl From<super::Mat3x3> for crate::core::types::FfiMat3x3 {
    fn from(m: super::Mat3x3) -> Self {
        Self { m: m.m }
    }
}
