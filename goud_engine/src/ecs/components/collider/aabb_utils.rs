//! Axis-Aligned Bounding Box (AABB) utility functions.
//!
//! These functions are used for broad-phase collision detection, spatial queries,
//! and efficient geometric tests.

use crate::core::math::{Rect, Vec2};
use crate::ecs::components::{ColliderShape, Transform2D};

/// Computes the world-space AABB for a collider shape with a transform.
///
/// This takes a collider's local AABB and transforms it to world space
/// using the entity's Transform2D. The result is always axis-aligned
/// even if the shape is rotated.
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::components::{Collider, Transform2D, collider};
/// use goud_engine::core::math::Vec2;
///
/// let collider = Collider::circle(1.0);
/// let transform = Transform2D::from_position(Vec2::new(10.0, 20.0));
///
/// let world_aabb = collider::aabb::compute_world_aabb(collider.shape(), &transform);
/// assert_eq!(world_aabb.center(), Vec2::new(10.0, 20.0));
/// ```
pub fn compute_world_aabb(shape: &ColliderShape, transform: &Transform2D) -> Rect {
    let local_aabb = shape.compute_aabb();

    // For circles and AABBs without rotation, we can optimize
    if matches!(shape, ColliderShape::Circle { .. })
        || (matches!(shape, ColliderShape::Aabb { .. }) && transform.rotation.abs() < f32::EPSILON)
    {
        // Simple translation + scale
        let half_size = local_aabb.size() * 0.5;
        let scaled_half_size = Vec2::new(
            half_size.x * transform.scale.x.abs(),
            half_size.y * transform.scale.y.abs(),
        );
        let center = transform.position;
        return Rect::from_min_max(center - scaled_half_size, center + scaled_half_size);
    }

    // For rotated shapes, transform all corners and compute bounding box
    let corners = [
        Vec2::new(local_aabb.x, local_aabb.y),
        Vec2::new(local_aabb.x + local_aabb.width, local_aabb.y),
        Vec2::new(
            local_aabb.x + local_aabb.width,
            local_aabb.y + local_aabb.height,
        ),
        Vec2::new(local_aabb.x, local_aabb.y + local_aabb.height),
    ];

    let matrix = transform.matrix();
    let transformed_corners: Vec<Vec2> = corners
        .iter()
        .map(|&corner| matrix.transform_point(corner))
        .collect();

    // Find min/max of transformed corners
    let mut min_x = transformed_corners[0].x;
    let mut min_y = transformed_corners[0].y;
    let mut max_x = transformed_corners[0].x;
    let mut max_y = transformed_corners[0].y;

    for corner in &transformed_corners[1..] {
        min_x = min_x.min(corner.x);
        min_y = min_y.min(corner.y);
        max_x = max_x.max(corner.x);
        max_y = max_y.max(corner.y);
    }

    Rect::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
}

/// Tests if two AABBs overlap.
///
/// Returns true if the rectangles intersect or touch.
#[inline]
pub fn overlaps(a: &Rect, b: &Rect) -> bool {
    a.intersects(b)
}

/// Computes the intersection of two AABBs.
///
/// Returns Some(Rect) with the overlapping region, or None if they don't overlap.
#[inline]
pub fn intersection(a: &Rect, b: &Rect) -> Option<Rect> {
    a.intersection(b)
}

/// Expands an AABB by a margin on all sides.
///
/// Useful for creating query regions or tolerance zones.
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::components::collider;
/// use goud_engine::core::math::Rect;
///
/// let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
/// let expanded = collider::aabb::expand(&aabb, 1.0);
/// assert_eq!(expanded.width, 12.0);
/// assert_eq!(expanded.height, 12.0);
/// ```
pub fn expand(aabb: &Rect, margin: f32) -> Rect {
    Rect::new(
        aabb.x - margin,
        aabb.y - margin,
        aabb.width + margin * 2.0,
        aabb.height + margin * 2.0,
    )
}

