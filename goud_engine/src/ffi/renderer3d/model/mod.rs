//! FFI functions for 3D model loading and manipulation.

pub mod batch;
#[allow(unused_imports)]
pub use batch::*;

use super::state::{ensure_renderer3d_state, with_renderer};
use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;

/// Invalid model handle constant.
pub const GOUD_INVALID_MODEL: u32 = u32::MAX;

/// Loads a 3D model file from disk and returns a model handle.
///
/// Supported formats: glTF (.gltf, .glb), OBJ (.obj), FBX (.fbx).
///
/// # Arguments
/// * `context_id` - The windowed context
/// * `path_ptr` - Path to the model file (null-terminated C string)
///
/// # Returns
/// Model handle on success, `GOUD_INVALID_MODEL` on error.
///
/// # Safety
/// `path_ptr` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer3d_load_model(
    context_id: GoudContextId,
    path_ptr: *const c_char,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_MODEL;
    }

    if path_ptr.is_null() {
        set_last_error(GoudError::InvalidHandle);
        return GOUD_INVALID_MODEL;
    }

    if let Err(e) = ensure_renderer3d_state(context_id) {
        set_last_error(e);
        return GOUD_INVALID_MODEL;
    }

    // SAFETY: caller guarantees path_ptr is a valid null-terminated C string.
    let path_str = match CStr::from_ptr(path_ptr).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Invalid UTF-8 in model path".to_string(),
            ));
            return GOUD_INVALID_MODEL;
        }
    };

    // Read file bytes from disk.
    let bytes = match std::fs::read(path_str) {
        Ok(b) => b,
        Err(e) => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Failed to read model file '{}': {}",
                path_str, e
            )));
            return GOUD_INVALID_MODEL;
        }
    };

    // Determine extension.
    let ext = match Path::new(path_str).extension().and_then(|e| e.to_str()) {
        Some(e) => e.to_ascii_lowercase(),
        None => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Model file has no extension: '{}'",
                path_str
            )));
            return GOUD_INVALID_MODEL;
        }
    };

    // Parse via the model provider registry.
    #[cfg(feature = "native")]
    let (model_data, embedded_assets) = {
        use crate::assets::loaders::mesh::default_registry;
        use crate::assets::{AssetPath, LoadContext};

        let registry = default_registry();
        let asset_path = AssetPath::from_string(path_str.to_string());
        let mut load_ctx = LoadContext::new(asset_path);

        match registry.load(&ext, &bytes, &mut load_ctx) {
            Ok(data) => {
                // Capture embedded image data before load_ctx drops.
                let embedded: Vec<(String, Vec<u8>)> = load_ctx
                    .embedded_assets()
                    .iter()
                    .map(|a| (a.path.clone(), a.bytes.clone()))
                    .collect();
                (data, embedded)
            }
            Err(e) => {
                set_last_error(GoudError::ResourceLoadFailed(format!(
                    "Failed to parse model '{}': {}",
                    path_str, e
                )));
                return GOUD_INVALID_MODEL;
            }
        }
    };

    #[cfg(not(feature = "native"))]
    {
        set_last_error(GoudError::ResourceLoadFailed(
            "Model loading requires the 'native' feature".to_string(),
        ));
        return GOUD_INVALID_MODEL;
    }

    // Load textures for each sub-mesh material.
    #[cfg(feature = "native")]
    {
        let model_dir = Path::new(path_str)
            .parent()
            .unwrap_or_else(|| Path::new("."));

        // Collect texture paths to load before borrowing the renderer mutably.
        let texture_loads: Vec<(usize, String, &str)> = model_data
            .mesh
            .sub_meshes
            .iter()
            .enumerate()
            .flat_map(|(i, sm)| {
                let mut paths = Vec::new();
                if let Some(ref mat) = sm.material {
                    if let Some(ref p) = mat.base_color_texture_path {
                        let resolved = model_dir.join(p).to_string_lossy().to_string();
                        paths.push((i, resolved, "albedo"));
                    }
                    if let Some(ref p) = mat.normal_texture_path {
                        let resolved = model_dir.join(p).to_string_lossy().to_string();
                        paths.push((i, resolved, "normal"));
                    }
                    if let Some(ref p) = mat.metallic_roughness_texture_path {
                        let resolved = model_dir.join(p).to_string_lossy().to_string();
                        paths.push((i, resolved, "metallic_roughness"));
                    }
                }
                paths
            })
            .collect();

        // Load the model geometry first.
        let model_id = with_renderer(context_id, |renderer| {
            renderer.load_model(model_data, path_str)
        })
        .unwrap_or(GOUD_INVALID_MODEL);

        if model_id == 0 || model_id == GOUD_INVALID_MODEL {
            return GOUD_INVALID_MODEL;
        }

        // Load textures and bind them to the model's materials.
        // Try embedded image data first (for GLB files), then fall back to filesystem.
        for (mesh_index, tex_path, tex_type) in &texture_loads {
            let tex_id = if let Some((_, img_bytes)) =
                embedded_assets.iter().find(|(p, _)| tex_path.ends_with(p))
            {
                load_texture_from_bytes(context_id, img_bytes, tex_path)
            } else {
                load_texture_from_path(context_id, tex_path)
            };
            if tex_id != 0 {
                with_renderer(context_id, |renderer| match *tex_type {
                    "albedo" => {
                        renderer.set_model_material_albedo_map(model_id, *mesh_index, tex_id);
                        renderer.set_model_texture(model_id, *mesh_index as i32, tex_id);
                    }
                    "normal" => {
                        renderer.set_model_material_normal_map(model_id, *mesh_index, tex_id);
                    }
                    "metallic_roughness" => {
                        renderer.set_model_material_metallic_roughness_map(
                            model_id,
                            *mesh_index,
                            tex_id,
                        );
                    }
                    _ => {}
                });
            }
        }

        model_id
    }
}

