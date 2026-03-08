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

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::core::math::Vec2;
    use crate::core::serialization::binary;

    use super::*;

    // =============================================================================
    // Binary serialization tests
    // =============================================================================

    #[test]
    fn test_binary_roundtrip_transform2d() {
        let original = Transform2D::new(Vec2::new(5.0, 15.0), 1.57, Vec2::new(2.0, 3.0));

        let bytes = binary::encode(&original).unwrap();
        let decoded: Transform2D = binary::decode(&bytes).unwrap();

        assert_eq!(original, decoded);
    }

    // =============================================================================
    // Delta encoding tests
    // =============================================================================

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
        let reconstructed = baseline.apply_delta(&delta);

        assert!((reconstructed.rotation - target.rotation).abs() < f32::EPSILON);
        assert_eq!(reconstructed.position, target.position);
        assert_eq!(reconstructed.scale, target.scale);
    }

    // =============================================================================
    // Property-based tests
    // =============================================================================

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
                let restored = baseline.apply_delta(&delta);
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
