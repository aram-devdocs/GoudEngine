//! Spatial ECS components for audio listener/emitter entities.

use crate::ecs::Component;

/// Marks an entity as the active audio listener.
///
/// The spatial audio system uses this component plus either `Transform` or
/// `Transform2D` to update the audio listener position each frame.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AudioListener {
    /// Whether this listener is active.
    pub enabled: bool,
}

impl AudioListener {
    /// Creates an enabled listener.
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Sets whether this listener is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for AudioListener {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for AudioListener {}

/// Marks an entity as a spatial audio emitter.
///
/// This component is evaluated alongside `AudioSource` and an active sink ID.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AudioEmitter {
    /// Whether this emitter participates in spatial updates.
    pub enabled: bool,
    /// Maximum audible distance for attenuation.
    pub max_distance: f32,
    /// Attenuation rolloff exponent.
    pub rolloff: f32,
}

impl AudioEmitter {
    /// Creates a default, enabled emitter.
    pub fn new() -> Self {
        Self {
            enabled: true,
            max_distance: 100.0,
            rolloff: 1.0,
        }
    }

    /// Sets whether this emitter is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets maximum spatial distance (clamped to `>= 0.1`).
    pub fn with_max_distance(mut self, max_distance: f32) -> Self {
        self.max_distance = max_distance.max(0.1);
        self
    }

    /// Sets attenuation rolloff (clamped to `>= 0.01`).
    pub fn with_rolloff(mut self, rolloff: f32) -> Self {
        self.rolloff = rolloff.max(0.01);
        self
    }
}

impl Default for AudioEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for AudioEmitter {}
