//! [`FontLoader`] implementing the [`AssetLoader`] trait for font files.

use crate::assets::{asset::Asset, AssetLoadError, AssetLoader, LoadContext};

use super::{asset::FontAsset, format::FontFormat, settings::FontSettings};

/// Font asset loader for TTF and OTF files.
///
/// Uses `fontdue` to parse font files and extract metadata. The raw bytes
/// are stored in the resulting [`FontAsset`] for later rasterization.
///
/// # Supported Formats
/// - TrueType (.ttf)
/// - OpenType (.otf)
///
/// # Example
/// ```no_run
/// use goud_engine::assets::{AssetServer, loaders::font::{FontLoader, FontAsset}};
///
/// let mut server = AssetServer::new();
/// server.register_loader(FontLoader::default());
/// ```
#[derive(Clone, Debug, Default)]
pub struct FontLoader;

impl AssetLoader for FontLoader {
    type Asset = FontAsset;
    type Settings = FontSettings;

    fn extensions(&self) -> &[&str] {
        FontAsset::extensions()
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        // Determine format from file extension
        let format = context
            .extension()
            .map(FontFormat::from_extension)
            .unwrap_or(FontFormat::Unknown);

        // Parse with fontdue to validate and extract metadata
        let font_settings = fontdue::FontSettings {
            collection_index: settings.collection_index,
            ..fontdue::FontSettings::default()
        };

        let font = fontdue::Font::from_bytes(bytes, font_settings)
            .map_err(|e| AssetLoadError::decode_failed(format!("Font parsing failed: {e}")))?;

        // Extract metadata from the parsed font
        let family_name = font.name().unwrap_or("Unknown").to_string();
        let units_per_em = font.units_per_em();
        let glyph_count = font.glyph_count();

        Ok(FontAsset::new(
            bytes.to_vec(),
            family_name,
            Default::default(), // FontStyle detection not available in fontdue
            format,
            units_per_em.round() as u16, // fontdue returns f32; standard values (1000, 2048) fit safely in u16
            glyph_count,
            settings.collection_index,
        ))
    }
}
