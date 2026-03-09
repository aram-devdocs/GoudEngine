use crate::assets::audio_manager::AudioManager;
use crate::assets::loaders::AudioAsset;
use crate::ecs::components::AudioChannel;

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
