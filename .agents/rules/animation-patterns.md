---
globs:
  - "**/animation*/**"
  - "**/sprite_animator/**"
  - "**/ffi/animation/**"
---

# Animation Subsystem Patterns

## Architecture

Three independent animation systems that compose together:

1. **SpriteAnimator** — frame-by-frame sprite sheet animation
2. **AnimationController** — state machine with parametric transitions
3. **AnimationLayerStack** — multi-layer blending

## SpriteAnimator

- Drives `Sprite.source_rect` based on `AnimationClip` frames
- `PlaybackMode::Loop` or `PlaybackMode::OneShot`
- `AnimationEvent` fires at specific frame indices with `EventPayload`
- Located in `ecs/components/sprite_animator/`
- FFI in `ffi/component_sprite_animator/`

## AnimationController

- State machine: named states, each holding an `AnimationClip`
- Transitions have conditions: `BoolEquals`, `FloatGreaterThan`, `FloatLessThan`
- Parameters set externally from game logic
- Blend duration for smooth crossfading between states
- Located in `ecs/components/animation_controller/`

## AnimationLayerStack

- Multiple layers with independent clips and weights
- `BlendMode::Override` — higher layers replace lower (scaled by weight)
- `BlendMode::Additive` — layer output added to layers below
- Located in `ecs/components/animation_layer/`

## Tweening

- Standalone tween functions in `ffi/animation/tween.rs`
- Easing: Linear, EaseIn, EaseOut, EaseInOut, EaseInBack, EaseOutBounce
- Create with `goud_tween_create()`, advance with `goud_tween_update()`

## FFI

- Animation FFI in `ffi/animation/` with separate modules: `controller.rs`, `events.rs`, `layer.rs`, `skeletal.rs`, `tween.rs`

## Testing

- Animation tests do not require GL context
- Test frame advancement, event firing, state transitions, and blend math independently
- Test files in `ecs/components/animation_controller/tests.rs` and `ecs/systems/animation/tests.rs`
