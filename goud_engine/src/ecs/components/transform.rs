//! Transform component for 3D spatial transformations.
//!
//! The [`Transform`] component represents an entity's position, rotation, and scale
//! in 3D space. It is one of the most commonly used components in game development,
//! essential for positioning and orienting objects in the world.
//!
//! # Design Philosophy
//!
//! Transform stores position, rotation (as quaternion), and scale separately rather
//! than as a combined matrix. This provides:
//!
//! - **Intuitive manipulation**: Modify position/rotation/scale independently
//! - **Numerical stability**: Quaternions avoid gimbal lock and numerical drift
//! - **Memory efficiency**: 10 floats vs 16 for a full matrix
//! - **Interpolation support**: Easy lerp/slerp for animations
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::components::Transform;
//! use goud_engine::core::math::Vec3;
//!
//! // Create a transform at position (10, 5, 0)
//! let mut transform = Transform::from_position(Vec3::new(10.0, 5.0, 0.0));
//!
//! // Modify the transform
//! transform.translate(Vec3::new(1.0, 0.0, 0.0));
//! transform.rotate_y(std::f32::consts::PI / 4.0);
//! transform.set_scale(Vec3::new(2.0, 2.0, 2.0));
//!
//! // Get the transformation matrix for rendering
//! let matrix = transform.matrix();
//! ```
//!
//! # Coordinate System
//!
//! GoudEngine uses a right-handed coordinate system:
//! - X axis: Right
//! - Y axis: Up
//! - Z axis: Out of the screen (towards viewer)
//!
//! Rotations follow the right-hand rule: positive rotation around an axis
//! goes counter-clockwise when looking down that axis.
//!
//! # FFI Safety
//!
//! Transform is `#[repr(C)]` and can be safely passed across FFI boundaries.
//! The quaternion is stored as (x, y, z, w) to match common conventions.

use crate::core::math::{Matrix4, Quaternion, Vec3};
use crate::ecs::Component;
use cgmath::InnerSpace;
use std::f32::consts::PI;

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

/// A quaternion for representing rotations.
///
/// Stored as (x, y, z, w) where:
/// - (x, y, z) is the vector part (imaginary components)
/// - w is the scalar part (real component)
///
/// The identity rotation is (0, 0, 0, 1).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    /// The x-component of the vector part.
    pub x: f32,
    /// The y-component of the vector part.
    pub y: f32,
    /// The z-component of the vector part.
    pub z: f32,
    /// The scalar (real) part.
    pub w: f32,
}

impl Quat {
    /// Identity quaternion representing no rotation.
    pub const IDENTITY: Quat = Quat {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    /// Creates a new quaternion from components.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Creates a quaternion from axis-angle representation.
    ///
    /// # Arguments
    ///
    /// * `axis` - The normalized rotation axis
    /// * `angle` - The rotation angle in radians
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::transform::Quat;
    /// use goud_engine::core::math::Vec3;
    /// use std::f32::consts::PI;
    ///
    /// // Rotate 90 degrees around Y axis
    /// let q = Quat::from_axis_angle(Vec3::unit_y(), PI / 2.0);
    /// ```
    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let s = half_angle.sin();
        let c = half_angle.cos();
        Self {
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
            w: c,
        }
    }

    /// Creates a quaternion from Euler angles (in radians).
    ///
    /// Uses XYZ rotation order (pitch, yaw, roll):
    /// - X: pitch (rotation around right axis)
    /// - Y: yaw (rotation around up axis)
    /// - Z: roll (rotation around forward axis)
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::transform::Quat;
    /// use std::f32::consts::PI;
    ///
    /// // 45 degree yaw rotation
    /// let q = Quat::from_euler(0.0, PI / 4.0, 0.0);
    /// ```
    #[inline]
    pub fn from_euler(pitch: f32, yaw: f32, roll: f32) -> Self {
        // Using XYZ convention
        let (sx, cx) = (pitch * 0.5).sin_cos();
        let (sy, cy) = (yaw * 0.5).sin_cos();
        let (sz, cz) = (roll * 0.5).sin_cos();

        Self {
            x: sx * cy * cz + cx * sy * sz,
            y: cx * sy * cz - sx * cy * sz,
            z: cx * cy * sz + sx * sy * cz,
            w: cx * cy * cz - sx * sy * sz,
        }
    }

