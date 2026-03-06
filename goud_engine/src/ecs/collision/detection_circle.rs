//! Circle-based collision detection algorithms.
//!
//! Provides collision detection for:
//! - Circle vs Circle
//! - Circle vs AABB (axis-aligned bounding box)
//! - Circle vs OBB (oriented bounding box)

use crate::core::math::Vec2;
use crate::ecs::collision::contact::Contact;

/// Detects collision between two circles.
///
/// This is the fastest collision detection algorithm, requiring only a single
/// distance check. Returns contact information if the circles overlap.
///
/// # Arguments
///
/// * `center_a` - Center of circle A
/// * `radius_a` - Radius of circle A (must be positive)
/// * `center_b` - Center of circle B
/// * `radius_b` - Radius of circle B (must be positive)
///
/// # Returns
///
/// - `Some(Contact)` if the circles are overlapping
/// - `None` if the circles are separated
///
/// # Example
///
/// ```
/// use goud_engine::ecs::collision::circle_circle_collision;
/// use goud_engine::core::math::Vec2;
///
/// // Overlapping circles
/// let contact = circle_circle_collision(
///     Vec2::new(0.0, 0.0), 1.0,
///     Vec2::new(1.5, 0.0), 1.0
/// );
/// assert!(contact.is_some());
/// assert!(contact.unwrap().penetration > 0.0);
///
/// // Separated circles
/// let no_contact = circle_circle_collision(
///     Vec2::new(0.0, 0.0), 1.0,
///     Vec2::new(5.0, 0.0), 1.0
/// );
/// assert!(no_contact.is_none());
/// ```
///
/// # Performance
///
/// O(1) - Single distance computation and comparison.
pub fn circle_circle_collision(
    center_a: Vec2,
    radius_a: f32,
    center_b: Vec2,
    radius_b: f32,
) -> Option<Contact> {
    // Vector from A to B
    let delta = center_b - center_a;
    let distance_squared = delta.length_squared();
    let combined_radius = radius_a + radius_b;
    let combined_radius_squared = combined_radius * combined_radius;

    // Early exit if circles are separated
    if distance_squared > combined_radius_squared {
        return None;
    }

    // Handle edge case: circles at exact same position
    if distance_squared < 1e-6 {
        // Circles are basically at the same position, use arbitrary normal
        return Some(Contact {
            point: center_a,
            normal: Vec2::unit_x(),
            penetration: combined_radius,
        });
    }

    // Compute distance and normal
    let distance = distance_squared.sqrt();
    let normal = delta / distance;

    // Penetration is how much they overlap
    let penetration = combined_radius - distance;

    // Contact point is halfway between the closest points on each circle
    let contact_point = center_a + normal * (radius_a - penetration * 0.5);

    Some(Contact {
        point: contact_point,
        normal,
        penetration,
    })
}

/// Detects collision between a circle and an axis-aligned bounding box (AABB).
///
/// This uses the "closest point" algorithm:
/// 1. Find the closest point on the AABB to the circle center
/// 2. Check if that point is within the circle's radius
///
/// This is more efficient than the full OBB version since it doesn't require
/// rotation transformations.
///
/// # Arguments
///
/// * `circle_center` - Center of the circle
/// * `circle_radius` - Radius of the circle (must be positive)
/// * `box_center` - Center of the AABB
/// * `box_half_extents` - Half-width and half-height of the AABB
///
/// # Returns
///
/// - `Some(Contact)` if the circle and AABB are overlapping
/// - `None` if they are separated
///
/// # Example
///
/// ```
/// use goud_engine::ecs::collision::circle_aabb_collision;
/// use goud_engine::core::math::Vec2;
///
/// // Circle overlapping with AABB
/// let contact = circle_aabb_collision(
///     Vec2::new(1.5, 0.0), 1.0,
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)
/// );
/// assert!(contact.is_some());
///
/// // Circle separated from AABB
/// let no_contact = circle_aabb_collision(
///     Vec2::new(5.0, 0.0), 1.0,
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)
/// );
/// assert!(no_contact.is_none());
/// ```
///
/// # Performance
///
/// O(1) - Simple clamping and distance check.
pub fn circle_aabb_collision(
    circle_center: Vec2,
    circle_radius: f32,
    box_center: Vec2,
    box_half_extents: Vec2,
) -> Option<Contact> {
    // Compute AABB bounds
    let box_min = box_center - box_half_extents;
    let box_max = box_center + box_half_extents;

    // Find the closest point on the AABB to the circle center
    let closest_point = Vec2::new(
        circle_center.x.clamp(box_min.x, box_max.x),
        circle_center.y.clamp(box_min.y, box_max.y),
    );

    // Vector from closest point to circle center
    let delta = circle_center - closest_point;
    let distance_squared = delta.length_squared();
    let radius_squared = circle_radius * circle_radius;

    // Early exit if separated
    if distance_squared > radius_squared {
        return None;
    }

    // Handle edge case: circle center inside box
    if distance_squared < 1e-6 {
        // Circle center is inside the box, find the axis of minimum penetration
        let penetration_x =
            (box_half_extents.x - (circle_center.x - box_center.x).abs()) + circle_radius;
        let penetration_y =
            (box_half_extents.y - (circle_center.y - box_center.y).abs()) + circle_radius;

        if penetration_x < penetration_y {
            // Push out along X axis
            let normal = if circle_center.x > box_center.x {
                Vec2::unit_x()
            } else {
                Vec2::new(-1.0, 0.0)
            };
            return Some(Contact {
                point: Vec2::new(
                    if circle_center.x > box_center.x {
                        box_max.x
                    } else {
                        box_min.x
                    },
                    circle_center.y,
                ),
                normal,
                penetration: penetration_x,
            });
        } else {
            // Push out along Y axis
            let normal = if circle_center.y > box_center.y {
                Vec2::unit_y()
            } else {
                Vec2::new(0.0, -1.0)
            };
            return Some(Contact {
                point: Vec2::new(
                    circle_center.x,
                    if circle_center.y > box_center.y {
                        box_max.y
                    } else {
                        box_min.y
                    },
                ),
                normal,
                penetration: penetration_y,
            });
        }
    }

    // Compute distance and normal
    let distance = distance_squared.sqrt();
    let normal = delta / distance;

    // Penetration is how much the circle overlaps with the box
    let penetration = circle_radius - distance;

    // Contact point is on the surface of the circle
    let contact_point = closest_point;

    Some(Contact {
        point: contact_point,
        normal,
        penetration,
    })
}

