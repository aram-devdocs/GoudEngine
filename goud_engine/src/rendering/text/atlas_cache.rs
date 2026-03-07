//! Glyph atlas caching layer.
//!
//! Caches generated [`GlyphAtlas`] instances keyed by `(font_handle, size)`
//! so that repeated text rendering at the same size reuses the same atlas.

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::assets::{loaders::FontAsset, AssetHandle};
use crate::libs::graphics::backend::render_backend::RenderBackend;
use crate::libs::graphics::backend::types::TextureHandle;

use super::glyph_atlas::GlyphAtlas;

/// Cache for glyph atlases, keyed by `(font_handle, size_px_u32)`.
///
/// The size is stored as a `u32` (rounded to the nearest integer from `f32`) so that the key
/// is `Hash + Eq`. Callers that need sub-pixel size variation should round
/// to the nearest integer before querying.
#[derive(Debug)]
pub struct GlyphAtlasCache {
    cache: HashMap<(AssetHandle<FontAsset>, u32), GlyphAtlas>,
    /// GPU texture handles from invalidated atlases, pending destruction.
    pending_destroy: Vec<TextureHandle>,
}

impl GlyphAtlasCache {
    /// Creates an empty cache.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            pending_destroy: Vec::new(),
        }
    }

    /// Returns a cached atlas or generates (and caches) a new one.
    ///
    /// # Arguments
    ///
    /// * `font`        - The loaded font asset (used to parse the font data).
    /// * `font_handle` - The asset handle identifying this font.
    /// * `size_px`     - Desired pixel size (rounded to the nearest integer for cache key).
    ///
    /// # Errors
    ///
    /// Returns an error if font parsing or atlas generation fails.
    pub fn get_or_create(
        &mut self,
        font: &FontAsset,
        font_handle: AssetHandle<FontAsset>,
        size_px: f32,
    ) -> Result<&GlyphAtlas, String> {
        let size_key = size_px.round() as u32;
        let key = (font_handle, size_key);

        if let Entry::Vacant(e) = self.cache.entry(key) {
            let parsed_font = font.parse()?;
            let atlas = GlyphAtlas::generate(&parsed_font, size_px)?;
            e.insert(atlas);
        }

        // Invariant: we just ensured the key exists above (either pre-existing or just inserted).
        self.cache
            .get(&key)
            .ok_or_else(|| "internal error: cache entry missing after insertion".to_string())
    }

    /// Removes all cached atlases for the given font handle (all sizes).
    ///
    /// Any GPU texture handles owned by the removed atlases are queued
    /// for destruction. Call [`destroy_gpu_textures`](Self::destroy_gpu_textures)
    /// to release them.
    pub fn invalidate_font(&mut self, font_handle: AssetHandle<FontAsset>) {
        self.cache.retain(|&(h, _), atlas| {
            if h == font_handle {
                if let Some(tex) = atlas.take_gpu_texture() {
                    self.pending_destroy.push(tex);
                }
                false
            } else {
                true
            }
        });
    }

    /// Processes a batch of font hot-reloads by invalidating every font
    /// in the provided slice.
    ///
    /// Returns the number of atlas entries that were removed.
    pub fn process_reloads(&mut self, reloaded: &[AssetHandle<FontAsset>]) -> usize {
        let before = self.cache.len();
        for &handle in reloaded {
            self.invalidate_font(handle);
        }
        before - self.cache.len()
    }

    /// Destroys all GPU textures that were queued by previous invalidations.
    ///
    /// This must be called with a valid render backend to release GPU memory.
    pub fn destroy_gpu_textures(&mut self, backend: &mut dyn RenderBackend) {
        for handle in self.pending_destroy.drain(..) {
            backend.destroy_texture(handle);
        }
    }

    /// Returns the number of GPU texture handles pending destruction.
    pub fn pending_destroy_count(&self) -> usize {
        self.pending_destroy.len()
    }

    /// Removes all cached atlases.
    pub fn clear(&mut self) {
        for atlas in self.cache.values_mut() {
            if let Some(tex) = atlas.take_gpu_texture() {
                self.pending_destroy.push(tex);
            }
        }
        self.cache.clear();
    }

    /// Returns the number of cached atlases.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.cache.len()
    }
}

