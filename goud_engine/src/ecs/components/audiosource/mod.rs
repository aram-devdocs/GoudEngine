//! ## Play a Sound Effect
//!
//! ```
//! use goud_engine::ecs::{World, Component};
//! use goud_engine::ecs::components::{AudioSource, AudioChannel};
//! use goud_engine::assets::AssetHandle;
//! use goud_engine::assets::loaders::audio::AudioAsset;
//!
//! let mut world = World::new();
//!
//! // Assume we have a loaded audio asset
//! let audio_handle: AssetHandle<AudioAsset> = AssetHandle::default();
//!
//! // Spawn entity with audio source
//! let entity = world.spawn_empty();
//! world.insert(entity, AudioSource::new(audio_handle)
//!     .with_volume(0.8)
//!     .with_looping(false)
//!     .with_auto_play(true)
//!     .with_channel(AudioChannel::SFX));
//! ```
//!
//! ## Background Music with Looping
//!
//! ```
//! use goud_engine::ecs::components::{AudioSource, AudioChannel};
//! use goud_engine::assets::AssetHandle;
//! use goud_engine::assets::loaders::audio::AudioAsset;
//!
//! let audio_handle: AssetHandle<AudioAsset> = AssetHandle::default();
//!
//! let music = AudioSource::new(audio_handle)
//!     .with_volume(0.5)
//!     .with_looping(true)
//!     .with_auto_play(true)
//!     .with_channel(AudioChannel::Music);
//! ```
//!
//! ## Spatial Audio with Attenuation
//!
//! ```
//! use goud_engine::ecs::components::{AudioSource, AudioChannel, AttenuationModel};
//! use goud_engine::assets::AssetHandle;
//! use goud_engine::assets::loaders::audio::AudioAsset;
//!
//! let audio_handle: AssetHandle<AudioAsset> = AssetHandle::default();
//!
//! let spatial_audio = AudioSource::new(audio_handle)
//!     .with_volume(1.0)
//!     .with_spatial(true)
//!     .with_max_distance(100.0)
//!     .with_attenuation(AttenuationModel::InverseDistance)
//!     .with_channel(AudioChannel::Ambience);
//! ```

pub mod attenuation;
pub mod channel;
pub mod source;
pub mod spatial;

#[cfg(test)]
mod tests;

pub use attenuation::AttenuationModel;
pub use channel::AudioChannel;
pub use source::AudioSource;
pub use spatial::{AudioEmitter, AudioListener};
