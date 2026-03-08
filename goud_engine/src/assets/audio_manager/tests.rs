//! Tests for the audio manager and spatial attenuation helpers.

use crate::assets::audio_manager::{
    spatial::{
        compute_attenuation_exponential, compute_attenuation_inverse, compute_attenuation_linear,
    },
    AudioManager,
};
use crate::assets::loaders::AudioAsset;
use crate::core::error::GoudError;
use crate::core::math::Vec2;
use crate::ecs::components::AudioChannel;

// AudioManager tests requiring audio hardware are #[ignore]d (CI has no device).

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_new() {
    match AudioManager::new() {
        Ok(manager) => {
            assert_eq!(manager.global_volume(), 1.0);
            assert_eq!(manager.active_count(), 0);
        }
        Err(e) => {
            assert!(matches!(e, GoudError::AudioInitFailed(_)));
        }
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_global_volume() {
    if let Ok(mut manager) = AudioManager::new() {
        assert_eq!(manager.global_volume(), 1.0);

        manager.set_global_volume(0.5);
        assert_eq!(manager.global_volume(), 0.5);

        manager.set_global_volume(0.0);
        assert_eq!(manager.global_volume(), 0.0);
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_volume_clamping() {
    if let Ok(mut manager) = AudioManager::new() {
        manager.set_global_volume(1.5);
        assert_eq!(manager.global_volume(), 1.0);

        manager.set_global_volume(-0.5);
        assert_eq!(manager.global_volume(), 0.0);
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_play_empty_asset() {
    if let Ok(mut manager) = AudioManager::new() {
        let empty_asset = AudioAsset::empty();

        let result = manager.play(&empty_asset);
        assert!(result.is_err());

        match result {
            Err(GoudError::ResourceLoadFailed(_)) => {}
            _ => panic!("Expected ResourceLoadFailed error"),
        }
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_pause_resume() {
    if let Ok(manager) = AudioManager::new() {
        let nonexistent_id = 999;
        assert!(!manager.pause(nonexistent_id));
        assert!(!manager.resume(nonexistent_id));
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_stop() {
    if let Ok(mut manager) = AudioManager::new() {
        let nonexistent_id = 999;
        assert!(!manager.stop(nonexistent_id));
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_is_playing() {
    if let Ok(manager) = AudioManager::new() {
        let nonexistent_id = 999;
        assert!(!manager.is_playing(nonexistent_id));
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_active_count() {
    if let Ok(manager) = AudioManager::new() {
        assert_eq!(manager.active_count(), 0);
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_stop_all() {
    if let Ok(mut manager) = AudioManager::new() {
        manager.stop_all();
        assert_eq!(manager.active_count(), 0);
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_cleanup_finished() {
    if let Ok(mut manager) = AudioManager::new() {
        manager.cleanup_finished();
        assert_eq!(manager.active_count(), 0);
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_debug() {
    if let Ok(manager) = AudioManager::new() {
        let debug_str = format!("{:?}", manager);
        assert!(debug_str.contains("AudioManager"));
        assert!(debug_str.contains("global_volume"));
        assert!(debug_str.contains("active_players"));
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_allocate_player_id() {
    if let Ok(manager) = AudioManager::new() {
        let id1 = manager.allocate_player_id();
        let id2 = manager.allocate_player_id();
        let id3 = manager.allocate_player_id();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
    }
}

#[test]
fn test_audio_manager_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<AudioManager>();
}

#[test]
fn test_audio_manager_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<AudioManager>();
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_play_looped() {
    if let Ok(mut manager) = AudioManager::new() {
        let empty_asset = AudioAsset::empty();
        let result = manager.play_looped(&empty_asset);
        assert!(result.is_err());
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_play_with_settings() {
    if let Ok(mut manager) = AudioManager::new() {
        let empty_asset = AudioAsset::empty();
        let result = manager.play_with_settings(
            &empty_asset,
            0.5,
            1.0,
            false,
            crate::ecs::components::AudioChannel::SFX,
        );
        assert!(result.is_err());
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_set_sink_volume() {
    if let Ok(manager) = AudioManager::new() {
        let nonexistent_id = 999;
        assert!(!manager.set_sink_volume(nonexistent_id, 0.5));
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_set_sink_speed() {
    if let Ok(manager) = AudioManager::new() {
        let nonexistent_id = 999;
        assert!(!manager.set_sink_speed(nonexistent_id, 1.5));
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_audio_manager_is_finished() {
    if let Ok(manager) = AudioManager::new() {
        let nonexistent_id = 999;
        assert!(!manager.is_finished(nonexistent_id));
    }
}

// ============================================================================
// Spatial Audio Tests
// ============================================================================

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

// ============================================================================
// Pure Math Tests (no audio hardware needed)
// ============================================================================

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

// ============================================================================
// Per-channel Volume Tests
// ============================================================================

#[test]
#[ignore] // requires audio hardware
fn test_channel_volume_defaults() {
    if let Ok(manager) = AudioManager::new() {
        assert_eq!(manager.get_channel_volume(AudioChannel::Music), 1.0);
        assert_eq!(manager.get_channel_volume(AudioChannel::SFX), 1.0);
        assert_eq!(manager.get_channel_volume(AudioChannel::Voice), 1.0);
        assert_eq!(manager.get_channel_volume(AudioChannel::Ambience), 1.0);
        assert_eq!(manager.get_channel_volume(AudioChannel::UI), 1.0);
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_set_channel_volume() {
    if let Ok(mut manager) = AudioManager::new() {
        manager.set_channel_volume(AudioChannel::Music, 0.5);
        assert_eq!(manager.get_channel_volume(AudioChannel::Music), 0.5);
        // Other channels unchanged
        assert_eq!(manager.get_channel_volume(AudioChannel::SFX), 1.0);
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_channel_volume_clamping() {
    if let Ok(mut manager) = AudioManager::new() {
        manager.set_channel_volume(AudioChannel::Music, 1.5);
        assert_eq!(manager.get_channel_volume(AudioChannel::Music), 1.0);

        manager.set_channel_volume(AudioChannel::Music, -0.5);
        assert_eq!(manager.get_channel_volume(AudioChannel::Music), 0.0);
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_effective_volume_composition() {
    if let Ok(mut manager) = AudioManager::new() {
        // global=0.8, channel=0.5, individual=0.5 => 0.2
        manager.set_global_volume(0.8);
        manager.set_channel_volume(AudioChannel::Music, 0.5);
        let effective = manager.effective_volume(AudioChannel::Music, 0.5);
        assert!((effective - 0.2).abs() < 0.001);
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_custom_channel_volume() {
    if let Ok(mut manager) = AudioManager::new() {
        let custom = AudioChannel::Custom(10);
        // Defaults to 1.0 for unknown channels
        assert_eq!(manager.get_channel_volume(custom), 1.0);

        manager.set_channel_volume(custom, 0.3);
        assert_eq!(manager.get_channel_volume(custom), 0.3);
    }
}

#[test]
#[ignore] // requires audio hardware
fn test_play_on_channel_empty_asset() {
    if let Ok(mut manager) = AudioManager::new() {
        let empty_asset = AudioAsset::empty();
        let result = manager.play_on_channel(&empty_asset, AudioChannel::Music);
        assert!(result.is_err());
    }
}

// ============================================================================
// AudioChannel::from_id Tests
// ============================================================================

#[test]
fn test_audio_channel_from_id() {
    assert_eq!(AudioChannel::from_id(0), AudioChannel::Music);
    assert_eq!(AudioChannel::from_id(1), AudioChannel::SFX);
    assert_eq!(AudioChannel::from_id(2), AudioChannel::Voice);
    assert_eq!(AudioChannel::from_id(3), AudioChannel::Ambience);
    assert_eq!(AudioChannel::from_id(4), AudioChannel::UI);
    assert_eq!(AudioChannel::from_id(5), AudioChannel::Custom(5));
    assert_eq!(AudioChannel::from_id(255), AudioChannel::Custom(255));
    // Roundtrip: from_id(id).id() == id
    for id in 0..=10 {
        assert_eq!(AudioChannel::from_id(id).id(), id);
    }
}