    /// Creates a quaternion that rotates from one direction to another.
    ///
    /// Both vectors should be normalized.
    #[inline]
    pub fn from_rotation_arc(from: Vec3, to: Vec3) -> Self {
        let from_cg: cgmath::Vector3<f32> = from.into();
        let to_cg: cgmath::Vector3<f32> = to.into();

        // Handle edge case where vectors are opposite
        let dot = from_cg.dot(to_cg);
        if dot < -0.999999 {
            // Vectors are opposite, rotate 180 degrees around any perpendicular axis
            let mut axis = cgmath::Vector3::unit_x().cross(from_cg);
            if axis.magnitude2() < 0.000001 {
                axis = cgmath::Vector3::unit_y().cross(from_cg);
            }
            axis = axis.normalize();
            return Self::from_axis_angle(Vec3::from(axis), PI);
        }

        let cross = from_cg.cross(to_cg);
        Self {
            x: cross.x,
            y: cross.y,
            z: cross.z,
            w: 1.0 + dot,
        }
        .normalize()
    }

    /// Returns the squared length of the quaternion.
    #[inline]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// Returns the length of the quaternion.
    #[inline]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Returns a normalized version of this quaternion.
    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::IDENTITY
        } else {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
                w: self.w / len,
            }
        }
    }

    /// Returns the conjugate of this quaternion.
    ///
    /// For a unit quaternion, the conjugate is also the inverse.
    #[inline]
    pub fn conjugate(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    /// Returns the inverse of this quaternion.
    ///
    /// For unit quaternions (normalized), this is equivalent to the conjugate.
    #[inline]
    pub fn inverse(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq == 0.0 {
            Self::IDENTITY
        } else {
            let conj = self.conjugate();
            Self {
                x: conj.x / len_sq,
                y: conj.y / len_sq,
                z: conj.z / len_sq,
                w: conj.w / len_sq,
            }
        }
    }

    /// Multiplies two quaternions (combines rotations).
    ///
    /// This represents combining two rotations, where `self` is applied first,
    /// then `other`. This is also available via the `*` operator.
    #[inline]
    pub fn multiply(self, other: Self) -> Self {
        self * other
    }

    /// Rotates a vector by this quaternion.
    #[inline]
    pub fn rotate_vector(self, v: Vec3) -> Vec3 {
        // q * v * q^-1, optimized formula
        let qv = Vec3::new(self.x, self.y, self.z);
        let uv = qv.cross(v);
        let uuv = qv.cross(uv);
        v + (uv * self.w + uuv) * 2.0
    }

    /// Spherical linear interpolation between two quaternions.
    ///
    /// This provides smooth interpolation between rotations.
    #[inline]
    pub fn slerp(self, other: Self, t: f32) -> Self {
        let dot = self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;

        // If the dot product is negative, negate one quaternion to take shorter path
        let (other, dot) = if dot < 0.0 {
            (
                Self {
                    x: -other.x,
                    y: -other.y,
                    z: -other.z,
                    w: -other.w,
                },
                -dot,
            )
        } else {
            (other, dot)
        };

        // If quaternions are very close, use linear interpolation
        if dot > 0.9995 {
            return Self {
                x: self.x + t * (other.x - self.x),
                y: self.y + t * (other.y - self.y),
                z: self.z + t * (other.z - self.z),
                w: self.w + t * (other.w - self.w),
            }
            .normalize();
        }

        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();

        let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;

        Self {
            x: self.x * s0 + other.x * s1,
            y: self.y * s0 + other.y * s1,
            z: self.z * s0 + other.z * s1,
            w: self.w * s0 + other.w * s1,
        }
    }

    /// Converts the quaternion to Euler angles (in radians).
    ///
    /// Returns (pitch, yaw, roll) using XYZ convention.
    #[inline]
    pub fn to_euler(self) -> (f32, f32, f32) {
        // Roll (x-axis rotation)
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        // Pitch (y-axis rotation)
        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            (PI / 2.0).copysign(sinp)
        } else {
            sinp.asin()
        };

        // Yaw (z-axis rotation)
        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        (pitch, yaw, roll)
    }

    /// Returns the forward direction vector (negative Z axis after rotation).
    #[inline]
    pub fn forward(self) -> Vec3 {
        self.rotate_vector(Vec3::new(0.0, 0.0, -1.0))
    }

    /// Returns the right direction vector (positive X axis after rotation).
    #[inline]
    pub fn right(self) -> Vec3 {
        self.rotate_vector(Vec3::new(1.0, 0.0, 0.0))
    }

    /// Returns the up direction vector (positive Y axis after rotation).
    #[inline]
    pub fn up(self) -> Vec3 {
        self.rotate_vector(Vec3::new(0.0, 1.0, 0.0))
    }
}

impl Default for Quat {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

// Implement Mul trait for quaternion multiplication
impl std::ops::Mul for Quat {
    type Output = Self;

