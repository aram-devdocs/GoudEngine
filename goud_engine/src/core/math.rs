//! FFI-safe mathematical types for the engine.
//!
//! This module provides `#[repr(C)]` mathematical types that are safe to pass across
//! FFI boundaries. These types wrap functionality from cgmath but guarantee a stable,
//! predictable memory layout for use with C#, Python, and other language bindings.
//!
//! # Design Decision
//!
//! We wrap cgmath rather than replacing it because:
//! 1. **Internal Operations**: cgmath provides battle-tested matrix/vector operations
//!    (look_at, quaternion math, etc.) that would be error-prone to reimplement
//! 2. **FFI Safety**: cgmath types like `Vector3<f32>` are newtypes over arrays and
//!    don't guarantee a specific memory layout suitable for FFI
//! 3. **Type Safety**: Our wrappers ensure compile-time FFI compatibility while
//!    maintaining ergonomic conversions for internal use
//!
//! # Usage
//!
//! ```rust
//! use goud_engine::core::math::{Vec3, Color};
//!
//! // Create FFI-safe types
//! let position = Vec3::new(1.0, 2.0, 3.0);
//! let color = Color::RED;
//!
//! // Convert to cgmath for internal math operations
//! let cgmath_vec: cgmath::Vector3<f32> = position.into();
//!
//! // Convert back from cgmath results
//! let result = Vec3::from(cgmath_vec);
//! ```

use std::ops::{Add, Div, Mul, Neg, Sub};

// Re-export cgmath types for internal use where FFI is not needed
pub use cgmath::{Matrix3, Matrix4, Point3, Quaternion};

// =============================================================================
// Vec2 - 2D Vector (FFI-Safe)
// =============================================================================

/// A 2D vector with FFI-safe memory layout.
///
/// This type is guaranteed to have the same memory layout as a C struct
/// with two consecutive f32 fields. Use this type for any 2D positions,
/// velocities, or texture coordinates that cross FFI boundaries.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
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

// Operator implementations for Vec2
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

// cgmath conversions for Vec2
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

// =============================================================================
// Vec3 - 3D Vector (FFI-Safe)
// =============================================================================

/// A 3D vector with FFI-safe memory layout.
///
/// This type is guaranteed to have the same memory layout as a C struct
/// with three consecutive f32 fields. Use this type for any 3D positions,
/// directions, or colors that cross FFI boundaries.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec3 {
    /// The x-component of the vector.
    pub x: f32,
    /// The y-component of the vector.
    pub y: f32,
    /// The z-component of the vector.
    pub z: f32,
}

impl Vec3 {
    /// Creates a new Vec3 from x, y, z components.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Returns the zero vector (0, 0, 0).
    #[inline]
    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Returns the one vector (1, 1, 1).
    #[inline]
    pub const fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }

    /// Returns the unit X vector (1, 0, 0).
    #[inline]
    pub const fn unit_x() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Returns the unit Y vector (0, 1, 0).
    #[inline]
    pub const fn unit_y() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    /// Returns the unit Z vector (0, 0, 1).
    #[inline]
    pub const fn unit_z() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }

    /// Computes the dot product of two vectors.
    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Computes the cross product of two vectors.
    ///
    /// The result is perpendicular to both input vectors, following the right-hand rule.
    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Returns the squared length of the vector.
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
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }
}

// Operator implementations for Vec3
impl Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;
    #[inline]
    fn mul(self, vec: Vec3) -> Vec3 {
        vec * self
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, scalar: f32) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// cgmath conversions for Vec3
impl From<cgmath::Vector3<f32>> for Vec3 {
    #[inline]
    fn from(v: cgmath::Vector3<f32>) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vec3> for cgmath::Vector3<f32> {
    #[inline]
    fn from(v: Vec3) -> Self {
        cgmath::Vector3::new(v.x, v.y, v.z)
    }
}

// =============================================================================
// Vec4 - 4D Vector (FFI-Safe)
// =============================================================================

/// A 4D vector with FFI-safe memory layout.
///
/// This type is commonly used for homogeneous coordinates in graphics
/// or RGBA colors. It is guaranteed to have the same memory layout as
/// a C struct with four consecutive f32 fields.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec4 {
    /// The x-component of the vector.
    pub x: f32,
    /// The y-component of the vector.
    pub y: f32,
    /// The z-component of the vector.
    pub z: f32,
    /// The w-component of the vector.
    pub w: f32,
}

