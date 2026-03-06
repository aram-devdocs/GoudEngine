//! Core `Transform` struct and constructor methods.

use crate::core::math::Matrix4;
use crate::core::math::Vec3;
use crate::ecs::components::transform::quat::Quat;
use crate::ecs::Component;
use cgmath::Quaternion;

/// A 3D spatial transformation component.
///
/// Represents position, rotation, and scale in 3D space. This is the primary
/// component for positioning entities in the game world.
///
/// # Memory Layout
///
/// The component is laid out as:
/// - `position`: 3 x f32 (12 bytes)
/// - `rotation`: 4 x f32 (16 bytes) - quaternion (x, y, z, w)
/// - `scale`: 3 x f32 (12 bytes)
/// - Total: 40 bytes
///
/// # Example
///
/// ```
/// use goud_engine::ecs::components::Transform;
/// use goud_engine::core::math::Vec3;
///
/// // Create at origin with default rotation and scale
/// let mut transform = Transform::default();
///
/// // Or create with specific position
/// let transform = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));
///
/// // Or with full control
/// let transform = Transform::new(
///     Vec3::new(10.0, 5.0, 0.0),     // position
///     Transform::IDENTITY_ROTATION,   // rotation (identity quaternion)
///     Vec3::one(),                    // scale
/// );
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// Position in world space (or local space if entity has a parent).
    pub position: Vec3,
    /// Rotation as a quaternion (x, y, z, w).
    ///
    /// The quaternion should be normalized. Use the rotation methods to ensure
    /// this invariant is maintained.
    pub rotation: Quat,
    /// Scale along each axis.
    ///
    /// A scale of (1, 1, 1) means no scaling. Negative values flip the object
    /// along that axis.
    pub scale: Vec3,
}

impl Transform {
    /// Identity rotation (no rotation).
    pub const IDENTITY_ROTATION: Quat = Quat::IDENTITY;

    /// Creates a new Transform with the specified position, rotation, and scale.
    #[inline]
    pub const fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Creates a Transform at the specified position with default rotation and scale.
    #[inline]
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::one(),
        }
    }

    /// Creates a Transform with the specified rotation and default position/scale.
    #[inline]
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            position: Vec3::zero(),
            rotation,
            scale: Vec3::one(),
        }
    }

    /// Creates a Transform with the specified scale and default position/rotation.
    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Quat::IDENTITY,
            scale,
        }
    }

    /// Creates a Transform with uniform scale.
    #[inline]
    pub fn from_scale_uniform(scale: f32) -> Self {
        Self::from_scale(Vec3::new(scale, scale, scale))
    }

    /// Creates a Transform with position and rotation.
    #[inline]
    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            scale: Vec3::one(),
        }
    }

    /// Creates a Transform looking at a target position.
    ///
    /// # Arguments
    ///
    /// * `eye` - The position of the transform
    /// * `target` - The point to look at
    /// * `up` - The up direction (usually Vec3::unit_y())
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Transform;
    /// use goud_engine::core::math::Vec3;
    ///
    /// let transform = Transform::look_at(
    ///     Vec3::new(0.0, 5.0, 10.0),  // eye position
    ///     Vec3::zero(),               // looking at origin
    ///     Vec3::unit_y(),             // up direction
    /// );
    /// ```
    #[inline]
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        // Calculate the forward direction (from eye to target)
        let forward = (target - eye).normalize();
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);

        // We want the rotation that transforms:
        //   local -Z -> world forward
        //   local +X -> world right
        //   local +Y -> world up
        //
        // Since forward() returns rotate_vector(-Z), we need a rotation matrix
        // where column 2 (Z basis) is -forward (negated), so that when we negate
        // Z and rotate, we get forward.
        //
        // The rotation matrix M has columns [right, up, -forward] so that:
        //   M * [0,0,-1] = forward
        //   M * [1,0,0] = right
        //   M * [0,1,0] = up
        let neg_forward = Vec3::new(-forward.x, -forward.y, -forward.z);

        // Build rotation matrix columns: [right, up, -forward]
        let m00 = right.x;
        let m01 = up.x;
        let m02 = neg_forward.x;
        let m10 = right.y;
        let m11 = up.y;
        let m12 = neg_forward.y;
        let m20 = right.z;
        let m21 = up.z;
        let m22 = neg_forward.z;

        let trace = m00 + m11 + m22;
        let rotation = if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Quat::new((m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s)
        } else if m00 > m11 && m00 > m22 {
            let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
            Quat::new(0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s)
        } else if m11 > m22 {
            let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
            Quat::new((m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s)
        } else {
            let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
            Quat::new((m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s)
        };

        Self {
            position: eye,
            rotation: rotation.normalize(),
            scale: Vec3::one(),
        }
    }

    /// Computes the 4x4 transformation matrix.
    ///
    /// The matrix represents the combined transformation: Scale * Rotation * Translation
    /// (applied in that order when transforming points).
    #[inline]
    pub fn matrix(&self) -> Matrix4<f32> {
        let rotation: Quaternion<f32> = self.rotation.into();
        let rotation_matrix: cgmath::Matrix3<f32> = rotation.into();

        // Build transformation matrix: T * R * S
        // Scale
        let sx = self.scale.x;
        let sy = self.scale.y;
        let sz = self.scale.z;

        // Rotation columns scaled
        let r = rotation_matrix;

        Matrix4::new(
            r.x.x * sx,
            r.x.y * sx,
            r.x.z * sx,
            0.0,
            r.y.x * sy,
            r.y.y * sy,
            r.y.z * sy,
            0.0,
            r.z.x * sz,
            r.z.y * sz,
            r.z.z * sz,
            0.0,
            self.position.x,
            self.position.y,
            self.position.z,
            1.0,
        )
    }

    /// Computes the inverse transformation matrix.
    ///
    /// Useful for view matrices or converting world-space to local-space.
    #[inline]
    pub fn matrix_inverse(&self) -> Matrix4<f32> {
        // For a TRS matrix, inverse is: S^-1 * R^-1 * T^-1
        let inv_rotation = self.rotation.inverse();
        let inv_rotation_cg: Quaternion<f32> = inv_rotation.into();
        let inv_rotation_matrix: cgmath::Matrix3<f32> = inv_rotation_cg.into();

        let inv_scale = Vec3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);

        // Rotated and scaled inverse translation
        let inv_pos = inv_rotation.rotate_vector(-self.position);
        let inv_pos_scaled = Vec3::new(
            inv_pos.x * inv_scale.x,
            inv_pos.y * inv_scale.y,
            inv_pos.z * inv_scale.z,
        );

        let r = inv_rotation_matrix;

        Matrix4::new(
            r.x.x * inv_scale.x,
            r.x.y * inv_scale.x,
            r.x.z * inv_scale.x,
            0.0,
            r.y.x * inv_scale.y,
            r.y.y * inv_scale.y,
            r.y.z * inv_scale.y,
            0.0,
            r.z.x * inv_scale.z,
            r.z.y * inv_scale.z,
            r.z.z * inv_scale.z,
            0.0,
            inv_pos_scaled.x,
            inv_pos_scaled.y,
            inv_pos_scaled.z,
            1.0,
        )
    }
}

impl Default for Transform {
    /// Returns a Transform at the origin with no rotation and unit scale.
    #[inline]
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Quat::IDENTITY,
            scale: Vec3::one(),
        }
    }
}

// Implement Component trait for Transform
impl Component for Transform {}
