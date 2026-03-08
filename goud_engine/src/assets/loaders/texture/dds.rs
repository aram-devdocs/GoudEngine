//! DDS container parsing for compressed texture formats.
//!
//! Parses DDS (DirectDraw Surface) file headers to extract compressed texture
//! data in BC1, BC3, BC5, and BC7 formats. This is pure byte manipulation
//! with no external crate dependencies.

use crate::assets::AssetLoadError;

/// DDS file magic number: "DDS " as a little-endian u32.
const DDS_MAGIC: u32 = 0x2053_4444;

/// Expected size of the DDS header structure (without magic).
const DDS_HEADER_SIZE: u32 = 124;

/// Expected size of the DDS pixel format structure.
const DDS_PIXEL_FORMAT_SIZE: u32 = 32;

/// Pixel format flag indicating a FourCC code is present.
const DDPF_FOURCC: u32 = 0x4;

/// FourCC code for DXT1 (BC1) compression.
const FOURCC_DXT1: u32 = u32::from_le_bytes(*b"DXT1");

/// FourCC code for DXT5 (BC3) compression.
const FOURCC_DXT5: u32 = u32::from_le_bytes(*b"DXT5");

/// FourCC code for DX10 extended header.
const FOURCC_DX10: u32 = u32::from_le_bytes(*b"DX10");

/// DXGI format value for BC5_UNORM.
const DXGI_FORMAT_BC5_UNORM: u32 = 83;

/// DXGI format value for BC7_UNORM.
const DXGI_FORMAT_BC7_UNORM: u32 = 98;

/// Supported block-compressed texture formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressedFormat {
    /// BC1 (DXT1): 4:1 compression, 1-bit alpha. 8 bytes per 4x4 block.
    BC1,
    /// BC3 (DXT5): 4:1 compression, full alpha. 16 bytes per 4x4 block.
    BC3,
    /// BC5 (ATI2/3Dc): Two-channel compression (normals). 16 bytes per 4x4 block.
    BC5,
    /// BC7: High-quality RGBA compression. 16 bytes per 4x4 block.
    BC7,
}

/// Returns the byte size of a single 4x4 block for the given compressed format.
pub const fn block_size(format: CompressedFormat) -> usize {
    match format {
        CompressedFormat::BC1 => 8,
        CompressedFormat::BC3 | CompressedFormat::BC5 | CompressedFormat::BC7 => 16,
    }
}

/// Parsed DDS pixel format from the header.
#[derive(Debug, Clone, Copy)]
struct DdsPixelFormat {
    /// Flags indicating which fields are valid.
    flags: u32,
    /// FourCC code identifying the format.
    four_cc: u32,
}

/// Parsed DX10 extended header (present when FourCC is "DX10").
#[derive(Debug, Clone, Copy)]
struct Dx10Header {
    /// DXGI_FORMAT enum value.
    dxgi_format: u32,
}

/// Result of parsing a DDS file header.
#[derive(Debug, Clone)]
pub struct DdsParseResult {
    /// Texture width in pixels.
    pub width: u32,
    /// Texture height in pixels.
    pub height: u32,
    /// Detected compressed format.
    pub format: CompressedFormat,
    /// Number of mipmap levels (0 or 1 means no mipmaps).
    pub mip_levels: u32,
    /// Byte offset where compressed data begins.
    pub data_offset: usize,
    /// Length of the compressed data in bytes.
    pub data_len: usize,
}

/// Reads a little-endian u32 from a byte slice at the given offset.
fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, AssetLoadError> {
    let end = offset + 4;
    if end > bytes.len() {
        return Err(AssetLoadError::decode_failed(format!(
            "DDS: unexpected end of data at offset {offset}"
        )));
    }
    Ok(u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]))
}