/// Detects collision between a circle and an oriented bounding box (OBB).
///
/// This is a more general version of circle-AABB collision that handles
/// rotated boxes. The algorithm:
/// 1. Transform the circle into the box's local coordinate space
/// 2. Perform circle-AABB collision in local space
/// 3. Transform the contact back to world space
///
/// # Arguments
///
/// * `circle_center` - Center of the circle in world space
/// * `circle_radius` - Radius of the circle (must be positive)
/// * `box_center` - Center of the OBB in world space
/// * `box_half_extents` - Half-width and half-height of the OBB
/// * `box_rotation` - Rotation angle of the OBB in radians
///
/// # Returns
///
/// - `Some(Contact)` if the circle and OBB are overlapping
/// - `None` if they are separated
///
/// # Example
///
/// ```
/// use goud_engine::ecs::collision::circle_obb_collision;
/// use goud_engine::core::math::Vec2;
/// use std::f32::consts::PI;
///
/// // Circle overlapping with rotated box
/// let contact = circle_obb_collision(
///     Vec2::new(1.5, 0.0), 1.0,
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), PI / 4.0
/// );
/// assert!(contact.is_some());
///
/// // Circle separated from rotated box
/// let no_contact = circle_obb_collision(
///     Vec2::new(5.0, 0.0), 1.0,
///     Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0), PI / 4.0
/// );
/// assert!(no_contact.is_none());
/// ```
///
/// # Performance
///
/// O(1) - Coordinate transformation + AABB collision check.
/// More expensive than AABB version due to sin/cos calculations.
pub fn circle_obb_collision(
    circle_center: Vec2,
    circle_radius: f32,
    box_center: Vec2,
    box_half_extents: Vec2,
    box_rotation: f32,
) -> Option<Contact> {
    // If rotation is near zero, use faster AABB version
    if box_rotation.abs() < 1e-6 {
        return circle_aabb_collision(circle_center, circle_radius, box_center, box_half_extents);
    }

    // Compute rotation matrix for box (inverse rotation to transform to local space)
    let cos_r = box_rotation.cos();
    let sin_r = box_rotation.sin();

    // Transform circle center to box's local coordinate space
    let delta = circle_center - box_center;
    let local_circle_center = Vec2::new(
        delta.x * cos_r + delta.y * sin_r,
        -delta.x * sin_r + delta.y * cos_r,
    );

    // Perform collision in local space (box is axis-aligned here)
    let local_contact = circle_aabb_collision(
        local_circle_center,
        circle_radius,
        Vec2::zero(), // Box is at origin in local space
        box_half_extents,
    )?;

    // Transform contact back to world space
    let world_normal = Vec2::new(
        local_contact.normal.x * cos_r - local_contact.normal.y * sin_r,
        local_contact.normal.x * sin_r + local_contact.normal.y * cos_r,
    );

    let world_point = Vec2::new(
        local_contact.point.x * cos_r - local_contact.point.y * sin_r,
        local_contact.point.x * sin_r + local_contact.point.y * cos_r,
    ) + box_center;

    Some(Contact {
        point: world_point,
        normal: world_normal,
        penetration: local_contact.penetration,
    })
}
