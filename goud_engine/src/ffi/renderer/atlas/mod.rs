//! # Texture Atlas FFI
//!
//! C-compatible functions for runtime texture atlas packing.
//! Atlases pack multiple textures into a single GPU texture to reduce
//! draw calls when used with the sprite batch renderer.

mod ffi;

use std::cell::RefCell;
use std::collections::HashMap;

use crate::ffi::context::GoudContextId;
use crate::ffi::window::with_window_state;
use crate::rendering::texture_atlas::TextureAtlas;

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
    /// Pixel X offset within the atlas.
    pub pixel_x: u32,
    /// Pixel Y offset within the atlas.
    pub pixel_y: u32,
    /// Width in pixels.
    pub pixel_w: u32,
    /// Height in pixels.
    pub pixel_h: u32,
}

/// FFI-safe atlas statistics.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiAtlasStats {
    /// Number of textures packed.
    pub texture_count: u32,
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Pixel area consumed by packed textures.
    pub used_pixels: u64,
    /// Total pixel area of the atlas.
    pub total_pixels: u64,
    /// Pack efficiency percentage (0.0-100.0).
    pub efficiency: f32,
    /// Wasted pixel area.
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
// Re-exports
// ============================================================================

pub use ffi::{
    goud_atlas_add_from_file, goud_atlas_add_pixels, goud_atlas_add_texture, goud_atlas_create,
    goud_atlas_destroy, goud_atlas_finalize, goud_atlas_get_entry, goud_atlas_get_stats,
    goud_atlas_get_texture,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_atlas_entry_layout() {
        assert_eq!(std::mem::size_of::<FfiAtlasEntry>(), 32);
    }

    #[test]
    fn test_ffi_atlas_stats_layout() {
        assert_eq!(std::mem::size_of::<FfiAtlasStats>(), 48);
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
