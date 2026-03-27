//! FFI functions for environment features: grid, skybox, fog, and the render call.

use super::state::{ensure_renderer3d_state, with_renderer};
use crate::core::debugger;
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::libs::graphics::renderer3d::TextureManagerTrait;
use crate::libs::graphics::renderer3d::{FogConfig, GridConfig, SkyboxConfig};
use cgmath::{Vector3, Vector4};

struct WindowTextureBridge {
    context_id: GoudContextId,
}

impl TextureManagerTrait for WindowTextureBridge {
    fn bind_texture(&self, texture_id: u32, slot: u32) {
        let _ = with_window_state(self.context_id, |state| {
            if let Err(err) = state.backend_mut().bind_texture_by_index(texture_id, slot) {
                log::warn!(
                    "ffi renderer3d failed to bind texture index {} on slot {}: {}",
                    texture_id,
                    slot,
                    err
                );
            }
        });
    }
}

fn resolved_runtime_debug_draw_shapes(
    context_id: GoudContextId,
) -> Vec<crate::core::providers::types::DebugShape3D> {
    let Some(route_id) = debugger::route_for_context(context_id) else {
        return Vec::new();
    };
    let Some(control) = debugger::control_state_for_route(&route_id) else {
        return Vec::new();
    };
    if !control.debug_draw_enabled {
        return Vec::new();
    }

    #[cfg(feature = "rapier3d")]
    let provider_shapes = crate::ffi::physics::physics3d_debug_shapes(context_id);
    #[cfg(not(feature = "rapier3d"))]
    let provider_shapes: Vec<crate::core::providers::types::DebugShape3D> = Vec::new();
    let _ = debugger::replace_provider_debug_draw_3d_for_context(context_id, &provider_shapes);

    let Some(payload) = debugger::debug_draw_payload_for_route(&route_id) else {
        return Vec::new();
    };

    payload
        .provider_3d
        .into_iter()
        .chain(payload.transient_3d)
        .map(|entry| entry.shape)
        .collect()
}

// ============================================================================
// FFI: Grid
// ============================================================================

/// Configures the ground grid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_configure_grid(
    context_id: GoudContextId,
    enabled: bool,
    size: f32,
    divisions: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if ensure_renderer3d_state(context_id).is_err() {
        set_last_error(GoudError::InternalError(
            "Renderer state not found".to_string(),
        ));
        return false;
    }

    with_renderer(context_id, |renderer| {
        let config = GridConfig {
            enabled,
            size,
            divisions,
            ..Default::default()
        };
        renderer.configure_grid(config);
        true
    })
    .unwrap_or(false)
}

/// Sets grid enabled state.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_grid_enabled(
    context_id: GoudContextId,
    enabled: bool,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_grid_enabled(enabled);
        true
    })
    .unwrap_or(false)
}

// ============================================================================
// FFI: Skybox
// ============================================================================

/// Configures the skybox/background color.
#[no_mangle]
pub extern "C" fn goud_renderer3d_configure_skybox(
    context_id: GoudContextId,
    enabled: bool,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if ensure_renderer3d_state(context_id).is_err() {
        set_last_error(GoudError::InternalError(
            "Renderer state not found".to_string(),
        ));
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.configure_skybox(SkyboxConfig {
            enabled,
            color: Vector4::new(r, g, b, a),
        });
        true
    })
    .unwrap_or(false)
}

// ============================================================================
// FFI: Fog
// ============================================================================

/// Configures fog settings.
#[no_mangle]
pub extern "C" fn goud_renderer3d_configure_fog(
    context_id: GoudContextId,
    enabled: bool,
    r: f32,
    g: f32,
    b: f32,
    density: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if ensure_renderer3d_state(context_id).is_err() {
        set_last_error(GoudError::InternalError(
            "Renderer state not found".to_string(),
        ));
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.configure_fog(FogConfig {
            enabled,
            color: Vector3::new(r, g, b),
            density,
        });
        true
    })
    .unwrap_or(false)
}

/// Sets fog enabled state.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_fog_enabled(
    context_id: GoudContextId,
    enabled: bool,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_fog_enabled(enabled);
        true
    })
    .unwrap_or(false)
}

// ============================================================================
// FFI: Rendering
// ============================================================================

