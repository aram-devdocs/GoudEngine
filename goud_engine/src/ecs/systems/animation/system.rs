//! Core animation system logic.

use crate::core::event::Events;
use crate::ecs::components::animation_layer::AnimationLayerStack;
use crate::ecs::components::sprite::Sprite;
use crate::ecs::components::sprite_animator::events::AnimationEventFired;
use crate::ecs::components::sprite_animator::{PlaybackMode, SpriteAnimator};
use crate::ecs::systems::animation::blend::compute_blended_rect;
use crate::ecs::World;

/// Advances all sprite animations by `dt` seconds.
///
/// For each entity with a [`SpriteAnimator`] component:
/// 1. Skip if not playing or already finished.
/// 2. Accumulate `dt` into `elapsed`.
/// 3. Advance `current_frame` when `elapsed >= frame_duration`.
/// 4. In `Loop` mode, wrap back to frame 0 after the last frame.
/// 5. In `OneShot` mode, stop at the last frame and set `finished = true`.
/// 6. If the entity also has a [`Sprite`] component, apply the current
///    frame's `Rect` to `sprite.source_rect`.
///
/// # Example
///
/// ```rust,ignore
/// use goud_engine::ecs::systems::update_sprite_animations;
///
/// // In your game loop:
/// update_sprite_animations(&mut world, delta_time);
/// ```
pub fn update_sprite_animations(world: &mut World, dt: f32) {
    // Collect entities that have a SpriteAnimator component.
    let entities: Vec<_> = world
        .archetypes()
        .iter()
        .flat_map(|archetype| archetype.entities().iter().copied())
        .filter(|&entity| world.has::<SpriteAnimator>(entity))
        .collect();

    // Collect pending events across all entities, then dispatch after.
    let mut pending_events: Vec<AnimationEventFired> = Vec::new();

    for entity in entities {
        // Advance the animator.
        let frame_rect = {
            let Some(animator) = world.get_mut::<SpriteAnimator>(entity) else {
                continue;
            };

            if !animator.playing || animator.finished {
                continue;
            }

            let frame_count = animator.clip.frames.len();
            if frame_count == 0 {
                continue;
            }

            if animator.clip.frame_duration <= 0.0 {
                continue;
            }

            let prev_frame = animator.current_frame;
            animator.elapsed += dt;
            let mut frames_advanced: usize = 0;

            while animator.elapsed >= animator.clip.frame_duration {
                animator.elapsed -= animator.clip.frame_duration;
                animator.current_frame += 1;
                frames_advanced += 1;

                if animator.current_frame >= frame_count {
                    match animator.clip.mode {
                        PlaybackMode::Loop => {
                            animator.current_frame = 0;
                        }
                        PlaybackMode::OneShot => {
                            animator.current_frame = frame_count - 1;
                            animator.finished = true;
                            animator.playing = false;
                            animator.elapsed = 0.0;
                            break;
                        }
                    }
                }
            }

            // Collect events for frames that were crossed.
            collect_animation_events(
                entity,
                prev_frame,
                animator.current_frame,
                frame_count,
                frames_advanced,
                &animator.clip.events,
                &mut pending_events,
            );

            animator.previous_frame = animator.current_frame;
            animator.current_rect()
        };

        // Apply the current frame's rect to the Sprite, if present.
        if let Some(rect) = frame_rect {
            if let Some(sprite) = world.get_mut::<Sprite>(entity) {
                sprite.source_rect = Some(rect);
            }
        }
    }

    // Dispatch collected events to the Events resource, if present.
    if !pending_events.is_empty() {
        if let Some(events) = world.get_resource_mut::<Events<AnimationEventFired>>() {
            for event in pending_events {
                events.send(event);
            }
        }
    }

    // Second pass: advance and blend AnimationLayerStack components.
    update_animation_layer_stacks(world, dt);
}

/// Advances all [`AnimationLayerStack`] components and applies the blended
/// result to each entity's [`Sprite`] source rect.
fn update_animation_layer_stacks(world: &mut World, dt: f32) {
    let entities: Vec<_> = world
        .archetypes()
        .iter()
        .flat_map(|archetype| archetype.entities().iter().copied())
        .filter(|&entity| world.has::<AnimationLayerStack>(entity))
        .collect();

    for entity in entities {
        let blended = {
            let Some(stack) = world.get_mut::<AnimationLayerStack>(entity) else {
                continue;
            };

            // Advance each layer's animation independently.
            for layer in &mut stack.layers {
                if !layer.playing || layer.finished {
                    continue;
                }
                let frame_count = layer.clip.frames.len();
                if frame_count == 0 || layer.clip.frame_duration <= 0.0 {
                    continue;
                }
                layer.elapsed += dt;
                while layer.elapsed >= layer.clip.frame_duration {
                    layer.elapsed -= layer.clip.frame_duration;
                    layer.current_frame += 1;
                    if layer.current_frame >= frame_count {
                        match layer.clip.mode {
                            PlaybackMode::Loop => {
                                layer.current_frame = 0;
                            }
                            PlaybackMode::OneShot => {
                                layer.current_frame = frame_count - 1;
                                layer.finished = true;
                                layer.playing = false;
                                layer.elapsed = 0.0;
                                break;
                            }
                        }
                    }
                }
            }

            // Collect (rect, weight, blend_mode) tuples for blending.
            let layer_data: Vec<_> = stack
                .layers
                .iter()
                .filter_map(|layer| {
                    layer
                        .clip
                        .frames
                        .get(layer.current_frame)
                        .map(|&rect| (rect, layer.weight, layer.blend_mode))
                })
                .collect();

            compute_blended_rect(&layer_data)
        };

        if let Some(rect) = blended {
            if let Some(sprite) = world.get_mut::<Sprite>(entity) {
                sprite.source_rect = Some(rect);
            }
        }
    }
}

/// Collects animation events for frames crossed between `prev_frame`
/// and `current_frame`, handling loop wrap-around.
///
/// When `frames_advanced >= frame_count`, a full cycle occurred and
/// all configured events fire exactly once.
fn collect_animation_events(
    entity: crate::ecs::Entity,
    prev_frame: usize,
    current_frame: usize,
    frame_count: usize,
    frames_advanced: usize,
    clip_events: &[crate::ecs::components::sprite_animator::events::AnimationEvent],
    pending: &mut Vec<AnimationEventFired>,
) {
    if clip_events.is_empty() || frames_advanced == 0 {
        return;
    }

    for ev in clip_events {
        // If we advanced through a full cycle or more, every event fires.
        let should_fire = if frames_advanced >= frame_count {
            true
        } else if current_frame > prev_frame {
            // Normal forward advance: fire for frames in (prev_frame, current_frame].
            ev.frame_index > prev_frame && ev.frame_index <= current_frame
        } else if current_frame < prev_frame {
            // Loop wrap: fire for (prev_frame, last] and [0, current_frame].
            ev.frame_index > prev_frame || ev.frame_index <= current_frame
        } else {
            // Same frame after advancing means we wrapped exactly to
            // the starting frame. All frames in (prev_frame, ..., prev_frame]
            // were visited, so fire all events.
            true
        };

        if should_fire {
            pending.push(AnimationEventFired::new(
                entity,
                ev.name.clone(),
                ev.payload.clone(),
                ev.frame_index,
            ));
        }
    }
}
