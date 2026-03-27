//! FFI render config setter functions.

use super::super::state::with_renderer;
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

/// Enables or disables frustum culling.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_frustum_culling_enabled(
    context_id: GoudContextId,
    enabled: bool,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.frustum_culling.enabled = enabled;
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Sets the skinning mode (0 = CPU, 1 = GPU).
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_skinning_mode(context_id: GoudContextId, mode: u32) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.skinning.mode = match mode {
            0 => crate::libs::graphics::renderer3d::config::SkinningMode::Cpu,
            _ => crate::libs::graphics::renderer3d::config::SkinningMode::Gpu,
        };
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Enables or disables material sorting.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_material_sorting_enabled(
    context_id: GoudContextId,
    enabled: bool,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.batching.material_sorting_enabled = enabled;
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Enables or disables animation LOD (distance-based update throttling).
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_animation_lod_enabled(
    context_id: GoudContextId,
    enabled: bool,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.skinning.animation_lod_enabled = enabled;
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Enables or disables shared animation evaluation (cache identical poses).
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_shared_animation_eval(
    context_id: GoudContextId,
    enabled: bool,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.skinning.shared_animation_eval = enabled;
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Sets the distance at which animation updates are half-rated (every other frame).
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_animation_lod_distance(
    context_id: GoudContextId,
    distance: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.skinning.animation_lod_distance = distance;
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Sets the distance at which animation updates are frozen entirely.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_animation_lod_skip_distance(
    context_id: GoudContextId,
    distance: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.skinning.animation_lod_skip_distance = distance;
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Enables or disables static batching.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_static_batching_enabled(
    context_id: GoudContextId,
    enabled: bool,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.batching.static_batching_enabled = enabled;
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Enables or disables instanced rendering.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_instancing_enabled(
    context_id: GoudContextId,
    enabled: bool,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.batching.instancing_enabled = enabled;
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Sets the baked animation sample rate in frames per second.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_baked_animation_sample_rate(
    context_id: GoudContextId,
    rate: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.skinning.baked_animation_sample_rate = rate.max(1.0);
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Sets the minimum instance count required for instanced rendering.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_min_instances_for_batching(
    context_id: GoudContextId,
    count: u32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.batching.min_instances_for_batching = count.max(1) as usize;
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Sets the fallback material color used when a mesh has no assigned material.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_default_material_color(
    context_id: GoudContextId,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.default_material_color = [r, g, b, a];
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Returns the fallback material color as four RGBA floats packed into an array.
///
/// Writes four `f32` values to `out_r`, `out_g`, `out_b`, `out_a`.
/// Returns `0` on success, `-1` on invalid context or null pointer.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer3d_get_default_material_color(
    context_id: GoudContextId,
    out_r: *mut f32,
    out_g: *mut f32,
    out_b: *mut f32,
    out_a: *mut f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    if out_r.is_null() || out_g.is_null() || out_b.is_null() || out_a.is_null() {
        set_last_error(GoudError::InvalidHandle);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let c = renderer.render_config().default_material_color;
        // SAFETY: caller guarantees out_r/g/b/a are valid writable pointers.
        *out_r = c[0];
        *out_g = c[1];
        *out_b = c[2];
        *out_a = c[3];
        0
    })
    .unwrap_or(-1)
}

/// Sets the grid transparency alpha (0.0 = invisible, 1.0 = opaque).
///
/// This controls the alpha value used when rendering the ground grid overlay.
/// Default: `0.4`.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_grid_alpha(context_id: GoudContextId, alpha: f32) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_grid_alpha(alpha.clamp(0.0, 1.0));
        0
    })
    .unwrap_or(-1)
}

/// Sets the vertical field of view for frustum culling, in degrees.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_frustum_culling_fov(
    context_id: GoudContextId,
    fov_degrees: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.frustum_culling.fov_degrees = fov_degrees.clamp(1.0, 179.0);
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Sets the near clipping plane for frustum culling.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_frustum_culling_near_plane(
    context_id: GoudContextId,
    near: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.frustum_culling.near_plane = near.max(0.001);
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Sets the far clipping plane for frustum culling.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_frustum_culling_far_plane(
    context_id: GoudContextId,
    far: f32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.frustum_culling.far_plane = far.max(1.0);
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Enables or disables shadow mapping.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_shadows_enabled(
    context_id: GoudContextId,
    enabled: bool,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.shadows.enabled = enabled;
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Sets the shadow map resolution.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_shadow_map_size(context_id: GoudContextId, size: u32) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.shadows.map_size = size.max(1);
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}

/// Sets the shadow depth bias.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_shadow_bias(context_id: GoudContextId, bias: f32) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        let mut config = renderer.render_config().clone();
        config.shadows.bias = bias.max(0.0);
        renderer.set_render_config(config);
        0
    })
    .unwrap_or(-1)
}