/// Renders all 3D objects in the scene.
///
/// Call this between goud_renderer_begin and goud_renderer_end (or in game loop).
#[no_mangle]
pub extern "C" fn goud_renderer3d_render(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return false;
    }

    let route_id = debugger::route_for_context(context_id);

    let rendered = with_renderer(context_id, |renderer| {
        let texture_bridge = WindowTextureBridge { context_id };

        if let Some(route_id) = route_id.as_ref() {
            debugger::scoped_route(Some(route_id.clone()), || {
                renderer.set_debug_draw_shapes(&resolved_runtime_debug_draw_shapes(context_id));
                renderer.render(Some(&texture_bridge));
            });
        } else {
            renderer.set_debug_draw_shapes(&[]);
            renderer.render(Some(&texture_bridge));
        }
        true
    })
    .unwrap_or(false);

    if rendered {
        let _ = debugger::update_render_stats_for_context(context_id, 1, 0, 0, 1);
    }

    rendered
}

/// Renders all 3D objects (alias for goud_renderer3d_render).
#[no_mangle]
pub extern "C" fn goud_renderer3d_render_all(context_id: GoudContextId) -> bool {
    goud_renderer3d_render(context_id)
}

// ============================================================================
// FFI: Render Config
// ============================================================================

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

/// Returns the current animation LOD half-rate distance.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_animation_lod_distance(context_id: GoudContextId) -> f32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
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
    if context_id == GOUD_INVALID_CONTEXT_ID {
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

// ============================================================================
// FFI: Render Config Getters
// ============================================================================

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

// ============================================================================
// FFI: Baked Animation Sample Rate
// ============================================================================

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

// ============================================================================
// FFI: Min Instances for Batching
// ============================================================================

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

/// Returns the minimum instance count for batching, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_min_instances_for_batching(context_id: GoudContextId) -> i32 {
    with_renderer(context_id, |r| {
        r.render_config().batching.min_instances_for_batching as i32
    })
    .unwrap_or(-1)
}

// ============================================================================
// FFI: Default Material Color
// ============================================================================

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

/// Returns the current grid transparency alpha.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_grid_alpha(context_id: GoudContextId) -> f32 {
    with_renderer(context_id, |renderer| renderer.grid_alpha()).unwrap_or(-1.0)
}

// ============================================================================
// FFI: Frustum Culling Config (FOV, Near, Far)
// ============================================================================

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

/// Returns the frustum culling FOV in degrees, or -1.0 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_frustum_culling_fov(context_id: GoudContextId) -> f32 {
    with_renderer(context_id, |r| {
        r.render_config().frustum_culling.fov_degrees
    })
    .unwrap_or(-1.0)
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

/// Returns the frustum culling near plane distance, or -1.0 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_frustum_culling_near_plane(context_id: GoudContextId) -> f32 {
    with_renderer(context_id, |r| r.render_config().frustum_culling.near_plane).unwrap_or(-1.0)
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

/// Returns the frustum culling far plane distance, or -1.0 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_frustum_culling_far_plane(context_id: GoudContextId) -> f32 {
    with_renderer(context_id, |r| r.render_config().frustum_culling.far_plane).unwrap_or(-1.0)
}

// ============================================================================
// FFI: Shadow Config
// ============================================================================

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

/// Returns whether shadows are enabled.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_shadows_enabled(context_id: GoudContextId) -> bool {
    with_renderer(context_id, |r| r.render_config().shadows.enabled).unwrap_or(false)
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

/// Returns the shadow map resolution, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_shadow_map_size(context_id: GoudContextId) -> i32 {
    with_renderer(context_id, |r| r.render_config().shadows.map_size as i32).unwrap_or(-1)
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

/// Returns the shadow depth bias, or -1.0 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_shadow_bias(context_id: GoudContextId) -> f32 {
    with_renderer(context_id, |r| r.render_config().shadows.bias).unwrap_or(-1.0)
}

// ============================================================================
// FFI: Stats
// ============================================================================

/// Returns the number of draw calls issued during the last `render()` call.
///
/// Resets to zero at the start of each `render()`. Returns `-1` if the
/// context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_draw_calls(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| renderer.stats().draw_calls as i32).unwrap_or(-1)
}

/// Returns the number of objects that passed frustum culling during the last
/// `render()` call.
///
/// Resets to zero at the start of each `render()`. Returns `-1` if the
/// context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_visible_object_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        renderer.stats().visible_objects as i32
    })
    .unwrap_or(-1)
}

