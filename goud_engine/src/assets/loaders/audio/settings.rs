//! [`AudioSettings`] for controlling audio load behavior.

/// Audio loading settings.
#[derive(Clone, Debug)]
pub struct AudioSettings {
    /// Whether to load the entire audio file into memory.
    /// When `false`, the loader may stream from disk for large files.
    pub preload: bool,
    /// Target sample rate (reserved for future resampling implementation).
    /// Currently has no effect — the audio asset will always use the sample rate
    /// of the encoded content.
    pub target_sample_rate: u32,
    /// Target channel count (reserved for future resampling implementation).
    /// Currently has no effect — the audio asset will always use the channel count
    /// of the encoded content (0 has no special meaning).
    pub target_channel_count: u16,
    /// File size threshold in bytes above which audio is streamed from disk
    /// instead of being loaded entirely into memory. Default: 1 MB.
    pub streaming_threshold: u64,
    /// When `true`, audio is always streamed from disk regardless of file size.
    pub force_streaming: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            preload: true,
            target_sample_rate: 0,          // Use original
            target_channel_count: 0,        // Use original
            streaming_threshold: 1_048_576, // 1 MB
            force_streaming: false,
        }
    }
}
