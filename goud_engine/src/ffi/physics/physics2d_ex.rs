//! Additive 2D physics FFI exports.
//!
//! Adds extended collider and raycast APIs without breaking existing signatures.

use crate::core::error::{set_last_error, GoudError};
use crate::core::providers::types::{BodyHandle, ColliderDesc};
use crate::ffi::context::GoudContextId;

use super::physics2d_common::{with_provider, with_provider_mut, INVALID_HANDLE};
use super::physics2d_state::{body_matches_layer_mask, register_collider};

pub(super) const DEFAULT_COLLISION_LAYER: u32 = 1;
pub(super) const DEFAULT_COLLISION_MASK: u32 = u32::MAX;

const RAYCAST_SKIP_EPSILON: f32 = 0.0001;
const MAX_FILTERED_RAYCAST_STEPS: usize = 128;

pub(super) fn add_collider_with_filter(
    ctx: GoudContextId,
    body_handle: u64,
    shape_type: u32,
    width: f32,
    height: f32,
    radius: f32,
    friction: f32,
    restitution: f32,
    is_sensor: bool,
    layer: u32,
    mask: u32,
) -> i64 {
    with_provider_mut(ctx, |p| {
        let desc = ColliderDesc {
            shape: shape_type,
            half_extents: [width, height],
            radius,
            friction,
            restitution,
            is_sensor,
            layer,
            mask,
        };

        match p.create_collider(BodyHandle(body_handle), &desc) {
            Ok(handle) => {
                register_collider(ctx, body_handle, handle.0, layer, mask, is_sensor);
                handle.0 as i64
            }
            Err(e) => {
                set_last_error(e);
                INVALID_HANDLE
            }
        }
    })
}

/// Attaches a collider with explicit sensor/layer/mask filtering data.
///
/// # Arguments
///
/// * `ctx` - Context ID
/// * `body_handle` - Handle of the body to attach to
/// * `shape_type` - 0 = circle, 1 = box, 2 = capsule
/// * `width`, `height` - Half-extents for box shapes
/// * `radius` - Radius for circle/capsule shapes
/// * `friction` - Friction coefficient
/// * `restitution` - Bounciness coefficient
/// * `is_sensor` - Whether collider is a trigger (no physical response)
/// * `layer` - Collision layer bits assigned to this collider
/// * `mask` - Collision mask bits this collider accepts
///
/// # Returns
///
/// A positive collider handle on success, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_physics_add_collider_ex(
    ctx: GoudContextId,
    body_handle: u64,
    shape_type: u32,
    width: f32,
    height: f32,
    radius: f32,
    friction: f32,
    restitution: f32,
    is_sensor: bool,
    layer: u32,
    mask: u32,
) -> i64 {
    add_collider_with_filter(
        ctx,
        body_handle,
        shape_type,
        width,
        height,
        radius,
        friction,
        restitution,
        is_sensor,
        layer,
        mask,
    )
}

/// Casts a filtered ray and returns full hit payload.
///
/// The function tests colliders in ray order and skips any hit whose body does
/// not match `layer_mask` (based on collider metadata registered via
/// `goud_physics_add_collider_ex`).
///
/// # Safety
///
/// All output pointers must be valid, non-null pointers to writable memory.
/// Ownership is not transferred.
///
/// # Returns
///
/// 1 if a hit occurred, 0 if no hit, negative on error.
#[no_mangle]
pub unsafe extern "C" fn goud_physics_raycast_ex(
    ctx: GoudContextId,
    ox: f32,
    oy: f32,
    dx: f32,
    dy: f32,
    max_dist: f32,
    layer_mask: u32,
    out_body_handle: *mut u64,
    out_collider_handle: *mut u64,
    out_hit_x: *mut f32,
    out_hit_y: *mut f32,
    out_normal_x: *mut f32,
    out_normal_y: *mut f32,
    out_distance: *mut f32,
) -> i32 {
    if out_body_handle.is_null()
        || out_collider_handle.is_null()
        || out_hit_x.is_null()
        || out_hit_y.is_null()
        || out_normal_x.is_null()
        || out_normal_y.is_null()
        || out_distance.is_null()
    {
        set_last_error(GoudError::InvalidState(
            "one or more output pointers are null".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    if layer_mask == 0 {
        return 0;
    }

    if !max_dist.is_finite() || max_dist <= 0.0 {
        set_last_error(GoudError::InvalidState(
            "max_dist must be positive and finite".to_string(),
        ));
        return GoudError::InvalidState(String::new()).error_code();
    }

    with_provider(ctx, |p| {
        let mut current_origin = [ox, oy];
        let mut accumulated_distance = 0.0f32;
        let mut remaining_distance = max_dist;

        for _ in 0..MAX_FILTERED_RAYCAST_STEPS {
            if remaining_distance <= 0.0 || !remaining_distance.is_finite() {
                return 0;
            }

            let Some(hit) = p.raycast(current_origin, [dx, dy], remaining_distance) else {
                return 0;
            };

            if body_matches_layer_mask(ctx, hit.body.0, layer_mask) {
                // SAFETY: Null checks above guarantee all output pointers are writable.
                *out_body_handle = hit.body.0;
                *out_collider_handle = hit.collider.0;
                *out_hit_x = hit.point[0];
                *out_hit_y = hit.point[1];
                *out_normal_x = hit.normal[0];
                *out_normal_y = hit.normal[1];
                *out_distance = accumulated_distance + hit.distance;
                return 1;
            }

            let advance = hit.distance.max(0.0) + RAYCAST_SKIP_EPSILON;
            if !advance.is_finite() || advance <= 0.0 || advance > remaining_distance {
                return 0;
            }

            accumulated_distance += advance;
            remaining_distance -= advance;
            current_origin[0] += dx * advance;
            current_origin[1] += dy * advance;
        }

        0
    })
}
