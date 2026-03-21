//! # Texture Atlas FFI
//!
//! C-compatible functions for runtime texture atlas packing.
//! Atlases pack multiple textures into a single GPU texture to reduce
//! draw calls when used with the sprite batch renderer.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;

use crate::core::error::{set_last_error, GoudError};
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use crate::ffi::window::with_window_state;
use crate::rendering::texture_atlas::TextureAtlas;

use super::texture::{GoudTextureHandle, GOUD_INVALID_TEXTURE};

// ============================================================================
// Handle types
// ============================================================================

/// Opaque atlas handle for FFI.
pub type GoudAtlasHandle = u64;

/// Invalid atlas handle constant.
pub const GOUD_INVALID_ATLAS: GoudAtlasHandle = u64::MAX;

// ============================================================================
// FFI result structs
// ============================================================================

/// FFI-safe entry describing a packed texture's position within the atlas.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiAtlasEntry {
    /// Left edge in UV space (0.0..1.0).
    pub u_min: f32,
    /// Top edge in UV space (0.0..1.0).
    pub v_min: f32,
    /// Right edge in UV space (0.0..1.0).
    pub u_max: f32,
    /// Bottom edge in UV space (0.0..1.0).
    pub v_max: f32,
    /// Pixel X offset of the packed texture within the atlas.
    pub pixel_x: u32,
    /// Pixel Y offset of the packed texture within the atlas.
    pub pixel_y: u32,
    /// Width of the packed texture in pixels.
    pub pixel_w: u32,
    /// Height of the packed texture in pixels.
    pub pixel_h: u32,
}

/// FFI-safe atlas statistics.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiAtlasStats {
    /// Number of textures packed into the atlas.
    pub texture_count: u32,
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Total pixel area consumed by packed textures.
    pub used_pixels: u64,
    /// Total pixel area of the atlas (width * height).
    pub total_pixels: u64,
    /// Pack efficiency as a percentage (0.0 - 100.0).
    pub efficiency: f32,
    /// Wasted pixel area (total - used).
    pub wasted_pixels: u64,
}

// ============================================================================
// Thread-local atlas storage (one map per context)
// ============================================================================

struct AtlasStore {
    atlases: HashMap<u64, TextureAtlas>,
    next_id: u64,
}

impl AtlasStore {
    fn new() -> Self {
        Self {
            atlases: HashMap::new(),
            next_id: 1,
        }
    }

    fn insert(&mut self, atlas: TextureAtlas) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.atlases.insert(id, atlas);
        id
    }
}

type ContextKey = (u32, u32);

thread_local! {
    static ATLAS_STORES: RefCell<HashMap<ContextKey, AtlasStore>> =
        RefCell::new(HashMap::new());
}

fn context_key(id: GoudContextId) -> ContextKey {
    (id.index(), id.generation())
}

fn with_store<F, R>(context_id: GoudContextId, f: F) -> R
where
    F: FnOnce(&mut AtlasStore) -> R,
{
    ATLAS_STORES.with(|cell| {
        let mut stores = cell.borrow_mut();
        let store = stores
            .entry(context_key(context_id))
            .or_insert_with(AtlasStore::new);
        f(store)
    })
}

// ============================================================================
// Helper: convert C string pointer to &str
// ============================================================================

/// # Safety
/// `ptr` must be a valid null-terminated C string.
unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Result<&'a str, ()> {
    // SAFETY: caller guarantees valid null-terminated C string
    CStr::from_ptr(ptr).to_str().map_err(|_| ())
}

// ============================================================================
// FFI functions
// ============================================================================

