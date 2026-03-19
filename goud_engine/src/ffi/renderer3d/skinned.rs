//! FFI functions for skinned mesh creation and manipulation.

use super::state::{ensure_renderer3d_state, with_renderer};
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::libs::graphics::renderer3d::Skeleton3D;

/// Invalid skinned mesh handle constant.
pub const GOUD_INVALID_SKINNED_MESH: u32 = u32::MAX;

/// Creates a skinned mesh from raw vertex data.
///
/// # Safety
/// `vertices_ptr` must point to `vertex_count` valid `f32` values.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer3d_create_skinned_mesh(
    context_id: GoudContextId,
    vertices_ptr: *const f32,
    vertex_count: u32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_SKINNED_MESH;
    }

    if vertices_ptr.is_null() {
        set_last_error(GoudError::InvalidHandle);
        return GOUD_INVALID_SKINNED_MESH;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return GOUD_INVALID_SKINNED_MESH;
    }

    // SAFETY: Caller guarantees vertices_ptr points to vertex_count valid f32 values.
    let vertices = unsafe { std::slice::from_raw_parts(vertices_ptr, vertex_count as usize) };

    with_renderer(context_id, |renderer| {
        renderer.create_skinned_mesh(vertices.to_vec(), Skeleton3D::new())
    })
    .unwrap_or(GOUD_INVALID_SKINNED_MESH)
}

/// Removes a skinned mesh.
#[no_mangle]
pub extern "C" fn goud_renderer3d_remove_skinned_mesh(
    context_id: GoudContextId,
    mesh_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| renderer.remove_skinned_mesh(mesh_id)).unwrap_or(false)
}

/// Sets the position of a skinned mesh.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_skinned_mesh_position(
    context_id: GoudContextId,
    mesh_id: u32,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_skinned_mesh_position(mesh_id, x, y, z)
    })
    .unwrap_or(false)
}

/// Sets the rotation of a skinned mesh.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_skinned_mesh_rotation(
    context_id: GoudContextId,
    mesh_id: u32,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_skinned_mesh_rotation(mesh_id, x, y, z)
    })
    .unwrap_or(false)
}

/// Sets the scale of a skinned mesh.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_skinned_mesh_scale(
    context_id: GoudContextId,
    mesh_id: u32,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_skinned_mesh_scale(mesh_id, x, y, z)
    })
    .unwrap_or(false)
}

/// Updates bone matrices for a skinned mesh.
///
/// # Safety
/// `matrices_ptr` must point to `bone_count * 16` valid `f32` values.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer3d_set_skinned_mesh_bones(
    context_id: GoudContextId,
    mesh_id: u32,
    matrices_ptr: *const f32,
    bone_count: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if matrices_ptr.is_null() {
        set_last_error(GoudError::InvalidHandle);
        return false;
    }

    // SAFETY: Caller guarantees matrices_ptr points to bone_count*16 valid f32 values.
    let flat = unsafe { std::slice::from_raw_parts(matrices_ptr, (bone_count as usize) * 16) };
    let matrices: Vec<[f32; 16]> = flat
        .chunks_exact(16)
        .map(|chunk| {
            let mut arr = [0.0f32; 16];
            arr.copy_from_slice(chunk);
            arr
        })
        .collect();

    with_renderer(context_id, |renderer| {
        renderer.set_skinned_mesh_bone_matrices(mesh_id, matrices)
    })
    .unwrap_or(false)
}
