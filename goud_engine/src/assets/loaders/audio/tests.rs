//! Tests for audio asset loading types.

use crate::assets::{asset::Asset, AssetLoader, AssetType, LoadContext};

use super::{asset::AudioAsset, format::AudioFormat, loader::AudioLoader, settings::AudioSettings};

// ============================================================================
// AudioAsset Tests
// ============================================================================

#[test]
fn test_audio_asset_empty() {
    let audio = AudioAsset::empty();
    assert!(audio.is_empty());
    assert_eq!(audio.sample_rate(), 44100);
    assert_eq!(audio.channel_count(), 2);
    assert_eq!(audio.format(), AudioFormat::Wav);
    assert_eq!(audio.size_bytes(), 0);
}

#[test]
fn test_audio_asset_new() {
    let data = vec![1, 2, 3, 4];
    let audio = AudioAsset::new(data.clone(), 48000, 1, AudioFormat::Mp3);

    assert_eq!(audio.data(), &data);
    assert_eq!(audio.sample_rate(), 48000);
    assert_eq!(audio.channel_count(), 1);
    assert_eq!(audio.format(), AudioFormat::Mp3);
    assert_eq!(audio.size_bytes(), 4);
    assert!(!audio.is_empty());
}

#[test]
fn test_audio_asset_is_mono() {
    let mono = AudioAsset::new(vec![], 44100, 1, AudioFormat::Wav);
    let stereo = AudioAsset::new(vec![], 44100, 2, AudioFormat::Wav);

    assert!(mono.is_mono());
    assert!(!mono.is_stereo());
    assert!(!stereo.is_mono());
    assert!(stereo.is_stereo());
}

#[test]
fn test_audio_asset_duration_secs() {
    // Stub always returns 0.0
    let audio = AudioAsset::new(vec![1, 2, 3, 4], 44100, 2, AudioFormat::Wav);
    assert_eq!(audio.duration_secs(), 0.0);
}

#[test]
fn test_audio_asset_clone() {
    let audio1 = AudioAsset::new(vec![1, 2, 3], 48000, 1, AudioFormat::Ogg);
    let audio2 = audio1.clone();

    assert_eq!(audio1, audio2);
}

#[test]
fn test_audio_asset_debug() {
    let audio = AudioAsset::empty();
    let debug_str = format!("{:?}", audio);
    assert!(debug_str.contains("AudioAsset"));
}

#[test]
fn test_audio_asset_trait() {
    assert_eq!(AudioAsset::asset_type_name(), "AudioAsset");
    assert_eq!(AudioAsset::asset_type(), AssetType::Audio);
    assert_eq!(AudioAsset::extensions(), &["wav", "mp3", "ogg", "flac"]);
}

// ============================================================================
// AudioFormat Tests
// ============================================================================

#[test]
fn test_audio_format_extension() {
    assert_eq!(AudioFormat::Wav.extension(), "wav");
    assert_eq!(AudioFormat::Mp3.extension(), "mp3");
    assert_eq!(AudioFormat::Ogg.extension(), "ogg");
    assert_eq!(AudioFormat::Flac.extension(), "flac");
    assert_eq!(AudioFormat::Unknown.extension(), "");
}

#[test]
fn test_audio_format_name() {
    assert_eq!(AudioFormat::Wav.name(), "WAV");
    assert_eq!(AudioFormat::Mp3.name(), "MP3");
    assert_eq!(AudioFormat::Ogg.name(), "OGG Vorbis");
    assert_eq!(AudioFormat::Flac.name(), "FLAC");
    assert_eq!(AudioFormat::Unknown.name(), "Unknown");
}

#[test]
fn test_audio_format_from_extension() {
    assert_eq!(AudioFormat::from_extension("wav"), AudioFormat::Wav);
    assert_eq!(AudioFormat::from_extension("WAV"), AudioFormat::Wav);
    assert_eq!(AudioFormat::from_extension("mp3"), AudioFormat::Mp3);
    assert_eq!(AudioFormat::from_extension("ogg"), AudioFormat::Ogg);
    assert_eq!(AudioFormat::from_extension("flac"), AudioFormat::Flac);
    assert_eq!(AudioFormat::from_extension("xyz"), AudioFormat::Unknown);
}

#[test]
fn test_audio_format_default() {
    assert_eq!(AudioFormat::default(), AudioFormat::Wav);
}

#[test]
fn test_audio_format_display() {
    assert_eq!(format!("{}", AudioFormat::Wav), "WAV");
    assert_eq!(format!("{}", AudioFormat::Mp3), "MP3");
    assert_eq!(format!("{}", AudioFormat::Ogg), "OGG Vorbis");
}

#[test]
fn test_audio_format_clone() {
    let format1 = AudioFormat::Mp3;
    let format2 = format1;
    assert_eq!(format1, format2);
}

