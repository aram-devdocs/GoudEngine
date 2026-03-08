//! Delta encoding for efficient network updates.
//!
//! Compares two values and produces a compact payload containing only the
//! fields that changed, identified by a bitmask. This dramatically reduces
//! bandwidth for incremental state updates.

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
    fn apply_delta(&self, delta: &DeltaPayload<Self::Mask>) -> Self;
}

/// Returns `true` if two f32 values differ by more than `f32::EPSILON`.
#[inline]
pub fn f32_changed(a: f32, b: f32) -> bool {
    (a - b).abs() > f32::EPSILON
}

// =============================================================================
// Vec2
// =============================================================================

impl DeltaEncode for Vec2 {
    type Mask = u8;

    fn delta_from(&self, baseline: &Self) -> Option<DeltaPayload<u8>> {
        let mut mask: u8 = 0;
        let mut data = Vec::new();

        if f32_changed(self.x, baseline.x) {
            mask |= 1 << 0;
            data.extend_from_slice(&self.x.to_le_bytes());
        }
        if f32_changed(self.y, baseline.y) {
            mask |= 1 << 1;
            data.extend_from_slice(&self.y.to_le_bytes());
        }

        if mask == 0 {
            None
        } else {
            Some(DeltaPayload { mask, data })
        }
    }

    fn apply_delta(&self, delta: &DeltaPayload<u8>) -> Self {
        let mut result = *self;
        let mut offset = 0;

        if delta.mask & (1 << 0) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.x = v;
            }
        }
        if delta.mask & (1 << 1) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.y = v;
            }
        }

        result
    }
}

// =============================================================================
// Vec3
// =============================================================================

impl DeltaEncode for Vec3 {
    type Mask = u8;

    fn delta_from(&self, baseline: &Self) -> Option<DeltaPayload<u8>> {
        let mut mask: u8 = 0;
        let mut data = Vec::new();

        if f32_changed(self.x, baseline.x) {
            mask |= 1 << 0;
            data.extend_from_slice(&self.x.to_le_bytes());
        }
        if f32_changed(self.y, baseline.y) {
            mask |= 1 << 1;
            data.extend_from_slice(&self.y.to_le_bytes());
        }
        if f32_changed(self.z, baseline.z) {
            mask |= 1 << 2;
            data.extend_from_slice(&self.z.to_le_bytes());
        }

        if mask == 0 {
            None
        } else {
            Some(DeltaPayload { mask, data })
        }
    }

    fn apply_delta(&self, delta: &DeltaPayload<u8>) -> Self {
        let mut result = *self;
        let mut offset = 0;

        if delta.mask & (1 << 0) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.x = v;
            }
        }
        if delta.mask & (1 << 1) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.y = v;
            }
        }
        if delta.mask & (1 << 2) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.z = v;
            }
        }

        result
    }
}

// =============================================================================
// Vec4
// =============================================================================

impl DeltaEncode for Vec4 {
    type Mask = u8;

    fn delta_from(&self, baseline: &Self) -> Option<DeltaPayload<u8>> {
        let mut mask: u8 = 0;
        let mut data = Vec::new();

        if f32_changed(self.x, baseline.x) {
            mask |= 1 << 0;
            data.extend_from_slice(&self.x.to_le_bytes());
        }
        if f32_changed(self.y, baseline.y) {
            mask |= 1 << 1;
            data.extend_from_slice(&self.y.to_le_bytes());
        }
        if f32_changed(self.z, baseline.z) {
            mask |= 1 << 2;
            data.extend_from_slice(&self.z.to_le_bytes());
        }
        if f32_changed(self.w, baseline.w) {
            mask |= 1 << 3;
            data.extend_from_slice(&self.w.to_le_bytes());
        }

        if mask == 0 {
            None
        } else {
            Some(DeltaPayload { mask, data })
        }
    }

    fn apply_delta(&self, delta: &DeltaPayload<u8>) -> Self {
        let mut result = *self;
        let mut offset = 0;

        if delta.mask & (1 << 0) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.x = v;
            }
        }
        if delta.mask & (1 << 1) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.y = v;
            }
        }
        if delta.mask & (1 << 2) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.z = v;
            }
        }
        if delta.mask & (1 << 3) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.w = v;
            }
        }

        result
    }
}

// =============================================================================
// Color
// =============================================================================

impl DeltaEncode for Color {
    type Mask = u8;

    fn delta_from(&self, baseline: &Self) -> Option<DeltaPayload<u8>> {
        let mut mask: u8 = 0;
        let mut data = Vec::new();

        if f32_changed(self.r, baseline.r) {
            mask |= 1 << 0;
            data.extend_from_slice(&self.r.to_le_bytes());
        }
        if f32_changed(self.g, baseline.g) {
            mask |= 1 << 1;
            data.extend_from_slice(&self.g.to_le_bytes());
        }
        if f32_changed(self.b, baseline.b) {
            mask |= 1 << 2;
            data.extend_from_slice(&self.b.to_le_bytes());
        }
        if f32_changed(self.a, baseline.a) {
            mask |= 1 << 3;
            data.extend_from_slice(&self.a.to_le_bytes());
        }

        if mask == 0 {
            None
        } else {
            Some(DeltaPayload { mask, data })
        }
    }

    fn apply_delta(&self, delta: &DeltaPayload<u8>) -> Self {
        let mut result = *self;
        let mut offset = 0;

        if delta.mask & (1 << 0) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.r = v;
            }
        }
        if delta.mask & (1 << 1) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.g = v;
            }
        }
        if delta.mask & (1 << 2) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.b = v;
            }
        }
        if delta.mask & (1 << 3) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.a = v;
            }
        }

        result
    }
}

// =============================================================================
// Rect
// =============================================================================

impl DeltaEncode for Rect {
    type Mask = u8;

    fn delta_from(&self, baseline: &Self) -> Option<DeltaPayload<u8>> {
        let mut mask: u8 = 0;
        let mut data = Vec::new();

        if f32_changed(self.x, baseline.x) {
            mask |= 1 << 0;
            data.extend_from_slice(&self.x.to_le_bytes());
        }
        if f32_changed(self.y, baseline.y) {
            mask |= 1 << 1;
            data.extend_from_slice(&self.y.to_le_bytes());
        }
        if f32_changed(self.width, baseline.width) {
            mask |= 1 << 2;
            data.extend_from_slice(&self.width.to_le_bytes());
        }
        if f32_changed(self.height, baseline.height) {
            mask |= 1 << 3;
            data.extend_from_slice(&self.height.to_le_bytes());
        }

        if mask == 0 {
            None
        } else {
            Some(DeltaPayload { mask, data })
        }
    }

    fn apply_delta(&self, delta: &DeltaPayload<u8>) -> Self {
        let mut result = *self;
        let mut offset = 0;

        if delta.mask & (1 << 0) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.x = v;
            }
        }
        if delta.mask & (1 << 1) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.y = v;
            }
        }
        if delta.mask & (1 << 2) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.width = v;
            }
        }
        if delta.mask & (1 << 3) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.height = v;
            }
        }

        result
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Reads an f32 from a byte slice at the given offset, advancing the offset.
///
/// Returns `None` if the slice is too short to contain a complete f32 at the
/// current offset, rather than panicking on malformed data.
#[inline]
pub fn read_f32(data: &[u8], offset: &mut usize) -> Option<f32> {
    let bytes: [u8; 4] = data.get(*offset..*offset + 4)?.try_into().ok()?;
    *offset += 4;
    Some(f32::from_le_bytes(bytes))
}
