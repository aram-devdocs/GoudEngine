//! Tests for audio source components.

use crate::assets::AssetHandle;
use crate::ecs::Component;

use super::attenuation::AttenuationModel;
use super::channel::AudioChannel;
use super::source::AudioSource;
use super::spatial::{AudioEmitter, AudioListener};

// AudioChannel tests
#[test]
fn test_audio_channel_id() {
    assert_eq!(AudioChannel::Music.id(), 0);
    assert_eq!(AudioChannel::SFX.id(), 1);
    assert_eq!(AudioChannel::Voice.id(), 2);
    assert_eq!(AudioChannel::Ambience.id(), 3);
    assert_eq!(AudioChannel::UI.id(), 4);
    assert_eq!(AudioChannel::Custom(10).id(), 10);
}

#[test]
fn test_audio_channel_name() {
    assert_eq!(AudioChannel::Music.name(), "Music");
    assert_eq!(AudioChannel::SFX.name(), "SFX");
    assert_eq!(AudioChannel::Voice.name(), "Voice");
    assert_eq!(AudioChannel::Ambience.name(), "Ambience");
    assert_eq!(AudioChannel::UI.name(), "UI");
    assert_eq!(AudioChannel::Custom(10).name(), "Custom(10)");
}

#[test]
fn test_audio_channel_default() {
    assert_eq!(AudioChannel::default(), AudioChannel::SFX);
}

#[test]
fn test_audio_channel_display() {
    assert_eq!(format!("{}", AudioChannel::Music), "Music");
    assert_eq!(format!("{}", AudioChannel::Custom(5)), "Custom(5)");
}

#[test]
fn test_audio_channel_clone_copy() {
    let channel = AudioChannel::Music;
    let cloned = channel;
    assert_eq!(channel, cloned);
}

#[test]
fn test_audio_channel_eq_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(AudioChannel::Music);
    set.insert(AudioChannel::SFX);
    assert!(set.contains(&AudioChannel::Music));
    assert!(!set.contains(&AudioChannel::Voice));
}

// AttenuationModel tests
#[test]
fn test_attenuation_model_name() {
    assert_eq!(AttenuationModel::Linear.name(), "Linear");
    assert_eq!(AttenuationModel::InverseDistance.name(), "InverseDistance");
    assert_eq!(
        AttenuationModel::Exponential { rolloff: 2.0 }.name(),
        "Exponential"
    );
    assert_eq!(AttenuationModel::None.name(), "None");
}

#[test]
fn test_attenuation_linear() {
    let model = AttenuationModel::Linear;
    assert_eq!(model.compute_attenuation(0.0, 100.0), 1.0);
    assert_eq!(model.compute_attenuation(50.0, 100.0), 0.5);
    assert_eq!(model.compute_attenuation(100.0, 100.0), 0.0);
    assert_eq!(model.compute_attenuation(150.0, 100.0), 0.0);
}

#[test]
fn test_attenuation_inverse_distance() {
    let model = AttenuationModel::InverseDistance;
    assert_eq!(model.compute_attenuation(0.0, 100.0), 1.0);
    assert!((model.compute_attenuation(1.0, 100.0) - 0.5).abs() < 0.01);
    assert!(model.compute_attenuation(50.0, 100.0) < 1.0);
}

#[test]
fn test_attenuation_exponential() {
    let model = AttenuationModel::Exponential { rolloff: 2.0 };
    assert_eq!(model.compute_attenuation(0.0, 100.0), 1.0);
    assert_eq!(model.compute_attenuation(50.0, 100.0), 0.25); // (0.5)^2
    assert_eq!(model.compute_attenuation(100.0, 100.0), 0.0);
}

#[test]
fn test_attenuation_none() {
    let model = AttenuationModel::None;
    assert_eq!(model.compute_attenuation(0.0, 100.0), 1.0);
    assert_eq!(model.compute_attenuation(50.0, 100.0), 1.0);
    assert_eq!(model.compute_attenuation(1000.0, 100.0), 1.0);
}

#[test]
fn test_attenuation_default() {
    let model = AttenuationModel::default();
    assert_eq!(model.name(), "InverseDistance");
}

#[test]
fn test_attenuation_display() {
    assert_eq!(format!("{}", AttenuationModel::Linear), "Linear");
    assert_eq!(
        format!("{}", AttenuationModel::Exponential { rolloff: 3.0 }),
        "Exponential(rolloff=3)"
    );
}

// AudioSource tests
#[test]
fn test_audio_source_new() {
    let handle = AssetHandle::default();
    let source = AudioSource::new(handle);

    assert_eq!(source.playing, false);
    assert_eq!(source.looping, false);
    assert_eq!(source.volume, 1.0);
    assert_eq!(source.pitch, 1.0);
    assert_eq!(source.channel, AudioChannel::SFX);
    assert_eq!(source.auto_play, false);
    assert_eq!(source.spatial, false);
    assert_eq!(source.max_distance, 100.0);
    assert_eq!(source.attenuation.name(), "InverseDistance");
    assert_eq!(source.sink_id, None);
}

