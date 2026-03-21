//! Runtime texture atlas — packs multiple textures into a single GPU texture.
//!
//! Follows the same pattern as [`crate::rendering::text::glyph_atlas::GlyphAtlas`]:
//! CPU-side pixel data with lazy GPU upload and a dirty flag.

use std::collections::HashMap;

use crate::libs::graphics::backend::render_backend::RenderBackend;
use crate::libs::graphics::backend::types::{
    TextureFilter, TextureFormat, TextureHandle, TextureWrap,
};

use super::packer::{PackedRect, ShelfPacker};
use super::stats::AtlasStats;

/// 1-pixel padding between packed textures to prevent bleeding.
const ATLAS_PADDING: u32 = 1;

/// Default maximum atlas dimension.
pub const DEFAULT_MAX_ATLAS_SIZE: u32 = 2048;

/// UV rectangle describing a packed texture's position within the atlas.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasUvRect {
    /// Left edge in UV space (0.0..1.0).
    pub u_min: f32,
    /// Top edge in UV space (0.0..1.0).
    pub v_min: f32,
    /// Right edge in UV space (0.0..1.0).
    pub u_max: f32,
    /// Bottom edge in UV space (0.0..1.0).
    pub v_max: f32,
}

/// Metadata for a single texture packed into the atlas.
#[derive(Debug, Clone)]
pub struct PackedTextureInfo {
    /// Unique key provided by the caller.
    pub key: String,
    /// Pixel-space placement in the atlas.
    pub rect: PackedRect,
    /// Normalized UV coordinates.
    pub uv_rect: AtlasUvRect,
    /// Original texture width before packing.
    pub original_width: u32,
    /// Original texture height before packing.
    pub original_height: u32,
}

/// A runtime-packed texture atlas.
///
/// Textures are added to the atlas via [`add_pixels`](Self::add_pixels).
/// After all textures are packed, call [`ensure_gpu_texture`](Self::ensure_gpu_texture)
/// to upload the atlas to the GPU.
#[derive(Debug, Clone)]
pub struct TextureAtlas {
    /// RGBA8 pixel data (4 bytes per pixel).
    texture_data: Vec<u8>,
    /// Atlas width in pixels.
    width: u32,
    /// Atlas height in pixels.
    height: u32,
    /// Category label (e.g. "terrain", "entities").
    category: String,
    /// Shelf packer for incremental rectangle placement.
    packer: ShelfPacker,
    /// Per-texture metadata, keyed by string identifier.
    entries: HashMap<String, PackedTextureInfo>,
    /// Cached GPU texture handle, lazily uploaded.
    gpu_texture: Option<TextureHandle>,
    /// True when CPU data changed since last GPU sync.
    dirty: bool,
    /// Incremented on each successful pack operation.
    version: u64,
}

impl TextureAtlas {
    /// Creates an empty atlas with the given category and dimensions.
    ///
    /// Dimensions default to [`DEFAULT_MAX_ATLAS_SIZE`] when 0 is passed.
    pub fn new(category: &str, max_width: u32, max_height: u32) -> Self {
        let w = if max_width == 0 {
            DEFAULT_MAX_ATLAS_SIZE
        } else {
            max_width
        };
        let h = if max_height == 0 {
            DEFAULT_MAX_ATLAS_SIZE
        } else {
            max_height
        };
        let pixel_count = (w as usize) * (h as usize) * 4;
        Self {
            texture_data: vec![0u8; pixel_count],
            width: w,
            height: h,
            category: category.to_string(),
            packer: ShelfPacker::new(w, h, ATLAS_PADDING),
            entries: HashMap::new(),
            gpu_texture: None,
            dirty: false,
            version: 0,
        }
    }

