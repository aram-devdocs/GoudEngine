//! Delta encoding for efficient network updates.
//!
//! Compares two values and produces a compact payload containing only the
//! fields that changed, identified by a bitmask. This dramatically reduces
//! bandwidth for incremental state updates.

use crate::core::error::GoudError;
use crate::core::math::{Color, Rect, Vec2, Vec3, Vec4};

/// A delta payload containing a change mask and the raw bytes of changed fields.
///
/// The mask type `M` is typically a `u8` where each bit indicates whether a
/// specific field changed. The `data` vector contains the little-endian bytes
/// of only the changed fields, in mask-bit order.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeltaPayload<M> {
    /// Bitmask indicating which fields changed.
    pub mask: M,
    /// Raw bytes of changed fields in mask-bit order.
    pub data: Vec<u8>,
}

/// Trait for types that support delta encoding against a baseline.
///
/// Implementations compare `self` against a `baseline` value and produce a
/// [`DeltaPayload`] containing only the changed fields. If nothing changed,
/// `delta_from` returns `None`.
pub trait DeltaEncode: Sized {
    /// The bitmask type used to identify changed fields.
    type Mask: Copy + Default + PartialEq + serde::Serialize + serde::de::DeserializeOwned;

    /// Computes the delta between `self` and `baseline`.
    ///
    /// Returns `None` if all fields are equal (within `f32::EPSILON`).
    fn delta_from(&self, baseline: &Self) -> Option<DeltaPayload<Self::Mask>>;

    /// Applies a delta payload to `self`, producing the updated value.
    ///
    /// # Errors
    ///
    /// Returns [`GoudError::InternalError`] if the payload data is truncated
    /// (a field indicated by the mask cannot be read).
    fn apply_delta(&self, delta: &DeltaPayload<Self::Mask>) -> Result<Self, GoudError>;
}

/// Returns `true` if two f32 values differ by more than `f32::EPSILON`.
#[inline]
pub(crate) fn f32_changed(a: f32, b: f32) -> bool {
    (a - b).abs() > f32::EPSILON
}

/// Reads an f32 from a byte slice at the given offset, advancing the offset.
///
/// Returns `None` if the slice is too short to contain a complete f32 at the
/// current offset, rather than panicking on malformed data.
#[inline]
pub(crate) fn read_f32(data: &[u8], offset: &mut usize) -> Option<f32> {
    let bytes: [u8; 4] = data.get(*offset..*offset + 4)?.try_into().ok()?;
    *offset += 4;
    Some(f32::from_le_bytes(bytes))
}

// =============================================================================
// Macro for DeltaEncode implementations on simple f32-field structs
// =============================================================================

/// Implements `DeltaEncode` for a struct whose fields are all `f32`.
///
/// Each field is specified as `(bit_index, field_path)`. The macro generates
/// both `delta_from` (compares each field, builds bitmask + data) and
/// `apply_delta` (reads changed fields from the payload, returning an error
/// on truncated data).
macro_rules! impl_delta_encode {
    ($ty:ty, $( ($bit:expr, $field:ident) ),+ $(,)?) => {
        impl DeltaEncode for $ty {
            type Mask = u8;

            fn delta_from(&self, baseline: &Self) -> Option<DeltaPayload<u8>> {
                let mut mask: u8 = 0;
                let mut data = Vec::new();

                $(
                    if f32_changed(self.$field, baseline.$field) {
                        mask |= 1 << $bit;
                        data.extend_from_slice(&self.$field.to_le_bytes());
                    }
                )+

                if mask == 0 {
                    None
                } else {
                    Some(DeltaPayload { mask, data })
                }
            }

            fn apply_delta(
                &self,
                delta: &DeltaPayload<u8>,
            ) -> Result<Self, GoudError> {
                let mut result = *self;
                let mut offset = 0;

                $(
                    if delta.mask & (1 << $bit) != 0 {
                        result.$field = read_f32(&delta.data, &mut offset)
                            .ok_or_else(|| GoudError::InternalError(
                                format!(
                                    "truncated delta payload for {}: missing field '{}' at bit {}",
                                    stringify!($ty),
                                    stringify!($field),
                                    $bit,
                                )
                            ))?;
                    }
                )+

                Ok(result)
            }
        }
    };
}

impl_delta_encode!(Vec2, (0, x), (1, y));
impl_delta_encode!(Vec3, (0, x), (1, y), (2, z));
impl_delta_encode!(Vec4, (0, x), (1, y), (2, z), (3, w));
impl_delta_encode!(Color, (0, r), (1, g), (2, b), (3, a));
impl_delta_encode!(Rect, (0, x), (1, y), (2, width), (3, height));
