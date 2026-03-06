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

// ============================================================================
// AudioManager Tests
// NOTE: Tests calling AudioManager::new() are #[ignore]d because they require
// audio hardware (rodio crashes with STATUS_ACCESS_VIOLATION on Windows CI)
// ============================================================================

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

// ============================================================================
// Thread Safety Tests
// ============================================================================

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

// ============================================================================
// Audio Playback Tests (with real audio data)
// ============================================================================

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
        let result = manager.play_with_settings(&empty_asset, 0.5, 1.0, false);
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
fn test_compute_attenuation_linear_zero_distance() {
    let attenuation = compute_attenuation_linear(0.0, 100.0, 1.0);
    assert!((attenuation - 1.0).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_linear_max_distance() {
    let attenuation = compute_attenuation_linear(100.0, 100.0, 1.0);
    assert!((attenuation - 0.0).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_linear_half_distance() {
    let attenuation = compute_attenuation_linear(50.0, 100.0, 1.0);
    assert!((attenuation - 0.5).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_linear_beyond_max() {
    let attenuation = compute_attenuation_linear(150.0, 100.0, 1.0);
    assert!((attenuation - 0.0).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_linear_quadratic_rolloff() {
    let attenuation = compute_attenuation_linear(50.0, 100.0, 2.0);
    // At half distance: 1 - (0.5)^2 = 1 - 0.25 = 0.75
    assert!((attenuation - 0.75).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_linear_zero_max_distance() {
    let attenuation = compute_attenuation_linear(100.0, 0.0, 1.0);
    assert!((attenuation - 1.0).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_inverse_zero_distance() {
    let attenuation = compute_attenuation_inverse(0.0, 100.0, 1.0);
    assert!((attenuation - 1.0).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_inverse_max_distance() {
    let attenuation = compute_attenuation_inverse(100.0, 100.0, 1.0);
    assert!((attenuation - 0.0).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_inverse_realistic() {
    let attenuation = compute_attenuation_inverse(10.0, 100.0, 1.0);
    // Formula: 1 / (1 + rolloff * (distance - ref_distance))
    // = 1 / (1 + 1 * (10 - 1)) = 1 / (1 + 9) = 1/10 = 0.1
    assert!(
        (attenuation - 0.1).abs() < 0.01,
        "Expected ~0.1, got {}",
        attenuation
    );
}

#[test]
fn test_compute_attenuation_inverse_beyond_max() {
    let attenuation = compute_attenuation_inverse(150.0, 100.0, 1.0);
    assert!((attenuation - 0.0).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_exponential_zero_distance() {
    let attenuation = compute_attenuation_exponential(0.0, 100.0, 1.0);
    assert!((attenuation - 1.0).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_exponential_max_distance() {
    let attenuation = compute_attenuation_exponential(100.0, 100.0, 1.0);
    assert!((attenuation - 0.0).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_exponential_half_distance() {
    // With rolloff=1.0, at half distance: (1 - 0.5)^1 = 0.5
    let attenuation = compute_attenuation_exponential(50.0, 100.0, 1.0);
    assert!((attenuation - 0.5).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_exponential_dramatic_falloff() {
    // With rolloff=3.0, falloff is dramatic: (1 - 0.5)^3 = 0.125
    let attenuation = compute_attenuation_exponential(50.0, 100.0, 3.0);
    assert!((attenuation - 0.125).abs() < 0.001);
}

#[test]
fn test_compute_attenuation_exponential_beyond_max() {
    let attenuation = compute_attenuation_exponential(150.0, 100.0, 1.0);
    assert!((attenuation - 0.0).abs() < 0.001);
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
fn test_spatial_audio_attenuation_at_source() {
    let source_pos = Vec2::new(100.0, 50.0);
    let listener_pos = Vec2::new(100.0, 50.0);
    let distance = (source_pos - listener_pos).length();

    let attenuation = compute_attenuation_linear(distance, 200.0, 1.0);
    assert!(
        (attenuation - 1.0).abs() < 0.001,
        "Attenuation at source should be 1.0"
    );
}

#[test]
fn test_spatial_audio_attenuation_at_max() {
    let source_pos = Vec2::new(100.0, 50.0);
    let listener_pos = Vec2::new(-100.0, 50.0); // 200 units away
    let distance = (source_pos - listener_pos).length();

    let attenuation = compute_attenuation_linear(distance, 200.0, 1.0);
    assert!(
        (attenuation - 0.0).abs() < 0.001,
        "Attenuation at max distance should be 0.0"
    );
}

#[test]
fn test_spatial_audio_attenuation_diagonal() {
    let source_pos = Vec2::new(100.0, 100.0);
    let listener_pos = Vec2::new(0.0, 0.0);
    let distance = (source_pos - listener_pos).length(); // sqrt(100^2 + 100^2) = ~141.42

    let attenuation = compute_attenuation_linear(distance, 200.0, 1.0);
    // Expected: 1 - 141.42/200 = 1 - 0.707 = 0.293
    assert!(
        (attenuation - 0.293).abs() < 0.01,
        "Attenuation for diagonal distance"
    );
}
