//! [`AudioFormat`] enum and related conversions.

use std::fmt;

/// Audio file format.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum AudioFormat {
    /// WAV (Waveform Audio File Format).
    #[default]
    Wav = 0,
    /// MP3 (MPEG-1 Audio Layer III).
    Mp3 = 1,
    /// OGG Vorbis.
    Ogg = 2,
    /// FLAC (Free Lossless Audio Codec).
    Flac = 3,
    /// Unknown format.
    Unknown = 255,
}

impl AudioFormat {
    /// Returns the file extension for this format.
    pub fn extension(self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Ogg => "ogg",
            AudioFormat::Flac => "flac",
            AudioFormat::Unknown => "",
        }
    }

    /// Returns the human-readable name of this format.
    pub fn name(self) -> &'static str {
        match self {
            AudioFormat::Wav => "WAV",
            AudioFormat::Mp3 => "MP3",
            AudioFormat::Ogg => "OGG Vorbis",
            AudioFormat::Flac => "FLAC",
            AudioFormat::Unknown => "Unknown",
        }
    }

    /// Returns the format corresponding to the given file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "wav" => AudioFormat::Wav,
            "mp3" => AudioFormat::Mp3,
            "ogg" => AudioFormat::Ogg,
            "flac" => AudioFormat::Flac,
            _ => AudioFormat::Unknown,
        }
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
