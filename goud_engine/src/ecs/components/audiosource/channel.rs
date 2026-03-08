//! Audio channel types for grouping and mixing audio sources.

/// Audio channel enumeration for audio mixing and grouping.
///
/// Channels allow you to group audio sources together for volume control,
/// filtering, and organization. Each audio source belongs to one channel.
///
/// # Built-in Channels
///
/// - **Music**: Background music tracks (typically looped)
/// - **SFX**: Sound effects (footsteps, impacts, UI clicks)
/// - **Voice**: Voice-overs, dialogue, speech
/// - **Ambience**: Ambient environment sounds (wind, rain, room tone)
/// - **UI**: User interface sounds (button clicks, menu navigation)
/// - **Custom**: User-defined channels (bits 5-31)
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::components::AudioChannel;
///
/// let music = AudioChannel::Music;
/// let sfx = AudioChannel::SFX;
/// let custom = AudioChannel::Custom(8); // Custom channel ID 8
///
/// assert_eq!(music.id(), 0);
/// assert_eq!(sfx.id(), 1);
/// assert_eq!(custom.id(), 8);
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AudioChannel {
    /// Background music tracks (channel ID: 0)
    Music = 0,
    /// Sound effects (channel ID: 1)
    SFX = 1,
    /// Voice-overs and dialogue (channel ID: 2)
    Voice = 2,
    /// Ambient environment sounds (channel ID: 3)
    Ambience = 3,
    /// User interface sounds (channel ID: 4)
    UI = 4,
    /// Custom channel (ID 5-31)
    Custom(u8),
}

impl AudioChannel {
    /// Constructs an `AudioChannel` from a numeric ID.
    ///
    /// Maps well-known IDs (0-4) to their named variants, and all others
    /// to `Custom(id)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::AudioChannel;
    ///
    /// assert_eq!(AudioChannel::from_id(0), AudioChannel::Music);
    /// assert_eq!(AudioChannel::from_id(1), AudioChannel::SFX);
    /// assert_eq!(AudioChannel::from_id(7), AudioChannel::Custom(7));
    /// ```
    pub fn from_id(id: u8) -> Self {
        match id {
            0 => AudioChannel::Music,
            1 => AudioChannel::SFX,
            2 => AudioChannel::Voice,
            3 => AudioChannel::Ambience,
            4 => AudioChannel::UI,
            other => AudioChannel::Custom(other),
        }
    }

    /// Returns the numeric channel ID (0-31).
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::AudioChannel;
    ///
    /// assert_eq!(AudioChannel::Music.id(), 0);
    /// assert_eq!(AudioChannel::SFX.id(), 1);
    /// assert_eq!(AudioChannel::Custom(10).id(), 10);
    /// ```
    pub fn id(&self) -> u8 {
        match self {
            AudioChannel::Music => 0,
            AudioChannel::SFX => 1,
            AudioChannel::Voice => 2,
            AudioChannel::Ambience => 3,
            AudioChannel::UI => 4,
            AudioChannel::Custom(id) => *id,
        }
    }

    /// Returns the channel name for debugging.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::AudioChannel;
    ///
    /// assert_eq!(AudioChannel::Music.name(), "Music");
    /// assert_eq!(AudioChannel::SFX.name(), "SFX");
    /// assert_eq!(AudioChannel::Custom(10).name(), "Custom(10)");
    /// ```
    pub fn name(&self) -> String {
        match self {
            AudioChannel::Music => "Music".to_string(),
            AudioChannel::SFX => "SFX".to_string(),
            AudioChannel::Voice => "Voice".to_string(),
            AudioChannel::Ambience => "Ambience".to_string(),
            AudioChannel::UI => "UI".to_string(),
            AudioChannel::Custom(id) => format!("Custom({})", id),
        }
    }
}

impl Default for AudioChannel {
    /// Returns `AudioChannel::SFX` as the default.
    fn default() -> Self {
        AudioChannel::SFX
    }
}

impl std::fmt::Display for AudioChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
