//! Glyph atlas generation and dynamic expansion.
//!
//! The atlas starts with printable ASCII by default, then can grow to include
//! new glyph indices discovered at runtime (Unicode/CJK/RTL shaping paths).

use std::collections::{BTreeSet, HashMap};

use crate::libs::graphics::backend::render_backend::RenderBackend;
use crate::libs::graphics::backend::types::{
    TextureFilter, TextureFormat, TextureHandle, TextureWrap,
};

use super::rasterizer::{rasterize_glyph_indices, GlyphMetrics, RasterizedGlyph};

/// UV rectangle describing a glyph's position within the atlas texture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvRect {
    /// Left edge in UV space (0.0..1.0).
    pub u_min: f32,
    /// Top edge in UV space (0.0..1.0).
    pub v_min: f32,
    /// Right edge in UV space (0.0..1.0).
    pub u_max: f32,
    /// Bottom edge in UV space (0.0..1.0).
    pub v_max: f32,
}

/// Glyph information stored in the atlas: UV coordinates and metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphInfo {
    /// UV rectangle for this glyph in the atlas texture.
    pub uv_rect: UvRect,
    /// Rasterization metrics for this glyph.
    pub metrics: GlyphMetrics,
}

/// A packed glyph atlas containing an RGBA8 texture and per-glyph metadata.
#[derive(Debug, Clone)]
pub struct GlyphAtlas {
    /// RGBA8 pixel data (4 bytes per pixel).
    texture_data: Vec<u8>,
    /// Atlas texture width in pixels.
    width: u32,
    /// Atlas texture height in pixels.
    height: u32,
    /// Font size used to rasterize this atlas.
    size_px: f32,
    /// Per-character glyph info (compat path for bitmap-like lookup).
    glyphs_by_char: HashMap<char, GlyphInfo>,
    /// Per-glyph-index info used by shaped text layout.
    glyphs_by_index: HashMap<u16, GlyphInfo>,
    /// Character to glyph-index lookup cache.
    char_to_index: HashMap<char, u16>,
    /// Cached rasterized bitmaps for all known glyph indices.
    rasterized_glyphs: HashMap<u16, RasterizedGlyph>,
    /// Cached GPU texture handle, lazily uploaded via `ensure_gpu_texture`.
    gpu_texture: Option<TextureHandle>,
    /// True when CPU atlas data changed and GPU texture must be synced.
    dirty: bool,
    /// Incremented whenever packed atlas pixels are rebuilt.
    version: u64,
}

/// The range of printable ASCII characters (space through tilde).
const PRINTABLE_ASCII_START: u8 = 32;
const PRINTABLE_ASCII_END: u8 = 126;

/// 1-pixel padding between glyphs to avoid texture bleeding.
const GLYPH_PADDING: u32 = 1;

/// Starting atlas dimension.
const INITIAL_ATLAS_SIZE: u32 = 256;

/// Maximum atlas dimension to prevent runaway allocation.
const MAX_ATLAS_SIZE: u32 = 4096;

impl GlyphAtlas {
    /// Generates an atlas for printable ASCII (32..=126) at the given size.
    pub fn generate(font: &fontdue::Font, size_px: f32) -> Result<Self, String> {
        let chars: Vec<char> = (PRINTABLE_ASCII_START..=PRINTABLE_ASCII_END)
            .map(|b| b as char)
            .collect();
        Self::generate_for_chars(font, size_px, &chars)
    }

    /// Generates an atlas containing only the provided characters.
    pub fn generate_for_chars(
        font: &fontdue::Font,
        size_px: f32,
        chars: &[char],
    ) -> Result<Self, String> {
        let mut char_to_index = HashMap::new();
        for &ch in chars {
            char_to_index.insert(ch, font.lookup_glyph_index(ch));
        }

        let unique_indices: Vec<u16> = char_to_index
            .values()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let rasterized_glyphs: HashMap<u16, RasterizedGlyph> =
            rasterize_glyph_indices(font, size_px, &unique_indices)
                .into_iter()
                .collect();

        let mut atlas_size = INITIAL_ATLAS_SIZE;
        let (texture_data, glyphs_by_index, final_size) = loop {
            if atlas_size > MAX_ATLAS_SIZE {
                return Err(format!(
                    "Glyphs do not fit in maximum atlas size ({0}x{0})",
                    MAX_ATLAS_SIZE
                ));
            }

            if let Some((pixels, glyphs)) =
                Self::try_pack(&rasterized_glyphs, atlas_size, atlas_size)
            {
                break (pixels, glyphs, atlas_size);
            }
            atlas_size *= 2;
        };

        let mut atlas = Self {
            texture_data,
            width: final_size,
            height: final_size,
            size_px,
            glyphs_by_char: HashMap::new(),
            glyphs_by_index,
            char_to_index,
            rasterized_glyphs,
            gpu_texture: None,
            dirty: true,
            version: 1,
        };
        atlas.rebuild_char_cache();
        Ok(atlas)
    }