    /// Multiplies two quaternions, combining their rotations.
    ///
    /// `self * other` applies `self` first, then `other`.
    #[inline]
    fn mul(self, other: Self) -> Self {
        Self {
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        }
    }
}

// Conversion to/from cgmath quaternion
impl From<Quaternion<f32>> for Quat {
    #[inline]
    fn from(q: Quaternion<f32>) -> Self {
        Self {
            x: q.v.x,
            y: q.v.y,
            z: q.v.z,
            w: q.s,
        }
    }
}

impl From<Quat> for Quaternion<f32> {
    #[inline]
    fn from(q: Quat) -> Self {
        Quaternion::new(q.w, q.x, q.y, q.z)
    }
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

    // =========================================================================
    // Position Methods
    // =========================================================================

    /// Translates the transform by the given offset.
    #[inline]
    pub fn translate(&mut self, offset: Vec3) {
        self.position = self.position + offset;
    }

    /// Translates the transform in local space.
    ///
    /// The offset is rotated by the transform's rotation before being applied.
    #[inline]
    pub fn translate_local(&mut self, offset: Vec3) {
        let world_offset = self.rotation.rotate_vector(offset);
        self.position = self.position + world_offset;
    }

    /// Sets the position of the transform.
    #[inline]
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    // =========================================================================
    // Rotation Methods
    // =========================================================================

    /// Rotates the transform by the given quaternion.
    ///
    /// The rotation is applied in world space (rotation is applied after the current rotation).
    #[inline]
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = (self.rotation * rotation).normalize();
    }