/// Merges two AABBs into a single AABB that contains both.
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::components::collider;
/// use goud_engine::core::math::Rect;
///
/// let a = Rect::new(0.0, 0.0, 5.0, 5.0);
/// let b = Rect::new(3.0, 3.0, 5.0, 5.0);
/// let merged = collider::aabb::merge(&a, &b);
/// assert_eq!(merged.x, 0.0);
/// assert_eq!(merged.y, 0.0);
/// assert_eq!(merged.width, 8.0);
/// assert_eq!(merged.height, 8.0);
/// ```
pub fn merge(a: &Rect, b: &Rect) -> Rect {
    let min_x = a.x.min(b.x);
    let min_y = a.y.min(b.y);
    let max_x = (a.x + a.width).max(b.x + b.width);
    let max_y = (a.y + a.height).max(b.y + b.height);
    Rect::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
}

/// Tests if a point is inside an AABB.
#[inline]
pub fn contains_point(aabb: &Rect, point: Vec2) -> bool {
    aabb.contains(point)
}

/// Performs a raycast against an AABB.
///
/// Returns Some(t) with the intersection parameter [0, 1] if the ray hits,
/// or None if it misses. The intersection point is: ray_origin + ray_direction * t.
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::components::collider;
/// use goud_engine::core::math::{Rect, Vec2};
///
/// let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
/// let ray_origin = Vec2::new(-5.0, 5.0);
/// let ray_direction = Vec2::new(1.0, 0.0);
///
/// let hit = collider::aabb::raycast(&aabb, ray_origin, ray_direction, 100.0);
/// assert!(hit.is_some());
/// ```
pub fn raycast(
    aabb: &Rect,
    ray_origin: Vec2,
    ray_direction: Vec2,
    max_distance: f32,
) -> Option<f32> {
    // Slab method for AABB raycast
    let inv_dir = Vec2::new(
        if ray_direction.x.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            1.0 / ray_direction.x
        },
        if ray_direction.y.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            1.0 / ray_direction.y
        },
    );

    let min = aabb.min();
    let max = aabb.max();

    let t1 = (min.x - ray_origin.x) * inv_dir.x;
    let t2 = (max.x - ray_origin.x) * inv_dir.x;
    let t3 = (min.y - ray_origin.y) * inv_dir.y;
    let t4 = (max.y - ray_origin.y) * inv_dir.y;

    let tmin = t1.min(t2).max(t3.min(t4)).max(0.0);
    let tmax = t1.max(t2).min(t3.max(t4)).min(max_distance);

    if tmax >= tmin && tmin <= max_distance {
        Some(tmin)
    } else {
        None
    }
}

/// Computes the closest point on an AABB to a given point.
///
/// If the point is inside the AABB, returns the point itself.
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::components::collider;
/// use goud_engine::core::math::{Rect, Vec2};
///
/// let aabb = Rect::new(0.0, 0.0, 10.0, 10.0);
/// let point = Vec2::new(-5.0, 5.0);
///
/// let closest = collider::aabb::closest_point(&aabb, point);
/// assert_eq!(closest, Vec2::new(0.0, 5.0));
/// ```
pub fn closest_point(aabb: &Rect, point: Vec2) -> Vec2 {
    Vec2::new(
        point.x.clamp(aabb.x, aabb.x + aabb.width),
        point.y.clamp(aabb.y, aabb.y + aabb.height),
    )
}

/// Computes the squared distance from a point to the surface of an AABB.
///
/// Returns 0.0 if the point is inside the AABB.
pub fn distance_squared_to_point(aabb: &Rect, point: Vec2) -> f32 {
    let closest = closest_point(aabb, point);
    let dx = point.x - closest.x;
    let dy = point.y - closest.y;
    dx * dx + dy * dy
}

/// Computes the area of an AABB.
#[inline]
pub fn area(aabb: &Rect) -> f32 {
    aabb.area()
}

/// Computes the perimeter of an AABB.
pub fn perimeter(aabb: &Rect) -> f32 {
    2.0 * (aabb.width + aabb.height)
}