#[test]
fn test_audio_format_eq() {
    assert_eq!(AudioFormat::Wav, AudioFormat::Wav);
    assert_ne!(AudioFormat::Wav, AudioFormat::Mp3);
}

#[test]
fn test_audio_format_debug() {
    let format = AudioFormat::Ogg;
    let debug_str = format!("{:?}", format);
    assert!(debug_str.contains("Ogg"));
}

// ============================================================================
// AudioSettings Tests
// ============================================================================

#[test]
fn test_audio_settings_default() {
    let settings = AudioSettings::default();
    assert!(settings.preload);
    assert_eq!(settings.target_sample_rate, 0);
    assert_eq!(settings.target_channel_count, 0);
}

#[test]
fn test_audio_settings_custom() {
    let settings = AudioSettings {
        preload: false,
        target_sample_rate: 22050,
        target_channel_count: 1,
    };

    assert!(!settings.preload);
    assert_eq!(settings.target_sample_rate, 22050);
    assert_eq!(settings.target_channel_count, 1);
}

#[test]
fn test_audio_settings_clone() {
    let settings1 = AudioSettings::default();
    let settings2 = settings1.clone();

    assert_eq!(settings1.preload, settings2.preload);
    assert_eq!(settings1.target_sample_rate, settings2.target_sample_rate);
}

#[test]
fn test_audio_settings_debug() {
    let settings = AudioSettings::default();
    let debug_str = format!("{:?}", settings);
    assert!(debug_str.contains("AudioSettings"));
}

// ============================================================================
// AudioLoader Tests
// ============================================================================

#[test]
fn test_audio_loader_new() {
    let loader = AudioLoader::new();
    assert!(loader.settings.preload);
}

#[test]
fn test_audio_loader_default() {
    let loader = AudioLoader::default();
    assert!(loader.settings.preload);
}

#[test]
fn test_audio_loader_with_settings() {
    let settings = AudioSettings {
        preload: false,
        target_sample_rate: 22050,
        target_channel_count: 1,
    };

    let loader = AudioLoader::with_settings(settings);
    assert!(!loader.settings.preload);
    assert_eq!(loader.settings.target_sample_rate, 22050);
}

#[test]
fn test_audio_loader_extensions() {
    let loader = AudioLoader::new();
    assert_eq!(loader.extensions(), &["wav", "mp3", "ogg", "flac"]);
}

#[test]
fn test_audio_loader_load_wav() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("test.wav".into());
    let bytes = vec![1, 2, 3, 4];
    let settings = AudioSettings::default();

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert_eq!(audio.format(), AudioFormat::Wav);
    assert_eq!(audio.data(), &bytes);
}

#[test]
fn test_audio_loader_load_mp3() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("sound.mp3".into());
    let bytes = vec![5, 6, 7, 8];
    let settings = AudioSettings::default();

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert_eq!(audio.format(), AudioFormat::Mp3);
}

#[test]
fn test_audio_loader_load_ogg() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("music.ogg".into());
    let bytes = vec![9, 10, 11, 12];
    let settings = AudioSettings::default();

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert_eq!(audio.format(), AudioFormat::Ogg);
}

#[test]
fn test_audio_loader_load_flac() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("track.flac".into());
    let bytes = vec![13, 14, 15, 16];
    let settings = AudioSettings::default();

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert_eq!(audio.format(), AudioFormat::Flac);
}

#[test]
fn test_audio_loader_load_unknown_extension() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("file.xyz".into());
    let bytes = vec![1, 2, 3, 4];
    let settings = AudioSettings::default();

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert_eq!(audio.format(), AudioFormat::Unknown);
}

#[test]
fn test_audio_loader_clone() {
    let loader1 = AudioLoader::new();
    let loader2 = loader1.clone();

    assert_eq!(loader1.settings.preload, loader2.settings.preload);
}

#[test]
fn test_audio_loader_debug() {
    let loader = AudioLoader::new();
    let debug_str = format!("{:?}", loader);
    assert!(debug_str.contains("AudioLoader"));
}

// ============================================================================
// Thread Safety Tests
// ============================================================================

#[test]
fn test_audio_asset_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<AudioAsset>();
}

#[test]
fn test_audio_asset_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<AudioAsset>();
}

#[test]
fn test_audio_format_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<AudioFormat>();
}

#[test]
fn test_audio_format_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<AudioFormat>();
}

#[test]
fn test_audio_settings_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<AudioSettings>();
}

#[test]
fn test_audio_settings_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<AudioSettings>();
}

#[test]
fn test_audio_loader_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<AudioLoader>();
}

#[test]
fn test_audio_loader_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<AudioLoader>();
}
