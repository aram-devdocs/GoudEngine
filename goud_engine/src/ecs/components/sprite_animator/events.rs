//! Animation event types for keyframe-triggered callbacks.
//!
//! This module defines the event types fired when an animation reaches
//! a configured keyframe. Systems can listen to these events to trigger
//! gameplay effects such as sound, particles, or state changes.
//!
//! # Usage
//!
//! ```rust
//! use goud_engine::ecs::components::sprite_animator::events::{
//!     AnimationEventFired, EventPayload,
//! };
//! use goud_engine::core::event::Events;
//!
//! fn handle_animation_events(events: &Events<AnimationEventFired>) {
//!     let mut reader = events.reader();
//!     for event in reader.read() {
//!         println!(
//!             "Animation event '{}' fired at frame {} for entity {:?}",
//!             event.event_name, event.frame_index, event.entity,
//!         );
//!     }
//! }
//! ```

use crate::ecs::Entity;

/// Payload data attached to an animation event.
///
/// Allows animation events to carry arbitrary typed data without
/// requiring downstream systems to parse strings.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EventPayload {
    /// No payload data.
    None,
    /// An integer payload.
    Int(i32),
    /// A floating-point payload.
    Float(f32),
    /// A string payload.
    String(String),
}

/// Configuration for an event attached to a specific animation frame.
///
/// Stored inside [`AnimationClip`](super::AnimationClip) to define
/// when and what events should fire during playback.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AnimationEvent {
    /// The frame index at which this event fires.
    pub frame_index: usize,
    /// A name identifying the event (e.g., "footstep", "attack_hit").
    pub name: String,
    /// Optional payload data for the event.
    pub payload: EventPayload,
}

impl AnimationEvent {
    /// Creates a new animation event configuration.
    #[must_use]
    pub fn new(frame_index: usize, name: impl Into<String>, payload: EventPayload) -> Self {
        Self {
            frame_index,
            name: name.into(),
            payload,
        }
    }
}

/// Event fired when an animation reaches a configured keyframe.
///
/// This event is emitted by the animation system when a
/// [`SpriteAnimator`](super::SpriteAnimator) advances past a frame
/// that has an [`AnimationEvent`] configured in its clip.
///
/// # Fields
///
/// - `entity`: The entity whose animation triggered the event
/// - `event_name`: The name from the `AnimationEvent` configuration
/// - `payload`: The payload data from the configuration
/// - `frame_index`: The frame index that triggered the event
#[derive(Debug, Clone)]
pub struct AnimationEventFired {
    /// The entity whose animation triggered this event.
    pub entity: Entity,
    /// The name identifying this event.
    pub event_name: String,
    /// The payload data attached to this event.
    pub payload: EventPayload,
    /// The frame index that triggered this event.
    pub frame_index: usize,
}

impl AnimationEventFired {
    /// Creates a new `AnimationEventFired` event.
    #[must_use]
    pub fn new(
        entity: Entity,
        event_name: String,
        payload: EventPayload,
        frame_index: usize,
    ) -> Self {
        Self {
            entity,
            event_name,
            payload,
            frame_index,
        }
    }

    /// Returns `true` if this event was fired for the given entity.
    #[must_use]
    pub fn involves(&self, entity: Entity) -> bool {
        self.entity == entity
    }
}
