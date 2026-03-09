//! Rendering methods for WasmGame.
//!
//! Covers sprite drawing, texture management, and render statistics.

use wasm_bindgen::prelude::*;

use image::GenericImageView;

use crate::core::types::TextAlignment;
use crate::rendering::text::{
    layout_shaped_text, shape_text, GlyphAtlas, TextDirection, TextLayoutConfig,
};

use super::sprite_renderer::create_texture_entry;
use super::{WasmGame, WasmRenderStats};

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

#[wasm_bindgen]
impl WasmGame {
    pub fn draw_sprite(
        &mut self,
        texture: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rotation: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        if let Some(rs) = &mut self.render_state {
            rs.renderer
                .draw_sprite(texture, x, y, w, h, rotation, r, g, b, a);
        }
    }

    pub fn draw_sprite_rect(
        &mut self,
        texture: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rotation: f32,
        src_x: f32,
        src_y: f32,
        src_w: f32,
        src_h: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> bool {
        if let Some(rs) = &mut self.render_state {
            rs.renderer.draw_sprite_rect(
                texture, x, y, w, h, rotation, src_x, src_y, src_w, src_h, r, g, b, a,
            )
        } else {
            false
        }
    }

    pub fn draw_quad(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, g: f32, b: f32, a: f32) {
        if let Some(rs) = &mut self.render_state {
            rs.renderer.draw_quad(x, y, w, h, r, g, b, a);
        }
    }

    // ======================================================================
    // Texture management
    // ======================================================================

    /// Registers a texture from raw image bytes (PNG, JPG, etc.).
    ///
    /// The browser-side SDK fetches the image data asynchronously, then
    /// passes the raw bytes here for synchronous decode + GPU upload.
    /// Returns the texture handle (1-based; 0 is reserved for the white
    /// fallback texture used by `draw_quad`).
    pub fn register_texture_from_bytes(&mut self, data: &[u8]) -> Result<u32, JsValue> {
        let rs = self
            .render_state
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Rendering not initialized"))?;

        let img = image::load_from_memory(data)
            .map_err(|e| JsValue::from_str(&format!("Image decode error: {e}")))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let entry = create_texture_entry(
            &rs.device,
            &rs.queue,
            &rs.renderer.texture_bind_group_layout,
            &rs.renderer.sampler,
            width,
            height,
            &rgba,
        );

        let idx = rs.textures.len();
        rs.textures.push(Some(entry));
        Ok((idx + 1) as u32)
    }

    pub fn destroy_texture(&mut self, handle: u32) {
        if handle == 0 {
            return;
        }
        if let Some(rs) = &mut self.render_state {
            let idx = (handle - 1) as usize;
            if idx < rs.textures.len() {
                rs.textures[idx] = None;
            }
        }
    }

    /// Registers a font from raw bytes.
    ///
    /// Returns a 1-based handle suitable for `draw_text`.
    pub fn register_font_from_bytes(&mut self, data: &[u8]) -> Result<u32, JsValue> {
        let bytes = data.to_vec();
        let font = fontdue::Font::from_bytes(bytes.as_slice(), fontdue::FontSettings::default())
            .map_err(|e| JsValue::from_str(&format!("Font decode error: {e}")))?;

        let entry = super::WasmFontEntry {
            font,
            bytes,
            atlases: std::collections::HashMap::new(),
        };

        if let Some((idx, slot)) = self
            .fonts
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
        {
            *slot = Some(entry);
            return Ok((idx + 1) as u32);
        }

        self.fonts.push(Some(entry));
        Ok(self.fonts.len() as u32)
    }

    /// Destroys a loaded font and any backing atlas textures.
    pub fn destroy_font(&mut self, handle: u32) -> bool {
        if handle == 0 {
            return false;
        }

        let idx = (handle - 1) as usize;
        let Some(slot) = self.fonts.get_mut(idx) else {
            return false;
        };
        let Some(mut font_entry) = slot.take() else {
            return false;
        };

        if let Some(rs) = &mut self.render_state {
            for atlas in font_entry.atlases.values_mut() {
                if let Some(texture_handle) = atlas.texture_handle.take() {
                    let texture_idx = (texture_handle - 1) as usize;
                    if texture_idx < rs.textures.len() {
                        rs.textures[texture_idx] = None;
                    }
                }
            }
        }

        true
    }

    /// Draws shaped text with alignment, wrapping, spacing, direction, and tint.
    ///
    /// `alignment`: 0 = Left, 1 = Center, 2 = Right  
    /// `direction`: 0 = Auto, 1 = LTR, 2 = RTL
    #[allow(clippy::too_many_arguments)]
    pub fn draw_text(
        &mut self,
        font_handle: u32,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        alignment: u8,
        max_width: f32,
        line_spacing: f32,
        direction: u8,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> bool {
        if font_handle == 0 || font_size <= 0.0 || line_spacing <= 0.0 {
            return false;
        }

        let Some(alignment) = parse_alignment(alignment) else {
            return false;
        };
        let Some(direction) = parse_direction(direction) else {
            return false;
        };

        if text.is_empty() {
            return true;
        }

        let (fonts, render_state) = (&mut self.fonts, &mut self.render_state);
        let Some(rs) = render_state.as_mut() else {
            return false;
        };

        let font_index = (font_handle - 1) as usize;
        let Some(font_entry) = fonts.get_mut(font_index).and_then(|slot| slot.as_mut()) else {
            return false;
        };

        let shaped = match shape_text(text, &font_entry.bytes, font_size, direction) {
            Ok(shaped) => shaped,
            Err(_) => return false,
        };

        let size_key = font_size.round().max(1.0) as u32;
        if !font_entry.atlases.contains_key(&size_key) {
            let atlas = match GlyphAtlas::generate(&font_entry.font, font_size) {
                Ok(atlas) => atlas,
                Err(_) => return false,
            };
            font_entry.atlases.insert(
                size_key,
                super::WasmFontAtlas {
                    atlas,
                    texture_handle: None,
                    synced_version: 0,
                },
            );
        }

        let atlas_state = font_entry
            .atlases
            .get_mut(&size_key)
            .expect("atlas inserted above");

        if atlas_state
            .atlas
            .ensure_glyph_indices(&font_entry.font, shaped.glyph_indices())
            .is_err()
        {
            return false;
        }

        let config = TextLayoutConfig {
            max_width: if max_width > 0.0 {
                Some(max_width)
            } else {
                None
            },
            line_spacing,
            alignment,
        };
        let layout = layout_shaped_text(&shaped, &atlas_state.atlas, font_size, &config);
        if layout.glyphs.is_empty() {
            return true;
        }

        let atlas_version = atlas_state.atlas.version();
        if atlas_state.texture_handle.is_none()
            || atlas_state.synced_version != atlas_version
            || atlas_state.atlas.is_dirty()
        {
            let entry = create_texture_entry(
                &rs.device,
                &rs.queue,
                &rs.renderer.texture_bind_group_layout,
                &rs.renderer.sampler,
                atlas_state.atlas.width(),
                atlas_state.atlas.height(),
                atlas_state.atlas.texture_data(),
            );

            match atlas_state.texture_handle {
                Some(handle) => {
                    let texture_idx = (handle - 1) as usize;
                    if texture_idx < rs.textures.len() {
                        rs.textures[texture_idx] = Some(entry);
                    } else {
                        rs.textures.push(Some(entry));
                        atlas_state.texture_handle = Some(rs.textures.len() as u32);
                    }
                }
                None => {
                    rs.textures.push(Some(entry));
                    atlas_state.texture_handle = Some(rs.textures.len() as u32);
                }
            }

            atlas_state.synced_version = atlas_version;
            atlas_state.atlas.mark_clean();
        }

        let Some(texture_handle) = atlas_state.texture_handle else {
            return false;
        };

        for glyph in &layout.glyphs {
            if glyph.size_x <= 0.0 || glyph.size_y <= 0.0 {
                continue;
            }

            let uv_x = glyph.uv_rect.u_min;
            let uv_y = glyph.uv_rect.v_min;
            let uv_w = glyph.uv_rect.u_max - glyph.uv_rect.u_min;
            let uv_h = glyph.uv_rect.v_max - glyph.uv_rect.v_min;
            let center_x = x + glyph.x + glyph.size_x * 0.5;
            let center_y = y + glyph.y + glyph.size_y * 0.5;

            if !rs.renderer.draw_sprite_rect(
                texture_handle,
                center_x,
                center_y,
                glyph.size_x,
                glyph.size_y,
                0.0,
                uv_x,
                uv_y,
                uv_w,
                uv_h,
                r,
                g,
                b,
                a,
            ) {
                return false;
            }
        }

        true
    }

    // ======================================================================
    // Render statistics
    // ======================================================================

    /// Returns render statistics for the current frame, or `None` if
    /// rendering is not active.
    pub fn get_render_stats(&self) -> Option<WasmRenderStats> {
        self.render_state.as_ref().map(|rs| {
            let stats = rs.renderer.render_stats();
            WasmRenderStats {
                draw_calls: stats.draw_calls,
                triangles: stats.triangles,
                texture_binds: stats.texture_binds,
            }
        })
    }
}

fn parse_alignment(alignment: u8) -> Option<TextAlignment> {
    match alignment {
        0 => Some(TextAlignment::Left),
        1 => Some(TextAlignment::Center),
        2 => Some(TextAlignment::Right),
        _ => None,
    }
}

fn parse_direction(direction: u8) -> Option<TextDirection> {
    TextDirection::from_u8(direction)
}
