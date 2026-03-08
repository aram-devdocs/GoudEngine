//! Compressed texture asset and loader for DDS files.
//!
//! Provides [`CompressedTextureAsset`] for GPU-compressed texture data (BC1/BC3/BC5/BC7)
//! and [`CompressedTextureLoader`] which parses DDS containers.

use crate::assets::{Asset, AssetLoadError, AssetLoader, AssetType, LoadContext};

use super::dds::{self, CompressedFormat};

/// A loaded compressed texture asset containing block-compressed data.
///
/// Unlike [`TextureAsset`](super::TextureAsset) which stores decoded RGBA8 pixels,
/// this struct holds data in a GPU-native compressed format (BC1/BC3/BC5/BC7).
/// The data is uploaded to the GPU without CPU-side decompression.
///
/// # Fields
///
/// - `data`: Raw block-compressed data
/// - `width`: Image width in pixels
/// - `height`: Image height in pixels
/// - `format`: The block compression format (BC1, BC3, BC5, or BC7)
/// - `mip_levels`: Number of mipmap levels stored in the data
#[derive(Debug, Clone)]
pub struct CompressedTextureAsset {
    /// Raw block-compressed texture data.
    pub data: Vec<u8>,

    /// Width of the texture in pixels.
    pub width: u32,

    /// Height of the texture in pixels.
    pub height: u32,

    /// Block compression format.
    pub format: CompressedFormat,

    /// Number of mipmap levels (1 means base level only).
    pub mip_levels: u32,
}

impl CompressedTextureAsset {
    /// Returns the block size in bytes for this texture's format.
    #[inline]
    pub fn block_size(&self) -> usize {
        dds::block_size(self.format)
    }

    /// Returns the number of 4x4 blocks along the width.
    #[inline]
    pub fn blocks_wide(&self) -> u32 {
        self.width.div_ceil(4)
    }

    /// Returns the number of 4x4 blocks along the height.
    #[inline]
    pub fn blocks_high(&self) -> u32 {
        self.height.div_ceil(4)
    }

    /// Returns the total size of the compressed data in bytes.
    #[inline]
    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }
}

impl Asset for CompressedTextureAsset {
    fn asset_type_name() -> &'static str {
        "CompressedTexture"
    }

    fn asset_type() -> AssetType {
        AssetType::Texture
    }

    fn extensions() -> &'static [&'static str] {
        &["dds"]
    }
}

/// Asset loader for DDS compressed texture files.
///
/// Parses DDS containers and produces [`CompressedTextureAsset`] values
/// containing block-compressed data ready for GPU upload.
///
/// # Supported Formats
///
/// - BC1 (DXT1): 4:1 compression, 1-bit alpha
/// - BC3 (DXT5): 4:1 compression, full alpha
/// - BC5 (via DX10 header): Two-channel compression for normal maps
/// - BC7 (via DX10 header): High-quality RGBA compression
#[derive(Debug, Clone, Default)]
pub struct CompressedTextureLoader;

impl CompressedTextureLoader {
    /// Creates a new compressed texture loader.
    pub fn new() -> Self {
        Self
    }
}

impl AssetLoader for CompressedTextureLoader {
    type Asset = CompressedTextureAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        CompressedTextureAsset::extensions()
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        _context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        let result = dds::parse_dds(bytes)?;

        let data = bytes[result.data_offset..result.data_offset + result.data_len].to_vec();

        Ok(CompressedTextureAsset {
            data,
            width: result.width,
            height: result.height,
            format: result.format,
            mip_levels: result.mip_levels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AssetPath;

    #[test]
    fn test_compressed_texture_asset_trait() {
        assert_eq!(
            CompressedTextureAsset::asset_type_name(),
            "CompressedTexture"
        );
        assert_eq!(CompressedTextureAsset::asset_type(), AssetType::Texture);
        assert!(CompressedTextureAsset::extensions().contains(&"dds"));
    }

    #[test]
    fn test_compressed_texture_block_dimensions() {
        let asset = CompressedTextureAsset {
            data: vec![0; 16],
            width: 7,
            height: 5,
            format: CompressedFormat::BC3,
            mip_levels: 1,
        };

        assert_eq!(asset.blocks_wide(), 2); // ceil(7/4) = 2
        assert_eq!(asset.blocks_high(), 2); // ceil(5/4) = 2
        assert_eq!(asset.block_size(), 16);
        assert_eq!(asset.size_bytes(), 16);
    }

    #[test]
    fn test_compressed_texture_loader_extensions() {
        let loader = CompressedTextureLoader::new();
        assert_eq!(loader.extensions(), &["dds"]);
        assert!(loader.supports_extension("dds"));
        assert!(loader.supports_extension("DDS"));
        assert!(!loader.supports_extension("png"));
    }

    #[test]
    fn test_compressed_texture_loader_load() {
        let loader = CompressedTextureLoader::new();

        // Build a minimal BC1 DDS file
        let mut dds_bytes = build_test_dds_bc1(4, 4, &[0xAA; 8]);

        let path = AssetPath::from_string("test.dds".to_string());
        let mut context = LoadContext::new(path);

        let asset = loader
            .load(&dds_bytes, &(), &mut context)
            .expect("should load BC1 DDS");

        assert_eq!(asset.width, 4);
        assert_eq!(asset.height, 4);
        assert_eq!(asset.format, CompressedFormat::BC1);
        assert_eq!(asset.mip_levels, 1);
        assert_eq!(asset.data, vec![0xAA; 8]);

        // Corrupt the magic to test error path
        dds_bytes[0] = 0xFF;
        let err = loader.load(&dds_bytes, &(), &mut context).unwrap_err();
        assert!(err.is_decode_failed());
    }

    /// Helper to build a minimal BC1 DDS file for testing.
    fn build_test_dds_bc1(width: u32, height: u32, data: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();

        // Magic
        buf.extend_from_slice(&0x2053_4444u32.to_le_bytes());
        // Header size
        buf.extend_from_slice(&124u32.to_le_bytes());
        // Flags
        buf.extend_from_slice(&0u32.to_le_bytes());
        // Height
        buf.extend_from_slice(&height.to_le_bytes());
        // Width
        buf.extend_from_slice(&width.to_le_bytes());
        // Pitch
        buf.extend_from_slice(&0u32.to_le_bytes());
        // Depth
        buf.extend_from_slice(&0u32.to_le_bytes());
        // Mip count
        buf.extend_from_slice(&1u32.to_le_bytes());
        // Reserved[11]
        for _ in 0..11 {
            buf.extend_from_slice(&0u32.to_le_bytes());
        }
        // Pixel format: size, flags (FOURCC), fourCC (DXT1), then zeros
        buf.extend_from_slice(&32u32.to_le_bytes());
        buf.extend_from_slice(&0x4u32.to_le_bytes()); // DDPF_FOURCC
        buf.extend_from_slice(&u32::from_le_bytes(*b"DXT1").to_le_bytes());
        for _ in 0..5 {
            buf.extend_from_slice(&0u32.to_le_bytes());
        }
        // Caps
        for _ in 0..5 {
            buf.extend_from_slice(&0u32.to_le_bytes());
        }

        assert_eq!(buf.len(), 128);
        buf.extend_from_slice(data);
        buf
    }
}
