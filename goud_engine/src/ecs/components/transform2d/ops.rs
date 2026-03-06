//! Operations and methods for [`Transform2D`].
//!
//! Provides mutation (translate, rotate, scale), direction queries,
//! matrix generation, point/direction transforms, and interpolation.

use std::f32::consts::PI;

use crate::core::math::Vec2;

use super::mat3x3::Mat3x3;
use super::types::Transform2D;

impl Transform2D {
    // =========================================================================
    // Position Methods
    // =========================================================================

    /// Translates the transform by the given offset in world space.
    #[inline]
    pub fn translate(&mut self, offset: Vec2) {
        self.position = self.position + offset;
    }

    /// Translates the transform in local space.
    ///
    /// The offset is rotated by the transform's rotation before being applied.
    #[inline]
    pub fn translate_local(&mut self, offset: Vec2) {
        let (sin, cos) = self.rotation.sin_cos();
        let rotated = Vec2::new(
            offset.x * cos - offset.y * sin,
            offset.x * sin + offset.y * cos,
        );
        self.position = self.position + rotated;
    }

    /// Sets the position of the transform.
    #[inline]
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    // =========================================================================
    // Rotation Methods
    // =========================================================================

    /// Rotates the transform by the given angle in radians.
    #[inline]
    pub fn rotate(&mut self, angle: f32) {
        self.rotation = normalize_angle(self.rotation + angle);
    }

    /// Rotates the transform by the given angle in degrees.
    #[inline]
    pub fn rotate_degrees(&mut self, degrees: f32) {
        self.rotate(degrees.to_radians());
    }

