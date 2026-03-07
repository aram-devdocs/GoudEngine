//! Quaternion type for representing 3D rotations.

use crate::core::math::{Quaternion, Vec3};
use cgmath::InnerSpace;
use std::f32::consts::PI;

/// A quaternion for representing rotations.
///
/// Stored as (x, y, z, w) where:
/// - (x, y, z) is the vector part (imaginary components)
/// - w is the scalar part (real component)
///
/// The identity rotation is (0, 0, 0, 1).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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
