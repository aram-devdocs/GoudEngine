//! [`FontAsset`] definition and implementation.

use std::fmt;

use crate::assets::{Asset, AssetType};

use super::format::FontFormat;

/// Font style classification.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum FontStyle {
    /// Regular (upright, normal weight).
    #[default]
    Regular,
    /// Bold weight.
    Bold,
    /// Italic (oblique) style.
    Italic,
    /// Bold weight with italic style.
    BoldItalic,
}

impl fmt::Display for FontStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FontStyle::Regular => "Regular",
                FontStyle::Bold => "Bold",
                FontStyle::Italic => "Italic",
                FontStyle::BoldItalic => "Bold Italic",
            }
        )
    }
}

/// Font asset containing parsed font data.
///
/// Stores raw font bytes along with extracted metadata. The raw bytes can be
/// re-parsed with `fontdue` when rasterization is needed.
///
/// # Example
/// ```
/// use goud_engine::assets::loaders::FontAsset;
///
/// let font = FontAsset::new(
///     vec![0u8; 64],
///     "Test".to_string(),
///     Default::default(),
///     Default::default(),
///     1000,
///     95,
///     0,
/// );
/// assert_eq!(font.family_name(), "Test");
/// assert_eq!(font.glyph_count(), 95);
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct FontAsset {
    /// Raw font file bytes (TTF/OTF).
    data: Vec<u8>,
    /// Font family name extracted from the font metadata.
    family_name: String,
    /// Font style (regular, bold, italic, bold-italic).
    style: FontStyle,
    /// Original font file format.
    format: FontFormat,
    /// Design units per em square.
    units_per_em: u16,
    /// Number of glyphs in the font.
    glyph_count: u16,
    /// Index within a font collection (TTC/OTC). 0 for standalone fonts.
    collection_index: u32,
}

impl FontAsset {
    /// Creates a new font asset with the given parameters.
    ///
    /// # Arguments
    /// * `data` - Raw font file bytes
    /// * `family_name` - Font family name
    /// * `style` - Font style
    /// * `format` - Original file format
    /// * `units_per_em` - Design units per em square
    /// * `glyph_count` - Number of glyphs in the font
    /// * `collection_index` - Index within a font collection (0 for standalone fonts)
    pub fn new(
        data: Vec<u8>,
        family_name: String,
        style: FontStyle,
        format: FontFormat,
        units_per_em: u16,
        glyph_count: u16,
        collection_index: u32,
    ) -> Self {
        Self {
            data,
            family_name,
            style,
            format,
            units_per_em,
            glyph_count,
            collection_index,
        }
    }

    /// Returns the raw font file bytes.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Returns the font family name.
    pub fn family_name(&self) -> &str {
        &self.family_name
    }

    /// Returns the font style.
    ///
    /// Note: currently always returns `FontStyle::Regular` as fontdue does not expose style metadata.
    pub fn style(&self) -> FontStyle {
        self.style
    }

    /// Returns the original font format.
    pub fn format(&self) -> FontFormat {
        self.format
    }

    /// Returns the size of the font data in bytes.
    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the font data is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the number of glyphs in the font.
    pub fn glyph_count(&self) -> u16 {
        self.glyph_count
    }

    /// Returns the design units per em square.
    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    /// Returns the collection index (0 for standalone fonts, >0 for TTC/OTC collections).
    pub fn collection_index(&self) -> u32 {
        self.collection_index
    }

    /// Re-parses the stored bytes into a `fontdue::Font`.
    ///
    /// Uses the same `collection_index` from the original load to ensure the
    /// correct face is selected from TTC/OTC font collections.
    ///
    /// # Errors
    /// Returns an error string if the font bytes cannot be parsed.
    pub fn parse(&self) -> Result<fontdue::Font, String> {
        let settings = fontdue::FontSettings {
            collection_index: self.collection_index,
            ..fontdue::FontSettings::default()
        };
        fontdue::Font::from_bytes(self.data.as_slice(), settings).map_err(|e| e.to_string())
    }
}

impl Asset for FontAsset {
    fn asset_type_name() -> &'static str {
        "Font"
    }

    fn asset_type() -> AssetType {
        AssetType::Font
    }

    fn extensions() -> &'static [&'static str] {
        &["ttf", "otf"]
    }
}
