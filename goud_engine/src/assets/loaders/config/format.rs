//! [`ConfigFormat`] -- configuration file format enumeration.

use std::fmt;

/// Configuration file format.
///
/// Represents the serialization format of a configuration asset.
/// Used to determine how to parse the raw bytes.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ConfigFormat {
    /// JSON format (default).
    #[default]
    Json = 0,
    /// TOML format (requires `native` feature).
    Toml = 1,
}

impl ConfigFormat {
    /// Returns the file extension associated with this format.
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Toml => "toml",
        }
    }

    /// Returns the human-readable format name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Toml => "TOML",
        }
    }

    /// Detects the format from a file extension.
    ///
    /// Returns `None` if the extension is not a recognized config format.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }
}

impl fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
