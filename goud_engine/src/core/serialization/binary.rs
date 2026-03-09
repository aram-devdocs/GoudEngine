//! Binary serialization using bincode.
//!
//! Provides thin wrappers around bincode for encoding and decoding any
//! `serde`-compatible type into a compact binary format.

use crate::core::error::GoudError;

/// Encodes a serializable value into a compact binary representation.
///
/// Uses bincode internally for fast, compact serialization. The resulting
/// bytes are not self-describing; the caller must know the type to decode.
///
/// # Errors
///
/// Returns [`GoudError::InternalError`] if bincode serialization fails.
///
/// # Example
///
/// ```
/// use goud_engine::core::serialization::binary;
/// use goud_engine::core::math::Vec2;
///
/// let v = Vec2::new(1.0, 2.0);
/// let bytes = binary::encode(&v).unwrap();
/// let decoded: Vec2 = binary::decode(&bytes).unwrap();
/// assert_eq!(v, decoded);
/// ```
pub fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, GoudError> {
    bincode::serde::encode_to_vec(value, bincode::config::standard())
        .map_err(|e| GoudError::InternalError(format!("Binary encode failed: {e}")))
}

/// Decodes a value from a binary representation produced by [`encode`].
///
/// # Errors
///
/// Returns [`GoudError::InternalError`] if the bytes are invalid or do not
/// match the expected type layout.
///
/// # Example
///
/// ```
/// use goud_engine::core::serialization::binary;
/// use goud_engine::core::math::Vec3;
///
/// let v = Vec3::new(1.0, 2.0, 3.0);
/// let bytes = binary::encode(&v).unwrap();
/// let decoded: Vec3 = binary::decode(&bytes).unwrap();
/// assert_eq!(v, decoded);
/// ```
pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, GoudError> {
    let (value, consumed) =
        bincode::serde::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|e| GoudError::InternalError(format!("Binary decode failed: {e}")))?;
    if consumed != bytes.len() {
        return Err(GoudError::InternalError(
            "Binary decode failed: trailing bytes present".to_string(),
        ));
    }
    Ok(value)
}
