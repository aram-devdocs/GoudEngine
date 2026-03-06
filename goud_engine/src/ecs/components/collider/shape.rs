//! Collision shape definitions.
//!
//! Defines the [`ColliderShape`] enum and associated geometry methods used by
//! the [`Collider`](super::Collider) component.

use crate::core::math::{Rect, Vec2};

// =============================================================================
// ColliderShape Enum
// =============================================================================

/// The geometric shape of a collider.
///
/// Each shape type has different performance characteristics and use cases:
///
/// - **Circle**: Fastest collision detection, best for balls, projectiles
/// - **Box**: Axis-aligned (AABB) or oriented (OBB), good for walls and platforms
/// - **Capsule**: Good for characters, combines efficiency with smooth edges
/// - **Polygon**: Most flexible but slowest, use sparingly for complex shapes
#[derive(Debug, Clone, PartialEq)]
pub enum ColliderShape {
    /// Circle collider defined by radius.
    ///
    /// Center is at the entity's position. Fastest collision detection.
    Circle {
        /// Radius of the circle in world units
        radius: f32,
    },

    /// Axis-Aligned Bounding Box (AABB).
    ///
    /// Defined by half-extents (half-width, half-height) from the center.
    /// Fast collision detection, no rotation support.
    Aabb {
        /// Half-extents (half-width, half-height)
        half_extents: Vec2,
    },

    /// Oriented Bounding Box (OBB).
    ///
    /// Similar to AABB but can be rotated. Slightly slower than AABB.
    Obb {
        /// Half-extents (half-width, half-height)
        half_extents: Vec2,
    },

    /// Capsule collider (rounded rectangle).
    ///
    /// Defined by half-height and radius. Good for character controllers.
    /// The capsule extends vertically from the center.
    Capsule {
        /// Half-height of the capsule's cylindrical section
        half_height: f32,
        /// Radius of the capsule's rounded ends
        radius: f32,
    },

    /// Convex polygon collider.
    ///
    /// Vertices must be in counter-clockwise order and form a convex hull.
    /// Slowest collision detection, use sparingly.
    Polygon {
        /// Vertices in local space, counter-clockwise order
        vertices: Vec<Vec2>,
    },
}

impl ColliderShape {
    /// Returns the type name of this shape.
    pub fn type_name(&self) -> &'static str {
        match self {
            ColliderShape::Circle { .. } => "Circle",
            ColliderShape::Aabb { .. } => "AABB",
            ColliderShape::Obb { .. } => "OBB",
            ColliderShape::Capsule { .. } => "Capsule",
            ColliderShape::Polygon { .. } => "Polygon",
        }
    }

    /// Computes the axis-aligned bounding box (AABB) for this shape.
    ///
    /// Returns a rectangle in local space that fully contains the shape.
    /// For rotated shapes (OBB, Polygon), this is a conservative bound.
    pub fn compute_aabb(&self) -> Rect {
        match self {
            ColliderShape::Circle { radius } => {
                Rect::new(-radius, -radius, radius * 2.0, radius * 2.0)
            }
            ColliderShape::Aabb { half_extents } | ColliderShape::Obb { half_extents } => {
                Rect::new(
                    -half_extents.x,
                    -half_extents.y,
                    half_extents.x * 2.0,
                    half_extents.y * 2.0,
                )
            }
            ColliderShape::Capsule {
                half_height,
                radius,
            } => {
                let width = radius * 2.0;
                let height = (half_height + radius) * 2.0;
                Rect::new(-radius, -(half_height + radius), width, height)
            }
            ColliderShape::Polygon { vertices } => {
                if vertices.is_empty() {
                    return Rect::unit();
                }

                let mut min_x = vertices[0].x;
                let mut min_y = vertices[0].y;
                let mut max_x = vertices[0].x;
                let mut max_y = vertices[0].y;

                for v in vertices.iter().skip(1) {
                    min_x = min_x.min(v.x);
                    min_y = min_y.min(v.y);
                    max_x = max_x.max(v.x);
                    max_y = max_y.max(v.y);
                }

                Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
            }
        }
    }

    /// Returns true if this shape is a circle.
    pub fn is_circle(&self) -> bool {
        matches!(self, ColliderShape::Circle { .. })
    }

    /// Returns true if this shape is an axis-aligned box (AABB).
    pub fn is_aabb(&self) -> bool {
        matches!(self, ColliderShape::Aabb { .. })
    }

    /// Returns true if this shape is an oriented box (OBB).
    pub fn is_obb(&self) -> bool {
        matches!(self, ColliderShape::Obb { .. })
    }

    /// Returns true if this shape is a capsule.
    pub fn is_capsule(&self) -> bool {
        matches!(self, ColliderShape::Capsule { .. })
    }

    /// Returns true if this shape is a polygon.
    pub fn is_polygon(&self) -> bool {
        matches!(self, ColliderShape::Polygon { .. })
    }

    /// Validates that the shape is well-formed.
    ///
    /// Returns `true` if:
    /// - Radii and extents are positive
    /// - Polygons have at least 3 vertices
    /// - Polygon vertices form a convex hull (not checked, assumed by user)
    pub fn is_valid(&self) -> bool {
        match self {
            ColliderShape::Circle { radius } => *radius > 0.0,
            ColliderShape::Aabb { half_extents } | ColliderShape::Obb { half_extents } => {
                half_extents.x > 0.0 && half_extents.y > 0.0
            }
            ColliderShape::Capsule {
                half_height,
                radius,
            } => *half_height > 0.0 && *radius > 0.0,
            ColliderShape::Polygon { vertices } => vertices.len() >= 3,
        }
    }
}

impl Default for ColliderShape {
    /// Returns a unit circle (radius 1.0) as the default shape.
    fn default() -> Self {
        ColliderShape::Circle { radius: 1.0 }
    }
}
