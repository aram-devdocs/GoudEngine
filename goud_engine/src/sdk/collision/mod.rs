//! # SDK Collision Detection API
//!
//! Ergonomic Rust functions for 2D collision detection. Thin wrappers over
//! [`crate::ecs::collision`] -- pure math, no engine state required.
//!
//! # Example
//!
//! ```rust
//! use goud_engine::sdk::collision::{circle_circle, point_in_rect, distance};
//! use goud_engine::core::math::Vec2;
//!
//! if let Some(contact) = circle_circle(
//!     Vec2::new(0.0, 0.0), 1.0,
//!     Vec2::new(1.5, 0.0), 1.0,
//! ) {
//!     println!("Collision! Penetration: {}", contact.penetration);
//! }
//!
//! assert!(point_in_rect(Vec2::new(5.0, 5.0), Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)));
//! assert!((distance(Vec2::new(0.0, 0.0), Vec2::new(3.0, 4.0)) - 5.0).abs() < 0.001);
//! ```

use crate::core::math::Vec2;

// Re-export Contact from the internal collision module so SDK users
// do not need to reach into `ecs::collision` directly.
pub use crate::ecs::collision::Contact;

// Also re-export CollisionResponse for physics resolution workflows.
pub use crate::ecs::collision::CollisionResponse;

// =============================================================================
// Collision API struct (provides FFI generation via proc-macro)
// =============================================================================

/// Zero-sized type that hosts collision detection functions.
///
/// All methods are static (no `self` receiver) and are used by the
/// `#[goud_api]` proc-macro to auto-generate `#[no_mangle] extern "C"`
/// FFI wrappers. The free functions below delegate to these methods.
pub struct Collision;

// NOTE: FFI wrappers are hand-written in ffi/collision.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl Collision {
    /// Tests collision between two axis-aligned bounding boxes.
    pub fn aabb_aabb(
        center_a: Vec2,
        half_extents_a: Vec2,
        center_b: Vec2,
        half_extents_b: Vec2,
    ) -> Option<Contact> {
        crate::ecs::collision::aabb_aabb_collision(
            center_a,
            half_extents_a,
            center_b,
            half_extents_b,
        )
    }

    /// Tests collision between two circles.
    pub fn circle_circle(
        center_a: Vec2,
        radius_a: f32,
        center_b: Vec2,
        radius_b: f32,
    ) -> Option<Contact> {
        crate::ecs::collision::circle_circle_collision(center_a, radius_a, center_b, radius_b)
    }

    /// Tests collision between a circle and an AABB.
    pub fn circle_aabb(
        circle_center: Vec2,
        circle_radius: f32,
        box_center: Vec2,
        box_half_extents: Vec2,
    ) -> Option<Contact> {
        crate::ecs::collision::circle_aabb_collision(
            circle_center,
            circle_radius,
            box_center,
            box_half_extents,
        )
    }

    /// Checks whether a point lies inside a rectangle.
    pub fn point_in_rect(point: Vec2, rect_origin: Vec2, rect_size: Vec2) -> bool {
        point.x >= rect_origin.x
            && point.x <= rect_origin.x + rect_size.x
            && point.y >= rect_origin.y
            && point.y <= rect_origin.y + rect_size.y
    }

    /// Checks whether a point lies inside a circle.
    pub fn point_in_circle(point: Vec2, circle_center: Vec2, circle_radius: f32) -> bool {
        let dx = point.x - circle_center.x;
        let dy = point.y - circle_center.y;
        (dx * dx + dy * dy) <= (circle_radius * circle_radius)
    }

    /// Fast boolean overlap test for two AABBs using min/max corners.
    pub fn aabb_overlap(min_a: Vec2, max_a: Vec2, min_b: Vec2, max_b: Vec2) -> bool {
        max_a.x >= min_b.x && min_a.x <= max_b.x && max_a.y >= min_b.y && min_a.y <= max_b.y
    }

    /// Fast boolean overlap test for two circles.
    pub fn circle_overlap(center_a: Vec2, radius_a: f32, center_b: Vec2, radius_b: f32) -> bool {
        let dx = center_b.x - center_a.x;
        let dy = center_b.y - center_a.y;
        let combined_radius = radius_a + radius_b;
        (dx * dx + dy * dy) <= (combined_radius * combined_radius)
    }

    /// Returns the Euclidean distance between two points.
    pub fn distance(a: Vec2, b: Vec2) -> f32 {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Returns the squared Euclidean distance between two points.
    pub fn distance_squared(a: Vec2, b: Vec2) -> f32 {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        dx * dx + dy * dy
    }
}

// =============================================================================
// Public free-function API (delegates to Collision methods)
// =============================================================================

/// Tests collision between two axis-aligned bounding boxes.
///
/// Returns contact information (penetration depth, normal, contact point) if
/// the boxes overlap, or `None` if they are separated.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::collision::aabb_aabb;
/// use goud_engine::core::math::Vec2;
///
/// let contact = aabb_aabb(
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
///     Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0),
/// );
/// assert!(contact.is_some());
/// ```
#[inline]
pub fn aabb_aabb(
    center_a: Vec2,
    half_extents_a: Vec2,
    center_b: Vec2,
    half_extents_b: Vec2,
) -> Option<Contact> {
    Collision::aabb_aabb(center_a, half_extents_a, center_b, half_extents_b)
}

