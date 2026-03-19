//! [`ScriptAsset`] -- loaded script source code ready for execution.

use crate::assets::{Asset, AssetType};

/// A loaded script asset containing source code.
///
/// The `source` field holds the full text of the script, and `file_name`
/// records the original file name for diagnostic messages.
#[derive(Debug, Clone)]
pub struct ScriptAsset {
    /// The script source code.
    pub source: String,
    /// The original file name (used for error messages).
    pub file_name: String,
}

impl Asset for ScriptAsset {
    fn asset_type_name() -> &'static str {
        "Script"
    }

    fn asset_type() -> AssetType {
        AssetType::Text
    }

    fn extensions() -> &'static [&'static str] {
        &["lua"]
    }
}
