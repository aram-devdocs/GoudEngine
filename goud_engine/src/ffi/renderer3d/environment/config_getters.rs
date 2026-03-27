//! FFI render config getter functions.

use super::super::state::with_renderer;
use crate::ffi::context::GoudContextId;

/// Returns whether frustum culling is enabled.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_frustum_culling_enabled(context_id: GoudContextId) -> bool {
    with_renderer(context_id, |r| r.render_config().frustum_culling.enabled).unwrap_or(false)
}

/// Returns the skinning mode (0 = CPU, 1 = GPU), or -1 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_skinning_mode(context_id: GoudContextId) -> i32 {
    with_renderer(context_id, |r| match r.render_config().skinning.mode {
        crate::libs::graphics::renderer3d::config::SkinningMode::Cpu => 0,
        crate::libs::graphics::renderer3d::config::SkinningMode::Gpu => 1,
    })
    .unwrap_or(-1)
}

/// Returns whether material sorting is enabled.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_material_sorting_enabled(context_id: GoudContextId) -> bool {
    with_renderer(context_id, |r| {
        r.render_config().batching.material_sorting_enabled
    })
    .unwrap_or(false)
}

/// Returns whether static batching is enabled.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_static_batching_enabled(context_id: GoudContextId) -> bool {
    with_renderer(context_id, |r| {
        r.render_config().batching.static_batching_enabled
    })
    .unwrap_or(false)
}

/// Returns whether instanced rendering is enabled.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_instancing_enabled(context_id: GoudContextId) -> bool {
    with_renderer(context_id, |r| {
        r.render_config().batching.instancing_enabled
    })
    .unwrap_or(false)
}

/// Returns whether animation LOD is enabled.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_animation_lod_enabled(context_id: GoudContextId) -> bool {
    with_renderer(context_id, |r| {
        r.render_config().skinning.animation_lod_enabled
    })
    .unwrap_or(false)
}

/// Returns whether shared animation evaluation is enabled.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_shared_animation_eval(context_id: GoudContextId) -> bool {
    with_renderer(context_id, |r| {
        r.render_config().skinning.shared_animation_eval
    })
    .unwrap_or(false)
}

/// Returns the current animation LOD half-rate distance.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_animation_lod_distance(context_id: GoudContextId) -> f32 {
    if context_id == crate::ffi::context::GOUD_INVALID_CONTEXT_ID {
        return -1.0;
    }

    with_renderer(context_id, |renderer| {
        renderer.render_config().skinning.animation_lod_distance
    })
    .unwrap_or(-1.0)
}

/// Returns the current animation LOD freeze distance.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_animation_lod_skip_distance(
    context_id: GoudContextId,
) -> f32 {
    if context_id == crate::ffi::context::GOUD_INVALID_CONTEXT_ID {
        return -1.0;
    }

    with_renderer(context_id, |renderer| {
        renderer
            .render_config()
            .skinning
            .animation_lod_skip_distance
    })
    .unwrap_or(-1.0)
}

/// Returns the baked animation sample rate, or -1.0 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_baked_animation_sample_rate(
    context_id: GoudContextId,
) -> f32 {
    with_renderer(context_id, |r| {
        r.render_config().skinning.baked_animation_sample_rate
    })
    .unwrap_or(-1.0)
}

/// Returns the minimum instance count for batching, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_min_instances_for_batching(context_id: GoudContextId) -> i32 {
    with_renderer(context_id, |r| {
        r.render_config().batching.min_instances_for_batching as i32
    })
    .unwrap_or(-1)
}

/// Returns the current grid transparency alpha.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_grid_alpha(context_id: GoudContextId) -> f32 {
    with_renderer(context_id, |renderer| renderer.grid_alpha()).unwrap_or(-1.0)
}

/// Returns the frustum culling FOV in degrees, or -1.0 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_frustum_culling_fov(context_id: GoudContextId) -> f32 {
    with_renderer(context_id, |r| {
        r.render_config().frustum_culling.fov_degrees
    })
    .unwrap_or(-1.0)
}

/// Returns the frustum culling near plane distance, or -1.0 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_frustum_culling_near_plane(context_id: GoudContextId) -> f32 {
    with_renderer(context_id, |r| r.render_config().frustum_culling.near_plane).unwrap_or(-1.0)
}

/// Returns the frustum culling far plane distance, or -1.0 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_frustum_culling_far_plane(context_id: GoudContextId) -> f32 {
    with_renderer(context_id, |r| r.render_config().frustum_culling.far_plane).unwrap_or(-1.0)
}

/// Returns whether shadows are enabled.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_shadows_enabled(context_id: GoudContextId) -> bool {
    with_renderer(context_id, |r| r.render_config().shadows.enabled).unwrap_or(false)
}

/// Returns the shadow map resolution, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_shadow_map_size(context_id: GoudContextId) -> i32 {
    with_renderer(context_id, |r| r.render_config().shadows.map_size as i32).unwrap_or(-1)
}

/// Returns the shadow depth bias, or -1.0 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_shadow_bias(context_id: GoudContextId) -> f32 {
    with_renderer(context_id, |r| r.render_config().shadows.bias).unwrap_or(-1.0)
}
