//! FFI functions for the 3D material system.

use super::state::{ensure_renderer3d_state, with_renderer};
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::graphics::renderer3d::{Material3D, MaterialType, PbrProperties};
use cgmath::Vector4;

/// Invalid material handle constant.
pub const GOUD_INVALID_MATERIAL: u32 = u32::MAX;

/// Material type: Phong shading.
pub const GOUD_MATERIAL_TYPE_PHONG: i32 = 0;
/// Material type: PBR shading.
pub const GOUD_MATERIAL_TYPE_PBR: i32 = 1;
/// Material type: Unlit / flat shading.
pub const GOUD_MATERIAL_TYPE_UNLIT: i32 = 2;

fn material_type_from_i32(v: i32) -> MaterialType {
    match v {
        1 => MaterialType::Pbr,
        2 => MaterialType::Unlit,
        _ => MaterialType::Phong,
    }
}

/// Creates a 3D material.
///
/// # Arguments
/// * `context_id` - The windowed context
/// * `material_type` - 0=Phong, 1=PBR, 2=Unlit
/// * `r`, `g`, `b`, `a` - Diffuse/albedo color
/// * `shininess` - Specular shininess (Phong)
/// * `metallic`, `roughness`, `ao` - PBR properties
///
/// # Returns
/// Material ID on success, `GOUD_INVALID_MATERIAL` on error.
#[no_mangle]
pub extern "C" fn goud_renderer3d_create_material(
    context_id: GoudContextId,
    material_type: i32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    shininess: f32,
    metallic: f32,
    roughness: f32,
    ao: f32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_MATERIAL;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return GOUD_INVALID_MATERIAL;
    }

    with_renderer(context_id, |renderer| {
        renderer.create_material(Material3D {
            material_type: material_type_from_i32(material_type),
            color: Vector4::new(r, g, b, a),
            shininess,
            pbr: PbrProperties {
                metallic,
                roughness,
                ao,
                ..Default::default()
            },
        })
    })
    .unwrap_or(GOUD_INVALID_MATERIAL)
}

/// Updates an existing 3D material.
#[no_mangle]
pub extern "C" fn goud_renderer3d_update_material(
    context_id: GoudContextId,
    material_id: u32,
    material_type: i32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    shininess: f32,
    metallic: f32,
    roughness: f32,
    ao: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.update_material(
            material_id,
            Material3D {
                material_type: material_type_from_i32(material_type),
                color: Vector4::new(r, g, b, a),
                shininess,
                pbr: PbrProperties {
                    metallic,
                    roughness,
                    ao,
                    ..Default::default()
                },
            },
        )
    })
    .unwrap_or(false)
}

/// Removes a 3D material.
#[no_mangle]
pub extern "C" fn goud_renderer3d_remove_material(
    context_id: GoudContextId,
    material_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| renderer.remove_material(material_id)).unwrap_or(false)
}

/// Binds a material to an object.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_object_material(
    context_id: GoudContextId,
    object_id: u32,
    material_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_object_material(object_id, material_id)
    })
    .unwrap_or(false)
}

/// Gets the material ID bound to an object. Returns `GOUD_INVALID_MATERIAL` if none.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_object_material(
    context_id: GoudContextId,
    object_id: u32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_MATERIAL;
    }

    with_renderer(context_id, |renderer| {
        renderer
            .get_object_material(object_id)
            .unwrap_or(GOUD_INVALID_MATERIAL)
    })
    .unwrap_or(GOUD_INVALID_MATERIAL)
}