/// Tests collision between two circles (fastest collision test).
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::collision::circle_circle;
/// use goud_engine::core::math::Vec2;
///
/// let contact = circle_circle(
///     Vec2::new(0.0, 0.0), 1.0,
///     Vec2::new(1.5, 0.0), 1.0,
/// );
/// assert!(contact.is_some());
/// ```
#[inline]
pub fn circle_circle(
    center_a: Vec2,
    radius_a: f32,
    center_b: Vec2,
    radius_b: f32,
) -> Option<Contact> {
    Collision::circle_circle(center_a, radius_a, center_b, radius_b)
}

/// Tests collision between a circle and an axis-aligned bounding box.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::collision::circle_aabb;
/// use goud_engine::core::math::Vec2;
///
/// let contact = circle_aabb(
///     Vec2::new(0.0, 0.0), 1.0,
///     Vec2::new(2.0, 0.0), Vec2::new(1.0, 1.0),
/// );
/// // Circle at origin with radius 1, box centered at (2,0) with half-extents (1,1)
/// ```
#[inline]
pub fn circle_aabb(
    circle_center: Vec2,
    circle_radius: f32,
    box_center: Vec2,
    box_half_extents: Vec2,
) -> Option<Contact> {
    Collision::circle_aabb(circle_center, circle_radius, box_center, box_half_extents)
}

/// Checks whether a point lies inside a rectangle.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::collision::point_in_rect;
/// use goud_engine::core::math::Vec2;
///
/// assert!(point_in_rect(
///     Vec2::new(5.0, 5.0),
///     Vec2::new(0.0, 0.0),
///     Vec2::new(10.0, 10.0),
/// ));
/// assert!(!point_in_rect(
///     Vec2::new(15.0, 5.0),
///     Vec2::new(0.0, 0.0),
///     Vec2::new(10.0, 10.0),
/// ));
/// ```
#[inline]
pub fn point_in_rect(point: Vec2, rect_origin: Vec2, rect_size: Vec2) -> bool {
    Collision::point_in_rect(point, rect_origin, rect_size)
}

/// Checks whether a point lies inside a circle.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::collision::point_in_circle;
/// use goud_engine::core::math::Vec2;
///
/// assert!(point_in_circle(Vec2::new(0.5, 0.0), Vec2::new(0.0, 0.0), 1.0));
/// assert!(!point_in_circle(Vec2::new(2.0, 0.0), Vec2::new(0.0, 0.0), 1.0));
/// ```
#[inline]
pub fn point_in_circle(point: Vec2, circle_center: Vec2, circle_radius: f32) -> bool {
    Collision::point_in_circle(point, circle_center, circle_radius)
}

/// Fast boolean overlap test for two AABBs using min/max corners.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::collision::aabb_overlap;
/// use goud_engine::core::math::Vec2;
///
/// let overlapping = aabb_overlap(
///     Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0),
///     Vec2::new(1.0, 1.0), Vec2::new(3.0, 3.0),
/// );
/// assert!(overlapping);
/// ```
#[inline]
pub fn aabb_overlap(min_a: Vec2, max_a: Vec2, min_b: Vec2, max_b: Vec2) -> bool {
    Collision::aabb_overlap(min_a, max_a, min_b, max_b)
}

/// Fast boolean overlap test for two circles.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::collision::circle_overlap;
/// use goud_engine::core::math::Vec2;
///
/// assert!(circle_overlap(Vec2::new(0.0, 0.0), 1.0, Vec2::new(1.5, 0.0), 1.0));
/// assert!(!circle_overlap(Vec2::new(0.0, 0.0), 1.0, Vec2::new(5.0, 0.0), 1.0));
/// ```
#[inline]
pub fn circle_overlap(center_a: Vec2, radius_a: f32, center_b: Vec2, radius_b: f32) -> bool {
    Collision::circle_overlap(center_a, radius_a, center_b, radius_b)
}

/// Returns the Euclidean distance between two points.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::collision::distance;
/// use goud_engine::core::math::Vec2;
///
/// let d = distance(Vec2::new(0.0, 0.0), Vec2::new(3.0, 4.0));
/// assert!((d - 5.0).abs() < 0.001);
/// ```
#[inline]
pub fn distance(a: Vec2, b: Vec2) -> f32 {
    Collision::distance(a, b)
}

/// Returns the squared Euclidean distance between two points.
///
/// # Example
///
/// ```rust
/// use goud_engine::sdk::collision::distance_squared;
/// use goud_engine::core::math::Vec2;
///
/// let d2 = distance_squared(Vec2::new(0.0, 0.0), Vec2::new(3.0, 4.0));
/// assert!((d2 - 25.0).abs() < 0.001);
/// ```
#[inline]
pub fn distance_squared(a: Vec2, b: Vec2) -> f32 {
    Collision::distance_squared(a, b)
}

#[cfg(test)]
#[path = "tests.rs"]
#[cfg(test)]
mod tests;
