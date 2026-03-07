//! Animation controller system logic.

use crate::ecs::components::animation_controller::{
    AnimParam, AnimationController, TransitionCondition,
};
use crate::ecs::components::sprite_animator::SpriteAnimator;
use crate::ecs::World;

/// Evaluates whether a single transition condition is satisfied.
fn evaluate_condition(condition: &TransitionCondition, controller: &AnimationController) -> bool {
    match condition {
        TransitionCondition::BoolEquals { param, value } => {
            matches!(
                controller.parameters.get(param),
                Some(AnimParam::Bool(v)) if *v == *value
            )
        }
        TransitionCondition::FloatGreaterThan { param, threshold } => {
            matches!(
                controller.parameters.get(param),
                Some(AnimParam::Float(v)) if *v > *threshold
            )
        }
        TransitionCondition::FloatLessThan { param, threshold } => {
            matches!(
                controller.parameters.get(param),
                Some(AnimParam::Float(v)) if *v < *threshold
            )
        }
    }
}

/// Advances all animation controllers by `dt` seconds.
///
/// For each entity with both an [`AnimationController`] and a
/// [`SpriteAnimator`]:
///
/// 1. If a transition is in progress, advance it. When complete, switch
///    to the target state and update the animator's clip.
/// 2. If no transition is active, evaluate all transitions from the
///    current state. Start a transition if all conditions are met.
/// 3. Ensure the animator's clip matches the controller's current state.
///
/// # Example
///
/// ```rust,ignore
/// use goud_engine::ecs::systems::update_animation_controllers;
///
/// // In your game loop:
/// update_animation_controllers(&mut world, delta_time);
/// ```
pub fn update_animation_controllers(world: &mut World, dt: f32) {
    let entities: Vec<_> = world
        .archetypes()
        .iter()
        .flat_map(|archetype| archetype.entities().iter().copied())
        .filter(|&entity| {
            world.has::<AnimationController>(entity) && world.has::<SpriteAnimator>(entity)
        })
        .collect();

    for entity in entities {
        // Phase 1: Read controller state and decide what action to take.
        let action = {
            let Some(controller) = world.get::<AnimationController>(entity) else {
                continue;
            };

            if let Some(ref progress) = controller.transition_progress {
                let new_elapsed = progress.elapsed + dt;
                if new_elapsed >= progress.duration {
                    // Transition complete
                    Action::CompleteTransition {
                        to_state: progress.to_state.clone(),
                    }
                } else {
                    // Advance transition
                    Action::AdvanceTransition { new_elapsed }
                }
            } else {
                // Check for new transitions
                find_matching_transition(controller)
            }
        };

        // Phase 2: Apply the action with mutable access.
        match action {
            Action::None => {}
            Action::AdvanceTransition { new_elapsed } => {
                if let Some(ctrl) = world.get_mut::<AnimationController>(entity) {
                    if let Some(ref mut progress) = ctrl.transition_progress {
                        progress.elapsed = new_elapsed;
                    }
                }
            }
            Action::CompleteTransition { to_state } => {
                let new_clip = {
                    let Some(ctrl) = world.get_mut::<AnimationController>(entity) else {
                        continue;
                    };
                    ctrl.current_state = to_state;
                    ctrl.transition_progress = None;
                    ctrl.current_clip().cloned()
                };
                if let Some(clip) = new_clip {
                    if let Some(animator) = world.get_mut::<SpriteAnimator>(entity) {
                        animator.clip = clip;
                        animator.current_frame = 0;
                        animator.elapsed = 0.0;
                    }
                }
            }
            Action::StartTransition {
                from_state,
                to_state,
                duration,
            } => {
                if let Some(ctrl) = world.get_mut::<AnimationController>(entity) {
                    ctrl.transition_progress = Some(
                        crate::ecs::components::animation_controller::TransitionProgress {
                            from_state,
                            to_state,
                            elapsed: 0.0,
                            duration,
                        },
                    );
                }
            }
        }

        // Phase 3: Sync animator clip with current state (if no transition).
        {
            let current_clip = {
                let Some(ctrl) = world.get::<AnimationController>(entity) else {
                    continue;
                };
                if ctrl.transition_progress.is_some() {
                    None
                } else {
                    ctrl.current_clip().cloned()
                }
            };
            if let Some(clip) = current_clip {
                if let Some(animator) = world.get_mut::<SpriteAnimator>(entity) {
                    if animator.clip != clip {
                        animator.clip = clip;
                        animator.current_frame = 0;
                        animator.elapsed = 0.0;
                    }
                }
            }
        }
    }
}

/// Internal action enum to separate read and write phases.
enum Action {
    None,
    AdvanceTransition {
        new_elapsed: f32,
    },
    CompleteTransition {
        to_state: String,
    },
    StartTransition {
        from_state: String,
        to_state: String,
        duration: f32,
    },
}

/// Finds the first transition whose conditions all match.
fn find_matching_transition(controller: &AnimationController) -> Action {
    for transition in &controller.transitions {
        if transition.from != controller.current_state {
            continue;
        }
        let all_met = transition
            .conditions
            .iter()
            .all(|c| evaluate_condition(c, controller));
        if all_met {
            return Action::StartTransition {
                from_state: transition.from.clone(),
                to_state: transition.to.clone(),
                duration: transition.blend_duration,
            };
        }
    }
    Action::None
}
