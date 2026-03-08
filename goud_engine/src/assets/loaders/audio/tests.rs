//! Tests for audio asset loading types.

use crate::assets::{asset::Asset, AssetLoader, AssetType, LoadContext};

use super::{
    asset::{AudioAsset, AudioData},
    format::AudioFormat,
    loader::AudioLoader,
    settings::AudioSettings,
};

/// Creates a valid WAV file in memory using hound.
#[cfg(feature = "native")]
fn create_test_wav_bytes(sample_rate: u32, channels: u16, num_samples: u32) -> Vec<u8> {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec).unwrap();
        for i in 0..num_samples {
            // Write a simple sine-ish pattern for each channel
            let sample = ((i as f32 * 0.1).sin() * 16000.0) as i16;
            for _ in 0..channels {
                writer.write_sample(sample).unwrap();
            }
        }
        writer.finalize().unwrap();
    }
    cursor.into_inner()
}

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
    assert_eq!(audio.duration_secs(), 0.0);
}

#[test]
fn test_audio_asset_new() {
    let data = vec![1, 2, 3, 4];
    let audio = AudioAsset::new(
        AudioData::InMemory(data.clone()),
        48000,
        1,
        AudioFormat::Mp3,
        2.5,
    );

    assert_eq!(audio.data(), Some(data.as_slice()));
    assert_eq!(audio.sample_rate(), 48000);
    assert_eq!(audio.channel_count(), 1);
    assert_eq!(audio.format(), AudioFormat::Mp3);
    assert_eq!(audio.size_bytes(), 4);
    assert!(!audio.is_empty());
    assert!(!audio.is_streaming());
    assert!(audio.file_path().is_none());
    assert_eq!(audio.duration_secs(), 2.5);
}

#[test]
fn test_audio_asset_is_mono() {
    let mono = AudioAsset::new(AudioData::InMemory(vec![]), 44100, 1, AudioFormat::Wav, 0.0);
    let stereo = AudioAsset::new(AudioData::InMemory(vec![]), 44100, 2, AudioFormat::Wav, 0.0);

    assert!(mono.is_mono());
    assert!(!mono.is_stereo());
    assert!(!stereo.is_mono());
    assert!(stereo.is_stereo());
}

#[test]
fn test_audio_asset_duration_secs() {
    let audio = AudioAsset::new(
        AudioData::InMemory(vec![1, 2, 3, 4]),
        44100,
        2,
        AudioFormat::Wav,
        1.5,
    );
    assert_eq!(audio.duration_secs(), 1.5);
}

#[test]
fn test_audio_asset_bits_per_sample() {
    let audio = AudioAsset::empty();
    assert_eq!(audio.bits_per_sample(), 16);
}