/// Parses a DDS file from raw bytes and returns header information.
///
/// Validates the DDS magic number, header structure, and detects the
/// compressed format from FourCC codes or the DX10 extended header.
///
/// # Errors
///
/// Returns `AssetLoadError::DecodeFailed` if the data is not a valid DDS file,
/// the header is malformed, or the compressed format is not supported.
pub fn parse_dds(bytes: &[u8]) -> Result<DdsParseResult, AssetLoadError> {
    // Minimum: magic(4) + header(124) = 128 bytes
    if bytes.len() < 128 {
        return Err(AssetLoadError::decode_failed(
            "DDS: file too small for DDS header",
        ));
    }

    // Validate magic number
    let magic = read_u32(bytes, 0)?;
    if magic != DDS_MAGIC {
        return Err(AssetLoadError::decode_failed(format!(
            "DDS: invalid magic number 0x{magic:08X}, expected 0x{DDS_MAGIC:08X}"
        )));
    }

    // Validate header size
    let header_size = read_u32(bytes, 4)?;
    if header_size != DDS_HEADER_SIZE {
        return Err(AssetLoadError::decode_failed(format!(
            "DDS: invalid header size {header_size}, expected {DDS_HEADER_SIZE}"
        )));
    }

    // Read dimensions
    let height = read_u32(bytes, 12)?;
    let width = read_u32(bytes, 16)?;

    if width == 0 || height == 0 {
        return Err(AssetLoadError::decode_failed(
            "DDS: width and height must be non-zero",
        ));
    }

    // Read mipmap count (offset 28 in header, +4 for magic = 32 in file)
    let mip_map_count = read_u32(bytes, 28)?;
    let mip_levels = if mip_map_count == 0 { 1 } else { mip_map_count };

    // Parse pixel format (starts at offset 76 in file: magic(4) + header fields before pf)
    // Header layout: size(4) + flags(4) + height(4) + width(4) + pitch(4) + depth(4)
    //   + mipcount(4) + reserved[11](44) = 72 bytes into header, +4 for magic = 76
    let pf_offset = 76;
    let pf = parse_pixel_format(bytes, pf_offset)?;

    // Determine format and data offset
    let (format, data_offset) = detect_format(bytes, &pf)?;

    // Calculate data size
    let data_len = bytes
        .len()
        .checked_sub(data_offset)
        .ok_or_else(|| AssetLoadError::decode_failed("DDS: data offset exceeds file size"))?;

    if data_len == 0 {
        return Err(AssetLoadError::decode_failed(
            "DDS: no compressed data after header",
        ));
    }

    Ok(DdsParseResult {
        width,
        height,
        format,
        mip_levels,
        data_offset,
        data_len,
    })
}

/// Parses the pixel format structure from the DDS header.
fn parse_pixel_format(bytes: &[u8], offset: usize) -> Result<DdsPixelFormat, AssetLoadError> {
    let pf_size = read_u32(bytes, offset)?;
    if pf_size != DDS_PIXEL_FORMAT_SIZE {
        return Err(AssetLoadError::decode_failed(format!(
            "DDS: invalid pixel format size {pf_size}, expected {DDS_PIXEL_FORMAT_SIZE}"
        )));
    }

    let flags = read_u32(bytes, offset + 4)?;
    let four_cc = read_u32(bytes, offset + 8)?;

    Ok(DdsPixelFormat { flags, four_cc })
}

