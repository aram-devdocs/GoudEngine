//! Transform2D component for 2D spatial transformations.
//!
//! The [`Transform2D`] component represents an entity's position, rotation, and scale
//! in 2D space. It is optimized for 2D games where entities exist on a flat plane.
//!
//! # Design Philosophy
//!
//! Transform2D uses a simpler representation than the 3D [`Transform`](crate::ecs::components::Transform):
//!
//! - **Position**: 2D vector (x, y)
//! - **Rotation**: Single angle in radians (counter-clockwise)
//! - **Scale**: 2D vector for non-uniform scaling
//!
//! This provides:
//! - **Simplicity**: No quaternions, just a rotation angle
//! - **Memory efficiency**: 20 bytes vs 40 bytes for Transform
//! - **Intuitive**: Rotation is a single value in radians
//! - **Performance**: Simpler matrix calculations for 2D
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::components::Transform2D;
//! use goud_engine::core::math::Vec2;
//! use std::f32::consts::PI;
//!
//! // Create a transform at position (100, 50)
//! let mut transform = Transform2D::from_position(Vec2::new(100.0, 50.0));
//!
//! // Modify the transform
//! transform.translate(Vec2::new(10.0, 0.0));
//! transform.rotate(PI / 4.0); // 45 degrees
//! transform.set_scale(Vec2::new(2.0, 2.0));
//!
//! // Get the transformation matrix for rendering
//! let matrix = transform.matrix();
//! ```
//!
//! # Coordinate System
//!
//! GoudEngine 2D uses a standard screen-space coordinate system:
//! - X axis: Right (positive)
//! - Y axis: Down (positive) or Up (positive) depending on camera
//!
//! Rotation is counter-clockwise when viewed from above (standard mathematical convention).
//!
//! # FFI Safety
//!
//! Transform2D is `#[repr(C)]` and can be safely passed across FFI boundaries.

use crate::core::math::Vec2;
use crate::ecs::Component;
use std::f32::consts::PI;

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

/// A 3x3 transformation matrix for 2D transforms.
///
/// Stored in column-major order for OpenGL compatibility.
/// The bottom row is always [0, 0, 1].
///
/// Layout:
/// ```text
/// | m[0] m[3] m[6] |   | cos*sx  -sin*sy  tx |
/// | m[1] m[4] m[7] | = | sin*sx   cos*sy  ty |
/// | m[2] m[5] m[8] |   |   0        0      1 |
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat3x3 {
    /// Matrix elements in column-major order.
    pub m: [f32; 9],
}

impl Mat3x3 {
    /// Identity matrix.
    pub const IDENTITY: Mat3x3 = Mat3x3 {
        m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };

    /// Creates a new matrix from column-major elements.
    #[inline]
    pub const fn new(m: [f32; 9]) -> Self {
        Self { m }
    }

    /// Creates a matrix from individual components.
    ///
    /// Arguments are in row-major order for readability:
    /// ```text
    /// | m00 m01 m02 |
    /// | m10 m11 m12 |
    /// | m20 m21 m22 |
    /// ```
    #[inline]
    pub const fn from_rows(
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
    ) -> Self {
        // Convert to column-major storage
        Self {
            m: [m00, m10, m20, m01, m11, m21, m02, m12, m22],
        }
    }

