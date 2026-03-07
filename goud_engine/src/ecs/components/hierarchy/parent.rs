//! [`Parent`] component for pointing to a parent entity.

use crate::ecs::entity::Entity;
use crate::ecs::Component;
use std::fmt;

// =============================================================================
// Parent Component
// =============================================================================

/// Component indicating the parent entity of this entity.
///
/// When an entity has a `Parent` component, its transform (if any) is
/// considered to be in the parent's local coordinate space. The hierarchy
/// propagation system will compute the global transform by combining
/// the parent's global transform with this entity's local transform.
///
/// # Memory Layout
///
/// ```text
/// Parent (8 bytes total):
/// ┌────────────────┬────────────────┐
/// │  index (u32)   │ generation(u32)│  <- Entity
/// └────────────────┴────────────────┘
/// ```
///
/// # Example
///
/// ```
/// use goud_engine::ecs::Entity;
/// use goud_engine::ecs::components::Parent;
///
/// let parent_entity = Entity::new(0, 1);
/// let parent_component = Parent::new(parent_entity);
///
/// assert_eq!(parent_component.get(), parent_entity);
/// ```
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Parent {
    /// The parent entity.
    entity: Entity,
}

impl Parent {
    /// Creates a new Parent component pointing to the given entity.
    ///
    /// # Arguments
    ///
    /// * `parent` - The entity that should be this entity's parent
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Parent;
    ///
    /// let parent_entity = Entity::new(42, 1);
    /// let parent = Parent::new(parent_entity);
    /// ```
    #[inline]
    pub const fn new(parent: Entity) -> Self {
        Self { entity: parent }
    }

    /// Returns the parent entity.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Parent;
    ///
    /// let parent_entity = Entity::new(10, 2);
    /// let parent = Parent::new(parent_entity);
    ///
    /// assert_eq!(parent.get(), parent_entity);
    /// ```
    #[inline]
    pub const fn get(&self) -> Entity {
        self.entity
    }

    /// Sets the parent entity.
    ///
    /// This allows changing the parent without removing and re-adding the component.
    ///
    /// # Arguments
    ///
    /// * `parent` - The new parent entity
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Parent;
    ///
    /// let mut parent = Parent::new(Entity::new(0, 1));
    /// parent.set(Entity::new(5, 1));
    ///
    /// assert_eq!(parent.get(), Entity::new(5, 1));
    /// ```
    #[inline]
    pub fn set(&mut self, parent: Entity) {
        self.entity = parent;
    }
}

impl fmt::Debug for Parent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parent({:?})", self.entity)
    }
}

impl fmt::Display for Parent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parent({})", self.entity)
    }
}

impl Default for Parent {
    /// Returns a Parent with PLACEHOLDER entity.
    ///
    /// This is primarily for initialization purposes. A valid parent should
    /// be set before the entity is used in a hierarchy.
    #[inline]
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}

impl From<Entity> for Parent {
    #[inline]
    fn from(entity: Entity) -> Self {
        Self::new(entity)
    }
}

impl From<Parent> for Entity {
    #[inline]
    fn from(parent: Parent) -> Self {
        parent.entity
    }
}

// Implement Component trait
impl Component for Parent {}
