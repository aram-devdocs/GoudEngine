# Animation

GoudEngine provides three animation systems: sprite animation, state machine controllers, and standalone tweening.

## Sprite Animation

### AnimationClip

Defines a sequence of frames from a sprite sheet.

| Field | Type | Description |
|---|---|---|
| `frames` | `Vec<Rect>` | Source rectangles in the sprite sheet |
| `frame_duration` | `f32` | Seconds per frame |
| `mode` | `PlaybackMode` | `Loop` or `OneShot` |
| `events` | `Vec<AnimationEvent>` | Events triggered at specific frames |

### SpriteAnimator

Attach `SpriteAnimator` to an entity to drive frame-by-frame animation. It updates the entity's `Sprite` source rectangle each frame based on the active clip.

| Field | Type | Description |
|---|---|---|
| `clip` | `AnimationClip` | Active animation clip |
| `current_frame` | `usize` | Current frame index |
| `elapsed` | `f32` | Time since last frame change |
| `playing` | `bool` | Whether animation is advancing |
| `finished` | `bool` | True when a OneShot clip reaches its last frame |

### Animation Events

`AnimationEvent` fires when the animator reaches a specific frame. Events carry an `EventPayload` for passing data to event handlers.

## Animation Controller

A state machine that manages transitions between animation states.

### States and Transitions

Each state holds an `AnimationClip`. Transitions between states are triggered by parameter conditions:

- `BoolEquals { param, value }` — transition when a bool parameter matches
- `FloatGreaterThan { param, threshold }` — transition when a float exceeds a threshold
- `FloatLessThan { param, threshold }` — transition when a float is below a threshold

Parameters are set externally (from game logic) and the controller evaluates transitions each frame.

### Blend Duration

Transitions can specify a `blend_duration` for smooth crossfading between states. During blending, both the outgoing and incoming clips contribute to the final frame.

## Animation Layers

`AnimationLayerStack` supports multiple animation layers with independent clips and blend weights.

| Field | Type | Description |
|---|---|---|
| `name` | `String` | Layer identifier |
| `weight` | `f32` | Blend weight (0.0–1.0) |
| `blend_mode` | `BlendMode` | `Override` or `Additive` |

- **Override**: higher layers replace lower layers, scaled by weight
- **Additive**: layer output is added to layers below

## Tweening

Standalone tween functions interpolate values over time with easing:

| Easing | Description |
|---|---|
| `Linear` | Constant speed |
| `EaseIn` | Starts slow, accelerates |
| `EaseOut` | Starts fast, decelerates |
| `EaseInOut` | Slow start and end |
| `EaseInBack` | Overshoots before settling |
| `EaseOutBounce` | Bounces at the end |

## FFI

Animation FFI is in `goud_engine/src/ffi/animation/`:

- `goud_animation_controller_*()` — state machine operations
- `goud_animation_layer_*()` — layer stack management
- `goud_tween_create()` / `goud_tween_update()` — tween lifecycle