impl Vec4 {
    /// Creates a new Vec4 from x, y, z, w components.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Returns the zero vector (0, 0, 0, 0).
    #[inline]
    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }
    }

    /// Returns the one vector (1, 1, 1, 1).
    #[inline]
    pub const fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
            w: 1.0,
        }
    }

    /// Creates a Vec4 from a Vec3 and w component.
    #[inline]
    pub const fn from_vec3(v: Vec3, w: f32) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w,
        }
    }

    /// Returns the xyz components as a Vec3.
    #[inline]
    pub const fn xyz(self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    /// Computes the dot product of two vectors.
    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    /// Returns the squared length of the vector.
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
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
            w: self.w + (other.w - self.w) * t,
        }
    }
}

// Operator implementations for Vec4
impl Add for Vec4 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl Sub for Vec4 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;
    #[inline]
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;
    #[inline]
    fn mul(self, vec: Vec4) -> Vec4 {
        vec * self
    }
}

impl Div<f32> for Vec4 {
    type Output = Self;
    #[inline]
    fn div(self, scalar: f32) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
            w: self.w / scalar,
        }
    }
}

impl Neg for Vec4 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

// cgmath conversions for Vec4
impl From<cgmath::Vector4<f32>> for Vec4 {
    #[inline]
    fn from(v: cgmath::Vector4<f32>) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}

impl From<Vec4> for cgmath::Vector4<f32> {
    #[inline]
    fn from(v: Vec4) -> Self {
        cgmath::Vector4::new(v.x, v.y, v.z, v.w)
    }
}

// =============================================================================
// Rect - 2D Rectangle (FFI-Safe)
// =============================================================================

/// A 2D rectangle with FFI-safe memory layout.
///
/// Defined by position (x, y) and size (width, height).
/// The position represents the top-left corner in screen space.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    /// The x-coordinate of the top-left corner of the rectangle.
    pub x: f32,
    /// The y-coordinate of the top-left corner of the rectangle.
    pub y: f32,
    /// The width of the rectangle.
    pub width: f32,
    /// The height of the rectangle.
    pub height: f32,
}

impl Rect {
    /// Creates a new rectangle from position and size.
    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a rectangle from min and max points.
    #[inline]
    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self {
            x: min.x,
            y: min.y,
            width: max.x - min.x,
            height: max.y - min.y,
        }
    }

    /// Creates a unit rectangle (0, 0, 1, 1).
    #[inline]
    pub const fn unit() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }

    /// Returns the minimum point (top-left corner).
    #[inline]
    pub const fn min(&self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    /// Returns the maximum point (bottom-right corner).
    #[inline]
    pub fn max(&self) -> Vec2 {
        Vec2 {
            x: self.x + self.width,
            y: self.y + self.height,
        }
    }

    /// Returns the center point of the rectangle.
    #[inline]
    pub fn center(&self) -> Vec2 {
        Vec2 {
            x: self.x + self.width * 0.5,
            y: self.y + self.height * 0.5,
        }
    }

    /// Returns the size as a Vec2.
    #[inline]
    pub const fn size(&self) -> Vec2 {
        Vec2 {
            x: self.width,
            y: self.height,
        }
    }

    /// Returns the area of the rectangle.
    #[inline]
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Checks if the rectangle contains a point.
    #[inline]
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    /// Checks if this rectangle intersects with another.
    #[inline]
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Returns the intersection of two rectangles, or None if they don't intersect.
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let max_x = (self.x + self.width).min(other.x + other.width);
        let max_y = (self.y + self.height).min(other.y + other.height);

        if x < max_x && y < max_y {
            Some(Rect {
                x,
                y,
                width: max_x - x,
                height: max_y - y,
            })
        } else {
            None
        }
    }
}

// =============================================================================
// Color - RGBA Color (FFI-Safe)
// =============================================================================

