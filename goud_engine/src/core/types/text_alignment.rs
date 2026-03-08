//! Horizontal text alignment enum.
//!
//! Defined in `core::types` (Layer 1) so it can be used by all layers
//! without introducing upward dependency violations.

/// Horizontal text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TextAlignment {
    /// Align text to the left edge.
    #[default]
    Left,
    /// Center text horizontally.
    Center,
    /// Align text to the right edge.
    Right,
}