/// Returns the number of objects rejected by frustum culling during the last
/// `render()` call.
///
/// Resets to zero at the start of each `render()`. Returns `-1` if the
/// context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_culled_object_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        renderer.stats().culled_objects as i32
    })
    .unwrap_or(-1)
}

/// Returns the number of instanced draw calls issued during the last
/// `render()` call.
///
/// Each instanced draw call renders multiple instances of the same mesh in a
/// single GPU submission. Resets to zero at the start of each `render()`.
/// Returns `-1` if the context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_instanced_draw_calls(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    with_renderer(context_id, |renderer| {
        renderer.stats().instanced_draw_calls as i32
    })
    .unwrap_or(-1)
}

/// Returns the total number of instances submitted via instanced draw calls
/// during the last `render()` call.
///
/// This is the sum of per-draw instance counts, not the number of draw calls.
/// Resets to zero at the start of each `render()`. Returns `-1` if the
/// context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_active_instance_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    with_renderer(context_id, |renderer| {
        renderer.stats().active_instances as i32
    })
    .unwrap_or(-1)
}

/// Returns the number of animation players that were fully evaluated (bone
/// matrix computation) during the last `update_animations()` call.
///
/// Resets to zero at the start of each `update_animations()`. Returns `-1`
/// if the context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_animation_evaluation_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    with_renderer(context_id, |renderer| {
        renderer.stats().animation_evaluations as i32
    })
    .unwrap_or(-1)
}

/// Returns the number of animation evaluations that were skipped during the
/// last `update_animations()` call thanks to shared evaluation cache hits,
/// baked animation lookups, and animation LOD distance culling.
///
/// Resets to zero at the start of each `update_animations()`. Returns `-1`
/// if the context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_animation_evaluation_saved_count(
    context_id: GoudContextId,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    with_renderer(context_id, |renderer| {
        renderer.stats().animation_evaluations_saved as i32
    })
    .unwrap_or(-1)
}

