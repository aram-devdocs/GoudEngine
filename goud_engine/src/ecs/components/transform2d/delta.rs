//! Delta encoding implementation for `Transform2D`.
//!
//! This lives in the ECS layer (rather than `core::serialization`) because
//! `Transform2D` is an ECS component. The `DeltaEncode` trait and helpers are
//! imported from the core serialization module.

use crate::core::serialization::delta::{f32_changed, read_f32, DeltaEncode, DeltaPayload};

use super::Transform2D;

impl DeltaEncode for Transform2D {
    type Mask = u8;

    fn delta_from(&self, baseline: &Self) -> Option<DeltaPayload<u8>> {
        let mut mask: u8 = 0;
        let mut data = Vec::new();

        if f32_changed(self.position.x, baseline.position.x) {
            mask |= 1 << 0;
            data.extend_from_slice(&self.position.x.to_le_bytes());
        }
        if f32_changed(self.position.y, baseline.position.y) {
            mask |= 1 << 1;
            data.extend_from_slice(&self.position.y.to_le_bytes());
        }
        if f32_changed(self.rotation, baseline.rotation) {
            mask |= 1 << 2;
            data.extend_from_slice(&self.rotation.to_le_bytes());
        }
        if f32_changed(self.scale.x, baseline.scale.x) {
            mask |= 1 << 3;
            data.extend_from_slice(&self.scale.x.to_le_bytes());
        }
        if f32_changed(self.scale.y, baseline.scale.y) {
            mask |= 1 << 4;
            data.extend_from_slice(&self.scale.y.to_le_bytes());
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
                result.position.x = v;
            }
        }
        if delta.mask & (1 << 1) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.position.y = v;
            }
        }
        if delta.mask & (1 << 2) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.rotation = v;
            }
        }
        if delta.mask & (1 << 3) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.scale.x = v;
            }
        }
        if delta.mask & (1 << 4) != 0 {
            if let Some(v) = read_f32(&delta.data, &mut offset) {
                result.scale.y = v;
            }
        }

        result
    }
}
