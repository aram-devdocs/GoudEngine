//! Builder and accessor methods for [`AnimationLayerStack`] and [`AnimationLayer`].

use crate::ecs::components::sprite_animator::AnimationClip;
use crate::ecs::systems::animation::BlendMode;

use super::types::{AnimationLayer, AnimationLayerStack};

impl AnimationLayer {
    /// Creates a new animation layer with the given name, clip, and blend mode.
    ///
    /// The layer starts playing with a weight of `1.0`.
    #[inline]
    pub fn new(name: impl Into<String>, clip: AnimationClip, blend_mode: BlendMode) -> Self {
        Self {
            name: name.into(),
            clip,
            weight: 1.0,
            blend_mode,
            current_frame: 0,
            elapsed: 0.0,
            playing: true,
            finished: false,
        }
    }
}

impl AnimationLayerStack {
    /// Creates an empty layer stack.
    #[inline]
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Adds a layer to the stack (builder pattern).
    #[inline]
    pub fn with_layer(
        mut self,
        name: impl Into<String>,
        clip: AnimationClip,
        weight: f32,
        blend_mode: BlendMode,
    ) -> Self {
        let mut layer = AnimationLayer::new(name, clip, blend_mode);
        layer.weight = weight;
        self.layers.push(layer);
        self
    }

    /// Sets the weight of the layer at `index`.
    ///
    /// Does nothing if `index` is out of bounds.
    #[inline]
    pub fn set_layer_weight(&mut self, index: usize, weight: f32) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.weight = weight.clamp(0.0, 1.0);
        }
    }

    /// Returns a reference to the layer at `index`, if it exists.
    #[inline]
    pub fn get_layer(&self, index: usize) -> Option<&AnimationLayer> {
        self.layers.get(index)
    }

    /// Returns a mutable reference to the layer at `index`, if it exists.
    #[inline]
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut AnimationLayer> {
        self.layers.get_mut(index)
    }
}

impl Default for AnimationLayerStack {
    fn default() -> Self {
        Self::new()
    }
}
