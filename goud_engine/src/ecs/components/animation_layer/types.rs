//! Types for the animation layer stack component.

use crate::ecs::components::sprite_animator::AnimationClip;
use crate::ecs::systems::animation::BlendMode;
use crate::ecs::Component;

/// A single animation layer with its own clip, weight, and blend mode.
///
/// Each layer maintains independent playback state (current frame, elapsed
/// time) and contributes to the final blended [`Rect`](crate::core::math::Rect)
/// according to its weight and blend mode.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnimationLayer {
    /// Human-readable name for this layer (e.g. "base", "upper_body").
    pub name: String,
    /// The animation clip driving this layer.
    pub clip: AnimationClip,
    /// Blend weight in `[0.0, 1.0]`. A weight of `0.0` means this layer
    /// has no effect on the final output.
    pub weight: f32,
    /// How this layer combines with layers below it.
    pub blend_mode: BlendMode,
    /// Index of the current frame in `clip.frames`.
    pub current_frame: usize,
    /// Accumulated time since the last frame advance.
    pub elapsed: f32,
    /// Whether this layer is currently playing.
    pub playing: bool,
    /// Whether a OneShot animation on this layer has completed.
    pub finished: bool,
}

/// ECS component that holds a stack of [`AnimationLayer`] instances.
///
/// The animation system iterates the layers in order, blending their
/// current frame rectangles to produce a single output rect applied
/// to the entity's [`Sprite`](crate::ecs::components::Sprite).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnimationLayerStack {
    /// Ordered list of animation layers (index 0 is the base layer).
    pub layers: Vec<AnimationLayer>,
}

impl Component for AnimationLayerStack {}