/// Returns the number of bone matrix buffer uploads performed during the last
/// `render()` call (GPU skinning path only).
///
/// Each upload transfers one model's bone matrices to the GPU storage buffer.
/// Resets to zero at the start of each `render()`. Returns `-1` if the
/// context is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_bone_matrix_upload_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    with_renderer(context_id, |renderer| {
        renderer.stats().bone_matrix_uploads as i32
    })
    .unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use super::resolved_runtime_debug_draw_shapes;
    use super::*;
    use crate::core::debugger::{
        self, dispatch_request_json_for_route, register_context, DebuggerConfig, RuntimeSurfaceKind,
    };
    use crate::ffi::context::GoudContextId;
    #[cfg(feature = "rapier3d")]
    use crate::ffi::physics::{
        goud_physics3d_add_collider, goud_physics3d_add_rigid_body, goud_physics3d_create,
        goud_physics3d_destroy,
    };
    use serde_json::json;

    #[test]
    fn resolved_runtime_debug_draw_shapes_returns_empty_until_debug_draw_enabled() {
        let _guard = debugger::test_lock();
        debugger::reset_for_tests();

        let context_id = GoudContextId::new(761, 1);
        let route = register_context(
            context_id,
            RuntimeSurfaceKind::HeadlessContext,
            &DebuggerConfig {
                enabled: true,
                publish_local_attach: true,
                route_label: Some("renderer3d-debug-draw".to_string()),
            },
        );

        #[cfg(feature = "rapier3d")]
        {
            assert_eq!(goud_physics3d_create(context_id, 0.0, -9.8, 0.0), 0);
            let body = goud_physics3d_add_rigid_body(context_id, 1, 0.0, 0.0, 0.0, 1.0);
            assert!(body > 0);
            let collider = goud_physics3d_add_collider(
                context_id,
                body as u64,
                1,
                1.0,
                1.0,
                1.0,
                0.0,
                0.5,
                0.0,
            );
            assert!(collider > 0);
        }

        assert!(resolved_runtime_debug_draw_shapes(context_id).is_empty());

        let response = dispatch_request_json_for_route(
            &route,
            &json!({ "verb": "set_debug_draw_enabled", "enabled": true }).to_string(),
        )
        .expect("dispatcher should return JSON");
        assert_eq!(response["ok"], true);

        #[cfg(feature = "rapier3d")]
        assert_eq!(resolved_runtime_debug_draw_shapes(context_id).len(), 1);
        #[cfg(not(feature = "rapier3d"))]
        assert!(resolved_runtime_debug_draw_shapes(context_id).is_empty());
        #[cfg(feature = "rapier3d")]
        assert_eq!(goud_physics3d_destroy(context_id), 0);
    }

    // =========================================================================
    // Config getter / setter tests with invalid context
    // =========================================================================

    #[test]
    fn test_config_getters_invalid_context_return_error_values() {
        // All getters with GOUD_INVALID_CONTEXT_ID should return their
        // documented error sentinel (-1.0 for floats, -1 for ints, false for bools).
        assert!(
            (goud_renderer3d_get_frustum_culling_fov(GOUD_INVALID_CONTEXT_ID) - (-1.0)).abs()
                < f32::EPSILON,
            "get_frustum_culling_fov should return -1.0 for invalid context"
        );
        assert!(
            (goud_renderer3d_get_frustum_culling_near_plane(GOUD_INVALID_CONTEXT_ID) - (-1.0))
                .abs()
                < f32::EPSILON,
            "get_frustum_culling_near_plane should return -1.0 for invalid context"
        );
        assert!(
            (goud_renderer3d_get_frustum_culling_far_plane(GOUD_INVALID_CONTEXT_ID) - (-1.0)).abs()
                < f32::EPSILON,
            "get_frustum_culling_far_plane should return -1.0 for invalid context"
        );
        assert!(
            (goud_renderer3d_get_animation_lod_distance(GOUD_INVALID_CONTEXT_ID) - (-1.0)).abs()
                < f32::EPSILON,
            "get_animation_lod_distance should return -1.0 for invalid context"
        );
        assert!(
            (goud_renderer3d_get_animation_lod_skip_distance(GOUD_INVALID_CONTEXT_ID) - (-1.0))
                .abs()
                < f32::EPSILON,
            "get_animation_lod_skip_distance should return -1.0 for invalid context"
        );
        assert!(
            (goud_renderer3d_get_baked_animation_sample_rate(GOUD_INVALID_CONTEXT_ID) - (-1.0))
                .abs()
                < f32::EPSILON,
            "get_baked_animation_sample_rate should return -1.0 for invalid context"
        );
        assert!(
            (goud_renderer3d_get_shadow_bias(GOUD_INVALID_CONTEXT_ID) - (-1.0)).abs()
                < f32::EPSILON,
            "get_shadow_bias should return -1.0 for invalid context"
        );
    }

    #[test]
    fn test_config_int_getters_invalid_context_return_error_values() {
        assert_eq!(
            goud_renderer3d_get_skinning_mode(GOUD_INVALID_CONTEXT_ID),
            -1,
            "get_skinning_mode should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_get_min_instances_for_batching(GOUD_INVALID_CONTEXT_ID),
            -1,
            "get_min_instances_for_batching should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_get_shadow_map_size(GOUD_INVALID_CONTEXT_ID),
            -1,
            "get_shadow_map_size should return -1 for invalid context"
        );
    }

    #[test]
    fn test_config_bool_getters_invalid_context_return_false() {
        assert!(
            !goud_renderer3d_get_frustum_culling_enabled(GOUD_INVALID_CONTEXT_ID),
            "get_frustum_culling_enabled should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_get_material_sorting_enabled(GOUD_INVALID_CONTEXT_ID),
            "get_material_sorting_enabled should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_get_static_batching_enabled(GOUD_INVALID_CONTEXT_ID),
            "get_static_batching_enabled should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_get_instancing_enabled(GOUD_INVALID_CONTEXT_ID),
            "get_instancing_enabled should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_get_animation_lod_enabled(GOUD_INVALID_CONTEXT_ID),
            "get_animation_lod_enabled should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_get_shared_animation_eval(GOUD_INVALID_CONTEXT_ID),
            "get_shared_animation_eval should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_get_shadows_enabled(GOUD_INVALID_CONTEXT_ID),
            "get_shadows_enabled should return false for invalid context"
        );
    }

    #[test]
    fn test_config_setters_invalid_context_return_error() {
        assert_eq!(
            goud_renderer3d_set_frustum_culling_enabled(GOUD_INVALID_CONTEXT_ID, true),
            -1,
            "set_frustum_culling_enabled should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_skinning_mode(GOUD_INVALID_CONTEXT_ID, 0),
            -1,
            "set_skinning_mode should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_material_sorting_enabled(GOUD_INVALID_CONTEXT_ID, true),
            -1,
            "set_material_sorting_enabled should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_animation_lod_enabled(GOUD_INVALID_CONTEXT_ID, true),
            -1,
            "set_animation_lod_enabled should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_shared_animation_eval(GOUD_INVALID_CONTEXT_ID, true),
            -1,
            "set_shared_animation_eval should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_animation_lod_distance(GOUD_INVALID_CONTEXT_ID, 50.0),
            -1,
            "set_animation_lod_distance should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_animation_lod_skip_distance(GOUD_INVALID_CONTEXT_ID, 100.0),
            -1,
            "set_animation_lod_skip_distance should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_static_batching_enabled(GOUD_INVALID_CONTEXT_ID, true),
            -1,
            "set_static_batching_enabled should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_instancing_enabled(GOUD_INVALID_CONTEXT_ID, true),
            -1,
            "set_instancing_enabled should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_baked_animation_sample_rate(GOUD_INVALID_CONTEXT_ID, 60.0),
            -1,
            "set_baked_animation_sample_rate should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_min_instances_for_batching(GOUD_INVALID_CONTEXT_ID, 4),
            -1,
            "set_min_instances_for_batching should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_default_material_color(GOUD_INVALID_CONTEXT_ID, 1.0, 0.0, 0.0, 1.0),
            -1,
            "set_default_material_color should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_frustum_culling_fov(GOUD_INVALID_CONTEXT_ID, 90.0),
            -1,
            "set_frustum_culling_fov should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_frustum_culling_near_plane(GOUD_INVALID_CONTEXT_ID, 0.5),
            -1,
            "set_frustum_culling_near_plane should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_frustum_culling_far_plane(GOUD_INVALID_CONTEXT_ID, 500.0),
            -1,
            "set_frustum_culling_far_plane should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_shadows_enabled(GOUD_INVALID_CONTEXT_ID, true),
            -1,
            "set_shadows_enabled should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_shadow_map_size(GOUD_INVALID_CONTEXT_ID, 1024),
            -1,
            "set_shadow_map_size should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_set_shadow_bias(GOUD_INVALID_CONTEXT_ID, 0.01),
            -1,
            "set_shadow_bias should return -1 for invalid context"
        );
    }

    #[test]
    fn test_stats_getters_invalid_context_return_error() {
        assert_eq!(
            goud_renderer3d_get_draw_calls(GOUD_INVALID_CONTEXT_ID),
            -1,
            "get_draw_calls should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_get_visible_object_count(GOUD_INVALID_CONTEXT_ID),
            -1,
            "get_visible_object_count should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_get_culled_object_count(GOUD_INVALID_CONTEXT_ID),
            -1,
            "get_culled_object_count should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_get_instanced_draw_calls(GOUD_INVALID_CONTEXT_ID),
            -1,
            "get_instanced_draw_calls should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_get_active_instance_count(GOUD_INVALID_CONTEXT_ID),
            -1,
            "get_active_instance_count should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_get_animation_evaluation_count(GOUD_INVALID_CONTEXT_ID),
            -1,
            "get_animation_evaluation_count should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_get_animation_evaluation_saved_count(GOUD_INVALID_CONTEXT_ID),
            -1,
            "get_animation_evaluation_saved_count should return -1 for invalid context"
        );
        assert_eq!(
            goud_renderer3d_get_bone_matrix_upload_count(GOUD_INVALID_CONTEXT_ID),
            -1,
            "get_bone_matrix_upload_count should return -1 for invalid context"
        );
    }

    #[test]
    fn test_environment_functions_invalid_context_return_false() {
        assert!(
            !goud_renderer3d_configure_grid(GOUD_INVALID_CONTEXT_ID, true, 10.0, 10),
            "configure_grid should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_configure_skybox(GOUD_INVALID_CONTEXT_ID, true, 0.5, 0.5, 0.8, 1.0),
            "configure_skybox should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_configure_fog(GOUD_INVALID_CONTEXT_ID, true, 0.5, 0.5, 0.5, 0.1),
            "configure_fog should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_set_fog_enabled(GOUD_INVALID_CONTEXT_ID, true),
            "set_fog_enabled should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_set_grid_enabled(GOUD_INVALID_CONTEXT_ID, true),
            "set_grid_enabled should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_render(GOUD_INVALID_CONTEXT_ID),
            "render should return false for invalid context"
        );
        assert!(
            !goud_renderer3d_render_all(GOUD_INVALID_CONTEXT_ID),
            "render_all should return false for invalid context"
        );
    }
}
