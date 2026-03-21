//! Atlas FFI function implementations.

use std::ffi::CStr;
use std::os::raw::c_char;

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::renderer::texture::{GoudTextureHandle, GOUD_INVALID_TEXTURE};
use crate::ffi::window::with_window_state;
use crate::rendering::texture_atlas::TextureAtlas;

use super::{context_key, GOUD_INVALID_ATLAS};
use super::{with_store, FfiAtlasEntry, FfiAtlasStats, GoudAtlasHandle, ATLAS_STORES};

/// # Safety
/// `ptr` must be a valid null-terminated C string.
unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Result<&'a str, ()> {
    // SAFETY: caller guarantees valid null-terminated C string
    CStr::from_ptr(ptr).to_str().map_err(|_| ())
}

/// Creates a new empty texture atlas.
///
/// # Safety
///
/// `category` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn goud_atlas_create(
    context_id: GoudContextId,
    category: *const c_char,
    max_width: u32,
    max_height: u32,
) -> GoudAtlasHandle {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_ATLAS;
    }
    if category.is_null() {
        set_last_error(GoudError::InternalError("Null pointer: category".into()));
        return GOUD_INVALID_ATLAS;
    }
    // SAFETY: caller guarantees category is a valid null-terminated C string
    let cat = match cstr_to_str(category) {
        Ok(s) => s,
        Err(()) => {
            set_last_error(GoudError::InternalError("Invalid UTF-8 in category".into()));
            return GOUD_INVALID_ATLAS;
        }
    };
    let atlas = TextureAtlas::new(cat, max_width, max_height);
    with_store(context_id, |store| store.insert(atlas))
}

/// Loads an image from a file and packs it directly into the atlas.
///
/// # Safety
///
/// `key` and `path` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn goud_atlas_add_from_file(
    context_id: GoudContextId,
    atlas: GoudAtlasHandle,
    key: *const c_char,
    path: *const c_char,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if key.is_null() || path.is_null() {
        set_last_error(GoudError::InternalError("Null pointer: key or path".into()));
        return false;
    }
    // SAFETY: caller guarantees key/path are valid null-terminated C strings
    let key_str = match cstr_to_str(key) {
        Ok(s) => s,
        Err(()) => {
            set_last_error(GoudError::InternalError("Invalid UTF-8 in key".into()));
            return false;
        }
    };
    let path_str = match cstr_to_str(path) {
        Ok(s) => s,
        Err(()) => {
            set_last_error(GoudError::InternalError("Invalid UTF-8 in path".into()));
            return false;
        }
    };
    let img = match image::open(path_str) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            set_last_error(GoudError::ResourceLoadFailed(format!(
                "Failed to load image '{}': {}",
                path_str, e
            )));
            return false;
        }
    };
    let (width, height) = (img.width(), img.height());
    let data = img.into_raw();
    with_store(context_id, |store| match store.atlases.get_mut(&atlas) {
        Some(a) => a.add_pixels(key_str, &data, width, height),
        None => {
            set_last_error(GoudError::InvalidHandle);
            false
        }
    })
}

/// Reserved — GPU pixel readback is not supported.
///
/// # Safety
///
/// `key` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn goud_atlas_add_texture(
    _context_id: GoudContextId,
    _atlas: GoudAtlasHandle,
    _key: *const c_char,
    _texture: GoudTextureHandle,
) -> bool {
    set_last_error(GoudError::InternalError(
        "goud_atlas_add_texture: GPU readback not supported. \
         Use goud_atlas_add_from_file or goud_atlas_add_pixels."
            .into(),
    ));
    false
}

/// Packs raw RGBA8 pixel data into the atlas under the given key.
///
/// # Safety
///
/// * `key` must be a valid null-terminated C string.
/// * `pixels` must point to at least `width * height * 4` valid bytes.
#[no_mangle]
pub unsafe extern "C" fn goud_atlas_add_pixels(
    context_id: GoudContextId,
    atlas: GoudAtlasHandle,
    key: *const c_char,
    pixels: *const u8,
    width: u32,
    height: u32,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if key.is_null() || pixels.is_null() {
        set_last_error(GoudError::InternalError(
            "Null pointer: key or pixels".into(),
        ));
        return false;
    }
    if width == 0 || height == 0 {
        set_last_error(GoudError::InternalError(
            "Width and height must be > 0".into(),
        ));
        return false;
    }
    // SAFETY: caller guarantees key is a valid null-terminated C string
    let key_str = match cstr_to_str(key) {
        Ok(s) => s,
        Err(()) => {
            set_last_error(GoudError::InternalError("Invalid UTF-8 in key".into()));
            return false;
        }
    };
    let byte_count = match (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
    {
        Some(n) => n,
        None => {
            set_last_error(GoudError::InternalError(
                "Pixel buffer size overflow".into(),
            ));
            return false;
        }
    };
    // SAFETY: caller guarantees pixels points to at least byte_count valid bytes
    let data = std::slice::from_raw_parts(pixels, byte_count);
    with_store(context_id, |store| match store.atlases.get_mut(&atlas) {
        Some(a) => a.add_pixels(key_str, data, width, height),
        None => {
            set_last_error(GoudError::InvalidHandle);
            false
        }
    })
}

/// Finalizes the atlas by uploading it to the GPU.
#[no_mangle]
pub extern "C" fn goud_atlas_finalize(
    context_id: GoudContextId,
    atlas: GoudAtlasHandle,
) -> GoudTextureHandle {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_TEXTURE;
    }
    ATLAS_STORES.with(|cell| {
        let mut stores = cell.borrow_mut();
        let key = context_key(context_id);
        let store = match stores.get_mut(&key) {
            Some(s) => s,
            None => {
                set_last_error(GoudError::InvalidHandle);
                return GOUD_INVALID_TEXTURE;
            }
        };
        let atlas_ref = match store.atlases.get_mut(&atlas) {
            Some(a) => a,
            None => {
                set_last_error(GoudError::InvalidHandle);
                return GOUD_INVALID_TEXTURE;
            }
        };
        with_window_state(context_id, |state| {
            match atlas_ref.ensure_gpu_texture(state.backend_mut()) {
                Ok(handle) => ((handle.generation() as u64) << 32) | (handle.index() as u64),
                Err(e) => {
                    set_last_error(GoudError::InternalError(e));
                    GOUD_INVALID_TEXTURE
                }
            }
        })
        .unwrap_or_else(|| {
            set_last_error(GoudError::InvalidContext);
            GOUD_INVALID_TEXTURE
        })
    })
}

