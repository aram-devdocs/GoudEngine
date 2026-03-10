use crate::core::math::{Color, Rect};

use super::node_id::UiNodeId;

/// Solid quad draw request emitted by UI.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiQuadCommand {
    /// Source UI node.
    pub node_id: UiNodeId,
    /// Destination rectangle.
    pub rect: Rect,
    /// Fill color.
    pub color: Color,
}

/// Textured quad draw request emitted by UI.
#[derive(Debug, Clone, PartialEq)]
pub struct UiTexturedQuadCommand {
    /// Source UI node.
    pub node_id: UiNodeId,
    /// Destination rectangle.
    pub rect: Rect,
    /// Texture source path.
    pub texture_path: String,
    /// Tint color.
    pub tint: Color,
}

/// Text draw request emitted by UI.
#[derive(Debug, Clone, PartialEq)]
pub struct UiTextCommand {
    /// Source UI node.
    pub node_id: UiNodeId,
    /// Text content to render.
    pub text: String,
    /// Top-left text position in viewport space.
    pub position: [f32; 2],
    /// Font size in pixels.
    pub font_size: f32,
    /// Text color.
    pub color: Color,
    /// Font family selection (pipeline-level lookup).
    pub font_family: String,
}

/// High-level render command stream emitted by UI.
#[derive(Debug, Clone)]
pub enum UiRenderCommand {
    /// Solid rectangle fill.
    Quad(UiQuadCommand),
    /// Textured rectangle.
    TexturedQuad(UiTexturedQuadCommand),
    /// Text draw request.
    Text(UiTextCommand),
}