#[test]
fn test_audio_asset_clone() {
    let audio1 = AudioAsset::new(
        AudioData::InMemory(vec![1, 2, 3]),
        48000,
        1,
        AudioFormat::Ogg,
        0.5,
    );
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
fn test_audio_format_extension_and_name() {
    assert_eq!(AudioFormat::Wav.extension(), "wav");
    assert_eq!(AudioFormat::Wav.name(), "WAV");
    assert_eq!(AudioFormat::Mp3.extension(), "mp3");
    assert_eq!(AudioFormat::Mp3.name(), "MP3");
    assert_eq!(AudioFormat::Ogg.extension(), "ogg");
    assert_eq!(AudioFormat::Ogg.name(), "OGG Vorbis");
    assert_eq!(AudioFormat::Flac.extension(), "flac");
    assert_eq!(AudioFormat::Flac.name(), "FLAC");
    assert_eq!(AudioFormat::Unknown.extension(), "");
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
fn test_audio_format_traits() {
    assert_eq!(AudioFormat::default(), AudioFormat::Wav);
    assert_eq!(format!("{}", AudioFormat::Wav), "WAV");
    assert_eq!(format!("{}", AudioFormat::Ogg), "OGG Vorbis");
    let f = AudioFormat::Mp3;
    assert_eq!(f, f);
    assert_ne!(AudioFormat::Wav, AudioFormat::Mp3);
    assert!(format!("{:?}", AudioFormat::Ogg).contains("Ogg"));
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
    assert_eq!(settings.streaming_threshold, 1_048_576);
    assert!(!settings.force_streaming);
}

#[test]
fn test_audio_settings_custom() {
    let settings = AudioSettings {
        preload: false,
        target_sample_rate: 22050,
        target_channel_count: 1,
        streaming_threshold: 512_000,
        force_streaming: true,
    };

    assert!(!settings.preload);
    assert_eq!(settings.target_sample_rate, 22050);
    assert_eq!(settings.target_channel_count, 1);
    assert_eq!(settings.streaming_threshold, 512_000);
    assert!(settings.force_streaming);
}

#[test]
fn test_audio_settings_traits() {
    let s1 = AudioSettings::default();
    let s2 = s1.clone();
    assert_eq!(s1.preload, s2.preload);
    assert_eq!(s1.target_sample_rate, s2.target_sample_rate);
    assert!(format!("{:?}", s1).contains("AudioSettings"));
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
        ..AudioSettings::default()
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

#[cfg(feature = "native")]
#[test]
fn test_audio_loader_load_wav() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("test.wav".into());
    let bytes = create_test_wav_bytes(44100, 2, 4410);
    let settings = AudioSettings::default();

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert_eq!(audio.format(), AudioFormat::Wav);
    assert_eq!(audio.data(), Some(bytes.as_slice()));
    assert_eq!(audio.sample_rate(), 44100);
    assert_eq!(audio.channel_count(), 2);
}

/// Proves load_native() uses one code path for all formats: rodio::Decoder::new()
/// does content-based detection, so WAV tests cover the full decode logic.
/// The only per-format difference is the AudioFormat enum from the file extension.
#[cfg(feature = "native")]
#[test]
fn test_audio_loader_shared_decode_path() {
    let loader = AudioLoader::new();
    let bytes = create_test_wav_bytes(44100, 2, 4410);
    let settings = AudioSettings::default();

    let mut ctx_wav = LoadContext::new("file.wav".into());
    let mut ctx_ogg_ext = LoadContext::new("file.ogg".into());

    let audio_wav = loader.load(&bytes, &settings, &mut ctx_wav).unwrap();
    let audio_ogg = loader.load(&bytes, &settings, &mut ctx_ogg_ext).unwrap();

    // Metadata from actual content is identical regardless of extension.
    assert_eq!(audio_wav.sample_rate(), audio_ogg.sample_rate());
    assert_eq!(audio_wav.channel_count(), audio_ogg.channel_count());
    assert_eq!(audio_wav.duration_secs(), audio_ogg.duration_secs());
    assert_eq!(audio_wav.data().unwrap(), audio_ogg.data().unwrap());

    // Only the format enum (from extension) differs.
    assert_eq!(audio_wav.format(), AudioFormat::Wav);
    assert_eq!(audio_ogg.format(), AudioFormat::Ogg);
}

#[test]
fn test_audio_loader_load_unknown_extension() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("file.xyz".into());
    // Use valid WAV bytes even for unknown extension (rodio still decodes)
    #[cfg(feature = "native")]
    let bytes = create_test_wav_bytes(44100, 1, 100);
    #[cfg(not(feature = "native"))]
    let bytes = vec![1, 2, 3, 4];
    let settings = AudioSettings::default();

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert_eq!(audio.format(), AudioFormat::Unknown);
}

#[cfg(feature = "native")]
#[test]
fn test_audio_loader_invalid_bytes_returns_error() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("bad.wav".into());
    let bytes = vec![0xFF, 0xFE, 0x00, 0x01, 0x99];
    let settings = AudioSettings::default();

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.is_decode_failed());
}

#[cfg(feature = "native")]
#[test]
fn test_audio_loader_empty_bytes_returns_error() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("empty.wav".into());
    let bytes: Vec<u8> = vec![];
    let settings = AudioSettings::default();

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.is_decode_failed());
}

#[cfg(feature = "native")]
#[test]
fn test_audio_loader_extracts_wav_metadata() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("meta.wav".into());
    let bytes = create_test_wav_bytes(22050, 1, 2205);
    let settings = AudioSettings::default();

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert_eq!(audio.sample_rate(), 22050);
    assert_eq!(audio.channel_count(), 1);
    assert_eq!(audio.format(), AudioFormat::Wav);
}