/// Internal helper: load a texture from a file path and return its texture ID (u32).
///
/// Returns `0` if loading fails.
fn load_texture_from_path(context_id: GoudContextId, path: &str) -> u32 {
    let img = match image::open(path) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            log::warn!("Failed to load model texture '{}': {}", path, e);
            return 0;
        }
    };

    let width = img.width();
    let height = img.height();
    let data = img.into_raw();

    crate::ffi::window::with_window_state(context_id, |state| {
        use crate::libs::graphics::backend::types::{TextureFilter, TextureFormat, TextureWrap};
        use crate::libs::graphics::backend::TextureOps;

        match state.backend_mut().create_texture(
            width,
            height,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
            &data,
        ) {
            Ok(handle) => handle.index(),
            Err(e) => {
                log::warn!("Failed to create GPU texture for '{}': {}", path, e);
                0
            }
        }
    })
    .unwrap_or(0)
}

/// Internal helper: load a texture from in-memory image bytes (PNG/JPG).
///
/// Used for embedded GLB textures. Returns `0` if decoding fails.
fn load_texture_from_bytes(context_id: GoudContextId, img_bytes: &[u8], label: &str) -> u32 {
    let img = match image::load_from_memory(img_bytes) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            log::warn!("Failed to decode embedded texture '{}': {}", label, e);
            return 0;
        }
    };

    let width = img.width();
    let height = img.height();
    let data = img.into_raw();

    crate::ffi::window::with_window_state(context_id, |state| {
        use crate::libs::graphics::backend::types::{TextureFilter, TextureFormat, TextureWrap};
        use crate::libs::graphics::backend::TextureOps;

        match state.backend_mut().create_texture(
            width,
            height,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
            &data,
        ) {
            Ok(handle) => handle.index(),
            Err(e) => {
                log::warn!("Failed to create GPU texture for '{}': {}", label, e);
                0
            }
        }
    })
    .unwrap_or(0)
}