/// Creates a new empty texture atlas with the given category and max size.
///
/// Pass 0 for `max_width` / `max_height` to use the default (2048).
///
/// # Returns
///
/// A valid atlas handle, or `GOUD_INVALID_ATLAS` on error.
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
/// This is the most efficient way to add textures — no intermediate GPU
/// texture is created.
///
/// # Returns
///
/// `true` on success, `false` if the texture does not fit or on error.
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

    // SAFETY: caller guarantees key and path are valid null-terminated C strings
    let key_str = match cstr_to_str(key) {
        Ok(s) => s,
        Err(()) => {
            set_last_error(GoudError::InternalError("Invalid UTF-8 in key".into()));
            return false;
        }
    };
    // SAFETY: caller guarantees path is a valid null-terminated C string
    let path_str = match cstr_to_str(path) {
        Ok(s) => s,
        Err(()) => {
            set_last_error(GoudError::InternalError("Invalid UTF-8 in path".into()));
            return false;
        }
    };

    // Load image data from file (CPU only, no GPU texture created).
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

    let width = img.width();
    let height = img.height();
    let data = img.into_raw();

    with_store(context_id, |store| match store.atlases.get_mut(&atlas) {
        Some(a) => a.add_pixels(key_str, &data, width, height),
        None => {
            set_last_error(GoudError::InvalidHandle);
            false
        }
    })
}

/// Reserved for future GPU pixel readback support.
///
/// GPU pixel readback is not currently supported. Use
/// `goud_atlas_add_from_file` or `goud_atlas_add_pixels` instead.
///
/// # Returns
///
/// Always returns `false` and sets an error.
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
/// `pixels` must point to `width * height * 4` bytes of RGBA8 data.
///
/// # Returns
///
/// `true` on success, `false` if the texture does not fit or on error.
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
///
/// Returns the GPU texture handle for use in `FfiSpriteCmd.texture`.
/// Returns `GOUD_INVALID_TEXTURE` on error.
#[no_mangle]
pub extern "C" fn goud_atlas_finalize(
    context_id: GoudContextId,
    atlas: GoudAtlasHandle,
) -> GoudTextureHandle {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return GOUD_INVALID_TEXTURE;
    }

    // Hold the atlas store borrow while calling ensure_gpu_texture.
    // ATLAS_STORES and WINDOW_STATES are separate thread-locals so
    // nesting their borrows is safe.
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
/// # Returns
///
/// `true` if the entry was found and written to `out_entry`, `false` otherwise.
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
                    "Atlas entry not found: '{}'",
                    key_str
                )));
                false
            }
        }
    })
}

/// Queries atlas packing statistics.
///
/// # Returns
///
/// `true` on success, `false` on error.
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
///
/// Returns `GOUD_INVALID_TEXTURE` if the atlas has not been finalized.
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
///
/// # Returns
///
/// `true` on success, `false` if the handle is invalid.
#[no_mangle]
pub extern "C" fn goud_atlas_destroy(context_id: GoudContextId, atlas: GoudAtlasHandle) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    // Remove atlas from store, then destroy its GPU texture via window state.
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

// ============================================================================
// Context cleanup
// ============================================================================

/// Removes all atlases for a context, destroying their GPU textures.
///
/// Called during `goud_window_destroy` to prevent GPU resource leaks.
pub(crate) fn cleanup_atlas_state(context_id: GoudContextId) {
    let removed = ATLAS_STORES.with(|cell| {
        let mut stores = cell.borrow_mut();
        stores.remove(&context_key(context_id))
    });

    if let Some(mut store) = removed {
        for (_, atlas) in store.atlases.iter_mut() {
            with_window_state(context_id, |state| {
                atlas.destroy_gpu_texture(state.backend_mut());
            });
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_atlas_entry_layout() {
        assert_eq!(
            std::mem::size_of::<FfiAtlasEntry>(),
            4 * 4 + 4 * 4, // 4 floats + 4 u32s = 32 bytes
            "FfiAtlasEntry should be 32 bytes"
        );
    }

    #[test]
    fn test_ffi_atlas_stats_layout() {
        // #[repr(C)] layout: 3 u32s (12) + 4 pad + 2 u64s (16) + f32 (4) + 4 pad + u64 (8) = 48
        let size = std::mem::size_of::<FfiAtlasStats>();
        assert_eq!(size, 48, "FfiAtlasStats should be 48 bytes, got {size}");
    }

    #[test]
    fn test_atlas_store_insert_increments() {
        let mut store = AtlasStore::new();
        let id1 = store.insert(TextureAtlas::new("a", 64, 64));
        let id2 = store.insert(TextureAtlas::new("b", 64, 64));
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_invalid_atlas_handle() {
        assert_eq!(GOUD_INVALID_ATLAS, u64::MAX);
    }
}
