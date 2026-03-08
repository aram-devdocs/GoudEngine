# Audio

Audio playback uses the Rodio library through the `AudioProvider` trait.

## Providers

| Provider | Backend | Notes |
|---|---|---|
| `RodioAudioProvider` | rodio | Full playback, spatial audio |
| `NullAudioProvider` | none | Silent fallback for headless testing |

## AudioSource Component

Attach `AudioSource` to an entity for audio playback.

| Field | Type | Default | Description |
|---|---|---|---|
| `volume` | `f32` | 1.0 | Playback volume (0.0–1.0) |
| `pitch` | `f32` | 1.0 | Speed multiplier |
| `looping` | `bool` | false | Loop playback |
| `channel` | `AudioChannel` | SFX | Mixing channel |
| `auto_play` | `bool` | false | Start playing on spawn |
| `spatial` | `bool` | false | Enable spatial positioning |
| `max_distance` | `f32` | 100.0 | Maximum audible distance |
| `attenuation` | `AttenuationModel` | InverseDistance | Distance falloff model |

## Channels

Audio is routed through named channels with independent volume control:

| Channel | Use case |
|---|---|
| `Music` | Background music |
| `SFX` | Sound effects |
| `Ambience` | Environmental audio |
| `UI` | Interface sounds |
| `Voice` | Dialogue and narration |

## Volume Control

Three levels of volume control, applied multiplicatively:

1. **Master volume** — global scaling for all audio
2. **Channel volume** — per-channel scaling (Music, SFX, etc.)
3. **Instance volume** — per-source scaling on individual playback instances

## Spatial Audio

When `spatial` is enabled on an `AudioSource`:

- Set listener position with `set_listener_position([x, y, z])`
- Source position is read from the entity's transform
- Two attenuation models: `InverseDistance` and `LinearDistance`
- `max_distance` controls the cutoff range

## FFI

Audio FFI functions are in `goud_engine/src/ffi/audio/`. Key functions:

- `goud_audio_play()` / `goud_audio_stop()`
- `goud_audio_pause()` / `goud_audio_resume()`
- `goud_audio_set_volume()` / `goud_audio_set_channel_volume()`
- `goud_audio_set_spatial_position()`
