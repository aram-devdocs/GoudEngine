//! 2D vector type with FFI-safe memory layout.

use std::ops::{Add, Div, Mul, Neg, Sub};

/// A 2D vector with FFI-safe memory layout.
///
/// This type is guaranteed to have the same memory layout as a C struct
/// with two consecutive f32 fields. Use this type for any 2D positions,
/// velocities, or texture coordinates that cross FFI boundaries.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Vec2 {
    /// The x-component of the vector.
    pub x: f32,
    /// The y-component of the vector.
    pub y: f32,
}

impl Vec2 {
    /// Creates a new Vec2 from x and y components.
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns the zero vector (0, 0).
    #[inline]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Returns the one vector (1, 1).
    #[inline]
    pub const fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    /// Returns the unit X vector (1, 0).
    #[inline]
    pub const fn unit_x() -> Self {
        Self { x: 1.0, y: 0.0 }
    }

    /// Returns the unit Y vector (0, 1).
    #[inline]
    pub const fn unit_y() -> Self {
        Self { x: 0.0, y: 1.0 }
    }

    /// Computes the dot product of two vectors.
    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Returns the squared length of the vector.
    ///
    /// This is more efficient than `length()` when you only need to compare lengths.
    #[inline]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    /// Returns the length (magnitude) of the vector.
    #[inline]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Returns a normalized (unit length) version of this vector.
    ///
    /// If the vector has zero length, returns the zero vector.
    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::zero()
        } else {
            self / len
        }
    }

    /// Linearly interpolates between two vectors.
    ///
    /// When `t = 0.0`, returns `self`. When `t = 1.0`, returns `other`.
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    /// Returns the perpendicular vector (rotated 90 degrees counter-clockwise).
    #[inline]
    pub fn perpendicular(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }
}

impl Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn mul(self, vec: Vec2) -> Vec2 {
        vec * self
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, scalar: f32) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl From<cgmath::Vector2<f32>> for Vec2 {
    #[inline]
    fn from(v: cgmath::Vector2<f32>) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Vec2> for cgmath::Vector2<f32> {
    #[inline]
    fn from(v: Vec2) -> Self {
        cgmath::Vector2::new(v.x, v.y)
    }
}
