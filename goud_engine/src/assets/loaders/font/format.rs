//! [`FontFormat`] enum and related conversions.

use std::fmt;

/// Font file format.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum FontFormat {
    /// TrueType Font.
    Ttf = 0,
    /// OpenType Font.
    Otf = 1,
    /// Unknown format.
    #[default]
    Unknown = 255,
}

impl FontFormat {
    /// Returns the file extension for this format.
    pub fn extension(self) -> &'static str {
        match self {
            FontFormat::Ttf => "ttf",
            FontFormat::Otf => "otf",
            FontFormat::Unknown => "",
        }
    }

    /// Returns the human-readable name of this format.
    pub fn name(self) -> &'static str {
        match self {
            FontFormat::Ttf => "TrueType",
            FontFormat::Otf => "OpenType",
            FontFormat::Unknown => "Unknown",
        }
    }

    /// Returns the format corresponding to the given file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "ttf" => FontFormat::Ttf,
            "otf" => FontFormat::Otf,
            _ => FontFormat::Unknown,
        }
    }
}

impl fmt::Display for FontFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
