//! FFI functions for environment features: grid, skybox, fog, and the render call.

pub mod config_getters;
pub mod config_setters;
pub mod stats;
#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use config_getters::*;
#[allow(unused_imports)]
pub use config_setters::*;
#[allow(unused_imports)]
pub use stats::*;

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
