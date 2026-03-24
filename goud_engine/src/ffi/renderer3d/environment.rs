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
pub extern "C" fn goud_renderer3d_set_skinning_mode(
    context_id: GoudContextId,
    mode: u32,
) -> i32 {
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

// ============================================================================
// FFI: Stats
// ============================================================================

/// Returns the number of draw calls last frame, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_draw_calls(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| renderer.stats().draw_calls as i32).unwrap_or(-1)
}

/// Returns the number of visible objects last frame, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_visible_object_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| renderer.stats().visible_objects as i32).unwrap_or(-1)
}

/// Returns the number of culled objects last frame, or -1 on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_culled_object_count(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| renderer.stats().culled_objects as i32).unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use super::resolved_runtime_debug_draw_shapes;
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
}
