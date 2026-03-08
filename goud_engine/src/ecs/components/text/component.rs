//! Core [`Text`] component type and its builder methods.

use crate::assets::{loaders::FontAsset, AssetHandle};
use crate::core::math::Color;
use crate::ecs::Component;
use crate::rendering::text::layout::TextAlignment;

// =============================================================================
// Text Component
// =============================================================================

/// A text component for rendering text strings.
///
/// The `Text` component defines how a text string should be rendered.
/// It must be paired with a [`Transform2D`](crate::ecs::components::Transform2D)
/// to define the text's position, rotation, and scale.
///
/// # Fields
///
/// - `content`: The text string to render
/// - `font_handle`: Handle to the font asset
/// - `font_path`: Optional path for serialization
/// - `font_size`: Size of the font in pixels (default: 16.0)
/// - `color`: Color of the text (default: white)
/// - `alignment`: Horizontal text alignment (default: Left)
/// - `max_width`: Optional max width for word-wrapping
/// - `line_spacing`: Line spacing multiplier (default: 1.0)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Text {
    /// The text string to render.
    pub content: String,

    /// Handle to the font asset.
    #[serde(skip)]
    pub font_handle: AssetHandle<FontAsset>,

    /// Optional path to the font asset for serialization.
    #[serde(default)]
    pub font_path: Option<String>,

    /// Font size in pixels.
    pub font_size: f32,

    /// Text color.
    pub color: Color,

    /// Horizontal text alignment.
    pub alignment: TextAlignment,

    /// Optional maximum width for word-wrapping.
    pub max_width: Option<f32>,

    /// Line spacing multiplier (1.0 = default spacing).
    pub line_spacing: f32,
}

impl Text {
    /// Creates a new text component with default settings.
    ///
    /// The text will render with:
    /// - Font size 16.0
    /// - White color
    /// - Left alignment
    /// - No max width (no word-wrapping)
    /// - Line spacing 1.0
    #[inline]
    pub fn new(font_handle: AssetHandle<FontAsset>, content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font_handle,
            font_path: None,
            font_size: 16.0,
            color: Color::WHITE,
            alignment: TextAlignment::Left,
            max_width: None,
            line_spacing: 1.0,
        }
    }

    /// Sets the font size in pixels.
    #[inline]
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Sets the text color.
    #[inline]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the horizontal text alignment.
    #[inline]
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets the maximum width for word-wrapping.
    #[inline]
    pub fn with_max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// Sets the line spacing multiplier.
    #[inline]
    pub fn with_line_spacing(mut self, line_spacing: f32) -> Self {
        self.line_spacing = line_spacing;
        self
    }

    /// Sets the font asset path for serialization.
    #[inline]
    pub fn with_font_path(mut self, path: impl Into<String>) -> Self {
        self.font_path = Some(path.into());
        self
    }
}

// Implement Component trait so Text can be used in the ECS
impl Component for Text {}

// =============================================================================
// Default Implementation
// =============================================================================

impl Default for Text {
    /// Creates a text component with an invalid font handle.
    ///
    /// Primarily useful for deserialization or when the font handle
    /// will be set later. The text will not render correctly until a
    /// valid font handle is assigned.
    fn default() -> Self {
        Self {
            content: String::new(),
            font_handle: AssetHandle::INVALID,
            font_path: None,
            font_size: 16.0,
            color: Color::WHITE,
            alignment: TextAlignment::Left,
            max_width: None,
            line_spacing: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_new_sets_content_and_handle() {
        let handle = AssetHandle::new(1, 1);
        let text = Text::new(handle, "Hello");

        assert_eq!(text.content, "Hello");
        assert_eq!(text.font_handle, handle);
        assert_eq!(text.font_size, 16.0);
        assert_eq!(text.color, Color::WHITE);
        assert_eq!(text.alignment, TextAlignment::Left);
        assert!(text.max_width.is_none());
        assert_eq!(text.line_spacing, 1.0);
    }

    #[test]
    fn test_text_builder_methods() {
        let handle = AssetHandle::new(1, 1);
        let text = Text::new(handle, "Test")
            .with_font_size(24.0)
            .with_color(Color::RED)
            .with_alignment(TextAlignment::Center)
            .with_max_width(200.0)
            .with_line_spacing(1.5)
            .with_font_path("fonts/test.ttf");

        assert_eq!(text.font_size, 24.0);
        assert_eq!(text.color, Color::RED);
        assert_eq!(text.alignment, TextAlignment::Center);
        assert_eq!(text.max_width, Some(200.0));
        assert_eq!(text.line_spacing, 1.5);
        assert_eq!(text.font_path.as_deref(), Some("fonts/test.ttf"));
    }

    #[test]
    fn test_text_default_has_invalid_handle() {
        let text = Text::default();

        assert!(!text.font_handle.is_valid());
        assert!(text.content.is_empty());
        assert_eq!(text.font_size, 16.0);
        assert_eq!(text.color, Color::WHITE);
        assert_eq!(text.alignment, TextAlignment::Left);
        assert!(text.max_width.is_none());
        assert_eq!(text.line_spacing, 1.0);
    }

    #[test]
    fn test_text_is_component() {
        fn assert_component<T: Component>() {}
        assert_component::<Text>();
    }

    #[test]
    fn test_text_clone() {
        let handle = AssetHandle::new(1, 1);
        let text = Text::new(handle, "Clone me").with_font_size(32.0);
        let cloned = text.clone();

        assert_eq!(cloned.content, "Clone me");
        assert_eq!(cloned.font_size, 32.0);
    }
}