#[cfg(feature = "native")]
#[test]
fn test_audio_loader_duration_calculated() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("dur.wav".into());
    // 44100 samples at 44100 Hz mono = 1.0 second
    let bytes = create_test_wav_bytes(44100, 1, 44100);
    let settings = AudioSettings::default();

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert!(audio.duration_secs() > 0.0, "duration should be positive");
    // WAV total_duration() should give us ~1.0 second
    let diff = (audio.duration_secs() - 1.0).abs();
    assert!(diff < 0.05, "expected ~1.0s, got {}", audio.duration_secs());
}

#[test]
fn test_audio_loader_traits() {
    let l1 = AudioLoader::new();
    let l2 = l1.clone();
    assert_eq!(l1.settings.preload, l2.settings.preload);
    assert!(format!("{:?}", l1).contains("AudioLoader"));
}

// ============================================================================
// AudioData / Streaming Tests
// ============================================================================

#[test]
fn test_audio_data_in_memory_variant() {
    let data = AudioData::InMemory(vec![10, 20, 30]);
    let asset = AudioAsset::new(data, 44100, 2, AudioFormat::Wav, 1.0);

    assert!(!asset.is_streaming());
    assert!(asset.file_path().is_none());
    assert_eq!(asset.data(), Some([10u8, 20, 30].as_slice()));
    assert_eq!(asset.size_bytes(), 3);
    assert!(!asset.is_empty());
}

#[test]
fn test_audio_data_streaming_variant() {
    let data = AudioData::Streaming {
        path: std::path::PathBuf::from("music/track.ogg"),
        size_bytes: 5_000_000,
    };
    let asset = AudioAsset::new(data, 44100, 2, AudioFormat::Ogg, 180.0);

    assert!(asset.is_streaming());
    assert_eq!(
        asset.file_path(),
        Some(std::path::Path::new("music/track.ogg"))
    );
    assert!(asset.data().is_none());
    assert_eq!(asset.size_bytes(), 5_000_000);
    assert!(!asset.is_empty());
}

#[test]
fn test_audio_data_streaming_empty() {
    let data = AudioData::Streaming {
        path: std::path::PathBuf::from("empty.wav"),
        size_bytes: 0,
    };
    let asset = AudioAsset::new(data, 44100, 2, AudioFormat::Wav, 0.0);

    assert!(asset.is_empty());
    assert_eq!(asset.size_bytes(), 0);
}

#[cfg(feature = "native")]
#[test]
fn test_audio_loader_force_streaming() {
    let settings = AudioSettings {
        force_streaming: true,
        ..AudioSettings::default()
    };
    let loader = AudioLoader::with_settings(settings.clone());
    let bytes = create_test_wav_bytes(44100, 1, 100);
    let mut context = LoadContext::new("sfx/click.wav".into());

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert!(audio.is_streaming());
    assert_eq!(
        audio.file_path(),
        Some(std::path::Path::new("sfx/click.wav"))
    );
    assert_eq!(audio.size_bytes(), bytes.len());
}

#[cfg(feature = "native")]
#[test]
fn test_audio_loader_threshold_triggers_streaming() {
    let settings = AudioSettings {
        streaming_threshold: 10, // Very low threshold
        ..AudioSettings::default()
    };
    let loader = AudioLoader::with_settings(settings.clone());
    let bytes = create_test_wav_bytes(44100, 1, 100); // >10 bytes
    let mut context = LoadContext::new("music/bg.wav".into());

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert!(audio.is_streaming());
}

#[cfg(feature = "native")]
#[test]
fn test_audio_loader_below_threshold_stays_in_memory() {
    let settings = AudioSettings {
        streaming_threshold: 100_000_000, // Very high threshold
        ..AudioSettings::default()
    };
    let loader = AudioLoader::with_settings(settings.clone());
    let bytes = create_test_wav_bytes(44100, 1, 100);
    let mut context = LoadContext::new("sfx/small.wav".into());

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert!(!audio.is_streaming());
    assert_eq!(audio.data(), Some(bytes.as_slice()));
}

// ============================================================================
// Thread Safety Tests
// ============================================================================

#[test]
fn test_audio_types_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AudioAsset>();
    assert_send_sync::<AudioFormat>();
    assert_send_sync::<AudioSettings>();
    assert_send_sync::<AudioLoader>();
}
