//! UI component types.
//!
//! Each variant represents a different kind of UI widget that can be
//! attached to a [`UiNode`](super::UiNode).

// =============================================================================
// UiComponent
// =============================================================================

/// The type of UI widget attached to a node.
///
/// Start with `Panel` as the foundational container. Additional variants
/// (Button, Label, Image, etc.) will be added as the UI system grows.
#[derive(Debug, Clone)]
pub enum UiComponent {
    /// A rectangular container that can hold child nodes.
    Panel,
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
    fn test_clone() {
        let comp = UiComponent::Panel;
        let cloned = comp.clone();
        assert!(matches!(cloned, UiComponent::Panel));
    }
}