    /// Sets the rotation angle in radians.
    #[inline]
    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = normalize_angle(rotation);
    }

    /// Sets the rotation angle in degrees.
    #[inline]
    pub fn set_rotation_degrees(&mut self, degrees: f32) {
        self.set_rotation(degrees.to_radians());
    }

    /// Returns the rotation angle in degrees.
    #[inline]
    pub fn rotation_degrees(&self) -> f32 {
        self.rotation.to_degrees()
    }

    /// Makes the transform look at a target position.
    #[inline]
    pub fn look_at_target(&mut self, target: Vec2) {
        let direction = target - self.position;
        self.rotation = direction.y.atan2(direction.x);
    }

    // =========================================================================
    // Scale Methods
    // =========================================================================

    /// Sets the scale of the transform.
    #[inline]
    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
    }

    /// Sets uniform scale on both axes.
    #[inline]
    pub fn set_scale_uniform(&mut self, scale: f32) {
        self.scale = Vec2::new(scale, scale);
    }

    /// Multiplies the current scale by the given factors.
    #[inline]
    pub fn scale_by(&mut self, factors: Vec2) {
        self.scale = Vec2::new(self.scale.x * factors.x, self.scale.y * factors.y);
    }

    // =========================================================================
    // Direction Vectors
    // =========================================================================

    /// Returns the forward direction vector (positive X axis after rotation).
    #[inline]
    pub fn forward(&self) -> Vec2 {
        let (sin, cos) = self.rotation.sin_cos();
        Vec2::new(cos, sin)
    }

    /// Returns the right direction vector (positive Y axis after rotation).
    ///
    /// This is perpendicular to forward, rotated 90 degrees counter-clockwise.
    #[inline]
    pub fn right(&self) -> Vec2 {
        let (sin, cos) = self.rotation.sin_cos();
        Vec2::new(-sin, cos)
    }

    /// Returns the backward direction vector (negative X axis after rotation).
    #[inline]
    pub fn backward(&self) -> Vec2 {
        -self.forward()
    }

    /// Returns the left direction vector (negative Y axis after rotation).
    #[inline]
    pub fn left(&self) -> Vec2 {
        -self.right()
    }

    // =========================================================================
    // Matrix Generation
    // =========================================================================

    /// Computes the 3x3 transformation matrix.
    ///
    /// The matrix represents the combined transformation: Scale * Rotation * Translation
    /// (applied in that order when transforming points).
    #[inline]
    pub fn matrix(&self) -> Mat3x3 {
        let (sin, cos) = self.rotation.sin_cos();
        let sx = self.scale.x;
        let sy = self.scale.y;

        // Combined SRT matrix in column-major order:
        // | cos*sx  -sin*sy  tx |
        // | sin*sx   cos*sy  ty |
        // |   0        0      1 |
        Mat3x3 {
            m: [
                cos * sx,
                sin * sx,
                0.0,
                -sin * sy,
                cos * sy,
                0.0,
                self.position.x,
                self.position.y,
                1.0,
            ],
        }
    }

    /// Computes the inverse transformation matrix.
    ///
    /// Useful for converting world-space to local-space.
    #[inline]
    pub fn matrix_inverse(&self) -> Mat3x3 {
        let (sin, cos) = self.rotation.sin_cos();
        let inv_sx = 1.0 / self.scale.x;
        let inv_sy = 1.0 / self.scale.y;

        // Inverse of TRS = S^-1 * R^-1 * T^-1
        // R^-1 is rotation by -angle, which has cos unchanged and sin negated

        // First, the inverse rotation matrix (transpose for orthogonal matrix)
        // R^-1 = [[cos, sin], [-sin, cos]]  (note: sin is now positive in top row)

        // The combined S^-1 * R^-1 matrix:
        // | cos/sx   sin/sx |
        // | -sin/sy  cos/sy |

        // Translation part: -(S^-1 * R^-1 * t)
        let inv_tx = -(cos * self.position.x + sin * self.position.y) * inv_sx;
        let inv_ty = -(-sin * self.position.x + cos * self.position.y) * inv_sy;

        Mat3x3 {
            m: [
                cos * inv_sx,  // m[0]
                -sin * inv_sy, // m[1]
                0.0,           // m[2]
                sin * inv_sx,  // m[3]
                cos * inv_sy,  // m[4]
                0.0,           // m[5]
                inv_tx,        // m[6]
                inv_ty,        // m[7]
                1.0,           // m[8]
            ],
        }
    }

    /// Converts to a 4x4 matrix for use with 3D rendering APIs.
    ///
    /// The result places the 2D transform in the XY plane at Z=0.
    #[inline]
    pub fn to_mat4(&self) -> [f32; 16] {
        self.matrix().to_mat4()
    }

    // =========================================================================
    // Point Transformation
    // =========================================================================

    /// Transforms a point from local space to world space.
    #[inline]
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        let (sin, cos) = self.rotation.sin_cos();
        let scaled = Vec2::new(point.x * self.scale.x, point.y * self.scale.y);
        let rotated = Vec2::new(
            scaled.x * cos - scaled.y * sin,
            scaled.x * sin + scaled.y * cos,
        );
        rotated + self.position
    }

    /// Transforms a direction from local space to world space.
    ///
    /// Unlike points, directions are not affected by translation.
    #[inline]
    pub fn transform_direction(&self, direction: Vec2) -> Vec2 {
        let (sin, cos) = self.rotation.sin_cos();
        Vec2::new(
            direction.x * cos - direction.y * sin,
            direction.x * sin + direction.y * cos,
        )
    }

    /// Transforms a point from world space to local space.
    #[inline]
    pub fn inverse_transform_point(&self, point: Vec2) -> Vec2 {
        let translated = point - self.position;
        let (sin, cos) = self.rotation.sin_cos();
        let rotated = Vec2::new(
            translated.x * cos + translated.y * sin,
            -translated.x * sin + translated.y * cos,
        );
        Vec2::new(rotated.x / self.scale.x, rotated.y / self.scale.y)
    }

    /// Transforms a direction from world space to local space.
    #[inline]
    pub fn inverse_transform_direction(&self, direction: Vec2) -> Vec2 {
        let (sin, cos) = self.rotation.sin_cos();
        Vec2::new(
            direction.x * cos + direction.y * sin,
            -direction.x * sin + direction.y * cos,
        )
    }

    // =========================================================================
    // Interpolation
    // =========================================================================

    /// Linearly interpolates between two transforms.
    ///
    /// Position and scale are linearly interpolated, rotation uses
    /// shortest-path angle interpolation.
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            rotation: lerp_angle(self.rotation, other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

/// Normalizes an angle to the range [-PI, PI).
#[inline]
pub(super) fn normalize_angle(angle: f32) -> f32 {
    let mut result = angle % (2.0 * PI);
    if result >= PI {
        result -= 2.0 * PI;
    } else if result < -PI {
        result += 2.0 * PI;
    }
    result
}

/// Linearly interpolates between two angles using shortest path.
#[inline]
pub(super) fn lerp_angle(from: f32, to: f32, t: f32) -> f32 {
    let mut diff = to - from;

    // Wrap to [-PI, PI]
    while diff > PI {
        diff -= 2.0 * PI;
    }
    while diff < -PI {
        diff += 2.0 * PI;
    }

    normalize_angle(from + diff * t)
}
