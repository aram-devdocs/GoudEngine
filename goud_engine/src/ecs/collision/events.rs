//! Collision event types for entity collision notifications.
//!
//! This module defines events that are fired when collisions are detected
//! between entities. Systems can listen to these events to trigger gameplay
//! effects, audio, particles, damage, etc.
//!
//! # Event Types
//!
//! - [`CollisionStarted`]: Fired when two entities start colliding (first contact)
//! - [`CollisionEnded`]: Fired when two entities stop colliding (separation)
//!
//! # Usage Example
//!
//! ```rust
//! use goud_engine::ecs::collision::{CollisionStarted, CollisionEnded};
//! use goud_engine::core::event::Events;
//! use goud_engine::ecs::{Entity, World};
//!
//! // System that listens for collision events
//! fn handle_collisions(world: &World) {
//!     // In a real system, you'd use EventReader<CollisionStarted>
//!     // For now, this shows the concept
//! }
//! ```

use crate::ecs::collision::contact::Contact;
use crate::ecs::Entity;

/// Event fired when two entities start colliding.
///
/// This event is emitted when collision detection determines that two
/// entities have just made contact (were not colliding in the previous
/// frame, but are colliding now).
///
/// # Fields
///
/// - `entity_a`, `entity_b`: The two entities involved in the collision
/// - `contact`: Detailed contact information (point, normal, penetration)
///
/// # Usage
///
/// ```rust
/// use goud_engine::ecs::collision::events::CollisionStarted;
/// use goud_engine::core::event::{Events, EventReader};
///
/// fn handle_collision_start(events: &Events<CollisionStarted>) {
///     let mut reader = events.reader();
///     for event in reader.read() {
///         println!("Collision started between {:?} and {:?}", event.entity_a, event.entity_b);
///         println!("Contact point: {:?}", event.contact.point);
///         println!("Penetration depth: {}", event.contact.penetration);
///
///         // Trigger gameplay effects
///         // - Play collision sound
///         // - Spawn particle effects
///         // - Apply damage
///         // - Trigger game events
///     }
/// }
/// ```
///
/// # Entity Ordering
///
/// The order of `entity_a` and `entity_b` is consistent within a frame
/// but may vary between frames. To check if a specific entity is involved:
///
/// ```rust
/// # use goud_engine::ecs::collision::events::CollisionStarted;
/// # use goud_engine::ecs::Entity;
/// # let event = CollisionStarted {
/// #     entity_a: Entity::PLACEHOLDER,
/// #     entity_b: Entity::PLACEHOLDER,
/// #     contact: goud_engine::ecs::collision::Contact::default(),
/// # };
/// # let my_entity = Entity::PLACEHOLDER;
/// if event.involves(my_entity) {
///     let other = event.other_entity(my_entity);
///     println!("My entity collided with {:?}", other);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionStarted {
    /// First entity in the collision pair.
    pub entity_a: Entity,
    /// Second entity in the collision pair.
    pub entity_b: Entity,
    /// Contact information for this collision.
    pub contact: Contact,
}

impl CollisionStarted {
    /// Creates a new `CollisionStarted` event.
    #[must_use]
    pub fn new(entity_a: Entity, entity_b: Entity, contact: Contact) -> Self {
        Self {
            entity_a,
            entity_b,
            contact,
        }
    }

    /// Checks if the given entity is involved in this collision.
    #[must_use]
    pub fn involves(&self, entity: Entity) -> bool {
        self.entity_a == entity || self.entity_b == entity
    }

