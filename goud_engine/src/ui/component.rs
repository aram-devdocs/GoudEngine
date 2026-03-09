//! UI component types.
//!
//! Each variant represents a different kind of UI widget that can be
//! attached to a [`UiNode`](super::UiNode).

// =============================================================================
// UiComponent
// =============================================================================

/// The type of UI widget attached to a node.
///
/// `Panel` is a non-interactive container; `Button` is interactive/focusable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiComponent {
    /// A rectangular container that can hold child nodes.
    Panel,
    /// A clickable, focusable button.
    Button(UiButton),
}

/// Button settings and state that are intrinsic to the button component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiButton {
    /// Whether this button can be interacted with.
    pub enabled: bool,
}

impl UiButton {
    /// Creates an enabled button.
    #[inline]
    pub const fn new() -> Self {
        Self { enabled: true }
    }
}

impl Default for UiButton {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl UiComponent {
    /// Returns `true` if this component is a button.
    #[inline]
    pub const fn is_button(self) -> bool {
        matches!(self, Self::Button(_))
    }

    /// Returns `true` if this component is interactive for pointer/keyboard input.
    #[inline]
    pub const fn is_interactive(self) -> bool {
        match self {
            Self::Panel => false,
            Self::Button(button) => button.enabled,
        }
    }

    /// Returns `true` if this component participates in keyboard focus traversal.
    #[inline]
    pub const fn is_focusable(self) -> bool {
        match self {
            Self::Panel => false,
            Self::Button(button) => button.enabled,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_debug() {
        let comp = UiComponent::Panel;
        assert_eq!(format!("{:?}", comp), "Panel");
    }

    #[test]
    fn test_clone_and_button_helpers() {
        let comp = UiComponent::Panel;
        let cloned = comp.clone();
        assert!(matches!(cloned, UiComponent::Panel));

        let button = UiComponent::Button(UiButton::default());
        assert!(button.is_button());
        assert!(button.is_interactive());
        assert!(button.is_focusable());
    }
}
