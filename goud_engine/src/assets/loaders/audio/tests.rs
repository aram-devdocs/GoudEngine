//! Tests for audio asset loading types.

use crate::assets::{asset::Asset, AssetLoader, AssetType, LoadContext};

use super::{asset::AudioAsset, format::AudioFormat, loader::AudioLoader, settings::AudioSettings};

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
    let audio = AudioAsset::new(data.clone(), 48000, 1, AudioFormat::Mp3, 2.5);

    assert_eq!(audio.data(), &data);
    assert_eq!(audio.sample_rate(), 48000);
    assert_eq!(audio.channel_count(), 1);
    assert_eq!(audio.format(), AudioFormat::Mp3);
    assert_eq!(audio.size_bytes(), 4);
    assert!(!audio.is_empty());
    assert_eq!(audio.duration_secs(), 2.5);
}

#[test]
fn test_audio_asset_is_mono() {
    let mono = AudioAsset::new(vec![], 44100, 1, AudioFormat::Wav, 0.0);
    let stereo = AudioAsset::new(vec![], 44100, 2, AudioFormat::Wav, 0.0);

    assert!(mono.is_mono());
    assert!(!mono.is_stereo());
    assert!(!stereo.is_mono());
    assert!(stereo.is_stereo());
}

#[test]
fn test_audio_asset_duration_secs() {
    let audio = AudioAsset::new(vec![1, 2, 3, 4], 44100, 2, AudioFormat::Wav, 1.5);
    assert_eq!(audio.duration_secs(), 1.5);
}

#[test]
fn test_audio_asset_bits_per_sample() {
    let audio = AudioAsset::empty();
    assert_eq!(audio.bits_per_sample(), 16);
}

#[test]
fn test_audio_asset_clone() {
    let audio1 = AudioAsset::new(vec![1, 2, 3], 48000, 1, AudioFormat::Ogg, 0.5);
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
    assert_eq!(audio.data(), &bytes);
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
    assert_eq!(audio_wav.data(), audio_ogg.data());

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

#[cfg(feature = "native")]
#[test]
fn test_audio_loader_settings_override_sample_rate() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("override.wav".into());
    let bytes = create_test_wav_bytes(44100, 2, 1000);
    let settings = AudioSettings {
        preload: true,
        target_sample_rate: 22050,
        target_channel_count: 0,
    };

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert_eq!(audio.sample_rate(), 22050);
    assert_eq!(audio.channel_count(), 2); // unchanged
}

#[cfg(feature = "native")]
#[test]
fn test_audio_loader_settings_override_channels() {
    let loader = AudioLoader::new();
    let mut context = LoadContext::new("override_ch.wav".into());
    let bytes = create_test_wav_bytes(44100, 2, 1000);
    let settings = AudioSettings {
        preload: true,
        target_sample_rate: 0,
        target_channel_count: 1,
    };

    let result = loader.load(&bytes, &settings, &mut context);
    assert!(result.is_ok());

    let audio = result.unwrap();
    assert_eq!(audio.sample_rate(), 44100); // unchanged
    assert_eq!(audio.channel_count(), 1);
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
fn test_audio_types_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AudioAsset>();
    assert_send_sync::<AudioFormat>();
    assert_send_sync::<AudioSettings>();
    assert_send_sync::<AudioLoader>();
}