/// An RGBA color with FFI-safe memory layout.
///
/// Components are stored as f32 values, typically in the range [0.0, 1.0].
/// Values outside this range are allowed for HDR rendering.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color {
    /// The red component of the color.
    pub r: f32,
    /// The green component of the color.
    pub g: f32,
    /// The blue component of the color.
    pub b: f32,
    /// The alpha (transparency) component of the color.
    pub a: f32,
}

impl Color {
    // Common color constants
    /// The color white (1.0, 1.0, 1.0, 1.0).
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    /// The color black (0.0, 0.0, 0.0, 1.0).
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    /// The color red (1.0, 0.0, 0.0, 1.0).
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    /// The color green (0.0, 1.0, 0.0, 1.0).
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    /// The color blue (0.0, 0.0, 1.0, 1.0).
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    /// The color yellow (1.0, 1.0, 0.0, 1.0).
    pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
    /// The color cyan (0.0, 1.0, 1.0, 1.0).
    pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
    /// The color magenta (1.0, 0.0, 1.0, 1.0).
    pub const MAGENTA: Color = Color::rgb(1.0, 0.0, 1.0);
    /// Transparent black (0.0, 0.0, 0.0, 0.0).
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
    /// The color gray (0.5, 0.5, 0.5, 1.0).
    pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);

    /// Creates a new RGBA color.
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates an RGB color with alpha = 1.0.
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Creates an RGBA color.
    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from 8-bit RGBA values (0-255).
    #[inline]
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Creates a color from a hex value (0xRRGGBB or 0xRRGGBBAA).
    #[inline]
    pub fn from_hex(hex: u32) -> Self {
        if hex > 0xFFFFFF {
            // Has alpha (0xRRGGBBAA)
            Self::from_u8(
                ((hex >> 24) & 0xFF) as u8,
                ((hex >> 16) & 0xFF) as u8,
                ((hex >> 8) & 0xFF) as u8,
                (hex & 0xFF) as u8,
            )
        } else {
            // No alpha (0xRRGGBB)
            Self::from_u8(
                ((hex >> 16) & 0xFF) as u8,
                ((hex >> 8) & 0xFF) as u8,
                (hex & 0xFF) as u8,
                255,
            )
        }
    }

    /// Returns the RGB components as a Vec3.
    #[inline]
    pub const fn to_vec3(&self) -> Vec3 {
        Vec3 {
            x: self.r,
            y: self.g,
            z: self.b,
        }
    }

    /// Returns all RGBA components as a Vec4.
    #[inline]
    pub const fn to_vec4(&self) -> Vec4 {
        Vec4 {
            x: self.r,
            y: self.g,
            z: self.b,
            w: self.a,
        }
    }

    /// Creates a color from a Vec3 (RGB) with alpha = 1.0.
    #[inline]
    pub const fn from_vec3(v: Vec3) -> Self {
        Self {
            r: v.x,
            g: v.y,
            b: v.z,
            a: 1.0,
        }
    }

    /// Creates a color from a Vec4 (RGBA).
    #[inline]
    pub const fn from_vec4(v: Vec4) -> Self {
        Self {
            r: v.x,
            g: v.y,
            b: v.z,
            a: v.w,
        }
    }

    /// Linearly interpolates between two colors.
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// Returns a new color with the specified alpha.
    #[inline]
    pub const fn with_alpha(self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    /// Clamps all components to the [0.0, 1.0] range.
    #[inline]
    pub fn clamp(self) -> Self {
        Self {
            r: self.r.clamp(0.0, 1.0),
            g: self.g.clamp(0.0, 1.0),
            b: self.b.clamp(0.0, 1.0),
            a: self.a.clamp(0.0, 1.0),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Vec2 Tests
    // =========================================================================

    #[test]
    fn test_vec2_constructors() {
        assert_eq!(Vec2::new(1.0, 2.0), Vec2 { x: 1.0, y: 2.0 });
        assert_eq!(Vec2::zero(), Vec2 { x: 0.0, y: 0.0 });
        assert_eq!(Vec2::one(), Vec2 { x: 1.0, y: 1.0 });
        assert_eq!(Vec2::unit_x(), Vec2 { x: 1.0, y: 0.0 });
        assert_eq!(Vec2::unit_y(), Vec2 { x: 0.0, y: 1.0 });
    }

    #[test]
    fn test_vec2_dot() {
        let a = Vec2::new(2.0, 3.0);
        let b = Vec2::new(4.0, 5.0);
        assert_eq!(a.dot(b), 23.0); // 2*4 + 3*5 = 8 + 15 = 23
    }

    #[test]
    fn test_vec2_length() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.length_squared(), 25.0);
        assert_eq!(v.length(), 5.0);
    }

    #[test]
    fn test_vec2_normalize() {
        let v = Vec2::new(3.0, 4.0);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 0.0001);
        assert_eq!(Vec2::zero().normalize(), Vec2::zero());
    }

    #[test]
    fn test_vec2_operators() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);

        assert_eq!(a + b, Vec2::new(4.0, 6.0));
        assert_eq!(a - b, Vec2::new(-2.0, -2.0));
        assert_eq!(a * 2.0, Vec2::new(2.0, 4.0));
        assert_eq!(2.0 * a, Vec2::new(2.0, 4.0));
        assert_eq!(a / 2.0, Vec2::new(0.5, 1.0));
        assert_eq!(-a, Vec2::new(-1.0, -2.0));
    }

    #[test]
    fn test_vec2_lerp() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 20.0);
        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
        assert_eq!(a.lerp(b, 0.5), Vec2::new(5.0, 10.0));
    }

    #[test]
    fn test_vec2_cgmath_conversion() {
        let goud = Vec2::new(1.0, 2.0);
        let cg: cgmath::Vector2<f32> = goud.into();
        assert_eq!(cg.x, 1.0);
        assert_eq!(cg.y, 2.0);

        let back: Vec2 = cg.into();
        assert_eq!(back, goud);
    }

    // =========================================================================
    // Vec3 Tests
    // =========================================================================

    #[test]
    fn test_vec3_constructors() {
        assert_eq!(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );
        assert_eq!(
            Vec3::zero(),
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0
            }
        );
        assert_eq!(
            Vec3::one(),
            Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0
            }
        );
        assert_eq!(
            Vec3::unit_x(),
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0
            }
        );
        assert_eq!(
            Vec3::unit_y(),
            Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0
            }
        );
        assert_eq!(
            Vec3::unit_z(),
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0
            }
        );
    }

    #[test]
    fn test_vec3_cross() {
        let x = Vec3::unit_x();
        let y = Vec3::unit_y();
        let z = x.cross(y);
        assert!((z - Vec3::unit_z()).length() < 0.0001);
    }

    #[test]
    fn test_vec3_operators() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(a - b, Vec3::new(-3.0, -3.0, -3.0));
        assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(2.0 * a, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(-a, Vec3::new(-1.0, -2.0, -3.0));
    }

    #[test]
    fn test_vec3_cgmath_conversion() {
        let goud = Vec3::new(1.0, 2.0, 3.0);
        let cg: cgmath::Vector3<f32> = goud.into();
        assert_eq!(cg.x, 1.0);
        assert_eq!(cg.y, 2.0);
        assert_eq!(cg.z, 3.0);

        let back: Vec3 = cg.into();
        assert_eq!(back, goud);
    }

    #[test]
    fn test_vec3_dot() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert_eq!(a.dot(b), 32.0);

        // Dot product with self is length squared
        assert_eq!(a.dot(a), a.length_squared());

        // Dot product is commutative
        assert_eq!(a.dot(b), b.dot(a));

        // Perpendicular vectors have dot product of 0
        let x = Vec3::unit_x();
        let y = Vec3::unit_y();
        assert_eq!(x.dot(y), 0.0);
    }

    #[test]
    fn test_vec3_cross_properties() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        // Cross product is anti-commutative: a × b = -(b × a)
        let ab = a.cross(b);
        let ba = b.cross(a);
        assert!((ab + ba).length() < 0.0001);

        // Cross product is perpendicular to both inputs
        assert!(ab.dot(a).abs() < 0.0001);
        assert!(ab.dot(b).abs() < 0.0001);

        // Right-hand rule: x × y = z
        assert!((Vec3::unit_x().cross(Vec3::unit_y()) - Vec3::unit_z()).length() < 0.0001);
        assert!((Vec3::unit_y().cross(Vec3::unit_z()) - Vec3::unit_x()).length() < 0.0001);
        assert!((Vec3::unit_z().cross(Vec3::unit_x()) - Vec3::unit_y()).length() < 0.0001);
    }

    #[test]
    fn test_vec3_length() {
        // 3-4-5 Pythagorean in 3D extended: sqrt(1 + 4 + 4) = 3
        let v = Vec3::new(1.0, 2.0, 2.0);
        assert_eq!(v.length_squared(), 9.0);
        assert_eq!(v.length(), 3.0);

        // Zero vector has zero length
        assert_eq!(Vec3::zero().length(), 0.0);

        // Unit vectors have length 1
        assert_eq!(Vec3::unit_x().length(), 1.0);
        assert_eq!(Vec3::unit_y().length(), 1.0);
        assert_eq!(Vec3::unit_z().length(), 1.0);
    }

    #[test]
    fn test_vec3_normalize() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 0.0001);
        assert!((n.x - 0.6).abs() < 0.0001);
        assert!((n.y - 0.8).abs() < 0.0001);
        assert_eq!(n.z, 0.0);

        // Zero vector normalizes to zero (safe behavior)
        assert_eq!(Vec3::zero().normalize(), Vec3::zero());

        // Unit vectors remain unchanged
        assert!((Vec3::unit_x().normalize() - Vec3::unit_x()).length() < 0.0001);
    }

    #[test]
    fn test_vec3_lerp() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(10.0, 20.0, 30.0);

        // t=0 returns start
        assert_eq!(a.lerp(b, 0.0), a);

        // t=1 returns end
        assert_eq!(a.lerp(b, 1.0), b);

        // t=0.5 returns midpoint
        assert_eq!(a.lerp(b, 0.5), Vec3::new(5.0, 10.0, 15.0));

        // Extrapolation works (t > 1)
        assert_eq!(a.lerp(b, 2.0), Vec3::new(20.0, 40.0, 60.0));

        // Negative t extrapolates backwards
        assert_eq!(a.lerp(b, -0.5), Vec3::new(-5.0, -10.0, -15.0));
    }

    #[test]
    fn test_vec3_ffi_layout() {
        use std::mem::{align_of, size_of};

        // Verify Vec3 has expected FFI layout
        assert_eq!(size_of::<Vec3>(), 12); // 3 * f32 = 12 bytes
        assert_eq!(align_of::<Vec3>(), 4); // f32 alignment

        // Verify fields are laid out consecutively with no padding
        let v = Vec3::new(1.0, 2.0, 3.0);
        let ptr = &v as *const Vec3 as *const f32;
        unsafe {
            assert_eq!(*ptr, 1.0); // x at offset 0
            assert_eq!(*ptr.add(1), 2.0); // y at offset 4
            assert_eq!(*ptr.add(2), 3.0); // z at offset 8
        }

        // Verify Default trait
        assert_eq!(Vec3::default(), Vec3::zero());
    }

    // =========================================================================
    // Vec4 Tests
    // =========================================================================

    #[test]
    fn test_vec4_constructors() {
        assert_eq!(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                w: 4.0
            }
        );
        assert_eq!(
            Vec4::zero(),
            Vec4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0
            }
        );
        assert_eq!(
            Vec4::one(),
            Vec4 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
                w: 1.0
            }
        );
    }

    #[test]
    fn test_vec4_from_vec3() {
        let v3 = Vec3::new(1.0, 2.0, 3.0);
        let v4 = Vec4::from_vec3(v3, 4.0);
        assert_eq!(v4, Vec4::new(1.0, 2.0, 3.0, 4.0));
        assert_eq!(v4.xyz(), v3);
    }

    #[test]
    fn test_vec4_cgmath_conversion() {
        let goud = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let cg: cgmath::Vector4<f32> = goud.into();
        let back: Vec4 = cg.into();
        assert_eq!(back, goud);
    }

    // =========================================================================
    // Rect Tests
    // =========================================================================

    #[test]
    fn test_rect_constructors() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.x, 10.0);
        assert_eq!(r.y, 20.0);
        assert_eq!(r.width, 100.0);
        assert_eq!(r.height, 50.0);
    }

    #[test]
    fn test_rect_from_min_max() {
        let r = Rect::from_min_max(Vec2::new(10.0, 20.0), Vec2::new(110.0, 70.0));
        assert_eq!(r.x, 10.0);
        assert_eq!(r.y, 20.0);
        assert_eq!(r.width, 100.0);
        assert_eq!(r.height, 50.0);
    }

    #[test]
    fn test_rect_accessors() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.min(), Vec2::new(10.0, 20.0));
        assert_eq!(r.max(), Vec2::new(110.0, 70.0));
        assert_eq!(r.center(), Vec2::new(60.0, 45.0));
        assert_eq!(r.size(), Vec2::new(100.0, 50.0));
        assert_eq!(r.area(), 5000.0);
    }

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(r.contains(Vec2::new(50.0, 50.0)));
        assert!(r.contains(Vec2::new(0.0, 0.0)));
        assert!(!r.contains(Vec2::new(100.0, 100.0))); // exclusive max
        assert!(!r.contains(Vec2::new(-1.0, 50.0)));
        assert!(!r.contains(Vec2::new(50.0, -1.0)));
    }

    #[test]
    fn test_rect_intersects() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);
        let c = Rect::new(200.0, 200.0, 10.0, 10.0);

        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
        assert!(!c.intersects(&a));
    }

    #[test]
    fn test_rect_intersection() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);

        let inter = a.intersection(&b).unwrap();
        assert_eq!(inter.x, 50.0);
        assert_eq!(inter.y, 50.0);
        assert_eq!(inter.width, 50.0);
        assert_eq!(inter.height, 50.0);

        let c = Rect::new(200.0, 200.0, 10.0, 10.0);
        assert!(a.intersection(&c).is_none());
    }

    // =========================================================================
    // Color Tests
    // =========================================================================

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::WHITE, Color::rgba(1.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::BLACK, Color::rgba(0.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::RED, Color::rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::TRANSPARENT, Color::rgba(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_color_from_u8() {
        let c = Color::from_u8(255, 128, 0, 255);
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.5).abs() < 0.01);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_color_from_hex() {
        let c1 = Color::from_hex(0xFF0000); // Red
        assert_eq!(c1.r, 1.0);
        assert_eq!(c1.g, 0.0);
        assert_eq!(c1.b, 0.0);
        assert_eq!(c1.a, 1.0);

        let c2 = Color::from_hex(0xFF000080); // Red with 50% alpha
        assert_eq!(c2.r, 1.0);
        assert_eq!(c2.g, 0.0);
        assert_eq!(c2.b, 0.0);
        assert!((c2.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_vec_conversions() {
        let c = Color::rgba(0.1, 0.2, 0.3, 0.4);
        let v3 = c.to_vec3();
        assert_eq!(v3, Vec3::new(0.1, 0.2, 0.3));

        let v4 = c.to_vec4();
        assert_eq!(v4, Vec4::new(0.1, 0.2, 0.3, 0.4));

        let c2 = Color::from_vec4(v4);
        assert_eq!(c2, c);
    }

    #[test]
    fn test_color_lerp() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = a.lerp(b, 0.5);
        assert!((mid.r - 0.5).abs() < 0.0001);
        assert!((mid.g - 0.5).abs() < 0.0001);
        assert!((mid.b - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_color_with_alpha() {
        let c = Color::RED.with_alpha(0.5);
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 0.5);
    }

    // =========================================================================
    // FFI Layout Tests
    // =========================================================================

    #[test]
    fn test_ffi_layout_sizes() {
        use std::mem::size_of;

        // Verify types have expected sizes for FFI
        assert_eq!(size_of::<Vec2>(), 8); // 2 * f32
        assert_eq!(size_of::<Vec3>(), 12); // 3 * f32
        assert_eq!(size_of::<Vec4>(), 16); // 4 * f32
        assert_eq!(size_of::<Rect>(), 16); // 4 * f32
        assert_eq!(size_of::<Color>(), 16); // 4 * f32
    }
}
