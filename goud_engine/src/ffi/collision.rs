//! # FFI Collision Module
//!
//! This module provides C-compatible functions for collision detection.
//! It wraps the internal ECS collision detection algorithms for use by the C# SDK.
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // Check if two AABBs are colliding
//! var contact = new GoudContact();
//! if (goud_collision_aabb_aabb(x1, y1, hw1, hh1, x2, y2, hw2, hh2, ref contact)) {
//!     // Handle collision
//!     float penetration = contact.penetration;
//!     float normalX = contact.normal_x;
//!     float normalY = contact.normal_y;
//! }
//!
//! // Check if point is inside a rectangle
//! if (goud_collision_point_in_rect(px, py, rx, ry, rw, rh)) {
//!     // Point is inside rectangle
//! }
//! ```

use crate::core::math::Vec2;
use crate::ecs::collision::{aabb_aabb_collision, circle_aabb_collision, circle_circle_collision};

// ============================================================================
// Contact Structure (FFI-compatible)
// ============================================================================

/// FFI-compatible contact information from a collision.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GoudContact {
    /// Contact point X coordinate
    pub point_x: f32,
    /// Contact point Y coordinate
    pub point_y: f32,
    /// Collision normal X component (unit vector pointing from A to B)
    pub normal_x: f32,
    /// Collision normal Y component
    pub normal_y: f32,
    /// Penetration depth (positive = overlapping)
    pub penetration: f32,
}

impl From<crate::ecs::collision::Contact> for GoudContact {
    fn from(contact: crate::ecs::collision::Contact) -> Self {
        Self {
            point_x: contact.point.x,
            point_y: contact.point.y,
            normal_x: contact.normal.x,
            normal_y: contact.normal.y,
            penetration: contact.penetration,
        }
    }
}

// ============================================================================
// AABB Collision
// ============================================================================

/// Checks collision between two axis-aligned bounding boxes.
///
/// # Arguments
///
/// * `center_a_x/y` - Center of the first AABB
/// * `half_w_a/h_a` - Half-width and half-height of the first AABB
/// * `center_b_x/y` - Center of the second AABB
/// * `half_w_b/h_b` - Half-width and half-height of the second AABB
/// * `out_contact` - Pointer to store contact information (can be null)
///
/// # Returns
///
/// `true` if the AABBs are colliding, `false` otherwise.
///
/// # Safety
///
/// `out_contact` can be null if collision info is not needed, otherwise must be valid.
#[no_mangle]
pub unsafe extern "C" fn goud_collision_aabb_aabb(
    center_a_x: f32,
    center_a_y: f32,
    half_w_a: f32,
    half_h_a: f32,
    center_b_x: f32,
    center_b_y: f32,
    half_w_b: f32,
    half_h_b: f32,
    out_contact: *mut GoudContact,
) -> bool {
    let center_a = Vec2::new(center_a_x, center_a_y);
    let half_extents_a = Vec2::new(half_w_a, half_h_a);
    let center_b = Vec2::new(center_b_x, center_b_y);
    let half_extents_b = Vec2::new(half_w_b, half_h_b);

    match aabb_aabb_collision(center_a, half_extents_a, center_b, half_extents_b) {
        Some(contact) => {
            if !out_contact.is_null() {
                *out_contact = contact.into();
            }
            true
        }
        None => false,
    }
}

// ============================================================================
// Circle-Circle Collision
// ============================================================================

/// Checks collision between two circles.
///
/// # Arguments
///
/// * `center_a_x/y` - Center of the first circle
/// * `radius_a` - Radius of the first circle
/// * `center_b_x/y` - Center of the second circle
/// * `radius_b` - Radius of the second circle
/// * `out_contact` - Pointer to store contact information (can be null)
///
/// # Returns
///
/// `true` if the circles are colliding, `false` otherwise.
///
/// # Safety
///
/// `out_contact` can be null if collision info is not needed, otherwise must be valid.
#[no_mangle]
pub unsafe extern "C" fn goud_collision_circle_circle(
    center_a_x: f32,
    center_a_y: f32,
    radius_a: f32,
    center_b_x: f32,
    center_b_y: f32,
    radius_b: f32,
    out_contact: *mut GoudContact,
) -> bool {
    let center_a = Vec2::new(center_a_x, center_a_y);
    let center_b = Vec2::new(center_b_x, center_b_y);

    match circle_circle_collision(center_a, radius_a, center_b, radius_b) {
        Some(contact) => {
            if !out_contact.is_null() {
                *out_contact = contact.into();
            }
            true
        }
        None => false,
    }
}

// ============================================================================
// Circle-AABB Collision
// ============================================================================

