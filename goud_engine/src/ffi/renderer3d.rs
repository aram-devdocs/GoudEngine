//! 3D Renderer FFI Module
//!
//! Provides C-compatible functions for 3D rendering operations.
//! Wraps the Renderer3D from libs/graphics/renderer3d.rs
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // Create a primitive
//! uint cubeId = game.CreateCube(textureId, 1.0f, 1.0f, 1.0f);
//! game.SetObjectPosition(cubeId, 0, 1, 0);
//!
//! // Add a light
//! uint lightId = game.AddLight(LightType.Point, x, y, z, ...);
//!
//! // In game loop
//! game.Render3D();
//! ```

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::libs::graphics::renderer3d::{
    FogConfig, GridConfig, Light, LightType, PrimitiveCreateInfo, PrimitiveType, Renderer3D,
    SkyboxConfig,
};
use cgmath::{Vector3, Vector4};
use std::collections::HashMap;

// ============================================================================
// Constants
// ============================================================================

/// Invalid object handle constant.
pub const GOUD_INVALID_OBJECT: u32 = u32::MAX;

/// Invalid light handle constant.
pub const GOUD_INVALID_LIGHT: u32 = u32::MAX;

/// Light types
pub const GOUD_LIGHT_TYPE_POINT: i32 = 0;
/// Directional light type
pub const GOUD_LIGHT_TYPE_DIRECTIONAL: i32 = 1;
/// Spot light type
pub const GOUD_LIGHT_TYPE_SPOT: i32 = 2;

/// Primitive types
pub const GOUD_PRIMITIVE_CUBE: i32 = 0;
/// Sphere primitive type
pub const GOUD_PRIMITIVE_SPHERE: i32 = 1;
/// Plane primitive type
pub const GOUD_PRIMITIVE_PLANE: i32 = 2;
/// Cylinder primitive type
pub const GOUD_PRIMITIVE_CYLINDER: i32 = 3;

// ============================================================================
// State Management
// ============================================================================

thread_local! {
    static RENDERER3D_STATE: std::cell::RefCell<HashMap<(u32, u32), Renderer3D>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Ensures 3D renderer state is initialized for a context
fn ensure_renderer3d_state(context_id: GoudContextId) -> Result<(), GoudError> {
    let context_key = (context_id.index(), context_id.generation());

    let already_initialized =
        RENDERER3D_STATE.with(|cell| cell.borrow().contains_key(&context_key));

    if already_initialized {
        return Ok(());
    }

    // Get window dimensions
    let (width, height) = with_window_state(context_id, |ws| ws.get_framebuffer_size())
        .ok_or(GoudError::InvalidContext)?;

    // Create renderer
    let renderer = Renderer3D::new(width as u32, height as u32)
        .map_err(|e| GoudError::InitializationFailed(e))?;

    RENDERER3D_STATE.with(|cell| {
        cell.borrow_mut().insert(context_key, renderer);
    });

    Ok(())
}

fn with_renderer<F, R>(context_id: GoudContextId, f: F) -> Option<R>
where
    F: FnOnce(&mut Renderer3D) -> R,
{
    let context_key = (context_id.index(), context_id.generation());
    RENDERER3D_STATE.with(|cell| {
        let mut states = cell.borrow_mut();
        states.get_mut(&context_key).map(f)
    })
}

// ============================================================================
// FFI: Primitive Creation
// ============================================================================

/// Creates a 3D cube object.
///
/// # Arguments
/// * `context_id` - The windowed context
/// * `texture_id` - Texture ID (0 for no texture)
/// * `width`, `height`, `depth` - Cube dimensions
///
/// # Returns
/// Object ID on success, GOUD_INVALID_OBJECT on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_create_cube(
    context_id: GoudContextId,
    texture_id: u32,
    width: f32,
    height: f32,
    depth: f32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_OBJECT;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return GOUD_INVALID_OBJECT;
    }

    with_renderer(context_id, |renderer| {
        renderer.create_primitive(PrimitiveCreateInfo {
            primitive_type: PrimitiveType::Cube,
            width,
            height,
            depth,
            segments: 1,
            texture_id,
        })
    })
    .unwrap_or(GOUD_INVALID_OBJECT)
}

/// Creates a 3D plane object.
#[no_mangle]
pub extern "C" fn goud_renderer3d_create_plane(
    context_id: GoudContextId,
    texture_id: u32,
    width: f32,
    depth: f32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_OBJECT;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return GOUD_INVALID_OBJECT;
    }

    with_renderer(context_id, |renderer| {
        renderer.create_primitive(PrimitiveCreateInfo {
            primitive_type: PrimitiveType::Plane,
            width,
            height: 0.0,
            depth,
            segments: 1,
            texture_id,
        })
    })
    .unwrap_or(GOUD_INVALID_OBJECT)
}

