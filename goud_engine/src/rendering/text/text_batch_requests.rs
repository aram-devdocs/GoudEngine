//! Immediate text draw request APIs for text batch rendering.

use crate::assets::loaders::FontAsset;
use crate::assets::AssetServer;
use crate::core::math::{Color, Vec2};
use crate::core::types::TextAlignment;
use crate::ecs::components::Transform2D;

use super::layout::TextLayoutConfig;
use super::text_batch::TextBatch;

/// Immediate text draw request for systems that do not use ECS text components.
#[derive(Debug, Clone)]
pub struct DirectTextDrawRequest {
    /// Text content to render.
    pub content: String,
    /// World-space text origin.
    pub position: Vec2,
    /// Font handle used for glyph atlas resolution.
    pub font_handle: crate::assets::AssetHandle<FontAsset>,
    /// Font size in pixels.
    pub font_size: f32,
    /// Glyph color.
    pub color: Color,
    /// Horizontal alignment.
    pub alignment: TextAlignment,
    /// Optional wrapping width.
    pub max_width: Option<f32>,
    /// Line spacing multiplier.
    pub line_spacing: f32,
}

impl TextBatch {
    /// Draws immediate text requests without requiring ECS text components.
    pub fn draw_text_requests(
        &mut self,
        requests: &[DirectTextDrawRequest],
        asset_server: &AssetServer,
        backend: &mut dyn crate::libs::graphics::backend::render_backend::RenderBackend,
    ) -> Result<(), String> {
        for request in requests {
            if request.content.is_empty() {
                continue;
            }

            let config = TextLayoutConfig {
                max_width: request.max_width,
                line_spacing: request.line_spacing,
                alignment: request.alignment,
            };

            let Some((layout, gpu_texture)) = self.resolve_truetype_font(
                &request.content,
                request.font_size,
                &config,
                &request.font_handle,
                asset_server,
                backend,
            )?
            else {
                continue;
            };

            if layout.glyphs.is_empty() {
                continue;
            }

            let transform = Transform2D::from_position(request.position);
            self.append_glyph_batch(&layout, request.color, &transform, gpu_texture);
        }

        Ok(())
    }
}