/// Destroys a 3D model and all its owned GPU resources.
#[no_mangle]
pub extern "C" fn goud_renderer3d_destroy_model(context_id: GoudContextId, model_id: u32) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| renderer.destroy_model(model_id)).unwrap_or(false)
}

/// Creates an independent instance of a source model.
///
/// Returns the instance handle, or `GOUD_INVALID_MODEL` on failure.
#[no_mangle]
pub extern "C" fn goud_renderer3d_instantiate_model(
    context_id: GoudContextId,
    source_model_id: u32,
) -> u32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_MODEL;
    }

    with_renderer(context_id, |renderer| {
        renderer
            .instantiate_model(source_model_id)
            .unwrap_or(GOUD_INVALID_MODEL)
    })
    .unwrap_or(GOUD_INVALID_MODEL)
}

/// Mark a model or instance as static for batching optimizations.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_model_static(
    context_id: GoudContextId,
    model_id: u32,
    is_static: bool,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_model_static(model_id, is_static)
    })
    .unwrap_or(false)
}

/// Overrides the material on a specific sub-mesh of a model or instance.
///
/// Pass `mesh_index = -1` to apply to all sub-meshes.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_model_material(
    context_id: GoudContextId,
    model_id: u32,
    mesh_index: i32,
    material_id: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_model_material(model_id, mesh_index, material_id)
    })
    .unwrap_or(false)
}

/// Returns the number of sub-meshes in a model or instance.
///
/// Returns `-1` if the model does not exist.
#[no_mangle]
pub extern "C" fn goud_renderer3d_get_model_mesh_count(
    context_id: GoudContextId,
    model_id: u32,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    with_renderer(context_id, |renderer| {
        renderer
            .get_model_mesh_count(model_id)
            .map(|c| c as i32)
            .unwrap_or(-1)
    })
    .unwrap_or(-1)
}

/// Returns the AABB bounding box of a model.
///
/// Writes min/max coordinates to the provided output pointers.
/// Returns `false` if the model does not exist or any pointer is null.
///
/// # Safety
/// All output pointers must be valid, aligned, and non-null.
#[no_mangle]
pub unsafe extern "C" fn goud_renderer3d_get_model_bounding_box(
    context_id: GoudContextId,
    model_id: u32,
    out_min_x: *mut f32,
    out_min_y: *mut f32,
    out_min_z: *mut f32,
    out_max_x: *mut f32,
    out_max_y: *mut f32,
    out_max_z: *mut f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    if out_min_x.is_null()
        || out_min_y.is_null()
        || out_min_z.is_null()
        || out_max_x.is_null()
        || out_max_y.is_null()
        || out_max_z.is_null()
    {
        set_last_error(GoudError::InvalidHandle);
        return false;
    }

    let bounds = with_renderer(context_id, |renderer| {
        renderer.get_model_bounding_box(model_id)
    })
    .flatten();

    match bounds {
        Some(b) => {
            // SAFETY: caller guarantees all output pointers are valid and aligned.
            *out_min_x = b.min[0];
            *out_min_y = b.min[1];
            *out_min_z = b.min[2];
            *out_max_x = b.max[0];
            *out_max_y = b.max[1];
            *out_max_z = b.max[2];
            true
        }
        None => false,
    }
}

/// Sets the position of a 3D model or instance.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_model_position(
    context_id: GoudContextId,
    model_id: u32,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_model_position(model_id, x, y, z)
    })
    .unwrap_or(false)
}

/// Sets the rotation of a 3D model or instance (in degrees).
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_model_rotation(
    context_id: GoudContextId,
    model_id: u32,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_model_rotation(model_id, x, y, z)
    })
    .unwrap_or(false)
}

/// Sets the scale of a 3D model or instance.
#[no_mangle]
pub extern "C" fn goud_renderer3d_set_model_scale(
    context_id: GoudContextId,
    model_id: u32,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    with_renderer(context_id, |renderer| {
        renderer.set_model_scale(model_id, x, y, z)
    })
    .unwrap_or(false)
}

