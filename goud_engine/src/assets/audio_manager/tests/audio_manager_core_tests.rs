use crate::assets::audio_manager::AudioManager;
use crate::assets::loaders::AudioAsset;
use crate::core::error::GoudError;

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
            Err(crate::core::error::GoudError::ResourceLoadFailed(_)) => {}
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
