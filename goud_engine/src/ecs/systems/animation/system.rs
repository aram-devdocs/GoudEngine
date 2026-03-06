//! Core animation system logic.

use crate::ecs::components::sprite::Sprite;
use crate::ecs::components::sprite_animator::{PlaybackMode, SpriteAnimator};
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

            animator.elapsed += dt;

            while animator.elapsed >= animator.clip.frame_duration {
                animator.elapsed -= animator.clip.frame_duration;
                animator.current_frame += 1;

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

            animator.current_rect()
        };

        // Apply the current frame's rect to the Sprite, if present.
        if let Some(rect) = frame_rect {
            if let Some(sprite) = world.get_mut::<Sprite>(entity) {
                sprite.source_rect = Some(rect);
            }
        }
    }
}