/// Queries the UV rect and pixel placement for a packed texture.
///
/// # Safety
///
/// * `key` must be a valid null-terminated C string.
/// * `out_entry` must point to a valid `FfiAtlasEntry`.
#[no_mangle]
pub unsafe extern "C" fn goud_atlas_get_entry(
    context_id: GoudContextId,
    atlas: GoudAtlasHandle,
    key: *const c_char,
    out_entry: *mut FfiAtlasEntry,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if key.is_null() || out_entry.is_null() {
        set_last_error(GoudError::InternalError(
            "Null pointer: key or out_entry".into(),
        ));
        return false;
    }
    // SAFETY: caller guarantees key is a valid null-terminated C string
    let key_str = match cstr_to_str(key) {
        Ok(s) => s,
        Err(()) => {
            set_last_error(GoudError::InternalError("Invalid UTF-8 in key".into()));
            return false;
        }
    };
    with_store(context_id, |store| {
        let a = match store.atlases.get(&atlas) {
            Some(a) => a,
            None => {
                set_last_error(GoudError::InvalidHandle);
                return false;
            }
        };
        match a.get_entry(key_str) {
            Some(info) => {
                // SAFETY: caller guarantees out_entry points to valid FfiAtlasEntry
                *out_entry = FfiAtlasEntry {
                    u_min: info.uv_rect.u_min,
                    v_min: info.uv_rect.v_min,
                    u_max: info.uv_rect.u_max,
                    v_max: info.uv_rect.v_max,
                    pixel_x: info.rect.x,
                    pixel_y: info.rect.y,
                    pixel_w: info.rect.width,
                    pixel_h: info.rect.height,
                };
                true
            }
            None => {
                set_last_error(GoudError::InternalError(format!(
                    "Atlas entry not found: '{key_str}'"
                )));
                false
            }
        }
    })
}

/// Queries atlas packing statistics.
///
/// # Safety
///
/// `out_stats` must point to a valid `FfiAtlasStats`.
#[no_mangle]
pub unsafe extern "C" fn goud_atlas_get_stats(
    context_id: GoudContextId,
    atlas: GoudAtlasHandle,
    out_stats: *mut FfiAtlasStats,
) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    if out_stats.is_null() {
        set_last_error(GoudError::InternalError("Null pointer: out_stats".into()));
        return false;
    }
    with_store(context_id, |store| {
        let a = match store.atlases.get(&atlas) {
            Some(a) => a,
            None => {
                set_last_error(GoudError::InvalidHandle);
                return false;
            }
        };
        let s = a.stats();
        // SAFETY: caller guarantees out_stats points to valid FfiAtlasStats
        *out_stats = FfiAtlasStats {
            texture_count: s.texture_count,
            width: s.width,
            height: s.height,
            used_pixels: s.used_pixels,
            total_pixels: s.total_pixels,
            efficiency: s.efficiency,
            wasted_pixels: s.wasted_pixels,
        };
        true
    })
}

/// Returns the GPU texture handle for the atlas.
#[no_mangle]
pub extern "C" fn goud_atlas_get_texture(
    context_id: GoudContextId,
    atlas: GoudAtlasHandle,
) -> GoudTextureHandle {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_TEXTURE;
    }
    with_store(context_id, |store| match store.atlases.get(&atlas) {
        Some(a) => match a.gpu_texture() {
            Some(handle) => ((handle.generation() as u64) << 32) | (handle.index() as u64),
            None => {
                set_last_error(GoudError::InternalError("Atlas not finalized".into()));
                GOUD_INVALID_TEXTURE
            }
        },
        None => {
            set_last_error(GoudError::InvalidHandle);
            GOUD_INVALID_TEXTURE
        }
    })
}

/// Destroys an atlas and frees its GPU + CPU resources.
#[no_mangle]
pub extern "C" fn goud_atlas_destroy(context_id: GoudContextId, atlas: GoudAtlasHandle) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }
    let removed = with_store(context_id, |store| store.atlases.remove(&atlas));
    match removed {
        Some(mut a) => {
            with_window_state(context_id, |state| {
                a.destroy_gpu_texture(state.backend_mut());
            });
            true
        }
        None => {
            set_last_error(GoudError::InvalidHandle);
            false
        }
    }
}
