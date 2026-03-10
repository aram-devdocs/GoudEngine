//! Shared audio enums and descriptors.

/// Audio channel for volume grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioChannel {
    /// Master channel (affects all audio).
    Master,
    /// Music/soundtrack channel.
    Music,
    /// Sound effects channel.
    Effects,
    /// Voice/dialogue channel.
    Voice,
    /// Ambient/environmental channel.
    Ambient,
}

/// Configuration for playing a sound.
#[derive(Debug, Clone)]
pub struct PlayConfig {
    /// Volume (0.0 to 1.0).
    pub volume: f32,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f32,
    /// Whether to loop the sound.
    pub looping: bool,
    /// Audio channel for volume grouping.
    pub channel: AudioChannel,
    /// Optional spatial position as [x, y, z]. `None` for non-spatial audio.
    pub position: Option<[f32; 3]>,
}

impl Default for PlayConfig {
    fn default() -> Self {
        Self {
            volume: 1.0,
            speed: 1.0,
            looping: false,
            channel: AudioChannel::Effects,
            position: None,
        }
    }
}
