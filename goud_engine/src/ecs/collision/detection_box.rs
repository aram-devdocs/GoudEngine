//! Box-based collision detection algorithms.
//!
//! Provides collision detection for:
//! - OBB vs OBB (Separating Axis Theorem)
//! - AABB vs AABB (axis-aligned fast path)

use crate::core::math::Vec2;
use crate::ecs::collision::contact::Contact;

/// Detects collision between two oriented bounding boxes (OBBs) using SAT.
///
/// The Separating Axis Theorem (SAT) is the standard algorithm for OBB collision
/// detection. It tests for separation along potential separating axes. If no
/// separating axis is found, the boxes are colliding.
///
/// # Arguments
///
/// * `center_a` - Center of box A
/// * `half_extents_a` - Half-width and half-height of box A
/// * `rotation_a` - Rotation angle of box A in radians
/// * `center_b` - Center of box B
/// * `half_extents_b` - Half-width and half-height of box B
/// * `rotation_b` - Rotation angle of box B in radians
///
/// # Returns
///
/// - `Some(Contact)` if the boxes are overlapping
/// - `None` if the boxes are separated
///
/// # Algorithm
///
/// SAT tests for separation along 4 potential axes:
/// 1. Box A's X axis (rotated)
/// 2. Box A's Y axis (rotated)
/// 3. Box B's X axis (rotated)
/// 4. Box B's Y axis (rotated)
///
/// For each axis, we project both boxes and check for overlap.
/// If any axis shows no overlap, the boxes are separated.
/// Otherwise, we track the axis with minimum overlap (the collision normal).
///
/// # Example
///
/// ```
/// use goud_engine::ecs::collision::box_box_collision;
/// use goud_engine::core::math::Vec2;
///
/// // Two axis-aligned boxes overlapping
/// let contact = box_box_collision(
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
///     Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0), 0.0
/// );
/// assert!(contact.is_some());
///
/// // Two rotated boxes separated
/// let no_contact = box_box_collision(
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), 0.0,
///     Vec2::new(5.0, 0.0), Vec2::new(1.0, 1.0), std::f32::consts::PI / 4.0
/// );
/// assert!(no_contact.is_none());
/// ```
///
/// # Performance
///
/// O(1) - Tests 4 axes, each with constant-time projection and overlap checks.
/// More expensive than AABB-AABB due to rotation handling.
pub fn box_box_collision(
    center_a: Vec2,
    half_extents_a: Vec2,
    rotation_a: f32,
    center_b: Vec2,
    half_extents_b: Vec2,
    rotation_b: f32,
) -> Option<Contact> {
    // Compute rotation matrices (cos/sin)
    let cos_a = rotation_a.cos();
    let sin_a = rotation_a.sin();
    let cos_b = rotation_b.cos();
    let sin_b = rotation_b.sin();

    // Box A axes (rotated)
    let axis_a_x = Vec2::new(cos_a, sin_a);
    let axis_a_y = Vec2::new(-sin_a, cos_a);

    // Box B axes (rotated)
    let axis_b_x = Vec2::new(cos_b, sin_b);
    let axis_b_y = Vec2::new(-sin_b, cos_b);

    // Center offset
    let delta = center_b - center_a;

    // Track minimum overlap and collision normal
    let mut min_overlap = f32::INFINITY;
    let mut collision_normal = Vec2::unit_x();

    // Test all 4 potential separating axes
    let axes = [
        (axis_a_x, half_extents_a.x, half_extents_a.y),
        (axis_a_y, half_extents_a.x, half_extents_a.y),
        (axis_b_x, half_extents_b.x, half_extents_b.y),
        (axis_b_y, half_extents_b.x, half_extents_b.y),
    ];

    for (axis, _hx_a, _hy_a) in &axes {
        // Project box A onto axis
        let r_a = half_extents_a.x * (axis.dot(axis_a_x)).abs()
            + half_extents_a.y * (axis.dot(axis_a_y)).abs();

        // Project box B onto axis
        let r_b = half_extents_b.x * (axis.dot(axis_b_x)).abs()
            + half_extents_b.y * (axis.dot(axis_b_y)).abs();

        // Project center offset onto axis
        let distance = axis.dot(delta).abs();

        // Check for separation
        let overlap = r_a + r_b - distance;

        if overlap < 0.0 {
            // Found separating axis - no collision
            return None;
        }

        // Track minimum overlap (shallowest penetration)
        if overlap < min_overlap {
            min_overlap = overlap;
            collision_normal = *axis;

            // Ensure normal points from A to B
            if axis.dot(delta) < 0.0 {
                collision_normal = collision_normal * -1.0;
            }
        }
    }

    // No separating axis found - boxes are colliding
    let contact_point = center_a + delta * 0.5; // Approximate contact point

    Some(Contact {
        point: contact_point,
        normal: collision_normal,
        penetration: min_overlap,
    })
}

/// Detects collision between two axis-aligned bounding boxes (AABBs).
///
/// This is a specialized, faster version of box-box collision for axis-aligned
/// boxes (rotation = 0). It's much simpler than the full SAT algorithm.
///
/// # Arguments
///
/// * `center_a` - Center of box A
/// * `half_extents_a` - Half-width and half-height of box A
/// * `center_b` - Center of box B
/// * `half_extents_b` - Half-width and half-height of box B
///
/// # Returns
///
/// - `Some(Contact)` if the boxes are overlapping
/// - `None` if the boxes are separated
///
/// # Example
///
/// ```
/// use goud_engine::ecs::collision::aabb_aabb_collision;
/// use goud_engine::core::math::Vec2;
///
/// // Two AABBs overlapping
/// let contact = aabb_aabb_collision(
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0),
///     Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0)
/// );
/// assert!(contact.is_some());
/// ```
///
/// # Performance
///
/// O(1) - Simple comparison checks, no trigonometry required.
pub fn aabb_aabb_collision(
    center_a: Vec2,
    half_extents_a: Vec2,
    center_b: Vec2,
    half_extents_b: Vec2,
) -> Option<Contact> {
    // Compute min/max bounds
    let min_a = center_a - half_extents_a;
    let max_a = center_a + half_extents_a;
    let min_b = center_b - half_extents_b;
    let max_b = center_b + half_extents_b;

    // Check for overlap on X axis
    if max_a.x < min_b.x || max_b.x < min_a.x {
        return None;
    }

    // Check for overlap on Y axis
    if max_a.y < min_b.y || max_b.y < min_a.y {
        return None;
    }

    // Compute overlap on each axis
    let overlap_x = (max_a.x.min(max_b.x) - min_a.x.max(min_b.x)).abs();
    let overlap_y = (max_a.y.min(max_b.y) - min_a.y.max(min_b.y)).abs();

    // Find minimum overlap axis (collision normal)
    let (penetration, normal) = if overlap_x < overlap_y {
        // X axis is minimum
        let normal = if center_b.x > center_a.x {
            Vec2::unit_x()
        } else {
            Vec2::new(-1.0, 0.0)
        };
        (overlap_x, normal)
    } else {
        // Y axis is minimum
        let normal = if center_b.y > center_a.y {
            Vec2::unit_y()
        } else {
            Vec2::new(0.0, -1.0)
        };
        (overlap_y, normal)
    };

    // Contact point is at the center of overlap region
    let contact_point = Vec2::new(
        (min_a.x.max(min_b.x) + max_a.x.min(max_b.x)) * 0.5,
        (min_a.y.max(min_b.y) + max_a.y.min(max_b.y)) * 0.5,
    );

    Some(Contact {
        point: contact_point,
        normal,
        penetration,
    })
}