#[test]
fn test_audio_source_with_volume() {
    let handle = AssetHandle::default();
    let source = AudioSource::new(handle).with_volume(0.5);
    assert_eq!(source.volume, 0.5);

    // Test clamping
    let source = AudioSource::new(handle).with_volume(-0.1);
    assert_eq!(source.volume, 0.0);

    let source = AudioSource::new(handle).with_volume(1.5);
    assert_eq!(source.volume, 1.0);
}

#[test]
fn test_audio_source_with_pitch() {
    let handle = AssetHandle::default();
    let source = AudioSource::new(handle).with_pitch(1.5);
    assert_eq!(source.pitch, 1.5);

    // Test clamping
    let source = AudioSource::new(handle).with_pitch(0.1);
    assert_eq!(source.pitch, 0.5);

    let source = AudioSource::new(handle).with_pitch(3.0);
    assert_eq!(source.pitch, 2.0);
}

#[test]
fn test_audio_source_builder_pattern() {
    let handle = AssetHandle::default();
    let source = AudioSource::new(handle)
        .with_volume(0.8)
        .with_pitch(1.2)
        .with_looping(true)
        .with_channel(AudioChannel::Music)
        .with_auto_play(true)
        .with_spatial(true)
        .with_max_distance(200.0)
        .with_attenuation(AttenuationModel::Linear);

    assert_eq!(source.volume, 0.8);
    assert_eq!(source.pitch, 1.2);
    assert_eq!(source.looping, true);
    assert_eq!(source.channel, AudioChannel::Music);
    assert_eq!(source.auto_play, true);
    assert_eq!(source.spatial, true);
    assert_eq!(source.max_distance, 200.0);
    assert_eq!(source.attenuation.name(), "Linear");
}

#[test]
fn test_audio_source_play_pause_stop() {
    let handle = AssetHandle::default();
    let mut source = AudioSource::new(handle);

    assert_eq!(source.is_playing(), false);

    source.play();
    assert_eq!(source.is_playing(), true);

    source.pause();
    assert_eq!(source.is_playing(), false);

    source.play();
    assert_eq!(source.is_playing(), true);

    source.stop();
    assert_eq!(source.is_playing(), false);
    assert_eq!(source.sink_id, None);
}

#[test]
fn test_audio_source_is_spatial() {
    let handle = AssetHandle::default();
    let source = AudioSource::new(handle);
    assert_eq!(source.is_spatial(), false);

    let source = source.with_spatial(true);
    assert_eq!(source.is_spatial(), true);
}

#[test]
fn test_audio_source_sink_id() {
    let handle = AssetHandle::default();
    let mut source = AudioSource::new(handle);

    assert_eq!(source.has_sink(), false);
    assert_eq!(source.sink_id(), None);

    source.set_sink_id(Some(42));
    assert_eq!(source.has_sink(), true);
    assert_eq!(source.sink_id(), Some(42));

    source.set_sink_id(None);
    assert_eq!(source.has_sink(), false);
}

#[test]
fn test_audio_source_default() {
    let source = AudioSource::default();
    assert_eq!(source.playing, false);
    assert_eq!(source.volume, 1.0);
}

#[test]
fn test_audio_source_display() {
    let handle = AssetHandle::default();
    let source = AudioSource::new(handle)
        .with_volume(0.75)
        .with_pitch(1.5)
        .with_channel(AudioChannel::Music);

    let display = format!("{}", source);
    assert!(display.contains("playing=false"));
    assert!(display.contains("volume=0.75"));
    assert!(display.contains("pitch=1.50"));
    assert!(display.contains("channel=Music"));
}

#[test]
fn test_audio_source_component() {
    let handle = AssetHandle::default();
    let _source: Box<dyn Component> = Box::new(AudioSource::new(handle));
}

#[test]
fn test_audio_source_clone() {
    let handle = AssetHandle::default();
    let source = AudioSource::new(handle).with_volume(0.5);
    let cloned = source.clone();
    assert_eq!(cloned.volume, 0.5);
}

#[test]
fn test_audio_source_debug() {
    let handle = AssetHandle::default();
    let source = AudioSource::new(handle);
    let debug = format!("{:?}", source);
    assert!(debug.contains("AudioSource"));
}

#[test]
fn test_audio_listener_defaults() {
    let listener = AudioListener::new();
    assert!(listener.enabled);
}

#[test]
fn test_audio_listener_builder() {
    let listener = AudioListener::new().with_enabled(false);
    assert!(!listener.enabled);
}

#[test]
fn test_audio_emitter_defaults() {
    let emitter = AudioEmitter::new();
    assert!(emitter.enabled);
    assert_eq!(emitter.max_distance, 100.0);
    assert_eq!(emitter.rolloff, 1.0);
}

#[test]
fn test_audio_emitter_builder_and_clamp() {
    let emitter = AudioEmitter::new()
        .with_enabled(false)
        .with_max_distance(-10.0)
        .with_rolloff(0.0);

    assert!(!emitter.enabled);
    assert_eq!(emitter.max_distance, 0.1);
    assert_eq!(emitter.rolloff, 0.01);
}