    /// Packs raw RGBA8 pixel data into the atlas under the given key.
    ///
    /// Returns `true` on success, `false` if the texture does not fit
    /// or the key is already used.
    pub fn add_pixels(&mut self, key: &str, pixels: &[u8], width: u32, height: u32) -> bool {
        if self.entries.contains_key(key) {
            return false;
        }

        let expected_len = (width as usize) * (height as usize) * 4;
        if pixels.len() != expected_len {
            return false;
        }

        let rect = match self.packer.pack(width, height) {
            Some(r) => r,
            None => return false,
        };

        // Blit source pixels into the atlas texture data.
        let atlas_stride = self.width as usize * 4;
        for row in 0..height as usize {
            let src_start = row * (width as usize) * 4;
            let src_end = src_start + (width as usize) * 4;
            let dst_start = (rect.y as usize + row) * atlas_stride + rect.x as usize * 4;
            self.texture_data[dst_start..dst_start + (width as usize) * 4]
                .copy_from_slice(&pixels[src_start..src_end]);
        }

        let uv_rect = AtlasUvRect {
            u_min: rect.x as f32 / self.width as f32,
            v_min: rect.y as f32 / self.height as f32,
            u_max: (rect.x + rect.width) as f32 / self.width as f32,
            v_max: (rect.y + rect.height) as f32 / self.height as f32,
        };

        let info = PackedTextureInfo {
            key: key.to_string(),
            rect,
            uv_rect,
            original_width: width,
            original_height: height,
        };

        self.entries.insert(key.to_string(), info);
        self.dirty = true;
        self.version += 1;

        true
    }

    /// Returns metadata for a packed texture by key.
    pub fn get_entry(&self, key: &str) -> Option<&PackedTextureInfo> {
        self.entries.get(key)
    }

    /// Returns the number of textures packed.
    pub fn texture_count(&self) -> u32 {
        self.entries.len() as u32
    }

    /// Returns the atlas category label.
    pub fn category(&self) -> &str {
        &self.category
    }

    /// Returns the atlas dimensions.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Computes atlas statistics.
    pub fn stats(&self) -> AtlasStats {
        AtlasStats::compute(
            self.entries.len() as u32,
            self.width,
            self.height,
            self.packer.used_area(),
        )
    }

    /// Returns the cached GPU texture handle, if uploaded.
    pub fn gpu_texture(&self) -> Option<TextureHandle> {
        self.gpu_texture
    }

    /// Lazily uploads (or updates) the atlas pixel data on the GPU.
    ///
    /// Mirrors the pattern from [`GlyphAtlas::ensure_gpu_texture`].
    pub fn ensure_gpu_texture(
        &mut self,
        backend: &mut dyn RenderBackend,
    ) -> Result<TextureHandle, String> {
        if let Some(handle) = self.gpu_texture {
            if !self.dirty {
                return Ok(handle);
            }
            // Same size — update in-place.
            if backend.texture_size(handle) == Some((self.width, self.height)) {
                backend
                    .update_texture(handle, 0, 0, self.width, self.height, &self.texture_data)
                    .map_err(|e| format!("failed to update texture atlas: {e}"))?;
                self.dirty = false;
                return Ok(handle);
            }
            // Size mismatch — recreate.
            backend.destroy_texture(handle);
            self.gpu_texture = None;
        }

        let handle = backend
            .create_texture(
                self.width,
                self.height,
                TextureFormat::RGBA8,
                TextureFilter::Nearest,
                TextureWrap::ClampToEdge,
                &self.texture_data,
            )
            .map_err(|e| format!("failed to create GPU texture for atlas: {e}"))?;

        self.gpu_texture = Some(handle);
        self.dirty = false;
        Ok(handle)
    }

    /// Destroys the GPU texture if one has been uploaded.
    pub fn destroy_gpu_texture(&mut self, backend: &mut dyn RenderBackend) {
        if let Some(handle) = self.gpu_texture.take() {
            backend.destroy_texture(handle);
        }
    }

    /// Returns true when CPU data changed since last GPU sync.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Returns the content version counter.
    pub fn version(&self) -> u64 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red_pixel_data(w: u32, h: u32) -> Vec<u8> {
        vec![255, 0, 0, 255].repeat((w * h) as usize)
    }

    fn green_pixel_data(w: u32, h: u32) -> Vec<u8> {
        vec![0, 255, 0, 255].repeat((w * h) as usize)
    }

    #[test]
    fn test_add_single_texture() {
        let mut atlas = TextureAtlas::new("test", 128, 128);
        let pixels = red_pixel_data(32, 32);
        assert!(atlas.add_pixels("red", &pixels, 32, 32));
        assert_eq!(atlas.texture_count(), 1);
        assert!(atlas.is_dirty());
    }

    #[test]
    fn test_add_duplicate_key_rejected() {
        let mut atlas = TextureAtlas::new("test", 128, 128);
        let pixels = red_pixel_data(16, 16);
        assert!(atlas.add_pixels("a", &pixels, 16, 16));
        assert!(!atlas.add_pixels("a", &pixels, 16, 16));
    }

