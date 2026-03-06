//! Rendering methods for WasmGame.
//!
//! Covers sprite drawing, texture management, and render statistics.

use wasm_bindgen::prelude::*;

use image::GenericImageView;

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
