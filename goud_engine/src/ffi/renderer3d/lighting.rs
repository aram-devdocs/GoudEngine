//! FFI functions for dynamic lighting in the 3D renderer.

use super::primitives::{GOUD_INVALID_LIGHT, GOUD_LIGHT_TYPE_DIRECTIONAL, GOUD_LIGHT_TYPE_SPOT};
use super::state::{ensure_renderer3d_state, with_renderer};
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::graphics::renderer3d::{Light, LightType};
use cgmath::Vector3;

// ============================================================================
// FFI: Lighting
// ============================================================================

/// Adds a light to the scene.
///
/// # Arguments
/// * `light_type` - 0=Point, 1=Directional, 2=Spot
/// * `pos_x/y/z` - Position
/// * `dir_x/y/z` - Direction
/// * `r/g/b` - Color (0-1)
/// * `intensity` - Light intensity
/// * `range` - Light range
/// * `spot_angle` - Spot light cone angle in degrees
#[no_mangle]
pub extern "C" fn goud_renderer3d_add_light(
    context_id: GoudContextId,
    light_type: i32,
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    dir_x: f32,
    dir_y: f32,
    dir_z: f32,
    r: f32,
    g: f32,
    b: f32,
    intensity: f32,
    range: f32,
    spot_angle: f32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_LIGHT;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return GOUD_INVALID_LIGHT;
    }

    let lt = map_light_type(light_type);

    with_renderer(context_id, |renderer| {
        renderer.add_light(Light {
            light_type: lt,
            position: Vector3::new(pos_x, pos_y, pos_z),
            direction: Vector3::new(dir_x, dir_y, dir_z),
            color: Vector3::new(r, g, b),
            intensity,
            range,
            spot_angle,
            enabled: true,
        })
    })
    .unwrap_or(GOUD_INVALID_LIGHT)
}

/// Updates a light's properties.
#[no_mangle]
pub extern "C" fn goud_renderer3d_update_light(
    context_id: GoudContextId,
    light_id: u32,
    light_type: i32,
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    dir_x: f32,
    dir_y: f32,
    dir_z: f32,
    r: f32,
    g: f32,
    b: f32,
    intensity: f32,
    range: f32,
    spot_angle: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    let lt = map_light_type(light_type);

    with_renderer(context_id, |renderer| {
        renderer.update_light(
            light_id,
            Light {
                light_type: lt,
                position: Vector3::new(pos_x, pos_y, pos_z),
                direction: Vector3::new(dir_x, dir_y, dir_z),
                color: Vector3::new(r, g, b),
                intensity,
                range,
                spot_angle,
                enabled: true,
            },
        )
    })
    .unwrap_or(false)
}

/// Removes a light from the scene.
#[no_mangle]
pub extern "C" fn goud_renderer3d_remove_light(context_id: GoudContextId, light_id: u32) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    with_renderer(context_id, |renderer| renderer.remove_light(light_id)).unwrap_or(false)
}

// ============================================================================
// Helpers
// ============================================================================

fn map_light_type(light_type: i32) -> LightType {
    match light_type {
        GOUD_LIGHT_TYPE_DIRECTIONAL => LightType::Directional,
        GOUD_LIGHT_TYPE_SPOT => LightType::Spot,
        _ => LightType::Point,
    }
}
