//! Text direction configuration for layout and shaping.

/// Text flow direction used by the shaping/layout pipeline.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextDirection {
    /// Use Unicode bidirectional analysis to determine run direction.
    #[default]
    Auto,
    /// Force left-to-right shaping/layout.
    LeftToRight,
    /// Force right-to-left shaping/layout.
    RightToLeft,
}

impl TextDirection {
    /// Parses a compact integer representation used by FFI and wasm boundaries.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Auto),
            1 => Some(Self::LeftToRight),
            2 => Some(Self::RightToLeft),
            _ => None,
        }
    }
}
