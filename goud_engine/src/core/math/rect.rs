//! 2D rectangle type with FFI-safe memory layout.

use super::vec2::Vec2;

/// A 2D rectangle with FFI-safe memory layout.
///
/// Defined by position (x, y) and size (width, height).
/// The position represents the top-left corner in screen space.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
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
