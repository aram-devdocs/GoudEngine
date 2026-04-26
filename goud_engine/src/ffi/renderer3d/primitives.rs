//! FFI functions for primitive creation and object manipulation.

use super::state::{ensure_renderer3d_state, with_renderer};
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::graphics::renderer3d::{PrimitiveCreateInfo, PrimitiveType};

// ============================================================================
// Constants
// ============================================================================

/// Invalid object handle constant.
pub const GOUD_INVALID_OBJECT: u32 = u32::MAX;

/// Invalid light handle constant.
pub const GOUD_INVALID_LIGHT: u32 = u32::MAX;

/// Point light type
pub const GOUD_LIGHT_TYPE_POINT: i32 = 0;
/// Directional light type
pub const GOUD_LIGHT_TYPE_DIRECTIONAL: i32 = 1;
/// Spot light type
pub const GOUD_LIGHT_TYPE_SPOT: i32 = 2;

/// Cube primitive type
pub const GOUD_PRIMITIVE_CUBE: i32 = 0;
/// Sphere primitive type
pub const GOUD_PRIMITIVE_SPHERE: i32 = 1;
/// Plane primitive type
pub const GOUD_PRIMITIVE_PLANE: i32 = 2;
/// Cylinder primitive type
pub const GOUD_PRIMITIVE_CYLINDER: i32 = 3;

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
        set_last_error(crate::core::error::GoudError::InvalidContext);
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
        set_last_error(crate::core::error::GoudError::InvalidContext);
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

/// Creates an instance of a source plane primitive.
///
/// Mirrors `goud_renderer3d_instantiate_model` for primitives. Every instance
/// of the same source plane renders through one instanced draw call, so a
/// terrain of identical-geometry tiles collapses to one batch per source plane
/// (issue #679).
///
/// The returned id behaves like an object id for transform updates
/// (`set_object_position`, `set_object_rotation`, `set_object_scale`,
/// `destroy_object`). Per-instance materials are not supported -- the source
/// plane's material/texture is captured when the first instance is created.
/// Use one source plane per material to draw multiple materials.
///
/// # Arguments
/// * `context_id` - The windowed context
/// * `source_plane_id` - Object id returned by `goud_renderer3d_create_plane`
///
/// # Returns
/// Plane-instance handle on success, GOUD_INVALID_OBJECT on failure.
#[no_mangle]
pub extern "C" fn goud_renderer3d_instantiate_plane(
    context_id: GoudContextId,
    source_plane_id: u32,
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
        renderer
            .instantiate_plane(source_plane_id)
            .unwrap_or(GOUD_INVALID_OBJECT)
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
        set_last_error(crate::core::error::GoudError::InvalidContext);
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
        set_last_error(crate::core::error::GoudError::InvalidContext);
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
        set_last_error(GoudError::InvalidContext);
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
        set_last_error(GoudError::InvalidContext);
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
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_object_scale(object_id, x, y, z)
    })
    .unwrap_or(false)
}

/// Marks a 3D object as static or dynamic.
///
/// Static objects are batched into a single VBO for reduced draw calls.
/// Once marked static, the object's transform should not change.
///
/// # Returns
/// `true` on success, `false` if context or object is invalid.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_object_static(
    context_id: GoudContextId,
    object_id: u32,
    is_static: bool,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_object_static(object_id, is_static)
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
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| renderer.remove_object(object_id)).unwrap_or(false)
}