impl Default for GlyphAtlasCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::loaders::font::format::FontFormat;
    use crate::assets::loaders::FontAsset;
    use crate::assets::loaders::FontStyle;

    /// Build a `FontAsset` from the test TTF fixture.
    fn test_font_asset() -> FontAsset {
        let bytes = include_bytes!("../../../test_assets/fonts/test_font.ttf").to_vec();
        let font = fontdue::Font::from_bytes(bytes.as_slice(), fontdue::FontSettings::default())
            .expect("parse");

        FontAsset::new(
            bytes,
            "TestFont".to_string(),
            FontStyle::Regular,
            FontFormat::Ttf,
            1000,
            font.glyph_count() as u16,
            0,
        )
    }

    fn handle_a() -> AssetHandle<FontAsset> {
        AssetHandle::new(0, 1)
    }

    fn handle_b() -> AssetHandle<FontAsset> {
        AssetHandle::new(1, 1)
    }

    #[test]
    fn test_cache_get_or_create_returns_atlas() {
        let mut cache = GlyphAtlasCache::new();
        let font = test_font_asset();

        let result = cache.get_or_create(&font, handle_a(), 16.0);
        assert!(result.is_ok());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_hit_returns_same_atlas() {
        let mut cache = GlyphAtlasCache::new();
        let font = test_font_asset();

        let _ = cache.get_or_create(&font, handle_a(), 16.0).unwrap();
        let _ = cache.get_or_create(&font, handle_a(), 16.0).unwrap();

        // Should still have only 1 entry (cache hit).
        assert_eq!(cache.len(), 1);

        // Verify the cached atlas contains 'A'.
        let atlas = cache.get_or_create(&font, handle_a(), 16.0).unwrap();
        assert!(atlas.glyph_info('A').is_some());
    }

    #[test]
    fn test_cache_different_sizes_get_different_entries() {
        let mut cache = GlyphAtlasCache::new();
        let font = test_font_asset();

        let _ = cache.get_or_create(&font, handle_a(), 16.0).unwrap();
        let _ = cache.get_or_create(&font, handle_a(), 32.0).unwrap();

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_different_handles_get_different_entries() {
        let mut cache = GlyphAtlasCache::new();
        let font = test_font_asset();

        let _ = cache.get_or_create(&font, handle_a(), 16.0).unwrap();
        let _ = cache.get_or_create(&font, handle_b(), 16.0).unwrap();

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_invalidate_font_removes_all_sizes() {
        let mut cache = GlyphAtlasCache::new();
        let font = test_font_asset();

        let _ = cache.get_or_create(&font, handle_a(), 16.0).unwrap();
        let _ = cache.get_or_create(&font, handle_a(), 32.0).unwrap();
        let _ = cache.get_or_create(&font, handle_b(), 16.0).unwrap();

        cache.invalidate_font(handle_a());

        // Only handle_b's entry should remain.
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_clear_removes_all_entries() {
        let mut cache = GlyphAtlasCache::new();
        let font = test_font_asset();

        let _ = cache.get_or_create(&font, handle_a(), 16.0).unwrap();
        let _ = cache.get_or_create(&font, handle_b(), 24.0).unwrap();

        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_default_creates_empty_cache() {
        let cache = GlyphAtlasCache::default();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_process_reloads_invalidates_multiple_fonts() {
        let mut cache = GlyphAtlasCache::new();
        let font = test_font_asset();

        // Populate: handle_a at two sizes, handle_b at one size.
        let _ = cache.get_or_create(&font, handle_a(), 16.0).unwrap();
        let _ = cache.get_or_create(&font, handle_a(), 32.0).unwrap();
        let _ = cache.get_or_create(&font, handle_b(), 16.0).unwrap();

        assert_eq!(cache.len(), 3);

        // Reload both fonts at once.
        let invalidated = cache.process_reloads(&[handle_a(), handle_b()]);

        assert_eq!(invalidated, 3, "all three atlases should be invalidated");
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_process_reloads_returns_zero_for_unknown_fonts() {
        let mut cache = GlyphAtlasCache::new();
        let font = test_font_asset();

        let _ = cache.get_or_create(&font, handle_a(), 16.0).unwrap();

        // Reload a font that is not in the cache.
        let invalidated = cache.process_reloads(&[handle_b()]);

        assert_eq!(invalidated, 0);
        assert_eq!(cache.len(), 1, "existing entry should remain");
    }

    #[test]
    fn test_process_reloads_partial_invalidation() {
        let mut cache = GlyphAtlasCache::new();
        let font = test_font_asset();

        let _ = cache.get_or_create(&font, handle_a(), 16.0).unwrap();
        let _ = cache.get_or_create(&font, handle_b(), 16.0).unwrap();

        let invalidated = cache.process_reloads(&[handle_a()]);

        assert_eq!(invalidated, 1);
        assert_eq!(cache.len(), 1, "handle_b entry should remain");
    }
}