    /// Creates a translation matrix.
    #[inline]
    pub fn translation(tx: f32, ty: f32) -> Self {
        Self {
            m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, tx, ty, 1.0],
        }
    }

    /// Creates a rotation matrix from an angle in radians.
    #[inline]
    pub fn rotation(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            m: [cos, sin, 0.0, -sin, cos, 0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Creates a scale matrix.
    #[inline]
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            m: [sx, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Returns the translation component.
    #[inline]
    pub fn get_translation(&self) -> Vec2 {
        Vec2::new(self.m[6], self.m[7])
    }

    /// Multiplies two matrices.
    #[inline]
    pub fn multiply(&self, other: &Self) -> Self {
        let a = &self.m;
        let b = &other.m;

        Self {
            m: [
                a[0] * b[0] + a[3] * b[1] + a[6] * b[2],
                a[1] * b[0] + a[4] * b[1] + a[7] * b[2],
                a[2] * b[0] + a[5] * b[1] + a[8] * b[2],
                a[0] * b[3] + a[3] * b[4] + a[6] * b[5],
                a[1] * b[3] + a[4] * b[4] + a[7] * b[5],
                a[2] * b[3] + a[5] * b[4] + a[8] * b[5],
                a[0] * b[6] + a[3] * b[7] + a[6] * b[8],
                a[1] * b[6] + a[4] * b[7] + a[7] * b[8],
                a[2] * b[6] + a[5] * b[7] + a[8] * b[8],
            ],
        }
    }

    /// Transforms a point by this matrix.
    #[inline]
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        Vec2::new(
            self.m[0] * point.x + self.m[3] * point.y + self.m[6],
            self.m[1] * point.x + self.m[4] * point.y + self.m[7],
        )
    }

    /// Transforms a direction by this matrix (ignores translation).
    #[inline]
    pub fn transform_direction(&self, direction: Vec2) -> Vec2 {
        Vec2::new(
            self.m[0] * direction.x + self.m[3] * direction.y,
            self.m[1] * direction.x + self.m[4] * direction.y,
        )
    }

    /// Computes the determinant of the matrix.
    #[inline]
    pub fn determinant(&self) -> f32 {
        let m = &self.m;
        m[0] * (m[4] * m[8] - m[7] * m[5]) - m[3] * (m[1] * m[8] - m[7] * m[2])
            + m[6] * (m[1] * m[5] - m[4] * m[2])
    }

    /// Computes the inverse of the matrix.
    ///
    /// Returns None if the matrix is not invertible (determinant is zero).
    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < f32::EPSILON {
            return None;
        }

        let m = &self.m;
        let inv_det = 1.0 / det;

        Some(Self {
            m: [
                (m[4] * m[8] - m[7] * m[5]) * inv_det,
                (m[7] * m[2] - m[1] * m[8]) * inv_det,
                (m[1] * m[5] - m[4] * m[2]) * inv_det,
                (m[6] * m[5] - m[3] * m[8]) * inv_det,
                (m[0] * m[8] - m[6] * m[2]) * inv_det,
                (m[3] * m[2] - m[0] * m[5]) * inv_det,
                (m[3] * m[7] - m[6] * m[4]) * inv_det,
                (m[6] * m[1] - m[0] * m[7]) * inv_det,
                (m[0] * m[4] - m[3] * m[1]) * inv_det,
            ],
        })
    }

    /// Converts to a 4x4 matrix for 3D rendering.
    ///
    /// The result is a 4x4 matrix with the 2D transform in the XY plane:
    /// ```text
    /// | m[0] m[3]  0  m[6] |
    /// | m[1] m[4]  0  m[7] |
    /// |  0    0    1   0   |
    /// |  0    0    0   1   |
    /// ```
    #[inline]
    pub fn to_mat4(&self) -> [f32; 16] {
        [
            self.m[0], self.m[1], 0.0, 0.0, // column 0
            self.m[3], self.m[4], 0.0, 0.0, // column 1
            0.0, 0.0, 1.0, 0.0, // column 2
            self.m[6], self.m[7], 0.0, 1.0, // column 3
        ]
    }
}