/// Creates a 3D sphere object.
#[no_mangle]
pub extern "C" fn goud_renderer3d_create_sphere(
    context_id: GoudContextId,
    texture_id: u32,
    diameter: f32,
    segments: u32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_OBJECT;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return GOUD_INVALID_OBJECT;
    }

    with_renderer(context_id, |renderer| {
        renderer.create_primitive(PrimitiveCreateInfo {
            primitive_type: PrimitiveType::Sphere,
            width: diameter,
            height: diameter,
            depth: diameter,
            segments,
            texture_id,
        })
    })
    .unwrap_or(GOUD_INVALID_OBJECT)
}

/// Creates a 3D cylinder object.
#[no_mangle]
pub extern "C" fn goud_renderer3d_create_cylinder(
    context_id: GoudContextId,
    texture_id: u32,
    radius: f32,
    height: f32,
    segments: u32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_OBJECT;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return GOUD_INVALID_OBJECT;
    }

    with_renderer(context_id, |renderer| {
        renderer.create_primitive(PrimitiveCreateInfo {
            primitive_type: PrimitiveType::Cylinder,
            width: radius * 2.0,
            height,
            depth: radius * 2.0,
            segments,
            texture_id,
        })
    })
    .unwrap_or(GOUD_INVALID_OBJECT)
}

// ============================================================================
// FFI: Object Manipulation
// ============================================================================

/// Sets the position of a 3D object.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_object_position(
    context_id: GoudContextId,
    object_id: u32,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_object_position(object_id, x, y, z)
    })
    .unwrap_or(false)
}

/// Sets the rotation of a 3D object (in degrees).
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_object_rotation(
    context_id: GoudContextId,
    object_id: u32,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_object_rotation(object_id, x, y, z)
    })
    .unwrap_or(false)
}

/// Sets the scale of a 3D object.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_object_scale(
    context_id: GoudContextId,
    object_id: u32,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_object_scale(object_id, x, y, z)
    })
    .unwrap_or(false)
}

/// Destroys a 3D object.
#[no_mangle]
pub extern "C" fn goud_renderer3d_destroy_object(
    context_id: GoudContextId,
    object_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    with_renderer(context_id, |renderer| renderer.remove_object(object_id)).unwrap_or(false)
}

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

    let lt = match light_type {
        GOUD_LIGHT_TYPE_DIRECTIONAL => LightType::Directional,
        GOUD_LIGHT_TYPE_SPOT => LightType::Spot,
        _ => LightType::Point,
    };

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

    let lt = match light_type {
        GOUD_LIGHT_TYPE_DIRECTIONAL => LightType::Directional,
        GOUD_LIGHT_TYPE_SPOT => LightType::Spot,
        _ => LightType::Point,
    };

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
// FFI: Camera
// ============================================================================

/// Sets the 3D camera position.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_camera_position(
    context_id: GoudContextId,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    if let Err(_) = ensure_renderer3d_state(context_id) {
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_camera_position(x, y, z);
        true
    })
    .unwrap_or(false)
}

/// Sets the 3D camera rotation (pitch, yaw, roll in degrees).
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_camera_rotation(
    context_id: GoudContextId,
    pitch: f32,
    yaw: f32,
    roll: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    if let Err(_) = ensure_renderer3d_state(context_id) {
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_camera_rotation(pitch, yaw, roll);
        true
    })
    .unwrap_or(false)
}

// ============================================================================
// FFI: Grid and Skybox
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
        return false;
    }

    if let Err(_) = ensure_renderer3d_state(context_id) {
        return false;
    }

    with_renderer(context_id, |renderer| {
        let mut config = GridConfig::default();
        config.enabled = enabled;
        config.size = size;
        config.divisions = divisions;
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
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_grid_enabled(enabled);
        true
    })
    .unwrap_or(false)
}

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
        return false;
    }

    if let Err(_) = ensure_renderer3d_state(context_id) {
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
        return false;
    }

    if let Err(_) = ensure_renderer3d_state(context_id) {
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

    // Render without texture manager for now (we'll add texture support later)
    with_renderer(context_id, |renderer| {
        renderer.render(None);
        true
    })
    .unwrap_or(false)
}

// Keep the old function name for backward compatibility
/// Renders all 3D objects (alias for goud_renderer3d_render).
#[no_mangle]
pub extern "C" fn goud_renderer3d_render_all(context_id: GoudContextId) -> bool {
    goud_renderer3d_render(context_id)
}