/// Detects the compressed format from the pixel format and optional DX10 header.
///
/// Returns the format and the byte offset where compressed data starts.
fn detect_format(
    bytes: &[u8],
    pf: &DdsPixelFormat,
) -> Result<(CompressedFormat, usize), AssetLoadError> {
    if pf.flags & DDPF_FOURCC == 0 {
        return Err(AssetLoadError::decode_failed(
            "DDS: pixel format does not contain a FourCC code; \
             only compressed (BC1/BC3/BC5/BC7) formats are supported",
        ));
    }

    // Standard header ends at magic(4) + header(124) = 128
    let base_data_offset = 128;

    match pf.four_cc {
        FOURCC_DXT1 => Ok((CompressedFormat::BC1, base_data_offset)),
        FOURCC_DXT5 => Ok((CompressedFormat::BC3, base_data_offset)),
        FOURCC_DX10 => {
            // DX10 extended header is 20 bytes after the standard header
            let dx10_offset = base_data_offset;
            let dx10_end = dx10_offset + 20;
            if bytes.len() < dx10_end {
                return Err(AssetLoadError::decode_failed(
                    "DDS: file too small for DX10 extended header",
                ));
            }

            let dx10 = Dx10Header {
                dxgi_format: read_u32(bytes, dx10_offset)?,
            };

            let data_offset = dx10_end;
            match dx10.dxgi_format {
                DXGI_FORMAT_BC5_UNORM => Ok((CompressedFormat::BC5, data_offset)),
                DXGI_FORMAT_BC7_UNORM => Ok((CompressedFormat::BC7, data_offset)),
                other => Err(AssetLoadError::decode_failed(format!(
                    "DDS: unsupported DXGI format {other}; \
                     only BC5_UNORM ({DXGI_FORMAT_BC5_UNORM}) and \
                     BC7_UNORM ({DXGI_FORMAT_BC7_UNORM}) are supported via DX10 header"
                ))),
            }
        }
        other => {
            let cc_bytes = other.to_le_bytes();
            let cc_str: String = cc_bytes
                .iter()
                .map(|&b| {
                    if b.is_ascii_graphic() || b == b' ' {
                        b as char
                    } else {
                        '?'
                    }
                })
                .collect();
            Err(AssetLoadError::decode_failed(format!(
                "DDS: unsupported FourCC '{cc_str}' (0x{other:08X}); \
                 supported: DXT1 (BC1), DXT5 (BC3), DX10 (BC5/BC7)"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal valid DDS file with the given parameters.
    fn build_dds(
        width: u32,
        height: u32,
        mip_count: u32,
        pf_flags: u32,
        four_cc: u32,
        dx10_dxgi_format: Option<u32>,
        data: &[u8],
    ) -> Vec<u8> {
        let mut buf = Vec::new();

        // Magic
        buf.extend_from_slice(&DDS_MAGIC.to_le_bytes());
        // Header size
        buf.extend_from_slice(&DDS_HEADER_SIZE.to_le_bytes());
        // Flags (offset 8)
        buf.extend_from_slice(&0u32.to_le_bytes());
        // Height (offset 12)
        buf.extend_from_slice(&height.to_le_bytes());
        // Width (offset 16)
        buf.extend_from_slice(&width.to_le_bytes());
        // Pitch/linear size (offset 20)
        buf.extend_from_slice(&0u32.to_le_bytes());
        // Depth (offset 24)
        buf.extend_from_slice(&0u32.to_le_bytes());
        // Mip map count (offset 28)
        buf.extend_from_slice(&mip_count.to_le_bytes());
        // Reserved[11] (offset 32..76)
        for _ in 0..11 {
            buf.extend_from_slice(&0u32.to_le_bytes());
        }

        // Pixel format (offset 76)
        buf.extend_from_slice(&DDS_PIXEL_FORMAT_SIZE.to_le_bytes()); // pf.size
        buf.extend_from_slice(&pf_flags.to_le_bytes()); // pf.flags
        buf.extend_from_slice(&four_cc.to_le_bytes()); // pf.fourCC
        buf.extend_from_slice(&0u32.to_le_bytes()); // rgbBitCount
        buf.extend_from_slice(&0u32.to_le_bytes()); // rMask
        buf.extend_from_slice(&0u32.to_le_bytes()); // gMask
        buf.extend_from_slice(&0u32.to_le_bytes()); // bMask
        buf.extend_from_slice(&0u32.to_le_bytes()); // aMask

        // Caps fields (offset 108..124+4=128)
        buf.extend_from_slice(&0u32.to_le_bytes()); // caps
        buf.extend_from_slice(&0u32.to_le_bytes()); // caps2
        buf.extend_from_slice(&0u32.to_le_bytes()); // caps3
        buf.extend_from_slice(&0u32.to_le_bytes()); // caps4
        buf.extend_from_slice(&0u32.to_le_bytes()); // reserved2

        assert_eq!(buf.len(), 128, "DDS header must be exactly 128 bytes");

        // Optional DX10 header
        if let Some(dxgi_format) = dx10_dxgi_format {
            buf.extend_from_slice(&dxgi_format.to_le_bytes()); // dxgiFormat
            buf.extend_from_slice(&3u32.to_le_bytes()); // resourceDimension (TEXTURE2D)
            buf.extend_from_slice(&0u32.to_le_bytes()); // miscFlag
            buf.extend_from_slice(&1u32.to_le_bytes()); // arraySize
            buf.extend_from_slice(&0u32.to_le_bytes()); // miscFlags2
        }

        // Compressed data
        buf.extend_from_slice(data);
        buf
    }

    #[test]
    fn test_parse_bc1_dxt1() {
        let data = vec![0xAA; 8]; // One 4x4 block of BC1
        let dds = build_dds(4, 4, 1, DDPF_FOURCC, FOURCC_DXT1, None, &data);

        let result = parse_dds(&dds).expect("should parse BC1 DDS");
        assert_eq!(result.width, 4);
        assert_eq!(result.height, 4);
        assert_eq!(result.format, CompressedFormat::BC1);
        assert_eq!(result.mip_levels, 1);
        assert_eq!(result.data_offset, 128);
        assert_eq!(result.data_len, 8);
    }

    #[test]
    fn test_parse_bc3_dxt5() {
        let data = vec![0xBB; 16]; // One 4x4 block of BC3
        let dds = build_dds(4, 4, 4, DDPF_FOURCC, FOURCC_DXT5, None, &data);

        let result = parse_dds(&dds).expect("should parse BC3 DDS");
        assert_eq!(result.format, CompressedFormat::BC3);
        assert_eq!(result.mip_levels, 4);
        assert_eq!(result.data_offset, 128);
        assert_eq!(result.data_len, 16);
    }

    #[test]
    fn test_parse_bc5_dx10() {
        let data = vec![0xCC; 16];
        let dds = build_dds(
            4,
            4,
            1,
            DDPF_FOURCC,
            FOURCC_DX10,
            Some(DXGI_FORMAT_BC5_UNORM),
            &data,
        );

        let result = parse_dds(&dds).expect("should parse BC5 DDS");
        assert_eq!(result.format, CompressedFormat::BC5);
        assert_eq!(result.data_offset, 148); // 128 + 20 (DX10 header)
        assert_eq!(result.data_len, 16);
    }

    #[test]
    fn test_parse_bc7_dx10() {
        let data = vec![0xDD; 32];
        let dds = build_dds(
            8,
            8,
            1,
            DDPF_FOURCC,
            FOURCC_DX10,
            Some(DXGI_FORMAT_BC7_UNORM),
            &data,
        );

        let result = parse_dds(&dds).expect("should parse BC7 DDS");
        assert_eq!(result.width, 8);
        assert_eq!(result.height, 8);
        assert_eq!(result.format, CompressedFormat::BC7);
        assert_eq!(result.data_len, 32);
    }

    #[test]
    fn test_parse_zero_mip_count_defaults_to_one() {
        let data = vec![0xAA; 8];
        let dds = build_dds(4, 4, 0, DDPF_FOURCC, FOURCC_DXT1, None, &data);

        let result = parse_dds(&dds).expect("should parse");
        assert_eq!(result.mip_levels, 1);
    }

    #[test]
    fn test_reject_too_small() {
        let err = parse_dds(&[0u8; 64]).unwrap_err();
        assert!(err.is_decode_failed());
    }

    #[test]
    fn test_reject_bad_magic() {
        let mut dds = vec![0u8; 128];
        dds[0..4].copy_from_slice(&0xDEAD_BEEFu32.to_le_bytes());
        let err = parse_dds(&dds).unwrap_err();
        assert!(err.is_decode_failed());
    }

    #[test]
    fn test_reject_no_fourcc_flag() {
        let data = vec![0xAA; 8];
        let dds = build_dds(4, 4, 1, 0, FOURCC_DXT1, None, &data);

        let err = parse_dds(&dds).unwrap_err();
        assert!(err.is_decode_failed());
    }

    #[test]
    fn test_reject_unsupported_fourcc() {
        let data = vec![0xAA; 8];
        let fourcc = u32::from_le_bytes(*b"DXT3");
        let dds = build_dds(4, 4, 1, DDPF_FOURCC, fourcc, None, &data);

        let err = parse_dds(&dds).unwrap_err();
        assert!(err.is_decode_failed());
    }

    #[test]
    fn test_reject_unsupported_dxgi_format() {
        let data = vec![0xAA; 16];
        let dds = build_dds(4, 4, 1, DDPF_FOURCC, FOURCC_DX10, Some(999), &data);

        let err = parse_dds(&dds).unwrap_err();
        assert!(err.is_decode_failed());
    }

    #[test]
    fn test_reject_zero_dimensions() {
        let data = vec![0xAA; 8];
        let dds = build_dds(0, 4, 1, DDPF_FOURCC, FOURCC_DXT1, None, &data);
        let err = parse_dds(&dds).unwrap_err();
        assert!(err.is_decode_failed());
    }

    #[test]
    fn test_reject_dx10_truncated() {
        // Build a DX10 DDS but cut off the DX10 header
        let dds = build_dds(4, 4, 1, DDPF_FOURCC, FOURCC_DX10, None, &[]);

        let err = parse_dds(&dds).unwrap_err();
        assert!(err.is_decode_failed());
    }

    #[test]
    fn test_block_size_values() {
        assert_eq!(block_size(CompressedFormat::BC1), 8);
        assert_eq!(block_size(CompressedFormat::BC3), 16);
        assert_eq!(block_size(CompressedFormat::BC5), 16);
        assert_eq!(block_size(CompressedFormat::BC7), 16);
    }
}