    #[test]
    fn test_wrong_pixel_data_length() {
        let mut atlas = TextureAtlas::new("test", 128, 128);
        let pixels = vec![0u8; 100]; // wrong length
        assert!(!atlas.add_pixels("bad", &pixels, 32, 32));
    }

    #[test]
    fn test_overflow_returns_false() {
        let mut atlas = TextureAtlas::new("test", 32, 32);
        let big = red_pixel_data(33, 33);
        assert!(!atlas.add_pixels("big", &big, 33, 33));
    }

    #[test]
    fn test_uv_rect_correctness() {
        let mut atlas = TextureAtlas::new("test", 256, 256);
        let pixels = red_pixel_data(64, 64);
        atlas.add_pixels("tile", &pixels, 64, 64);
        let entry = atlas.get_entry("tile").unwrap();
        assert!((entry.uv_rect.u_min - 0.0).abs() < 0.001);
        assert!((entry.uv_rect.v_min - 0.0).abs() < 0.001);
        assert!((entry.uv_rect.u_max - 64.0 / 256.0).abs() < 0.001);
        assert!((entry.uv_rect.v_max - 64.0 / 256.0).abs() < 0.001);
    }

    #[test]
    fn test_pixel_blit_correctness() {
        let mut atlas = TextureAtlas::new("test", 64, 64);
        let red = red_pixel_data(2, 2);
        atlas.add_pixels("red", &red, 2, 2);

        // First pixel at (0,0) should be red
        assert_eq!(atlas.texture_data[0], 255); // R
        assert_eq!(atlas.texture_data[1], 0); // G
        assert_eq!(atlas.texture_data[2], 0); // B
        assert_eq!(atlas.texture_data[3], 255); // A
    }

    #[test]
    fn test_multiple_textures_distinct_pixels() {
        let mut atlas = TextureAtlas::new("test", 128, 128);
        let red = red_pixel_data(16, 16);
        let green = green_pixel_data(16, 16);
        atlas.add_pixels("red", &red, 16, 16);
        atlas.add_pixels("green", &green, 16, 16);

        let r_entry = atlas.get_entry("red").unwrap();
        let g_entry = atlas.get_entry("green").unwrap();

        // Verify they are at different positions
        assert_ne!(r_entry.rect.x, g_entry.rect.x);

        // Verify red pixel data at red entry position
        let stride = 128 * 4;
        let r_off = (r_entry.rect.y as usize) * stride + (r_entry.rect.x as usize) * 4;
        assert_eq!(atlas.texture_data[r_off], 255);
        assert_eq!(atlas.texture_data[r_off + 1], 0);

        // Verify green pixel data at green entry position
        let g_off = (g_entry.rect.y as usize) * stride + (g_entry.rect.x as usize) * 4;
        assert_eq!(atlas.texture_data[g_off], 0);
        assert_eq!(atlas.texture_data[g_off + 1], 255);
    }

    #[test]
    fn test_stats_after_packing() {
        let mut atlas = TextureAtlas::new("test", 256, 256);
        let pixels = red_pixel_data(64, 64);
        atlas.add_pixels("a", &pixels, 64, 64);
        atlas.add_pixels("b", &pixels, 64, 64);

        let s = atlas.stats();
        assert_eq!(s.texture_count, 2);
        assert_eq!(s.used_pixels, 64 * 64 * 2);
        assert_eq!(s.total_pixels, 256 * 256);
    }

    #[test]
    fn test_version_increments() {
        let mut atlas = TextureAtlas::new("test", 128, 128);
        assert_eq!(atlas.version(), 0);
        atlas.add_pixels("a", &red_pixel_data(8, 8), 8, 8);
        assert_eq!(atlas.version(), 1);
        atlas.add_pixels("b", &red_pixel_data(8, 8), 8, 8);
        assert_eq!(atlas.version(), 2);
    }

    #[test]
    fn test_default_dimensions() {
        let atlas = TextureAtlas::new("test", 0, 0);
        assert_eq!(
            atlas.dimensions(),
            (DEFAULT_MAX_ATLAS_SIZE, DEFAULT_MAX_ATLAS_SIZE)
        );
    }
}
