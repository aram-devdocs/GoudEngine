use crate::core::math::Color;

/// Widget kinds used by theme visual resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiComponentVisual {
    /// Container panel visuals.
    Panel,
    /// Button visuals.
    Button,
    /// Label visuals.
    Label,
    /// Image visuals.
    Image,
    /// Slider visuals.
    Slider,
}

/// Visual style for a single widget state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiVisualStyle {
    /// Fill/background color.
    pub background: Color,
    /// Foreground color (text, tint, or fill accent).
    pub text: Color,
    /// Border/accent outline color.
    pub border: Color,
}

/// Four-state visuals for a widget.
#[derive(Debug, Clone, PartialEq)]
pub struct UiStateVisuals {
    /// Default resting state.
    pub normal: UiVisualStyle,
    /// Hover state.
    pub hovered: UiVisualStyle,
    /// Pressed/active state.
    pub pressed: UiVisualStyle,
    /// Disabled state.
    pub disabled: UiVisualStyle,
}

/// Palette tokens for UI theming.
#[derive(Debug, Clone, PartialEq)]
pub struct UiPalette {
    /// App-level background.
    pub background: Color,
    /// Primary surface.
    pub surface: Color,
    /// Secondary surface.
    pub surface_alt: Color,
    /// Primary text color.
    pub text: Color,
    /// Muted text/accent color.
    pub muted: Color,
    /// Highlight/accent color.
    pub accent: Color,
}

/// Font defaults used by UI text emission.
#[derive(Debug, Clone, PartialEq)]
pub struct UiTypography {
    /// Default font family name.
    pub default_font_family: String,
    /// Default font size in pixels.
    pub default_font_size: f32,
}

/// Spacing tokens used by widgets.
#[derive(Debug, Clone, PartialEq)]
pub struct UiSpacingTokens {
    /// Small spacing token.
    pub small: f32,
    /// Medium spacing token.
    pub medium: f32,
    /// Large spacing token.
    pub large: f32,
    /// Inner control spacing.
    pub control_inner: f32,
}

/// Widget style tokens grouped by widget kind.
#[derive(Debug, Clone, PartialEq)]
pub struct UiWidgetTheme {
    /// Panel visuals.
    pub panel: UiStateVisuals,
    /// Button visuals.
    pub button: UiStateVisuals,
    /// Label visuals.
    pub label: UiStateVisuals,
    /// Image visuals.
    pub image: UiStateVisuals,
    /// Slider visuals.
    pub slider: UiStateVisuals,
}

/// Complete UI theme.
#[derive(Debug, Clone, PartialEq)]
pub struct UiTheme {
    /// Palette tokens.
    pub palette: UiPalette,
    /// Typography defaults.
    pub typography: UiTypography,
    /// Spacing defaults.
    pub spacing: UiSpacingTokens,
    /// Widget visuals.
    pub widgets: UiWidgetTheme,
}

/// Per-node visual overrides.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiStyleOverrides {
    /// Override background fill color.
    pub background_color: Option<Color>,
    /// Override foreground/tint/fill color.
    pub foreground_color: Option<Color>,
    /// Override text color.
    pub text_color: Option<Color>,
    /// Override border color.
    pub border_color: Option<Color>,
    /// Override font family for label rendering.
    pub font_family: Option<String>,
    /// Override font size for label rendering.
    pub font_size: Option<f32>,
    /// Override texture source for image rendering.
    pub texture_path: Option<String>,
    /// Override widget internal spacing token.
    pub widget_spacing: Option<f32>,
}