    /// Returns the other entity in the collision pair.
    ///
    /// # Returns
    ///
    /// - `Some(Entity)` if the given entity is involved in the collision
    /// - `None` if the given entity is not part of this collision
    ///
    /// # Example
    ///
    /// ```rust
    /// # use goud_engine::ecs::collision::events::CollisionStarted;
    /// # use goud_engine::ecs::Entity;
    /// # use goud_engine::ecs::collision::Contact;
    /// # let entity_a = Entity::from_bits(1);
    /// # let entity_b = Entity::from_bits(2);
    /// # let contact = Contact::default();
    /// let event = CollisionStarted::new(entity_a, entity_b, contact);
    ///
    /// assert_eq!(event.other_entity(entity_a), Some(entity_b));
    /// assert_eq!(event.other_entity(entity_b), Some(entity_a));
    /// assert_eq!(event.other_entity(Entity::from_bits(999)), None);
    /// ```
    #[must_use]
    pub fn other_entity(&self, entity: Entity) -> Option<Entity> {
        if self.entity_a == entity {
            Some(self.entity_b)
        } else if self.entity_b == entity {
            Some(self.entity_a)
        } else {
            None
        }
    }

    /// Returns an ordered pair of entities (lower entity ID first).
    ///
    /// This is useful for consistent lookups in hash maps or sets
    /// regardless of which entity was `entity_a` vs `entity_b`.
    #[must_use]
    pub fn ordered_pair(&self) -> (Entity, Entity) {
        if self.entity_a.to_bits() < self.entity_b.to_bits() {
            (self.entity_a, self.entity_b)
        } else {
            (self.entity_b, self.entity_a)
        }
    }
}

/// Event fired when two entities stop colliding.
///
/// This event is emitted when collision detection determines that two
/// entities that were colliding in the previous frame are no longer
/// in contact.
///
/// # Fields
///
/// - `entity_a`, `entity_b`: The two entities that separated
///
/// # Usage
///
/// ```rust
/// use goud_engine::ecs::collision::events::CollisionEnded;
/// use goud_engine::core::event::{Events, EventReader};
///
/// fn handle_collision_end(events: &Events<CollisionEnded>) {
///     let mut reader = events.reader();
///     for event in reader.read() {
///         println!("Collision ended between {:?} and {:?}", event.entity_a, event.entity_b);
///
///         // Stop gameplay effects
///         // - Stop looping collision sounds
///         // - Stop particle emitters
///         // - Update collision state tracking
///     }
/// }
/// ```
///
/// # Note on Contact Information
///
/// Unlike `CollisionStarted`, this event does not include contact
/// information since the entities are no longer touching. If you need
/// to track the last known contact point, store it when receiving
/// `CollisionStarted` events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollisionEnded {
    /// First entity in the collision pair.
    pub entity_a: Entity,
    /// Second entity in the collision pair.
    pub entity_b: Entity,
}

impl CollisionEnded {
    /// Creates a new `CollisionEnded` event.
    #[must_use]
    pub fn new(entity_a: Entity, entity_b: Entity) -> Self {
        Self { entity_a, entity_b }
    }

    /// Checks if the given entity is involved in this collision end.
    #[must_use]
    pub fn involves(&self, entity: Entity) -> bool {
        self.entity_a == entity || self.entity_b == entity
    }

    /// Returns the other entity in the collision pair.
    ///
    /// # Returns
    ///
    /// - `Some(Entity)` if the given entity was involved in the collision
    /// - `None` if the given entity was not part of this collision
    #[must_use]
    pub fn other_entity(&self, entity: Entity) -> Option<Entity> {
        if self.entity_a == entity {
            Some(self.entity_b)
        } else if self.entity_b == entity {
            Some(self.entity_a)
        } else {
            None
        }
    }

    /// Returns an ordered pair of entities (lower entity ID first).
    ///
    /// This is useful for consistent lookups in hash maps or sets
    /// regardless of which entity was `entity_a` vs `entity_b`.
    #[must_use]
    pub fn ordered_pair(&self) -> (Entity, Entity) {
        if self.entity_a.to_bits() < self.entity_b.to_bits() {
            (self.entity_a, self.entity_b)
        } else {
            (self.entity_b, self.entity_a)
        }
    }
}
