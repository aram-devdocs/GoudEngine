//! Audio system FFI functions.
//!
//! Provides C-compatible functions for playing, pausing, stopping, and
//! controlling audio through the engine's `AudioManager` resource. Audio
//! data is passed as raw bytes (WAV, OGG, MP3, FLAC) and decoded at
//! playback time.
//!
//! Split across submodules:
//! - `playback`: play, play_on_channel, play_with_settings
//! - `controls`: stop, pause, resume, stop_all, volume, queries
//! - `spatial`: spatial play/update, listener/source positioning, per-player mix,
//!   timed crossfade, and additive mix helpers

/// FFI functions for playback control, channel/volume state, and playback state queries.
pub(crate) mod controls;
pub(crate) mod playback;
pub(crate) mod spatial;

/// Error sentinel for functions returning `i64` player IDs.
const ERR_AUDIO: i64 = -1;

/// Error sentinel for functions returning `i32` status codes.
const ERR_I32: i32 = -1;