/// Checks collision between a circle and an axis-aligned bounding box.
///
/// # Arguments
///
/// * `circle_x/y` - Center of the circle
/// * `circle_radius` - Radius of the circle
/// * `box_x/y` - Center of the AABB
/// * `box_hw/hh` - Half-width and half-height of the AABB
/// * `out_contact` - Pointer to store contact information (can be null)
///
/// # Returns
///
/// `true` if colliding, `false` otherwise.
///
/// # Safety
///
/// `out_contact` can be null if collision info is not needed, otherwise must be valid.
#[no_mangle]
pub unsafe extern "C" fn goud_collision_circle_aabb(
    circle_x: f32,
    circle_y: f32,
    circle_radius: f32,
    box_x: f32,
    box_y: f32,
    box_hw: f32,
    box_hh: f32,
    out_contact: *mut GoudContact,
) -> bool {
    let circle_center = Vec2::new(circle_x, circle_y);
    let box_center = Vec2::new(box_x, box_y);
    let box_half_extents = Vec2::new(box_hw, box_hh);

    match circle_aabb_collision(circle_center, circle_radius, box_center, box_half_extents) {
        Some(contact) => {
            if !out_contact.is_null() {
                *out_contact = contact.into();
            }
            true
        }
        None => false,
    }
}

// ============================================================================
// Point-in-Shape Tests
// ============================================================================

/// Checks if a point is inside a rectangle.
///
/// # Arguments
///
/// * `point_x/y` - The point coordinates
/// * `rect_x/y` - Top-left corner of the rectangle
/// * `rect_w/h` - Width and height of the rectangle
///
/// # Returns
///
/// `true` if the point is inside the rectangle, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_collision_point_in_rect(
    point_x: f32,
    point_y: f32,
    rect_x: f32,
    rect_y: f32,
    rect_w: f32,
    rect_h: f32,
) -> bool {
    point_x >= rect_x
        && point_x <= rect_x + rect_w
        && point_y >= rect_y
        && point_y <= rect_y + rect_h
}

/// Checks if a point is inside a circle.
///
/// # Arguments
///
/// * `point_x/y` - The point coordinates
/// * `circle_x/y` - Center of the circle
/// * `circle_radius` - Radius of the circle
///
/// # Returns
///
/// `true` if the point is inside the circle, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_collision_point_in_circle(
    point_x: f32,
    point_y: f32,
    circle_x: f32,
    circle_y: f32,
    circle_radius: f32,
) -> bool {
    let dx = point_x - circle_x;
    let dy = point_y - circle_y;
    (dx * dx + dy * dy) <= (circle_radius * circle_radius)
}

// ============================================================================
// Simple Overlap Tests (no contact info needed)
// ============================================================================

/// Fast check if two AABBs overlap (no contact info).
///
/// This is faster than `goud_collision_aabb_aabb` when you only need
/// a boolean result and don't need penetration depth or collision normal.
///
/// # Arguments
///
/// * `min_a_x/y` - Minimum corner (top-left) of the first AABB
/// * `max_a_x/y` - Maximum corner (bottom-right) of the first AABB
/// * `min_b_x/y` - Minimum corner of the second AABB
/// * `max_b_x/y` - Maximum corner of the second AABB
///
/// # Returns
///
/// `true` if the AABBs overlap, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_collision_aabb_overlap(
    min_a_x: f32,
    min_a_y: f32,
    max_a_x: f32,
    max_a_y: f32,
    min_b_x: f32,
    min_b_y: f32,
    max_b_x: f32,
    max_b_y: f32,
) -> bool {
    max_a_x >= min_b_x && min_a_x <= max_b_x && max_a_y >= min_b_y && min_a_y <= max_b_y
}

/// Fast check if two circles overlap (no contact info).
///
/// # Arguments
///
/// * `x1/y1` - Center of the first circle
/// * `r1` - Radius of the first circle
/// * `x2/y2` - Center of the second circle
/// * `r2` - Radius of the second circle
///
/// # Returns
///
/// `true` if the circles overlap, `false` otherwise.
#[no_mangle]
pub extern "C" fn goud_collision_circle_overlap(
    x1: f32,
    y1: f32,
    r1: f32,
    x2: f32,
    y2: f32,
    r2: f32,
) -> bool {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let combined_radius = r1 + r2;
    (dx * dx + dy * dy) <= (combined_radius * combined_radius)
}

// ============================================================================
// Distance Functions
// ============================================================================

/// Returns the distance between two points.
///
/// # Arguments
///
/// * `x1/y1` - First point
/// * `x2/y2` - Second point
///
/// # Returns
///
/// The Euclidean distance between the points.
#[no_mangle]
pub extern "C" fn goud_collision_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

/// Returns the squared distance between two points.
///
/// This is faster than `goud_collision_distance` when you only need
/// to compare distances (no square root needed).
///
/// # Arguments
///
/// * `x1/y1` - First point
/// * `x2/y2` - Second point
///
/// # Returns
///
/// The squared Euclidean distance between the points.
#[no_mangle]
pub extern "C" fn goud_collision_distance_squared(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    dx * dx + dy * dy
}
