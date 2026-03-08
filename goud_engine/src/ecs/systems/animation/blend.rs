//! Animation blending utilities.
//!
//! Provides functions for interpolating between [`Rect`] values, used by both
//! the animation controller crossfade system and the animation layer stack.

use crate::core::math::Rect;

/// Blend mode controlling how a layer's rect combines with previous layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BlendMode {
    /// Replace the previous result entirely (weighted by the layer's weight).
    Override,
    /// Add the layer's rect offset on top of the previous result.
    Additive,
}

/// Linearly interpolates each field of two [`Rect`] values.
///
/// `weight` is clamped to `[0.0, 1.0]`. A weight of `0.0` returns `from`,
/// and a weight of `1.0` returns `to`.
#[inline]
pub fn blend_rects(from: Rect, to: Rect, weight: f32) -> Rect {
    let w = weight.clamp(0.0, 1.0);
    let inv = 1.0 - w;
    Rect::new(
        from.x * inv + to.x * w,
        from.y * inv + to.y * w,
        from.width * inv + to.width * w,
        from.height * inv + to.height * w,
    )
}

/// Computes a final blended [`Rect`] from a slice of `(rect, weight, blend_mode)` tuples.
///
/// Layers are processed in order. The first layer with a non-zero weight
/// initialises the accumulator. Subsequent layers blend into the accumulator
/// according to their [`BlendMode`]:
///
/// - **Override**: lerp between the accumulator and the layer's rect by weight.
/// - **Additive**: add `(layer_rect - base) * weight` to the accumulator, where
///   `base` is an identity rect `(0, 0, 0, 0)`.
///
/// Returns `None` if the slice is empty or all weights are effectively zero.
pub fn compute_blended_rect(layers: &[(Rect, f32, BlendMode)]) -> Option<Rect> {
    let mut result: Option<Rect> = None;

    for &(rect, weight, mode) in layers {
        let w = weight.clamp(0.0, 1.0);
        if w < f32::EPSILON {
            continue;
        }

        match result {
            None => {
                // First contributing layer initialises the accumulator.
                result = Some(rect);
            }
            Some(acc) => {
                let blended = match mode {
                    BlendMode::Override => blend_rects(acc, rect, w),
                    BlendMode::Additive => Rect::new(
                        acc.x + rect.x * w,
                        acc.y + rect.y * w,
                        acc.width + rect.width * w,
                        acc.height + rect.height * w,
                    ),
                };
                result = Some(blended);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Rect;

    #[test]
    fn test_blend_rects_50_percent() {
        let from = Rect::new(0.0, 0.0, 32.0, 32.0);
        let to = Rect::new(32.0, 0.0, 64.0, 64.0);
        let result = blend_rects(from, to, 0.5);

        assert!((result.x - 16.0).abs() < f32::EPSILON);
        assert!((result.y - 0.0).abs() < f32::EPSILON);
        assert!((result.width - 48.0).abs() < f32::EPSILON);
        assert!((result.height - 48.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_blend_rects_zero() {
        let from = Rect::new(10.0, 20.0, 30.0, 40.0);
        let to = Rect::new(100.0, 200.0, 300.0, 400.0);
        let result = blend_rects(from, to, 0.0);

        assert!((result.x - 10.0).abs() < f32::EPSILON);
        assert!((result.y - 20.0).abs() < f32::EPSILON);
        assert!((result.width - 30.0).abs() < f32::EPSILON);
        assert!((result.height - 40.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_blend_rects_full() {
        let from = Rect::new(10.0, 20.0, 30.0, 40.0);
        let to = Rect::new(100.0, 200.0, 300.0, 400.0);
        let result = blend_rects(from, to, 1.0);

        assert!((result.x - 100.0).abs() < f32::EPSILON);
        assert!((result.y - 200.0).abs() < f32::EPSILON);
        assert!((result.width - 300.0).abs() < f32::EPSILON);
        assert!((result.height - 400.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_blend_rects_clamps_negative_weight() {
        let from = Rect::new(0.0, 0.0, 32.0, 32.0);
        let to = Rect::new(64.0, 64.0, 64.0, 64.0);
        let result = blend_rects(from, to, -0.5);

        // Clamped to 0.0, so result == from
        assert!((result.x - 0.0).abs() < f32::EPSILON);
        assert!((result.width - 32.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_blend_rects_clamps_over_one() {
        let from = Rect::new(0.0, 0.0, 32.0, 32.0);
        let to = Rect::new(64.0, 64.0, 64.0, 64.0);
        let result = blend_rects(from, to, 1.5);

        // Clamped to 1.0, so result == to
        assert!((result.x - 64.0).abs() < f32::EPSILON);
        assert!((result.width - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_blended_rect_empty() {
        let result = compute_blended_rect(&[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_compute_blended_rect_single_layer() {
        let rect = Rect::new(10.0, 20.0, 30.0, 40.0);
        let result = compute_blended_rect(&[(rect, 1.0, BlendMode::Override)]);
        assert_eq!(result, Some(rect));
    }

    #[test]
    fn test_compute_blended_rect_override_two_layers() {
        let base = Rect::new(0.0, 0.0, 32.0, 32.0);
        let overlay = Rect::new(64.0, 64.0, 64.0, 64.0);
        let result = compute_blended_rect(&[
            (base, 1.0, BlendMode::Override),
            (overlay, 0.5, BlendMode::Override),
        ]);

        let expected = blend_rects(base, overlay, 0.5);
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_compute_blended_rect_zero_weight_ignored() {
        let base = Rect::new(10.0, 20.0, 30.0, 40.0);
        let ignored = Rect::new(999.0, 999.0, 999.0, 999.0);
        let result = compute_blended_rect(&[
            (base, 1.0, BlendMode::Override),
            (ignored, 0.0, BlendMode::Override),
        ]);

        assert_eq!(result, Some(base));
    }

    #[test]
    fn test_compute_blended_rect_additive() {
        let base = Rect::new(10.0, 10.0, 32.0, 32.0);
        let additive = Rect::new(5.0, 5.0, 0.0, 0.0);
        let result = compute_blended_rect(&[
            (base, 1.0, BlendMode::Override),
            (additive, 1.0, BlendMode::Additive),
        ]);

        // Additive: acc + rect * weight
        let expected = Rect::new(15.0, 15.0, 32.0, 32.0);
        assert_eq!(result, Some(expected));
    }
}