    /// Ensures all provided characters are present in the atlas.
    ///
    /// Returns `true` if new character mappings or glyphs were added.
    pub fn ensure_chars<I>(&mut self, font: &fontdue::Font, chars: I) -> Result<bool, String>
    where
        I: IntoIterator<Item = char>,
    {
        let mut changed = false;
        let mut new_indices = BTreeSet::new();

        for ch in chars {
            if self.char_to_index.contains_key(&ch) {
                continue;
            }
            let index = font.lookup_glyph_index(ch);
            self.char_to_index.insert(ch, index);
            changed = true;
            if !self.rasterized_glyphs.contains_key(&index) {
                new_indices.insert(index);
            }
        }

        if !new_indices.is_empty() {
            self.add_glyph_indices(font, new_indices.into_iter().collect())?;
        } else if changed {
            self.rebuild_char_cache();
        }

        Ok(changed)
    }

    /// Ensures all provided glyph indices are present in the atlas.
    ///
    /// Returns `true` when the atlas pixel data was rebuilt.
    pub fn ensure_glyph_indices<I>(
        &mut self,
        font: &fontdue::Font,
        glyph_indices: I,
    ) -> Result<bool, String>
    where
        I: IntoIterator<Item = u16>,
    {
        let new_indices: Vec<u16> = glyph_indices
            .into_iter()
            .filter(|idx| !self.rasterized_glyphs.contains_key(idx))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        if new_indices.is_empty() {
            return Ok(false);
        }

        self.add_glyph_indices(font, new_indices)?;
        Ok(true)
    }

    /// Returns glyph info (UV + metrics) for the given character, if present.
    pub fn glyph_info(&self, ch: char) -> Option<&GlyphInfo> {
        self.glyphs_by_char.get(&ch)
    }

    /// Returns glyph info (UV + metrics) for the given glyph index, if present.
    pub fn glyph_info_indexed(&self, glyph_id: u16) -> Option<&GlyphInfo> {
        self.glyphs_by_index.get(&glyph_id)
    }

    /// Returns the raw RGBA8 texture data.
    pub fn texture_data(&self) -> &[u8] {
        &self.texture_data
    }

    /// Returns the atlas texture width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the atlas texture height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the atlas content version.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Returns true when CPU-side texture data changed since last GPU sync.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the atlas as clean after an external GPU sync path (WASM).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Returns the cached GPU texture handle, if one has been uploaded.
    pub fn gpu_texture(&self) -> Option<TextureHandle> {
        self.gpu_texture
    }

    /// Lazily uploads (or updates) the atlas pixel data on GPU.
    ///
    /// If the atlas grew, the old texture is destroyed and recreated.
    pub fn ensure_gpu_texture(
        &mut self,
        backend: &mut dyn RenderBackend,
    ) -> Result<TextureHandle, String> {
        if let Some(handle) = self.gpu_texture {
            if !self.dirty {
                return Ok(handle);
            }

            if backend.texture_size(handle) == Some((self.width, self.height)) {
                backend
                    .update_texture(handle, 0, 0, self.width, self.height, &self.texture_data)
                    .map_err(|e| format!("failed to update glyph atlas texture: {e}"))?;
                self.dirty = false;
                return Ok(handle);
            }

            backend.destroy_texture(handle);
            self.gpu_texture = None;
        }

        let handle = backend
            .create_texture(
                self.width,
                self.height,
                TextureFormat::RGBA8Linear,
                TextureFilter::Linear,
                TextureWrap::ClampToEdge,
                &self.texture_data,
            )
            .map_err(|e| format!("failed to create GPU texture for glyph atlas: {e}"))?;

        self.gpu_texture = Some(handle);
        self.dirty = false;
        Ok(handle)
    }

    /// Takes the GPU texture handle out of this atlas, if present.
    ///
    /// After this call `gpu_texture()` returns `None`. The caller is
    /// responsible for destroying the returned handle via the backend.
    pub(crate) fn take_gpu_texture(&mut self) -> Option<TextureHandle> {
        self.gpu_texture.take()
    }

    fn add_glyph_indices(
        &mut self,
        font: &fontdue::Font,
        new_indices: Vec<u16>,
    ) -> Result<(), String> {
        let newly_rasterized = rasterize_glyph_indices(font, self.size_px, &new_indices);
        for (index, glyph) in newly_rasterized {
            self.rasterized_glyphs.insert(index, glyph);
        }
        self.repack_all()
    }

