use super::theme::{UiComponentVisual, UiStyleOverrides, UiTheme, UiVisualStyle};

/// Interaction state used to resolve theme visuals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiInteractionState {
    /// Default resting state.
    Normal,
    /// Pointer hover state.
    Hovered,
    /// Pressed/active state.
    Pressed,
    /// Disabled state.
    Disabled,
}

/// Resolves the final visual style for a widget type/state combination.
pub fn resolve_widget_visual(
    widget: UiComponentVisual,
    state: UiInteractionState,
    theme: &UiTheme,
    overrides: Option<&UiStyleOverrides>,
) -> UiVisualStyle {
    let state_set = match widget {
        UiComponentVisual::Panel => &theme.widgets.panel,
        UiComponentVisual::Button => &theme.widgets.button,
        UiComponentVisual::Label => &theme.widgets.label,
        UiComponentVisual::Image => &theme.widgets.image,
        UiComponentVisual::Slider => &theme.widgets.slider,
    };

    let mut base = match state {
        UiInteractionState::Normal => state_set.normal,
        UiInteractionState::Hovered => state_set.hovered,
        UiInteractionState::Pressed => state_set.pressed,
        UiInteractionState::Disabled => state_set.disabled,
    };

    if let Some(overrides) = overrides {
        if let Some(color) = overrides.background_color {
            base.background = color;
        }
        if let Some(color) = overrides.foreground_color {
            base.text = color;
        }
        if let Some(color) = overrides.border_color {
            base.border = color;
        }
    }

    base
}