impl UiTheme {
    /// Built-in light theme preset.
    pub fn light() -> Self {
        let palette = UiPalette {
            background: Color::from_hex(0xF3F5F7),
            surface: Color::from_hex(0xFFFFFF),
            surface_alt: Color::from_hex(0xE7ECF2),
            text: Color::from_hex(0x1F2937),
            muted: Color::from_hex(0x667085),
            accent: Color::from_hex(0x2563EB),
        };

        let typography = UiTypography {
            default_font_family: "F05".to_string(),
            default_font_size: 16.0,
        };

        let spacing = UiSpacingTokens {
            small: 4.0,
            medium: 8.0,
            large: 16.0,
            control_inner: 6.0,
        };

        let widgets = UiWidgetTheme {
            panel: UiStateVisuals {
                normal: UiVisualStyle {
                    background: palette.surface,
                    text: palette.text,
                    border: palette.surface_alt,
                },
                hovered: UiVisualStyle {
                    background: palette.surface_alt,
                    text: palette.text,
                    border: palette.muted,
                },
                pressed: UiVisualStyle {
                    background: palette.surface_alt,
                    text: palette.text,
                    border: palette.accent,
                },
                disabled: UiVisualStyle {
                    background: palette.surface_alt,
                    text: palette.muted,
                    border: palette.muted,
                },
            },
            button: UiStateVisuals {
                normal: UiVisualStyle {
                    background: palette.accent,
                    text: Color::WHITE,
                    border: palette.accent,
                },
                hovered: UiVisualStyle {
                    background: palette.accent.lerp(Color::WHITE, 0.15),
                    text: Color::WHITE,
                    border: palette.accent,
                },
                pressed: UiVisualStyle {
                    background: palette.accent.lerp(Color::BLACK, 0.2),
                    text: Color::WHITE,
                    border: palette.accent,
                },
                disabled: UiVisualStyle {
                    background: palette.surface_alt,
                    text: palette.muted,
                    border: palette.muted,
                },
            },
            label: UiStateVisuals {
                normal: UiVisualStyle {
                    background: Color::TRANSPARENT,
                    text: palette.text,
                    border: Color::TRANSPARENT,
                },
                hovered: UiVisualStyle {
                    background: Color::TRANSPARENT,
                    text: palette.text,
                    border: Color::TRANSPARENT,
                },
                pressed: UiVisualStyle {
                    background: Color::TRANSPARENT,
                    text: palette.text,
                    border: Color::TRANSPARENT,
                },
                disabled: UiVisualStyle {
                    background: Color::TRANSPARENT,
                    text: palette.muted,
                    border: Color::TRANSPARENT,
                },
            },
            image: UiStateVisuals {
                normal: UiVisualStyle {
                    background: Color::TRANSPARENT,
                    text: Color::WHITE,
                    border: Color::TRANSPARENT,
                },
                hovered: UiVisualStyle {
                    background: Color::TRANSPARENT,
                    text: Color::WHITE,
                    border: palette.muted,
                },
                pressed: UiVisualStyle {
                    background: Color::TRANSPARENT,
                    text: Color::WHITE,
                    border: palette.accent,
                },
                disabled: UiVisualStyle {
                    background: Color::TRANSPARENT,
                    text: palette.muted,
                    border: Color::TRANSPARENT,
                },
            },
            slider: UiStateVisuals {
                normal: UiVisualStyle {
                    background: palette.surface_alt,
                    text: palette.accent,
                    border: palette.muted,
                },
                hovered: UiVisualStyle {
                    background: palette.surface_alt,
                    text: palette.accent.lerp(Color::WHITE, 0.15),
                    border: palette.accent,
                },
                pressed: UiVisualStyle {
                    background: palette.surface_alt,
                    text: palette.accent.lerp(Color::BLACK, 0.2),
                    border: palette.accent,
                },
                disabled: UiVisualStyle {
                    background: palette.surface_alt,
                    text: palette.muted,
                    border: palette.muted,
                },
            },
        };

        Self {
            palette,
            typography,
            spacing,
            widgets,
        }
    }

    /// Built-in dark theme preset.
    pub fn dark() -> Self {
        let palette = UiPalette {
            background: Color::from_hex(0x0D1117),
            surface: Color::from_hex(0x161B22),
            surface_alt: Color::from_hex(0x21262D),
            text: Color::from_hex(0xE6EDF3),
            muted: Color::from_hex(0x8B949E),
            accent: Color::from_hex(0x2F81F7),
        };

        let mut theme = Self::light();
        theme.palette = palette.clone();
        theme.widgets.panel.normal.background = palette.surface;
        theme.widgets.panel.normal.text = palette.text;
        theme.widgets.panel.normal.border = palette.surface_alt;
        theme.widgets.panel.hovered.background = palette.surface_alt;
        theme.widgets.panel.hovered.text = palette.text;
        theme.widgets.panel.hovered.border = palette.muted;
        theme.widgets.panel.pressed.background = palette.surface_alt;
        theme.widgets.panel.pressed.text = palette.text;
        theme.widgets.panel.pressed.border = palette.accent;
        theme.widgets.panel.disabled.background = palette.surface_alt;
        theme.widgets.panel.disabled.text = palette.muted;
        theme.widgets.panel.disabled.border = palette.muted;

        theme.widgets.button.normal.background = palette.accent;
        theme.widgets.button.normal.text = Color::WHITE;
        theme.widgets.button.normal.border = palette.accent;
        theme.widgets.button.hovered.background = palette.accent.lerp(Color::WHITE, 0.15);
        theme.widgets.button.hovered.text = Color::WHITE;
        theme.widgets.button.hovered.border = palette.accent;
        theme.widgets.button.pressed.background = palette.accent.lerp(Color::BLACK, 0.2);
        theme.widgets.button.pressed.text = Color::WHITE;
        theme.widgets.button.pressed.border = palette.accent;
        theme.widgets.button.disabled.background = palette.surface_alt;
        theme.widgets.button.disabled.text = palette.muted;
        theme.widgets.button.disabled.border = palette.muted;

        theme.widgets.label.normal.text = palette.text;
        theme.widgets.label.hovered.text = palette.text;
        theme.widgets.label.pressed.text = palette.text;
        theme.widgets.label.disabled.text = palette.muted;

        theme.widgets.image.hovered.border = palette.muted;
        theme.widgets.image.pressed.border = palette.accent;
        theme.widgets.image.disabled.text = palette.muted;

        theme.widgets.slider.normal.background = palette.surface_alt;
        theme.widgets.slider.normal.text = palette.accent;
        theme.widgets.slider.normal.border = palette.muted;
        theme.widgets.slider.hovered.background = palette.surface_alt;
        theme.widgets.slider.hovered.text = palette.accent.lerp(Color::WHITE, 0.15);
        theme.widgets.slider.hovered.border = palette.accent;
        theme.widgets.slider.pressed.background = palette.surface_alt;
        theme.widgets.slider.pressed.text = palette.accent.lerp(Color::BLACK, 0.2);
        theme.widgets.slider.pressed.border = palette.accent;
        theme.widgets.slider.disabled.background = palette.surface_alt;
        theme.widgets.slider.disabled.text = palette.muted;
        theme.widgets.slider.disabled.border = palette.muted;

        theme
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::light()
    }
}
