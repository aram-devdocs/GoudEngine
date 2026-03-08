//! Property-based tests for serialization round-trips and delta encoding.
//!
//! Uses `proptest` to verify that encoding/decoding and delta operations are
//! correct across a wide range of inputs, avoiding NaN/Inf edge cases.

use proptest::prelude::*;

use crate::core::math::{Color, Rect, Vec2, Vec3, Vec4};
use crate::core::serialization::{binary, DeltaEncode};

// =============================================================================
// Strategies
// =============================================================================

const RANGE: std::ops::Range<f32> = -1e6_f32..1e6_f32;

prop_compose! {
    fn arb_vec2()(x in RANGE, y in RANGE) -> Vec2 {
        Vec2::new(x, y)
    }
}

prop_compose! {
    fn arb_vec3()(x in RANGE, y in RANGE, z in RANGE) -> Vec3 {
        Vec3::new(x, y, z)
    }
}

prop_compose! {
    fn arb_vec4()(x in RANGE, y in RANGE, z in RANGE, w in RANGE) -> Vec4 {
        Vec4::new(x, y, z, w)
    }
}

prop_compose! {
    fn arb_color()(r in RANGE, g in RANGE, b in RANGE, a in RANGE) -> Color {
        Color { r, g, b, a }
    }
}

prop_compose! {
    fn arb_rect()(x in RANGE, y in RANGE, width in RANGE, height in RANGE) -> Rect {
        Rect { x, y, width, height }
    }
}

// =============================================================================
// Binary round-trip: decode(encode(value)) == value
// =============================================================================

proptest! {
    #[test]
    fn binary_roundtrip_vec2(v in arb_vec2()) {
        let bytes = binary::encode(&v).unwrap();
        let decoded: Vec2 = binary::decode(&bytes).unwrap();
        prop_assert_eq!(v, decoded);
    }

    #[test]
    fn binary_roundtrip_vec3(v in arb_vec3()) {
        let bytes = binary::encode(&v).unwrap();
        let decoded: Vec3 = binary::decode(&bytes).unwrap();
        prop_assert_eq!(v, decoded);
    }

    #[test]
    fn binary_roundtrip_vec4(v in arb_vec4()) {
        let bytes = binary::encode(&v).unwrap();
        let decoded: Vec4 = binary::decode(&bytes).unwrap();
        prop_assert_eq!(v, decoded);
    }

    #[test]
    fn binary_roundtrip_color(c in arb_color()) {
        let bytes = binary::encode(&c).unwrap();
        let decoded: Color = binary::decode(&bytes).unwrap();
        prop_assert_eq!(c, decoded);
    }

    #[test]
    fn binary_roundtrip_rect(r in arb_rect()) {
        let bytes = binary::encode(&r).unwrap();
        let decoded: Rect = binary::decode(&bytes).unwrap();
        prop_assert_eq!(r, decoded);
    }

}

// =============================================================================
// Delta round-trip: apply_delta(delta_from(baseline, target)) == target
// =============================================================================

/// Checks approximate f32 equality within `f32::EPSILON`.
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= f32::EPSILON
}

proptest! {
    #[test]
    fn delta_roundtrip_vec2(baseline in arb_vec2(), target in arb_vec2()) {
        if let Some(delta) = target.delta_from(&baseline) {
            let restored = baseline.apply_delta(&delta).unwrap();
            prop_assert!(approx_eq(restored.x, target.x), "x mismatch");
            prop_assert!(approx_eq(restored.y, target.y), "y mismatch");
        }
    }

    #[test]
    fn delta_roundtrip_vec3(baseline in arb_vec3(), target in arb_vec3()) {
        if let Some(delta) = target.delta_from(&baseline) {
            let restored = baseline.apply_delta(&delta).unwrap();
            prop_assert!(approx_eq(restored.x, target.x), "x mismatch");
            prop_assert!(approx_eq(restored.y, target.y), "y mismatch");
            prop_assert!(approx_eq(restored.z, target.z), "z mismatch");
        }
    }

    #[test]
    fn delta_roundtrip_vec4(baseline in arb_vec4(), target in arb_vec4()) {
        if let Some(delta) = target.delta_from(&baseline) {
            let restored = baseline.apply_delta(&delta).unwrap();
            prop_assert!(approx_eq(restored.x, target.x), "x mismatch");
            prop_assert!(approx_eq(restored.y, target.y), "y mismatch");
            prop_assert!(approx_eq(restored.z, target.z), "z mismatch");
            prop_assert!(approx_eq(restored.w, target.w), "w mismatch");
        }
    }

    #[test]
    fn delta_roundtrip_color(baseline in arb_color(), target in arb_color()) {
        if let Some(delta) = target.delta_from(&baseline) {
            let restored = baseline.apply_delta(&delta).unwrap();
            prop_assert!(approx_eq(restored.r, target.r), "r mismatch");
            prop_assert!(approx_eq(restored.g, target.g), "g mismatch");
            prop_assert!(approx_eq(restored.b, target.b), "b mismatch");
            prop_assert!(approx_eq(restored.a, target.a), "a mismatch");
        }
    }

    #[test]
    fn delta_roundtrip_rect(baseline in arb_rect(), target in arb_rect()) {
        if let Some(delta) = target.delta_from(&baseline) {
            let restored = baseline.apply_delta(&delta).unwrap();
            prop_assert!(approx_eq(restored.x, target.x), "x mismatch");
            prop_assert!(approx_eq(restored.y, target.y), "y mismatch");
            prop_assert!(approx_eq(restored.width, target.width), "width mismatch");
            prop_assert!(approx_eq(restored.height, target.height), "height mismatch");
        }
    }

}

// =============================================================================
// Delta of identical values returns None
// =============================================================================

proptest! {
    #[test]
    fn delta_identical_vec2(v in arb_vec2()) {
        prop_assert!(v.delta_from(&v).is_none());
    }

    #[test]
    fn delta_identical_vec3(v in arb_vec3()) {
        prop_assert!(v.delta_from(&v).is_none());
    }

    #[test]
    fn delta_identical_vec4(v in arb_vec4()) {
        prop_assert!(v.delta_from(&v).is_none());
    }

    #[test]
    fn delta_identical_color(c in arb_color()) {
        prop_assert!(c.delta_from(&c).is_none());
    }

    #[test]
    fn delta_identical_rect(r in arb_rect()) {
        prop_assert!(r.delta_from(&r).is_none());
    }

}

// =============================================================================
// Delta payload size <= full encode size
// =============================================================================

proptest! {
    #[test]
    fn delta_size_leq_full_vec2(baseline in arb_vec2(), target in arb_vec2()) {
        if let Some(delta) = target.delta_from(&baseline) {
            let delta_bytes = binary::encode(&delta).unwrap();
            let full_bytes = binary::encode(&target).unwrap();
            prop_assert!(delta_bytes.len() <= full_bytes.len() + 12,
                "delta {} vs full {} (overhead: 8-byte bincode len + 1 mask + ~3 data-len prefix)",
                delta_bytes.len(), full_bytes.len());
        }
    }

}