    fn repack_all(&mut self) -> Result<(), String> {
        let mut atlas_size = self.width.max(INITIAL_ATLAS_SIZE);
        let (texture_data, glyphs_by_index, final_size) = loop {
            if atlas_size > MAX_ATLAS_SIZE {
                return Err(format!(
                    "Glyphs do not fit in maximum atlas size ({0}x{0})",
                    MAX_ATLAS_SIZE
                ));
            }

            if let Some((pixels, glyphs)) =
                Self::try_pack(&self.rasterized_glyphs, atlas_size, atlas_size)
            {
                break (pixels, glyphs, atlas_size);
            }
            atlas_size *= 2;
        };

        self.texture_data = texture_data;
        self.width = final_size;
        self.height = final_size;
        self.glyphs_by_index = glyphs_by_index;
        self.rebuild_char_cache();
        self.dirty = true;
        self.version = self.version.saturating_add(1);
        Ok(())
    }

    fn rebuild_char_cache(&mut self) {
        self.glyphs_by_char.clear();
        for (ch, idx) in &self.char_to_index {
            if let Some(info) = self.glyphs_by_index.get(idx) {
                self.glyphs_by_char.insert(*ch, *info);
            }
        }
    }

    /// Attempts row-based bin packing of rasterized glyphs into an atlas of
    /// the given dimensions. Returns `None` if the glyphs don't fit.
    fn try_pack(
        rasterized: &HashMap<u16, RasterizedGlyph>,
        atlas_w: u32,
        atlas_h: u32,
    ) -> Option<(Vec<u8>, HashMap<u16, GlyphInfo>)> {
        let mut texture_data = vec![0u8; (atlas_w * atlas_h * 4) as usize];
        let mut glyphs = HashMap::new();

        let mut cursor_x: u32 = GLYPH_PADDING;
        let mut cursor_y: u32 = GLYPH_PADDING;
        let mut row_height: u32 = 0;

        let mut glyph_indices: Vec<u16> = rasterized.keys().copied().collect();
        glyph_indices.sort_unstable();

        for glyph_id in glyph_indices {
            let glyph = rasterized.get(&glyph_id)?;

            // Zero-size glyphs (e.g., spaces) get a degenerate UV rect.
            if glyph.width == 0 || glyph.height == 0 {
                glyphs.insert(
                    glyph_id,
                    GlyphInfo {
                        uv_rect: UvRect {
                            u_min: 0.0,
                            v_min: 0.0,
                            u_max: 0.0,
                            v_max: 0.0,
                        },
                        metrics: glyph.metrics,
                    },
                );
                continue;
            }

            // Advance to next row if this glyph doesn't fit on the current one.
            if cursor_x + glyph.width + GLYPH_PADDING > atlas_w {
                cursor_x = GLYPH_PADDING;
                cursor_y += row_height + GLYPH_PADDING;
                row_height = 0;
            }

            // Check vertical overflow.
            if cursor_y + glyph.height + GLYPH_PADDING > atlas_h {
                return None;
            }

            // Blit glyph bitmap into atlas as RGBA8 (white + alpha).
            for gy in 0..glyph.height {
                for gx in 0..glyph.width {
                    let src_idx = (gy * glyph.width + gx) as usize;
                    let dst_x = cursor_x + gx;
                    let dst_y = cursor_y + gy;
                    let dst_idx = ((dst_y * atlas_w + dst_x) * 4) as usize;

                    let alpha = glyph.bitmap[src_idx];
                    texture_data[dst_idx] = 255; // R
                    texture_data[dst_idx + 1] = 255; // G
                    texture_data[dst_idx + 2] = 255; // B
                    texture_data[dst_idx + 3] = alpha; // A
                }
            }

            // Record UV coordinates.
            let u_min = cursor_x as f32 / atlas_w as f32;
            let v_min = cursor_y as f32 / atlas_h as f32;
            let u_max = (cursor_x + glyph.width) as f32 / atlas_w as f32;
            let v_max = (cursor_y + glyph.height) as f32 / atlas_h as f32;

            glyphs.insert(
                glyph_id,
                GlyphInfo {
                    uv_rect: UvRect {
                        u_min,
                        v_min,
                        u_max,
                        v_max,
                    },
                    metrics: glyph.metrics,
                },
            );

            cursor_x += glyph.width + GLYPH_PADDING;
            row_height = row_height.max(glyph.height);
        }

        Some((texture_data, glyphs))
    }
}

#[cfg(test)]
#[path = "glyph_atlas_tests.rs"]
mod tests;
