---
globs:
  - "**/audio*/**"
  - "**/audiosource/**"
  - "**/audio_manager/**"
  - "**/ffi/audio/**"
---

# Audio Subsystem Patterns

## Architecture

- Audio uses the provider pattern: `AudioProvider` trait
- `RodioAudioProvider` wraps rodio directly (Layer 1 dependency)
- `NullAudioProvider` provides silent fallback for testing
- Layer 2 bridges between the asset system and the audio provider via `audio_manager/`

## Components

- `AudioSource` — clip handle, playback state, volume, pitch, channel, spatial settings
- Five channels: Music, SFX, Ambience, UI, Voice — each with independent volume
- Spatial audio: listener position + source position + attenuation model

## Volume Model

Three levels applied multiplicatively: master → channel → instance

## Attenuation

- `InverseDistance` — realistic falloff (default)
- `LinearDistance` — uniform falloff to max_distance
- Defined in `ecs/components/audiosource/attenuation.rs`

## FFI

- Audio FFI in `ffi/audio/` with controls and playback modules
- Provider stored globally per context ID
- `audio_update()` called per frame for stream refill and listener sync

## Testing

- Audio tests can use `NullAudioProvider` — no sound device required
- Test spatial math independently of playback
- Test files in `assets/audio_manager/tests.rs` and `assets/loaders/audio/tests.rs`
