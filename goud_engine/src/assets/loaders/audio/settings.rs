//! [`AudioSettings`] for controlling audio load behavior.

/// Audio loading settings.
#[derive(Clone, Debug)]
pub struct AudioSettings {
    /// Whether to load the entire audio file into memory.
    /// Currently always true — streaming is not yet implemented.
    /// Setting this to `false` has no effect.
    pub preload: bool,
    /// Target sample rate (reserved for future resampling implementation).
    /// Currently has no effect — the audio asset will always use the sample rate
    /// of the encoded content.
    pub target_sample_rate: u32,
    /// Target channel count (reserved for future resampling implementation).
    /// Currently has no effect — the audio asset will always use the channel count
    /// of the encoded content (0 has no special meaning).
    pub target_channel_count: u16,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            preload: true,
            target_sample_rate: 0,   // Use original
            target_channel_count: 0, // Use original
        }
    }
}
