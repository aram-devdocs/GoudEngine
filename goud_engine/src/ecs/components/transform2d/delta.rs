//! Delta encoding implementation for `Transform2D`.
//!
//! This lives in the ECS layer (rather than `core::serialization`) because
//! `Transform2D` is an ECS component. The `DeltaEncode` trait and helpers are
//! imported from the core serialization module.

use crate::core::error::GoudError;
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

    fn apply_delta(&self, delta: &DeltaPayload<u8>) -> Result<Self, GoudError> {
        let mut result = *self;
        let mut offset = 0;

        let read = |data: &[u8], off: &mut usize, field: &str, bit: u8| -> Result<f32, GoudError> {
            read_f32(data, off).ok_or_else(|| {
                GoudError::InternalError(format!(
                    "truncated delta payload for Transform2D: missing field '{}' at bit {}",
                    field, bit,
                ))
            })
        };

        if delta.mask & (1 << 0) != 0 {
            result.position.x = read(&delta.data, &mut offset, "position.x", 0)?;
        }
        if delta.mask & (1 << 1) != 0 {
            result.position.y = read(&delta.data, &mut offset, "position.y", 1)?;
        }
        if delta.mask & (1 << 2) != 0 {
            result.rotation = read(&delta.data, &mut offset, "rotation", 2)?;
        }
        if delta.mask & (1 << 3) != 0 {
            result.scale.x = read(&delta.data, &mut offset, "scale.x", 3)?;
        }
        if delta.mask & (1 << 4) != 0 {
            result.scale.y = read(&delta.data, &mut offset, "scale.y", 4)?;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::core::math::Vec2;
    use crate::core::serialization::binary;

    use super::*;

    // =========================================================================
    // Binary serialization tests
    // =========================================================================

    #[test]
    fn test_binary_roundtrip_transform2d() {
        let original = Transform2D::new(Vec2::new(5.0, 15.0), 1.57, Vec2::new(2.0, 3.0));

        let bytes = binary::encode(&original).unwrap();
        let decoded: Transform2D = binary::decode(&bytes).unwrap();

        assert_eq!(original, decoded);
    }

    // =========================================================================
    // Delta encoding tests
    // =========================================================================

    #[test]
    fn test_transform2d_no_changes_returns_none() {
        let a = Transform2D::new(Vec2::new(1.0, 2.0), 0.5, Vec2::one());
        assert!(a.delta_from(&a).is_none());
    }

    #[test]
    fn test_transform2d_all_fields_changed() {
        let baseline = Transform2D::new(Vec2::new(0.0, 0.0), 0.0, Vec2::one());
        let current = Transform2D::new(Vec2::new(10.0, 20.0), 1.5, Vec2::new(2.0, 3.0));

        let delta = current.delta_from(&baseline).unwrap();

        assert_eq!(delta.mask, 0b11111);
        assert_eq!(delta.data.len(), 20); // 5 x f32
    }

    #[test]
    fn test_transform2d_position_only_changed() {
        let baseline = Transform2D::new(Vec2::new(0.0, 0.0), 0.0, Vec2::one());
        let current = Transform2D::new(Vec2::new(5.0, 10.0), 0.0, Vec2::one());

        let delta = current.delta_from(&baseline).unwrap();

        assert_eq!(delta.mask, 0b00011);
        assert_eq!(delta.data.len(), 8); // 2 x f32
    }

    #[test]
    fn test_transform2d_rotation_only_changed() {
        let baseline = Transform2D::new(Vec2::zero(), 0.0, Vec2::one());
        let current = Transform2D::new(Vec2::zero(), 3.14, Vec2::one());

        let delta = current.delta_from(&baseline).unwrap();

        assert_eq!(delta.mask, 0b00100);
        assert_eq!(delta.data.len(), 4);
    }

    #[test]
    fn test_transform2d_apply_delta_reconstructs() {
        let baseline = Transform2D::new(Vec2::new(1.0, 2.0), 0.0, Vec2::one());
        let target = Transform2D::new(Vec2::new(1.0, 2.0), 1.5, Vec2::new(2.0, 1.0));

        let delta = target.delta_from(&baseline).unwrap();
        let reconstructed = baseline.apply_delta(&delta).unwrap();

        assert!((reconstructed.rotation - target.rotation).abs() < f32::EPSILON);
        assert_eq!(reconstructed.position, target.position);
        assert_eq!(reconstructed.scale, target.scale);
    }

    #[test]
    fn test_transform2d_apply_delta_truncated_payload_returns_error() {
        let baseline = Transform2D::new(Vec2::zero(), 0.0, Vec2::one());
        let delta = DeltaPayload {
            mask: 0b00011,             // claims position.x and position.y changed
            data: vec![0, 0, 128, 63], // only 4 bytes (one f32), not enough for two
        };

        let result = baseline.apply_delta(&delta);
        assert!(result.is_err());
    }

    // =========================================================================
    // Property-based tests
    // =========================================================================

    const RANGE: std::ops::Range<f32> = -1e6_f32..1e6_f32;

    prop_compose! {
        fn arb_transform2d()(
            px in RANGE, py in RANGE,
            rotation in RANGE,
            sx in RANGE, sy in RANGE,
        ) -> Transform2D {
            Transform2D {
                position: Vec2::new(px, py),
                rotation,
                scale: Vec2::new(sx, sy),
            }
        }
    }

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() <= f32::EPSILON
    }

    proptest! {
        #[test]
        fn binary_roundtrip_transform2d_prop(t in arb_transform2d()) {
            let bytes = binary::encode(&t).unwrap();
            let decoded: Transform2D = binary::decode(&bytes).unwrap();
            prop_assert_eq!(t, decoded);
        }

        #[test]
        fn delta_roundtrip_transform2d(
            baseline in arb_transform2d(),
            target in arb_transform2d(),
        ) {
            if let Some(delta) = target.delta_from(&baseline) {
                let restored = baseline.apply_delta(&delta).unwrap();
                prop_assert!(approx_eq(restored.position.x, target.position.x));
                prop_assert!(approx_eq(restored.position.y, target.position.y));
                prop_assert!(approx_eq(restored.rotation, target.rotation));
                prop_assert!(approx_eq(restored.scale.x, target.scale.x));
                prop_assert!(approx_eq(restored.scale.y, target.scale.y));
            }
        }

        #[test]
        fn delta_identical_transform2d(t in arb_transform2d()) {
            prop_assert!(t.delta_from(&t).is_none());
        }

        #[test]
        fn delta_size_leq_full_transform2d(
            baseline in arb_transform2d(),
            target in arb_transform2d(),
        ) {
            if let Some(delta) = target.delta_from(&baseline) {
                let full_bytes = binary::encode(&target).unwrap();
                // Delta payload raw data should be <= full value bytes
                prop_assert!(delta.data.len() <= full_bytes.len(),
                    "delta data {} vs full {}", delta.data.len(), full_bytes.len());
            }
        }
    }
}
