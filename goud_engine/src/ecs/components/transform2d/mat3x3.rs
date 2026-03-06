//! 3x3 matrix type for 2D transformations.

use crate::core::math::Vec2;

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
