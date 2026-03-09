use crate::assets::audio_manager::{
    mixing::{clamp_duration, crossfade_pair, crossfade_progress},
    spatial::{
        compute_attenuation_exponential, compute_attenuation_inverse, compute_attenuation_linear,
        compute_stereo_pan, spatial_attenuation_3d, stereo_gains_from_pan,
    },
    AudioManager,
};
use crate::assets::loaders::AudioAsset;
use crate::core::math::Vec2;

#[test]
#[ignore] // requires audio hardware
fn test_spatial_audio_play_empty_asset() {
    if let Ok(mut manager) = AudioManager::new() {
        let empty_asset = AudioAsset::empty();
        let result = manager.play_spatial(
            &empty_asset,
            Vec2::new(100.0, 50.0),
            Vec2::new(0.0, 0.0),
            200.0,
            1.0,
        );
        assert!(result.is_err());
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_spatial_audio_update_volume_nonexistent() {
    if let Ok(manager) = AudioManager::new() {
        let nonexistent_id = 999;
        assert!(!manager.update_spatial_volume(
            nonexistent_id,
            Vec2::new(100.0, 50.0),
            Vec2::new(0.0, 0.0),
            200.0,
            1.0
        ));
    }
}

#[test]
fn test_compute_attenuation_linear() {
    // (distance, max_distance, rolloff, expected)
    let cases: &[(f32, f32, f32, f32)] = &[
        (0.0, 100.0, 1.0, 1.0),   // zero distance
        (100.0, 100.0, 1.0, 0.0), // max distance
        (50.0, 100.0, 1.0, 0.5),  // half distance
        (150.0, 100.0, 1.0, 0.0), // beyond max
        (50.0, 100.0, 2.0, 0.75), // quadratic rolloff: 1 - (0.5)^2
        (100.0, 0.0, 1.0, 1.0),   // zero max distance
    ];
    for &(dist, max, rolloff, expected) in cases {
        let att = compute_attenuation_linear(dist, max, rolloff);
        assert!(
            (att - expected).abs() < 0.001,
            "linear({dist}, {max}, {rolloff}): expected {expected}, got {att}"
        );
    }
}

#[test]
fn test_compute_attenuation_inverse() {
    // (distance, max_distance, rolloff, expected, tolerance)
    let cases: &[(f32, f32, f32, f32, f32)] = &[
        (0.0, 100.0, 1.0, 1.0, 0.001),   // zero distance
        (100.0, 100.0, 1.0, 0.0, 0.001), // max distance
        (10.0, 100.0, 1.0, 0.1, 0.01),   // 1/(1+1*(10-1)) = 0.1
        (150.0, 100.0, 1.0, 0.0, 0.001), // beyond max
    ];
    for &(dist, max, rolloff, expected, tol) in cases {
        let att = compute_attenuation_inverse(dist, max, rolloff);
        assert!(
            (att - expected).abs() < tol,
            "inverse({dist}, {max}, {rolloff}): expected {expected}, got {att}"
        );
    }
}

#[test]
fn test_compute_attenuation_exponential() {
    // (distance, max_distance, rolloff, expected)
    let cases: &[(f32, f32, f32, f32)] = &[
        (0.0, 100.0, 1.0, 1.0),    // zero distance
        (100.0, 100.0, 1.0, 0.0),  // max distance
        (50.0, 100.0, 1.0, 0.5),   // half distance: (1-0.5)^1
        (50.0, 100.0, 3.0, 0.125), // dramatic falloff: (1-0.5)^3
        (150.0, 100.0, 1.0, 0.0),  // beyond max
    ];
    for &(dist, max, rolloff, expected) in cases {
        let att = compute_attenuation_exponential(dist, max, rolloff);
        assert!(
            (att - expected).abs() < 0.001,
            "exponential({dist}, {max}, {rolloff}): expected {expected}, got {att}"
        );
    }
}

#[test]
fn test_attenuation_comparison() {
    let distance = 50.0;
    let max_distance = 100.0;
    let rolloff = 1.0;

    let linear = compute_attenuation_linear(distance, max_distance, rolloff);
    let inverse = compute_attenuation_inverse(distance, max_distance, rolloff);
    let exponential = compute_attenuation_exponential(distance, max_distance, rolloff);

    // Linear: 0.5, Inverse: ~0.02, Exponential: 0.5
    assert!((linear - 0.5).abs() < 0.001);
    assert!(inverse < 0.05); // Inverse falloff is much faster
    assert!((exponential - 0.5).abs() < 0.001);
}

#[test]
fn test_spatial_audio_attenuation_positions() {
    // (source, listener, max_distance, expected, tolerance)
    let cases: &[(Vec2, Vec2, f32, f32, f32)] = &[
        (
            Vec2::new(100.0, 50.0),
            Vec2::new(100.0, 50.0),
            200.0,
            1.0,
            0.001,
        ), // at source
        (
            Vec2::new(100.0, 50.0),
            Vec2::new(-100.0, 50.0),
            200.0,
            0.0,
            0.001,
        ), // at max
        (
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 0.0),
            200.0,
            0.293,
            0.01,
        ), // diagonal
    ];
    for &(source, listener, max_dist, expected, tol) in cases {
        let distance = (source - listener).length();
        let att = compute_attenuation_linear(distance, max_dist, 1.0);
        assert!(
            (att - expected).abs() < tol,
            "spatial at dist {distance}: expected {expected}, got {att}"
        );
    }
}

#[test]
fn test_spatial_audio_attenuation_3d() {
    let at_source = spatial_attenuation_3d([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 100.0, 1.0);
    assert!((at_source - 1.0).abs() < 0.001);

    let at_max = spatial_attenuation_3d([100.0, 0.0, 0.0], [0.0, 0.0, 0.0], 100.0, 1.0);
    assert!((at_max - 0.0).abs() < 0.001);

    let diagonal = spatial_attenuation_3d([50.0, 50.0, 0.0], [0.0, 0.0, 0.0], 100.0, 1.0);
    assert!(diagonal > 0.25 && diagonal < 0.35);
}

#[test]
fn test_stereo_pan_and_gains_math() {
    let left_pan = compute_stereo_pan([-10.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
    let center_pan = compute_stereo_pan([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
    let right_pan = compute_stereo_pan([10.0, 0.0, 0.0], [0.0, 0.0, 0.0]);

    assert!(left_pan < 0.0);
    assert_eq!(center_pan, 0.0);
    assert!(right_pan > 0.0);

    let (left_gain, right_gain) = stereo_gains_from_pan(0.0);
    assert!((left_gain - right_gain).abs() < 0.001);
    assert!((left_gain * left_gain + right_gain * right_gain - 1.0).abs() < 0.001);
}

#[test]
fn test_crossfade_and_mix_math_helpers() {
    assert_eq!(clamp_duration(-5.0), 0.0);
    assert_eq!(clamp_duration(1.5), 1.5);
    assert_eq!(crossfade_progress(0.5, 1.0), 0.5);

    let (from, to) = crossfade_pair(1.0, 0.8, 0.25);
    assert!((from - 0.75).abs() < 0.001);
    assert!((to - 0.2).abs() < 0.001);
}

/// Validates the effective_volume composition formula:
/// effective = global * channel * individual.
/// Covers identity, muted global/channel, and various combinations.
#[test]
fn test_effective_volume_math() {
    let cases: &[(f32, f32, f32, f32)] = &[
        (1.0, 1.0, 1.0, 1.0),  // identity
        (0.8, 0.5, 0.5, 0.2),  // mixed composition
        (0.0, 1.0, 1.0, 0.0),  // muted global
        (1.0, 0.0, 0.75, 0.0), // muted channel
        (1.0, 1.0, 0.5, 0.5),
        (0.5, 1.0, 1.0, 0.5),
        (0.5, 0.5, 0.5, 0.125),
        (0.25, 0.4, 1.0, 0.1),
    ];
    for &(global, channel, individual, expected) in cases {
        let effective = global * channel * individual;
        assert!(
            (effective - expected).abs() < 0.001,
            "global={global}, channel={channel}, individual={individual}: \
             expected {expected}, got {effective}"
        );
    }
}