    /// Rotates around the X axis by the given angle in radians.
    #[inline]
    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate(Quat::from_axis_angle(Vec3::unit_x(), angle));
    }

    /// Rotates around the Y axis by the given angle in radians.
    #[inline]
    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate(Quat::from_axis_angle(Vec3::unit_y(), angle));
    }

    /// Rotates around the Z axis by the given angle in radians.
    #[inline]
    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate(Quat::from_axis_angle(Vec3::unit_z(), angle));
    }

    /// Rotates around an arbitrary axis by the given angle in radians.
    #[inline]
    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        self.rotate(Quat::from_axis_angle(axis.normalize(), angle));
    }

    /// Rotates in local space around the X axis.
    #[inline]
    pub fn rotate_local_x(&mut self, angle: f32) {
        let local_rotation = Quat::from_axis_angle(Vec3::unit_x(), angle);
        self.rotation = (self.rotation * local_rotation).normalize();
    }

    /// Rotates in local space around the Y axis.
    #[inline]
    pub fn rotate_local_y(&mut self, angle: f32) {
        let local_rotation = Quat::from_axis_angle(Vec3::unit_y(), angle);
        self.rotation = (self.rotation * local_rotation).normalize();
    }

    /// Rotates in local space around the Z axis.
    #[inline]
    pub fn rotate_local_z(&mut self, angle: f32) {
        let local_rotation = Quat::from_axis_angle(Vec3::unit_z(), angle);
        self.rotation = (self.rotation * local_rotation).normalize();
    }

    /// Sets the rotation from Euler angles (in radians).
    #[inline]
    pub fn set_rotation_euler(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.rotation = Quat::from_euler(pitch, yaw, roll);
    }

    /// Sets the rotation.
    #[inline]
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation.normalize();
    }

    /// Makes the transform look at a target position.
    #[inline]
    pub fn look_at_target(&mut self, target: Vec3, up: Vec3) {
        let looking = Transform::look_at(self.position, target, up);
        self.rotation = looking.rotation;
    }

    // =========================================================================
    // Scale Methods
    // =========================================================================

    /// Sets the scale of the transform.
    #[inline]
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
    }

    /// Sets uniform scale on all axes.
    #[inline]
    pub fn set_scale_uniform(&mut self, scale: f32) {
        self.scale = Vec3::new(scale, scale, scale);
    }

    /// Multiplies the current scale by the given factors.
    #[inline]
    pub fn scale_by(&mut self, factors: Vec3) {
        self.scale = Vec3::new(
            self.scale.x * factors.x,
            self.scale.y * factors.y,
            self.scale.z * factors.z,
        );
    }

    // =========================================================================
    // Direction Vectors
    // =========================================================================

    /// Returns the forward direction vector (negative Z in local space).
    #[inline]
    pub fn forward(&self) -> Vec3 {
        self.rotation.forward()
    }

    /// Returns the right direction vector (positive X in local space).
    #[inline]
    pub fn right(&self) -> Vec3 {
        self.rotation.right()
    }

    /// Returns the up direction vector (positive Y in local space).
    #[inline]
    pub fn up(&self) -> Vec3 {
        self.rotation.up()
    }

    /// Returns the back direction vector (positive Z in local space).
    #[inline]
    pub fn back(&self) -> Vec3 {
        self.rotation.rotate_vector(Vec3::new(0.0, 0.0, 1.0))
    }

    /// Returns the left direction vector (negative X in local space).
    #[inline]
    pub fn left(&self) -> Vec3 {
        self.rotation.rotate_vector(Vec3::new(-1.0, 0.0, 0.0))
    }

    /// Returns the down direction vector (negative Y in local space).
    #[inline]
    pub fn down(&self) -> Vec3 {
        self.rotation.rotate_vector(Vec3::new(0.0, -1.0, 0.0))
    }

    // =========================================================================
    // Matrix Generation
    // =========================================================================

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

    // =========================================================================
    // Point Transformation
    // =========================================================================

    /// Transforms a point from local space to world space.
    #[inline]
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        let scaled = Vec3::new(
            point.x * self.scale.x,
            point.y * self.scale.y,
            point.z * self.scale.z,
        );
        let rotated = self.rotation.rotate_vector(scaled);
        rotated + self.position
    }

    /// Transforms a direction from local space to world space.
    ///
    /// Unlike points, directions are not affected by translation.
    #[inline]
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.rotation.rotate_vector(direction)
    }

    /// Transforms a point from world space to local space.
    #[inline]
    pub fn inverse_transform_point(&self, point: Vec3) -> Vec3 {
        let translated = point - self.position;
        let inv_rotation = self.rotation.inverse();
        let rotated = inv_rotation.rotate_vector(translated);
        Vec3::new(
            rotated.x / self.scale.x,
            rotated.y / self.scale.y,
            rotated.z / self.scale.z,
        )
    }

    /// Transforms a direction from world space to local space.
    #[inline]
    pub fn inverse_transform_direction(&self, direction: Vec3) -> Vec3 {
        let inv_rotation = self.rotation.inverse();
        inv_rotation.rotate_vector(direction)
    }

    // =========================================================================
    // Interpolation
    // =========================================================================

    /// Linearly interpolates between two transforms.
    ///
    /// Position and scale are linearly interpolated, rotation uses slerp.
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;
    use std::f32::consts::FRAC_PI_4;

    // =========================================================================
    // Quat Tests
    // =========================================================================

    mod quat_tests {
        use super::*;

        #[test]
        fn test_quat_identity() {
            let q = Quat::IDENTITY;
            assert_eq!(q.x, 0.0);
            assert_eq!(q.y, 0.0);
            assert_eq!(q.z, 0.0);
            assert_eq!(q.w, 1.0);
        }

        #[test]
        fn test_quat_new() {
            let q = Quat::new(1.0, 2.0, 3.0, 4.0);
            assert_eq!(q.x, 1.0);
            assert_eq!(q.y, 2.0);
            assert_eq!(q.z, 3.0);
            assert_eq!(q.w, 4.0);
        }

        #[test]
        fn test_quat_from_axis_angle() {
            let q = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2);
            assert!((q.length() - 1.0).abs() < 0.0001);

            // Rotating unit_z by 90 degrees around Y should give unit_x
            let rotated = q.rotate_vector(Vec3::unit_z());
            assert!((rotated.x - 1.0).abs() < 0.0001);
            assert!(rotated.y.abs() < 0.0001);
            assert!(rotated.z.abs() < 0.0001);
        }

        #[test]
        fn test_quat_from_euler() {
            // 90 degree yaw rotation
            let q = Quat::from_euler(0.0, FRAC_PI_2, 0.0);
            assert!((q.length() - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_quat_normalize() {
            let q = Quat::new(1.0, 2.0, 3.0, 4.0);
            let n = q.normalize();
            assert!((n.length() - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_quat_conjugate() {
            let q = Quat::new(1.0, 2.0, 3.0, 4.0);
            let c = q.conjugate();
            assert_eq!(c.x, -1.0);
            assert_eq!(c.y, -2.0);
            assert_eq!(c.z, -3.0);
            assert_eq!(c.w, 4.0);
        }

        #[test]
        fn test_quat_mul_identity() {
            let q = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
            let result = q * Quat::IDENTITY;
            assert!((result.x - q.x).abs() < 0.0001);
            assert!((result.y - q.y).abs() < 0.0001);
            assert!((result.z - q.z).abs() < 0.0001);
            assert!((result.w - q.w).abs() < 0.0001);
        }

        #[test]
        fn test_quat_rotate_vector() {
            let q = Quat::from_axis_angle(Vec3::unit_y(), PI);
            let v = Vec3::new(1.0, 0.0, 0.0);
            let rotated = q.rotate_vector(v);
            // 180 degree rotation around Y should negate X
            assert!((rotated.x - (-1.0)).abs() < 0.0001);
            assert!(rotated.y.abs() < 0.0001);
            assert!(rotated.z.abs() < 0.0001);
        }

        #[test]
        fn test_quat_slerp() {
            // Test slerp with a smaller rotation (avoid 180-degree edge case)
            let a = Quat::IDENTITY;
            let b = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2);
            let mid = a.slerp(b, 0.5);
            // Midpoint should be 45 degrees
            let expected = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
            // Compare quaternion components (accounting for sign flip equivalence)
            let dot =
                mid.x * expected.x + mid.y * expected.y + mid.z * expected.z + mid.w * expected.w;
            assert!(
                dot.abs() > 0.999,
                "slerp midpoint should represent same rotation"
            );
        }

        #[test]
        fn test_quat_directions() {
            let q = Quat::IDENTITY;
            let fwd = q.forward();
            let right = q.right();
            let up = q.up();

            assert!((fwd - Vec3::new(0.0, 0.0, -1.0)).length() < 0.0001);
            assert!((right - Vec3::new(1.0, 0.0, 0.0)).length() < 0.0001);
            assert!((up - Vec3::new(0.0, 1.0, 0.0)).length() < 0.0001);
        }

        #[test]
        fn test_quat_default() {
            let q = Quat::default();
            assert_eq!(q, Quat::IDENTITY);
        }

        #[test]
        fn test_quat_cgmath_conversion() {
            let our_quat = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
            let cg_quat: Quaternion<f32> = our_quat.into();
            let back: Quat = cg_quat.into();

            assert!((back.x - our_quat.x).abs() < 0.0001);
            assert!((back.y - our_quat.y).abs() < 0.0001);
            assert!((back.z - our_quat.z).abs() < 0.0001);
            assert!((back.w - our_quat.w).abs() < 0.0001);
        }

        #[test]
        fn test_quat_inverse() {
            let q = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4).normalize();
            let inv = q.inverse();
            let result = q * inv;
            // q * q^-1 should be identity
            assert!((result.x).abs() < 0.0001);
            assert!((result.y).abs() < 0.0001);
            assert!((result.z).abs() < 0.0001);
            assert!((result.w - 1.0).abs() < 0.0001);
        }
    }

    // =========================================================================
    // Transform Construction Tests
    // =========================================================================

    mod construction_tests {
        use super::*;

        #[test]
        fn test_transform_default() {
            let t = Transform::default();
            assert_eq!(t.position, Vec3::zero());
            assert_eq!(t.rotation, Quat::IDENTITY);
            assert_eq!(t.scale, Vec3::one());
        }

        #[test]
        fn test_transform_new() {
            let pos = Vec3::new(1.0, 2.0, 3.0);
            let rot = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
            let scale = Vec3::new(2.0, 2.0, 2.0);

            let t = Transform::new(pos, rot, scale);
            assert_eq!(t.position, pos);
            assert_eq!(t.rotation, rot);
            assert_eq!(t.scale, scale);
        }

        #[test]
        fn test_transform_from_position() {
            let pos = Vec3::new(10.0, 20.0, 30.0);
            let t = Transform::from_position(pos);
            assert_eq!(t.position, pos);
            assert_eq!(t.rotation, Quat::IDENTITY);
            assert_eq!(t.scale, Vec3::one());
        }

        #[test]
        fn test_transform_from_rotation() {
            let rot = Quat::from_axis_angle(Vec3::unit_x(), FRAC_PI_2);
            let t = Transform::from_rotation(rot);
            assert_eq!(t.position, Vec3::zero());
            assert_eq!(t.rotation, rot);
            assert_eq!(t.scale, Vec3::one());
        }

        #[test]
        fn test_transform_from_scale() {
            let scale = Vec3::new(2.0, 3.0, 4.0);
            let t = Transform::from_scale(scale);
            assert_eq!(t.position, Vec3::zero());
            assert_eq!(t.rotation, Quat::IDENTITY);
            assert_eq!(t.scale, scale);
        }

        #[test]
        fn test_transform_from_scale_uniform() {
            let t = Transform::from_scale_uniform(5.0);
            assert_eq!(t.scale, Vec3::new(5.0, 5.0, 5.0));
        }

        #[test]
        fn test_transform_from_position_rotation() {
            let pos = Vec3::new(1.0, 2.0, 3.0);
            let rot = Quat::from_axis_angle(Vec3::unit_z(), FRAC_PI_4);
            let t = Transform::from_position_rotation(pos, rot);
            assert_eq!(t.position, pos);
            assert_eq!(t.rotation, rot);
            assert_eq!(t.scale, Vec3::one());
        }

        #[test]
        fn test_transform_look_at() {
            let eye = Vec3::new(0.0, 0.0, 10.0);
            let target = Vec3::zero();
            let up = Vec3::unit_y();

            let t = Transform::look_at(eye, target, up);
            assert_eq!(t.position, eye);

            // Forward direction should point towards target
            let fwd = t.forward();
            let expected_fwd = (target - eye).normalize();
            assert!((fwd - expected_fwd).length() < 0.01);
        }
    }

    // =========================================================================
    // Transform Mutation Tests
    // =========================================================================

    mod mutation_tests {
        use super::*;

        #[test]
        fn test_translate() {
            let mut t = Transform::default();
            t.translate(Vec3::new(5.0, 0.0, 0.0));
            assert_eq!(t.position, Vec3::new(5.0, 0.0, 0.0));

            t.translate(Vec3::new(0.0, 3.0, 0.0));
            assert_eq!(t.position, Vec3::new(5.0, 3.0, 0.0));
        }

        #[test]
        fn test_translate_local() {
            let mut t = Transform::default();
            // Rotate 90 degrees around Y, so local X becomes world Z
            t.rotate_y(FRAC_PI_2);
            t.translate_local(Vec3::new(1.0, 0.0, 0.0));

            // Local X (1,0,0) should become world Z direction after 90 degree Y rotation
            assert!(t.position.x.abs() < 0.0001);
            assert!(t.position.y.abs() < 0.0001);
            assert!((t.position.z - (-1.0)).abs() < 0.0001);
        }

        #[test]
        fn test_set_position() {
            let mut t = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));
            t.set_position(Vec3::new(10.0, 20.0, 30.0));
            assert_eq!(t.position, Vec3::new(10.0, 20.0, 30.0));
        }

        #[test]
        fn test_rotate_x() {
            let mut t = Transform::default();
            t.rotate_x(FRAC_PI_2);

            // After +90 degree X rotation (counter-clockwise looking down X),
            // up (Y) should rotate towards +Z
            let up = t.up();
            assert!(up.x.abs() < 0.0001);
            assert!(up.y.abs() < 0.0001);
            assert!((up.z - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_rotate_y() {
            let mut t = Transform::default();
            t.rotate_y(FRAC_PI_2);

            // After +90 degree Y rotation (counter-clockwise looking down Y),
            // forward (-Z) should rotate towards -X
            let fwd = t.forward();
            assert!((fwd.x - (-1.0)).abs() < 0.0001);
            assert!(fwd.y.abs() < 0.0001);
            assert!(fwd.z.abs() < 0.0001);
        }

        #[test]
        fn test_rotate_z() {
            let mut t = Transform::default();
            t.rotate_z(FRAC_PI_2);

            // After 90 degree Z rotation, right (X) should become up (Y)
            let right = t.right();
            assert!(right.x.abs() < 0.0001);
            assert!((right.y - 1.0).abs() < 0.0001);
            assert!(right.z.abs() < 0.0001);
        }

        #[test]
        fn test_set_rotation() {
            let mut t = Transform::default();
            let rot = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
            t.set_rotation(rot);
            assert!((t.rotation.x - rot.x).abs() < 0.0001);
            assert!((t.rotation.y - rot.y).abs() < 0.0001);
            assert!((t.rotation.z - rot.z).abs() < 0.0001);
            assert!((t.rotation.w - rot.w).abs() < 0.0001);
        }

        #[test]
        fn test_set_scale() {
            let mut t = Transform::default();
            t.set_scale(Vec3::new(2.0, 3.0, 4.0));
            assert_eq!(t.scale, Vec3::new(2.0, 3.0, 4.0));
        }

        #[test]
        fn test_set_scale_uniform() {
            let mut t = Transform::default();
            t.set_scale_uniform(3.0);
            assert_eq!(t.scale, Vec3::new(3.0, 3.0, 3.0));
        }

        #[test]
        fn test_scale_by() {
            let mut t = Transform::from_scale(Vec3::new(2.0, 2.0, 2.0));
            t.scale_by(Vec3::new(3.0, 4.0, 5.0));
            assert_eq!(t.scale, Vec3::new(6.0, 8.0, 10.0));
        }

        #[test]
        fn test_look_at_target() {
            let mut t = Transform::from_position(Vec3::new(0.0, 0.0, 10.0));
            t.look_at_target(Vec3::zero(), Vec3::unit_y());

            let fwd = t.forward();
            let expected = Vec3::new(0.0, 0.0, -1.0);
            assert!((fwd - expected).length() < 0.01);
        }
    }

    // =========================================================================
    // Transform Direction Tests
    // =========================================================================

    mod direction_tests {
        use super::*;

        #[test]
        fn test_directions_identity() {
            let t = Transform::default();

            assert!((t.forward() - Vec3::new(0.0, 0.0, -1.0)).length() < 0.0001);
            assert!((t.back() - Vec3::new(0.0, 0.0, 1.0)).length() < 0.0001);
            assert!((t.right() - Vec3::new(1.0, 0.0, 0.0)).length() < 0.0001);
            assert!((t.left() - Vec3::new(-1.0, 0.0, 0.0)).length() < 0.0001);
            assert!((t.up() - Vec3::new(0.0, 1.0, 0.0)).length() < 0.0001);
            assert!((t.down() - Vec3::new(0.0, -1.0, 0.0)).length() < 0.0001);
        }

        #[test]
        fn test_directions_rotated() {
            let mut t = Transform::default();
            t.rotate_y(FRAC_PI_2);

            // After +90 degree Y rotation (counter-clockwise looking down Y):
            // forward (-Z) -> -X
            // right (X) -> -Z
            let fwd = t.forward();
            assert!((fwd.x - (-1.0)).abs() < 0.0001);

            let right = t.right();
            assert!((right.z - (-1.0)).abs() < 0.0001);
        }
    }

    // =========================================================================
    // Matrix Tests
    // =========================================================================

    mod matrix_tests {
        use super::*;

        #[test]
        fn test_matrix_identity() {
            let t = Transform::default();
            let m = t.matrix();

            // Identity transform should produce identity matrix
            assert!((m.x.x - 1.0).abs() < 0.0001);
            assert!((m.y.y - 1.0).abs() < 0.0001);
            assert!((m.z.z - 1.0).abs() < 0.0001);
            assert!((m.w.w - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_matrix_translation() {
            let t = Transform::from_position(Vec3::new(10.0, 20.0, 30.0));
            let m = t.matrix();

            // Translation should be in the last column
            assert!((m.w.x - 10.0).abs() < 0.0001);
            assert!((m.w.y - 20.0).abs() < 0.0001);
            assert!((m.w.z - 30.0).abs() < 0.0001);
        }

        #[test]
        fn test_matrix_scale() {
            let t = Transform::from_scale(Vec3::new(2.0, 3.0, 4.0));
            let m = t.matrix();

            // Scale should affect diagonal elements
            assert!((m.x.x - 2.0).abs() < 0.0001);
            assert!((m.y.y - 3.0).abs() < 0.0001);
            assert!((m.z.z - 4.0).abs() < 0.0001);
        }

        #[test]
        fn test_matrix_inverse() {
            let t = Transform::new(
                Vec3::new(5.0, 10.0, 15.0),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4),
                Vec3::new(2.0, 2.0, 2.0),
            );

            let m = t.matrix();
            let m_inv = t.matrix_inverse();

            // M * M^-1 should be identity
            let identity = m * m_inv;

            assert!((identity.x.x - 1.0).abs() < 0.001);
            assert!((identity.y.y - 1.0).abs() < 0.001);
            assert!((identity.z.z - 1.0).abs() < 0.001);
            assert!((identity.w.w - 1.0).abs() < 0.001);
        }
    }

    // =========================================================================
    // Point Transformation Tests
    // =========================================================================

    mod point_transform_tests {
        use super::*;

        #[test]
        fn test_transform_point_translation() {
            let t = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));
            let p = Vec3::zero();
            let transformed = t.transform_point(p);
            assert_eq!(transformed, Vec3::new(10.0, 0.0, 0.0));
        }

        #[test]
        fn test_transform_point_scale() {
            let t = Transform::from_scale(Vec3::new(2.0, 2.0, 2.0));
            let p = Vec3::new(5.0, 5.0, 5.0);
            let transformed = t.transform_point(p);
            assert_eq!(transformed, Vec3::new(10.0, 10.0, 10.0));
        }

        #[test]
        fn test_transform_point_rotation() {
            let t = Transform::from_rotation(Quat::from_axis_angle(Vec3::unit_y(), PI));
            let p = Vec3::new(1.0, 0.0, 0.0);
            let transformed = t.transform_point(p);
            // 180 degree rotation should negate X
            assert!((transformed.x - (-1.0)).abs() < 0.0001);
            assert!(transformed.y.abs() < 0.0001);
            assert!(transformed.z.abs() < 0.0001);
        }

        #[test]
        fn test_transform_direction() {
            let t = Transform::new(
                Vec3::new(100.0, 0.0, 0.0), // Translation should not affect direction
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2),
                Vec3::one(),
            );

            let dir = Vec3::new(0.0, 0.0, 1.0);
            let transformed = t.transform_direction(dir);

            // After +90 degree Y rotation (counter-clockwise looking down Y),
            // +Z direction should rotate towards +X
            assert!((transformed.x - 1.0).abs() < 0.0001);
            assert!(transformed.y.abs() < 0.0001);
            assert!(transformed.z.abs() < 0.0001);
        }

        #[test]
        fn test_inverse_transform_point() {
            let t = Transform::new(
                Vec3::new(10.0, 20.0, 30.0),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4),
                Vec3::new(2.0, 2.0, 2.0),
            );

            let world_point = Vec3::new(5.0, 5.0, 5.0);
            let local = t.inverse_transform_point(world_point);
            let back_to_world = t.transform_point(local);

            assert!((back_to_world - world_point).length() < 0.001);
        }

        #[test]
        fn test_inverse_transform_direction() {
            let t = Transform::from_rotation(Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2));

            let world_dir = Vec3::new(1.0, 0.0, 0.0);
            let local = t.inverse_transform_direction(world_dir);
            let back = t.transform_direction(local);

            assert!((back - world_dir).length() < 0.0001);
        }
    }

    // =========================================================================
    // Interpolation Tests
    // =========================================================================

    mod interpolation_tests {
        use super::*;

        #[test]
        fn test_lerp_position() {
            let a = Transform::from_position(Vec3::zero());
            let b = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));

            let mid = a.lerp(b, 0.5);
            assert_eq!(mid.position, Vec3::new(5.0, 0.0, 0.0));
        }

        #[test]
        fn test_lerp_scale() {
            let a = Transform::from_scale(Vec3::one());
            let b = Transform::from_scale(Vec3::new(3.0, 3.0, 3.0));

            let mid = a.lerp(b, 0.5);
            assert_eq!(mid.scale, Vec3::new(2.0, 2.0, 2.0));
        }

        #[test]
        fn test_lerp_rotation() {
            // Test lerp with a smaller rotation (avoid 180-degree edge case)
            let a = Transform::default();
            let b = Transform::from_rotation(Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_2));

            let mid = a.lerp(b, 0.5);
            // Midpoint should be 45 degrees
            let expected = Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4);
            // Compare using dot product (handles sign flip)
            let dot = mid.rotation.x * expected.x
                + mid.rotation.y * expected.y
                + mid.rotation.z * expected.z
                + mid.rotation.w * expected.w;
            assert!(
                dot.abs() > 0.999,
                "lerp midpoint rotation should match expected"
            );
        }

        #[test]
        fn test_lerp_endpoints() {
            let a = Transform::new(
                Vec3::new(0.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::new(1.0, 1.0, 1.0),
            );
            let b = Transform::new(
                Vec3::new(10.0, 10.0, 10.0),
                Quat::from_axis_angle(Vec3::unit_y(), PI),
                Vec3::new(2.0, 2.0, 2.0),
            );

            let start = a.lerp(b, 0.0);
            assert_eq!(start.position, a.position);
            assert_eq!(start.scale, a.scale);

            let end = a.lerp(b, 1.0);
            assert_eq!(end.position, b.position);
            assert_eq!(end.scale, b.scale);
        }
    }

    // =========================================================================
    // Component Trait Tests
    // =========================================================================

    mod component_tests {
        use super::*;

        #[test]
        fn test_transform_is_component() {
            fn assert_component<T: Component>() {}
            assert_component::<Transform>();
        }

        #[test]
        fn test_transform_is_send() {
            fn assert_send<T: Send>() {}
            assert_send::<Transform>();
        }

        #[test]
        fn test_transform_is_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<Transform>();
        }

        #[test]
        fn test_transform_clone() {
            let t = Transform::new(
                Vec3::new(1.0, 2.0, 3.0),
                Quat::from_axis_angle(Vec3::unit_y(), FRAC_PI_4),
                Vec3::new(2.0, 2.0, 2.0),
            );
            let cloned = t.clone();
            assert_eq!(t, cloned);
        }

        #[test]
        fn test_transform_copy() {
            let t = Transform::default();
            let copied = t;
            assert_eq!(t, copied);
        }
    }

    // =========================================================================
    // FFI Layout Tests
    // =========================================================================

    mod ffi_tests {
        use super::*;
        use std::mem::{align_of, size_of};

        #[test]
        fn test_quat_size() {
            assert_eq!(size_of::<Quat>(), 16); // 4 * f32
        }

        #[test]
        fn test_quat_align() {
            assert_eq!(align_of::<Quat>(), 4); // f32 alignment
        }

        #[test]
        fn test_transform_size() {
            // Vec3 (12) + Quat (16) + Vec3 (12) = 40 bytes
            assert_eq!(size_of::<Transform>(), 40);
        }

        #[test]
        fn test_transform_align() {
            assert_eq!(align_of::<Transform>(), 4); // f32 alignment
        }

        #[test]
        fn test_quat_field_layout() {
            let q = Quat::new(1.0, 2.0, 3.0, 4.0);
            let ptr = &q as *const Quat as *const f32;
            unsafe {
                assert_eq!(*ptr, 1.0);
                assert_eq!(*ptr.add(1), 2.0);
                assert_eq!(*ptr.add(2), 3.0);
                assert_eq!(*ptr.add(3), 4.0);
            }
        }
    }
}
