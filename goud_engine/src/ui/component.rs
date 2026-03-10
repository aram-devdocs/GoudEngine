//! UI component types.
//!
//! Each variant represents a different kind of UI widget that can be
//! attached to a [`UiNode`](super::UiNode).

use super::theme::UiComponentVisual;

/// The type of UI widget attached to a node.
#[derive(Debug, Clone, PartialEq)]
pub enum UiComponent {
    /// A rectangular container that can hold child nodes.
    Panel,
    /// A clickable, focusable button.
    Button(UiButton),
    /// A text label.
    Label(UiLabel),
    /// A textured image.
    Image(UiImage),
    /// A value slider.
    Slider(UiSlider),
}

/// Button settings and state intrinsic to button components.
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

/// Label component data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLabel {
    /// Label text content.
    pub text: String,
}

impl UiLabel {
    /// Creates a new label with text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl Default for UiLabel {
    #[inline]
    fn default() -> Self {
        Self::new("")
    }
}

/// Image component data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiImage {
    /// Texture source path.
    pub texture_path: String,
}

impl UiImage {
    /// Creates a new image widget.
    pub fn new(texture_path: impl Into<String>) -> Self {
        Self {
            texture_path: texture_path.into(),
        }
    }
}

impl Default for UiImage {
    #[inline]
    fn default() -> Self {
        Self::new("")
    }
}

/// Slider component data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiSlider {
    /// Whether this slider can be interacted with.
    pub enabled: bool,
    /// Minimum value.
    pub min: f32,
    /// Maximum value.
    pub max: f32,
    /// Current value.
    pub value: f32,
}

impl UiSlider {
    /// Creates a new slider, normalizing bounds and clamping value.
    pub fn new(min: f32, max: f32, value: f32) -> Self {
        let (min, max) = normalized_range(min, max);
        Self {
            enabled: true,
            min,
            max,
            value: value.clamp(min, max),
        }
    }

    /// Sets a new range, normalizing bounds and clamping the current value.
    pub fn set_range(&mut self, min: f32, max: f32) {
        let (min, max) = normalized_range(min, max);
        self.min = min;
        self.max = max;
        self.value = self.value.clamp(min, max);
    }

    /// Sets the current value, clamped to the slider range.
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }

    /// Returns normalized value in [0, 1].
    pub fn normalized_value(&self) -> f32 {
        let span = (self.max - self.min).abs();
        if span <= f32::EPSILON {
            0.0
        } else {
            ((self.value - self.min) / span).clamp(0.0, 1.0)
        }
    }
}

fn normalized_range(a: f32, b: f32) -> (f32, f32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

impl UiComponent {
    /// Returns `true` if this component is a button.
    #[inline]
    pub fn is_button(&self) -> bool {
        matches!(self, Self::Button(_))
    }

    /// Returns `true` if this component is interactive for pointer/keyboard input.
    #[inline]
    pub fn is_interactive(&self) -> bool {
        match self {
            Self::Panel | Self::Label(_) | Self::Image(_) => false,
            Self::Button(button) => button.enabled,
            Self::Slider(slider) => slider.enabled,
        }
    }

    /// Returns `true` if this component participates in keyboard focus traversal.
    #[inline]
    pub fn is_focusable(&self) -> bool {
        match self {
            Self::Panel | Self::Label(_) | Self::Image(_) => false,
            Self::Button(button) => button.enabled,
            Self::Slider(slider) => slider.enabled,
        }
    }

    /// Returns whether this component has an explicit enabled/disabled state.
    #[inline]
    pub fn has_enabled_state(&self) -> bool {
        matches!(self, Self::Button(_) | Self::Slider(_))
    }

    /// Returns whether this component is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        match self {
            Self::Panel | Self::Label(_) | Self::Image(_) => true,
            Self::Button(button) => button.enabled,
            Self::Slider(slider) => slider.enabled,
        }
    }

    /// Returns the theme visual category for this component.
    #[inline]
    pub fn visual_kind(&self) -> UiComponentVisual {
        match self {
            Self::Panel => UiComponentVisual::Panel,
            Self::Button(_) => UiComponentVisual::Button,
            Self::Label(_) => UiComponentVisual::Label,
            Self::Image(_) => UiComponentVisual::Image,
            Self::Slider(_) => UiComponentVisual::Slider,
        }
    }
}

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

    #[test]
    fn test_slider_normalizes_reversed_range_and_clamps_initial_value() {
        let slider = UiSlider::new(10.0, -10.0, 42.0);

        assert_eq!(slider.min, -10.0);
        assert_eq!(slider.max, 10.0);
        assert_eq!(slider.value, 10.0);
    }

    #[test]
    fn test_slider_set_range_preserves_normalized_bounds_and_clamps_value() {
        let mut slider = UiSlider::new(0.0, 1.0, 0.5);

        slider.set_range(8.0, 2.0);

        assert_eq!(slider.min, 2.0);
        assert_eq!(slider.max, 8.0);
        assert_eq!(slider.value, 2.0);
    }

    #[test]
    fn test_slider_set_value_clamps_to_range() {
        let mut slider = UiSlider::new(0.0, 10.0, 5.0);

        slider.set_value(-5.0);
        assert_eq!(slider.value, 0.0);

        slider.set_value(99.0);
        assert_eq!(slider.value, 10.0);
    }

    #[test]
    fn test_label_default_and_property_mutation() {
        let mut label = UiLabel::default();
        assert_eq!(label.text, "");

        label.text = "Score".to_owned();
        assert_eq!(label.text, "Score");
    }

    #[test]
    fn test_image_default_and_property_mutation() {
        let mut image = UiImage::default();
        assert_eq!(image.texture_path, "");

        image.texture_path = "assets/ui/icon.png".to_owned();
        assert_eq!(image.texture_path, "assets/ui/icon.png");
    }
}