impl Default for Mat3x3 {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl std::ops::Mul for Mat3x3 {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        self.multiply(&other)
    }
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

/// Normalizes an angle to the range [-PI, PI).
#[inline]
fn normalize_angle(angle: f32) -> f32 {
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
fn lerp_angle(from: f32, to: f32, t: f32) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    // =========================================================================
    // Mat3x3 Tests
    // =========================================================================

    mod mat3x3_tests {
        use super::*;

        #[test]
        fn test_identity() {
            let m = Mat3x3::IDENTITY;
            assert_eq!(m.m[0], 1.0);
            assert_eq!(m.m[4], 1.0);
            assert_eq!(m.m[8], 1.0);
        }

        #[test]
        fn test_translation() {
            let m = Mat3x3::translation(10.0, 20.0);
            let p = m.transform_point(Vec2::zero());
            assert!((p.x - 10.0).abs() < 0.0001);
            assert!((p.y - 20.0).abs() < 0.0001);
        }

        #[test]
        fn test_rotation() {
            let m = Mat3x3::rotation(FRAC_PI_2);
            let p = m.transform_point(Vec2::unit_x());
            // 90 degree rotation: (1, 0) -> (0, 1)
            assert!(p.x.abs() < 0.0001);
            assert!((p.y - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_scale() {
            let m = Mat3x3::scale(2.0, 3.0);
            let p = m.transform_point(Vec2::new(1.0, 1.0));
            assert!((p.x - 2.0).abs() < 0.0001);
            assert!((p.y - 3.0).abs() < 0.0001);
        }

        #[test]
        fn test_multiply() {
            let t = Mat3x3::translation(10.0, 0.0);
            let r = Mat3x3::rotation(FRAC_PI_2);
            let combined = t * r;

            let p = combined.transform_point(Vec2::unit_x());
            // First rotate: (1, 0) -> (0, 1)
            // Then translate: (0, 1) -> (10, 1)
            assert!((p.x - 10.0).abs() < 0.0001);
            assert!((p.y - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_inverse() {
            let m = Mat3x3::translation(10.0, 20.0);
            let inv = m.inverse().unwrap();
            let result = m * inv;

            // Should be close to identity
            assert!((result.m[0] - 1.0).abs() < 0.0001);
            assert!((result.m[4] - 1.0).abs() < 0.0001);
            assert!((result.m[8] - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_inverse_rotation() {
            let m = Mat3x3::rotation(FRAC_PI_4);
            let inv = m.inverse().unwrap();
            let result = m * inv;

            assert!((result.m[0] - 1.0).abs() < 0.0001);
            assert!((result.m[4] - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_determinant() {
            let m = Mat3x3::IDENTITY;
            assert!((m.determinant() - 1.0).abs() < 0.0001);

            let s = Mat3x3::scale(2.0, 3.0);
            assert!((s.determinant() - 6.0).abs() < 0.0001);
        }

        #[test]
        fn test_transform_direction() {
            let m = Mat3x3::translation(100.0, 100.0);
            let d = m.transform_direction(Vec2::unit_x());
            // Direction should not be affected by translation
            assert!((d.x - 1.0).abs() < 0.0001);
            assert!(d.y.abs() < 0.0001);
        }

        #[test]
        fn test_to_mat4() {
            let m = Mat3x3::translation(10.0, 20.0);
            let m4 = m.to_mat4();

            // Check diagonal
            assert_eq!(m4[0], 1.0);
            assert_eq!(m4[5], 1.0);
            assert_eq!(m4[10], 1.0);
            assert_eq!(m4[15], 1.0);

            // Check translation
            assert_eq!(m4[12], 10.0);
            assert_eq!(m4[13], 20.0);
            assert_eq!(m4[14], 0.0);
        }

        #[test]
        fn test_default() {
            assert_eq!(Mat3x3::default(), Mat3x3::IDENTITY);
        }
    }

    // =========================================================================
    // Transform2D Construction Tests
    // =========================================================================

    mod construction_tests {
        use super::*;

        #[test]
        fn test_default() {
            let t = Transform2D::default();
            assert_eq!(t.position, Vec2::zero());
            assert_eq!(t.rotation, 0.0);
            assert_eq!(t.scale, Vec2::one());
        }

        #[test]
        fn test_new() {
            let pos = Vec2::new(10.0, 20.0);
            let rot = FRAC_PI_4;
            let scale = Vec2::new(2.0, 3.0);

            let t = Transform2D::new(pos, rot, scale);
            assert_eq!(t.position, pos);
            assert_eq!(t.rotation, rot);
            assert_eq!(t.scale, scale);
        }

        #[test]
        fn test_from_position() {
            let pos = Vec2::new(100.0, 50.0);
            let t = Transform2D::from_position(pos);
            assert_eq!(t.position, pos);
            assert_eq!(t.rotation, 0.0);
            assert_eq!(t.scale, Vec2::one());
        }

        #[test]
        fn test_from_rotation() {
            let t = Transform2D::from_rotation(FRAC_PI_2);
            assert_eq!(t.position, Vec2::zero());
            assert_eq!(t.rotation, FRAC_PI_2);
            assert_eq!(t.scale, Vec2::one());
        }

        #[test]
        fn test_from_rotation_degrees() {
            let t = Transform2D::from_rotation_degrees(90.0);
            assert!((t.rotation - FRAC_PI_2).abs() < 0.0001);
        }

        #[test]
        fn test_from_scale() {
            let scale = Vec2::new(2.0, 3.0);
            let t = Transform2D::from_scale(scale);
            assert_eq!(t.position, Vec2::zero());
            assert_eq!(t.rotation, 0.0);
            assert_eq!(t.scale, scale);
        }

        #[test]
        fn test_from_scale_uniform() {
            let t = Transform2D::from_scale_uniform(2.0);
            assert_eq!(t.scale, Vec2::new(2.0, 2.0));
        }

        #[test]
        fn test_from_position_rotation() {
            let pos = Vec2::new(10.0, 20.0);
            let t = Transform2D::from_position_rotation(pos, FRAC_PI_4);
            assert_eq!(t.position, pos);
            assert_eq!(t.rotation, FRAC_PI_4);
            assert_eq!(t.scale, Vec2::one());
        }

        #[test]
        fn test_look_at() {
            let t = Transform2D::look_at(Vec2::zero(), Vec2::new(1.0, 0.0));
            assert!(t.rotation.abs() < 0.0001); // Should be 0 (looking right)

            let t2 = Transform2D::look_at(Vec2::zero(), Vec2::new(0.0, 1.0));
            assert!((t2.rotation - FRAC_PI_2).abs() < 0.0001); // Should be 90 degrees
        }
    }

    // =========================================================================
    // Transform2D Mutation Tests
    // =========================================================================

    mod mutation_tests {
        use super::*;

        #[test]
        fn test_translate() {
            let mut t = Transform2D::default();
            t.translate(Vec2::new(5.0, 10.0));
            assert_eq!(t.position, Vec2::new(5.0, 10.0));

            t.translate(Vec2::new(3.0, 2.0));
            assert_eq!(t.position, Vec2::new(8.0, 12.0));
        }

        #[test]
        fn test_translate_local() {
            let mut t = Transform2D::from_rotation(FRAC_PI_2);
            t.translate_local(Vec2::new(1.0, 0.0));

            // 90 degree rotation: local X (1, 0) -> world (0, 1)
            assert!(t.position.x.abs() < 0.0001);
            assert!((t.position.y - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_set_position() {
            let mut t = Transform2D::from_position(Vec2::new(10.0, 20.0));
            t.set_position(Vec2::new(100.0, 200.0));
            assert_eq!(t.position, Vec2::new(100.0, 200.0));
        }

        #[test]
        fn test_rotate() {
            let mut t = Transform2D::default();
            t.rotate(FRAC_PI_4);
            assert!((t.rotation - FRAC_PI_4).abs() < 0.0001);

            t.rotate(FRAC_PI_4);
            assert!((t.rotation - FRAC_PI_2).abs() < 0.0001);
        }

        #[test]
        fn test_rotate_degrees() {
            let mut t = Transform2D::default();
            t.rotate_degrees(45.0);
            assert!((t.rotation - FRAC_PI_4).abs() < 0.0001);
        }

        #[test]
        fn test_set_rotation() {
            let mut t = Transform2D::default();
            t.set_rotation(FRAC_PI_2);
            assert!((t.rotation - FRAC_PI_2).abs() < 0.0001);
        }

        #[test]
        fn test_set_rotation_degrees() {
            let mut t = Transform2D::default();
            t.set_rotation_degrees(90.0);
            assert!((t.rotation - FRAC_PI_2).abs() < 0.0001);
        }

        #[test]
        fn test_rotation_degrees() {
            let t = Transform2D::from_rotation(FRAC_PI_2);
            assert!((t.rotation_degrees() - 90.0).abs() < 0.01);
        }

        #[test]
        fn test_look_at_target() {
            let mut t = Transform2D::from_position(Vec2::new(10.0, 10.0));
            t.look_at_target(Vec2::new(20.0, 10.0));
            assert!(t.rotation.abs() < 0.0001); // Looking right = 0 degrees
        }

        #[test]
        fn test_set_scale() {
            let mut t = Transform2D::default();
            t.set_scale(Vec2::new(2.0, 3.0));
            assert_eq!(t.scale, Vec2::new(2.0, 3.0));
        }

        #[test]
        fn test_set_scale_uniform() {
            let mut t = Transform2D::default();
            t.set_scale_uniform(5.0);
            assert_eq!(t.scale, Vec2::new(5.0, 5.0));
        }

        #[test]
        fn test_scale_by() {
            let mut t = Transform2D::from_scale(Vec2::new(2.0, 3.0));
            t.scale_by(Vec2::new(2.0, 2.0));
            assert_eq!(t.scale, Vec2::new(4.0, 6.0));
        }
    }

    // =========================================================================
    // Direction Tests
    // =========================================================================

    mod direction_tests {
        use super::*;

        #[test]
        fn test_directions_identity() {
            let t = Transform2D::default();

            let fwd = t.forward();
            assert!((fwd.x - 1.0).abs() < 0.0001);
            assert!(fwd.y.abs() < 0.0001);

            let right = t.right();
            assert!(right.x.abs() < 0.0001);
            assert!((right.y - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_directions_rotated() {
            let t = Transform2D::from_rotation(FRAC_PI_2);

            // After 90 degree rotation:
            // forward (1, 0) -> (0, 1)
            let fwd = t.forward();
            assert!(fwd.x.abs() < 0.0001);
            assert!((fwd.y - 1.0).abs() < 0.0001);

            // right (0, 1) -> (-1, 0)
            let right = t.right();
            assert!((right.x - (-1.0)).abs() < 0.0001);
            assert!(right.y.abs() < 0.0001);
        }

        #[test]
        fn test_backward_and_left() {
            let t = Transform2D::default();

            let back = t.backward();
            assert!((back.x - (-1.0)).abs() < 0.0001);

            let left = t.left();
            assert!((left.y - (-1.0)).abs() < 0.0001);
        }
    }

    // =========================================================================
    // Matrix Tests
    // =========================================================================

    mod matrix_tests {
        use super::*;

        #[test]
        fn test_matrix_identity() {
            let t = Transform2D::default();
            let m = t.matrix();

            // Should be close to identity
            assert!((m.m[0] - 1.0).abs() < 0.0001);
            assert!((m.m[4] - 1.0).abs() < 0.0001);
            assert!((m.m[8] - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_matrix_translation() {
            let t = Transform2D::from_position(Vec2::new(10.0, 20.0));
            let m = t.matrix();

            assert!((m.m[6] - 10.0).abs() < 0.0001);
            assert!((m.m[7] - 20.0).abs() < 0.0001);
        }

        #[test]
        fn test_matrix_scale() {
            let t = Transform2D::from_scale(Vec2::new(2.0, 3.0));
            let m = t.matrix();

            assert!((m.m[0] - 2.0).abs() < 0.0001);
            assert!((m.m[4] - 3.0).abs() < 0.0001);
        }

        #[test]
        fn test_matrix_rotation() {
            let t = Transform2D::from_rotation(FRAC_PI_2);
            let m = t.matrix();

            let p = m.transform_point(Vec2::new(1.0, 0.0));
            // 90 degree rotation: (1, 0) -> (0, 1)
            assert!(p.x.abs() < 0.0001);
            assert!((p.y - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_matrix_inverse() {
            let t = Transform2D::new(Vec2::new(10.0, 20.0), FRAC_PI_4, Vec2::new(2.0, 3.0));

            let m = t.matrix();
            let m_inv = t.matrix_inverse();

            let result = m * m_inv;

            // Should be close to identity
            assert!((result.m[0] - 1.0).abs() < 0.001);
            assert!((result.m[4] - 1.0).abs() < 0.001);
            assert!((result.m[8] - 1.0).abs() < 0.001);
            assert!(result.m[6].abs() < 0.001);
            assert!(result.m[7].abs() < 0.001);
        }

        #[test]
        fn test_to_mat4() {
            let t = Transform2D::from_position(Vec2::new(5.0, 10.0));
            let m4 = t.to_mat4();

            // Check translation
            assert_eq!(m4[12], 5.0);
            assert_eq!(m4[13], 10.0);
            assert_eq!(m4[14], 0.0);

            // Check diagonal
            assert_eq!(m4[0], 1.0);
            assert_eq!(m4[5], 1.0);
            assert_eq!(m4[10], 1.0);
            assert_eq!(m4[15], 1.0);
        }
    }

    // =========================================================================
    // Point Transformation Tests
    // =========================================================================

    mod point_transform_tests {
        use super::*;

        #[test]
        fn test_transform_point_translation() {
            let t = Transform2D::from_position(Vec2::new(10.0, 20.0));
            let p = t.transform_point(Vec2::zero());
            assert_eq!(p, Vec2::new(10.0, 20.0));
        }

        #[test]
        fn test_transform_point_scale() {
            let t = Transform2D::from_scale(Vec2::new(2.0, 3.0));
            let p = t.transform_point(Vec2::new(5.0, 5.0));
            assert_eq!(p, Vec2::new(10.0, 15.0));
        }

        #[test]
        fn test_transform_point_rotation() {
            let t = Transform2D::from_rotation(FRAC_PI_2);
            let p = t.transform_point(Vec2::new(1.0, 0.0));
            // 90 degree rotation: (1, 0) -> (0, 1)
            assert!(p.x.abs() < 0.0001);
            assert!((p.y - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_transform_direction() {
            let t = Transform2D::new(
                Vec2::new(100.0, 100.0), // Translation should not affect direction
                FRAC_PI_2,
                Vec2::one(),
            );

            let dir = Vec2::new(1.0, 0.0);
            let transformed = t.transform_direction(dir);

            // 90 degree rotation: (1, 0) -> (0, 1)
            assert!(transformed.x.abs() < 0.0001);
            assert!((transformed.y - 1.0).abs() < 0.0001);
        }

        #[test]
        fn test_inverse_transform_point() {
            let t = Transform2D::new(Vec2::new(10.0, 20.0), FRAC_PI_4, Vec2::new(2.0, 2.0));

            let world_point = Vec2::new(5.0, 5.0);
            let local = t.inverse_transform_point(world_point);
            let back_to_world = t.transform_point(local);

            assert!((back_to_world.x - world_point.x).abs() < 0.001);
            assert!((back_to_world.y - world_point.y).abs() < 0.001);
        }

        #[test]
        fn test_inverse_transform_direction() {
            let t = Transform2D::from_rotation(FRAC_PI_2);

            let world_dir = Vec2::new(1.0, 0.0);
            let local = t.inverse_transform_direction(world_dir);
            let back = t.transform_direction(local);

            assert!((back.x - world_dir.x).abs() < 0.0001);
            assert!((back.y - world_dir.y).abs() < 0.0001);
        }
    }

    // =========================================================================
    // Interpolation Tests
    // =========================================================================

    mod interpolation_tests {
        use super::*;

        #[test]
        fn test_lerp_position() {
            let a = Transform2D::from_position(Vec2::zero());
            let b = Transform2D::from_position(Vec2::new(10.0, 20.0));

            let mid = a.lerp(b, 0.5);
            assert_eq!(mid.position, Vec2::new(5.0, 10.0));
        }

        #[test]
        fn test_lerp_scale() {
            let a = Transform2D::from_scale(Vec2::one());
            let b = Transform2D::from_scale(Vec2::new(3.0, 3.0));

            let mid = a.lerp(b, 0.5);
            assert_eq!(mid.scale, Vec2::new(2.0, 2.0));
        }

        #[test]
        fn test_lerp_rotation() {
            let a = Transform2D::from_rotation(0.0);
            let b = Transform2D::from_rotation(FRAC_PI_2);

            let mid = a.lerp(b, 0.5);
            assert!((mid.rotation - FRAC_PI_4).abs() < 0.0001);
        }

        #[test]
        fn test_lerp_rotation_shortest_path() {
            // From -170 degrees to 170 degrees should go through 180, not through 0
            let a = Transform2D::from_rotation(-170.0_f32.to_radians());
            let b = Transform2D::from_rotation(170.0_f32.to_radians());

            let mid = a.lerp(b, 0.5);
            // Should be close to 180 degrees (PI or -PI)
            assert!(mid.rotation.abs() > 3.0); // Close to PI
        }

        #[test]
        fn test_lerp_endpoints() {
            let a = Transform2D::new(Vec2::zero(), 0.0, Vec2::one());
            let b = Transform2D::new(Vec2::new(10.0, 10.0), PI, Vec2::new(2.0, 2.0));

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
        fn test_transform2d_is_component() {
            fn assert_component<T: Component>() {}
            assert_component::<Transform2D>();
        }

        #[test]
        fn test_transform2d_is_send() {
            fn assert_send<T: Send>() {}
            assert_send::<Transform2D>();
        }

        #[test]
        fn test_transform2d_is_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<Transform2D>();
        }

        #[test]
        fn test_transform2d_clone() {
            let t = Transform2D::new(Vec2::new(1.0, 2.0), FRAC_PI_4, Vec2::new(2.0, 3.0));
            let cloned = t.clone();
            assert_eq!(t, cloned);
        }

        #[test]
        fn test_transform2d_copy() {
            let t = Transform2D::default();
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
        fn test_transform2d_size() {
            // Vec2 (8) + f32 (4) + Vec2 (8) = 20 bytes
            assert_eq!(size_of::<Transform2D>(), 20);
        }

        #[test]
        fn test_transform2d_align() {
            assert_eq!(align_of::<Transform2D>(), 4); // f32 alignment
        }

        #[test]
        fn test_mat3x3_size() {
            // 9 * f32 = 36 bytes
            assert_eq!(size_of::<Mat3x3>(), 36);
        }

        #[test]
        fn test_mat3x3_align() {
            assert_eq!(align_of::<Mat3x3>(), 4);
        }

        #[test]
        fn test_transform2d_field_layout() {
            let t = Transform2D::new(Vec2::new(1.0, 2.0), 3.0, Vec2::new(4.0, 5.0));
            let ptr = &t as *const Transform2D as *const f32;
            unsafe {
                assert_eq!(*ptr, 1.0); // position.x
                assert_eq!(*ptr.add(1), 2.0); // position.y
                assert_eq!(*ptr.add(2), 3.0); // rotation
                assert_eq!(*ptr.add(3), 4.0); // scale.x
                assert_eq!(*ptr.add(4), 5.0); // scale.y
            }
        }
    }

    // =========================================================================
    // Utility Function Tests
    // =========================================================================

    mod utility_tests {
        use super::*;

        #[test]
        fn test_normalize_angle() {
            // Within range
            assert!((normalize_angle(0.0) - 0.0).abs() < 0.0001);
            assert!((normalize_angle(1.0) - 1.0).abs() < 0.0001);

            // Above PI
            assert!((normalize_angle(PI + 0.5) - (-PI + 0.5)).abs() < 0.0001);

            // Below -PI
            assert!((normalize_angle(-PI - 0.5) - (PI - 0.5)).abs() < 0.0001);

            // Large positive
            let result = normalize_angle(3.0 * PI);
            assert!(result >= -PI && result < PI);

            // Large negative
            let result = normalize_angle(-3.0 * PI);
            assert!(result >= -PI && result < PI);
        }

        #[test]
        fn test_lerp_angle_same_direction() {
            let result = lerp_angle(0.0, FRAC_PI_2, 0.5);
            assert!((result - FRAC_PI_4).abs() < 0.0001);
        }

        #[test]
        fn test_lerp_angle_across_boundary() {
            // From -170 to 170 should go through 180
            let from = -170.0_f32.to_radians();
            let to = 170.0_f32.to_radians();
            let mid = lerp_angle(from, to, 0.5);

            // Should be close to 180 degrees (PI or -PI)
            assert!(mid.abs() > 3.0);
        }

        #[test]
        fn test_lerp_angle_endpoints() {
            let from = 0.5;
            let to = 1.5;

            assert!((lerp_angle(from, to, 0.0) - from).abs() < 0.0001);
            assert!((lerp_angle(from, to, 1.0) - to).abs() < 0.0001);
        }
    }
}
