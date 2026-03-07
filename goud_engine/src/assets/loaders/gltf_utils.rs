//! Shared utilities for GLTF/GLB asset loaders.

#[cfg(feature = "native")]
use crate::assets::AssetLoadError;

/// Decodes a `data:` URI into raw bytes.
///
/// Only base64-encoded data URIs are supported (any MIME type).
#[cfg(feature = "native")]
pub(crate) fn decode_data_uri(uri: &str) -> Result<Vec<u8>, AssetLoadError> {
    let marker = ";base64,";
    let pos = uri.find(marker).ok_or_else(|| {
        AssetLoadError::decode_failed(format!("Unsupported data URI format: {uri}"))
    })?;
    let encoded = &uri[pos + marker.len()..];
    decode_base64(encoded)
}

/// Minimal base64 decoder (standard alphabet, padding optional).
///
/// Avoids pulling in the `base64` crate for a single use case.
#[cfg(feature = "native")]
pub(crate) fn decode_base64(input: &str) -> Result<Vec<u8>, AssetLoadError> {
    const TABLE: [u8; 128] = {
        let mut t = [255u8; 128];
        let mut i = 0u8;
        while i < 26 {
            t[(b'A' + i) as usize] = i;
            t[(b'a' + i) as usize] = i + 26;
            i += 1;
        }
        let mut d = 0u8;
        while d < 10 {
            t[(b'0' + d) as usize] = d + 52;
            d += 1;
        }
        t[b'+' as usize] = 62;
        t[b'/' as usize] = 63;
        t
    };

    let bytes: Vec<u8> = input
        .bytes()
        .filter(|&b| b != b'=' && b != b'\n' && b != b'\r' && b != b' ')
        .collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);

    for chunk in bytes.chunks(4) {
        let mut buf = 0u32;
        let len = chunk.len();
        for (i, &b) in chunk.iter().enumerate() {
            if b >= 128 {
                return Err(AssetLoadError::decode_failed(format!(
                    "Invalid base64 character: {b}"
                )));
            }
            let val = TABLE[b as usize];
            if val == 255 {
                return Err(AssetLoadError::decode_failed(format!(
                    "Invalid base64 character: {b}"
                )));
            }
            buf |= (val as u32) << (6 * (3 - i));
        }
        out.push((buf >> 16) as u8);
        if len > 2 {
            out.push((buf >> 8) as u8);
        }
        if len > 3 {
            out.push(buf as u8);
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "native")]
    use super::*;

    #[cfg(feature = "native")]
    #[test]
    fn test_decode_base64_simple() {
        // "Hello" in base64 is "SGVsbG8="
        let result = decode_base64("SGVsbG8=").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_decode_base64_no_padding() {
        let result = decode_base64("SGVsbG8").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_decode_base64_empty() {
        let result = decode_base64("").unwrap();
        assert!(result.is_empty());
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_decode_base64_invalid() {
        let result = decode_base64("!!!invalid!!!");
        assert!(result.is_err());
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_decode_data_uri() {
        let uri = "data:application/octet-stream;base64,SGVsbG8=";
        let result = decode_data_uri(uri).unwrap();
        assert_eq!(result, b"Hello");
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_decode_data_uri_unsupported() {
        let result = decode_data_uri("not-a-data-uri");
        assert!(result.is_err());
    }
}
