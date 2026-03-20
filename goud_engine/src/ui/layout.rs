//! UI layout primitives shared by nodes and the UI manager.

/// Whether a node uses layout-relative or absolute screen-space positioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionMode {
    /// Node position is computed by the layout system (default).
    Relative,
    /// Node position is set explicitly via `set_position()`.
    Absolute,
}

impl Default for PositionMode {
    #[inline]
    fn default() -> Self {
        Self::Relative
    }
}

/// Node anchoring relative to the parent content rect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAnchor {
    /// Place node at parent content top-left plus margins.
    TopLeft,
    /// Center node in parent content, offset by margins.
    Center,
    /// Place node at parent content bottom-right minus margins.
    BottomRight,
    /// Stretch node to fill parent content minus margins.
    Stretch,
}

impl Default for UiAnchor {
    #[inline]
    fn default() -> Self {
        Self::TopLeft
    }
}

/// Edge insets used for margins and padding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiEdges {
    /// Left edge in pixels.
    pub left: f32,
    /// Right edge in pixels.
    pub right: f32,
    /// Top edge in pixels.
    pub top: f32,
    /// Bottom edge in pixels.
    pub bottom: f32,
}

impl UiEdges {
    /// Zero insets.
    pub const ZERO: Self = Self {
        left: 0.0,
        right: 0.0,
        top: 0.0,
        bottom: 0.0,
    };

    /// Creates edge insets with explicit values.
    #[inline]
    pub const fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    /// Creates edge insets with a single value for all sides.
    #[inline]
    pub const fn all(value: f32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    /// Sum of left + right.
    #[inline]
    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    /// Sum of top + bottom.
    #[inline]
    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

impl Default for UiEdges {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

/// Direction of a flex container's main axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiFlexDirection {
    /// Horizontal layout (X axis main).
    Row,
    /// Vertical layout (Y axis main).
    Column,
}

/// Main-axis group alignment for flex children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiJustify {
    /// Start of main axis.
    Start,
    /// Center of main axis.
    Center,
    /// End of main axis.
    End,
}

/// Cross-axis alignment for flex children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAlign {
    /// Start of cross axis.
    Start,
    /// Center of cross axis.
    Center,
    /// End of cross axis.
    End,
    /// Stretch to fill cross axis minus margins.
    Stretch,
}

/// Full flex configuration for a layout container.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiFlexLayout {
    /// Flex direction.
    pub direction: UiFlexDirection,
    /// Main-axis group alignment.
    pub justify: UiJustify,
    /// Cross-axis child alignment.
    pub align_items: UiAlign,
    /// Fixed gap between children.
    pub spacing: f32,
}

impl Default for UiFlexLayout {
    #[inline]
    fn default() -> Self {
        Self {
            direction: UiFlexDirection::Row,
            justify: UiJustify::Start,
            align_items: UiAlign::Start,
            spacing: 0.0,
        }
    }
}

/// Layout mode for a node's children.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiLayout {
    /// Children are laid out individually by anchor.
    None,
    /// Children are laid out by flex rules.
    Flex(UiFlexLayout),
}

impl Default for UiLayout {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_helpers_work() {
        let edges = UiEdges::new(1.0, 2.0, 3.0, 4.0);
        assert!((edges.horizontal() - 3.0).abs() < 0.001);
        assert!((edges.vertical() - 7.0).abs() < 0.001);

        assert_eq!(UiEdges::all(5.0), UiEdges::new(5.0, 5.0, 5.0, 5.0));
    }
}
