//! Hierarchy components for parent-child entity relationships.
//!
//! This module provides components for building entity hierarchies, enabling
//! parent-child relationships between entities. This is essential for:
//!
//! - Scene graphs where child transforms are relative to parent transforms
//! - UI layouts where child widgets are positioned within parent containers
//! - Game object grouping where destroying a parent also destroys children
//!
//! # Components
//!
//! - [`Parent`]: Points to the parent entity (stored on child entities)
//! - [`Children`]: Lists all child entities (stored on parent entities)
//! - [`Name`]: Human-readable name for debugging and editor integration
//!
//! # Design Philosophy
//!
//! The hierarchy system uses a bidirectional pointer approach:
//! - Children point up to their parent via [`Parent`]
//! - Parents point down to their children via [`Children`]
//!
//! This redundancy enables efficient traversal in both directions and
//! matches the design of major engines like Bevy, Unity, and Godot.
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::{World, Entity};
//! use goud_engine::ecs::components::{Parent, Children, Name};
//!
//! let mut world = World::new();
//!
//! // Create parent entity
//! let parent = world.spawn_empty();
//! world.insert(parent, Name::new("Player"));
//! world.insert(parent, Children::new());
//!
//! // Create child entity
//! let child = world.spawn_empty();
//! world.insert(child, Name::new("Weapon"));
//! world.insert(child, Parent::new(parent));
//!
//! // Update parent's children list (normally done by a hierarchy system)
//! if let Some(children) = world.get_mut::<Children>(parent) {
//!     children.push(child);
//! }
//! ```
//!
//! # FFI Safety
//!
//! All hierarchy components are `#[repr(C)]` where applicable for FFI
//! compatibility. String data uses Rust's `String` type internally but
//! provides FFI-safe accessor methods.
//!
//! # Consistency
//!
//! **Important**: The hierarchy must be kept consistent. When adding a child:
//! 1. Add `Parent` component to the child pointing to parent
//! 2. Add/update `Children` component on parent to include child
//!
//! When removing a child:
//! 1. Remove `Parent` component from child (or update to new parent)
//! 2. Remove child from parent's `Children` list
//!
//! A hierarchy maintenance system should be used to ensure consistency.

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
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

// =============================================================================
// Children Component
// =============================================================================

/// Component containing a list of child entities.
///
/// This component stores references to all immediate children of an entity.
/// The order of children is preserved and can be significant for rendering
/// order or other order-dependent operations.
///
/// # Capacity and Performance
///
/// Internally uses a `Vec<Entity>`, so:
/// - Adding children is O(1) amortized
/// - Removing children is O(n) where n is the number of children
/// - Iteration is cache-friendly
///
/// For entities with many children, consider using `with_capacity` to
/// pre-allocate memory.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::Entity;
/// use goud_engine::ecs::components::Children;
///
/// let mut children = Children::new();
///
/// let child1 = Entity::new(1, 1);
/// let child2 = Entity::new(2, 1);
///
/// children.push(child1);
/// children.push(child2);
///
/// assert_eq!(children.len(), 2);
/// assert!(children.contains(child1));
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Children {
    /// The list of child entities in order.
    children: Vec<Entity>,
}

impl Children {
    /// Creates an empty Children component.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Children;
    ///
    /// let children = Children::new();
    /// assert!(children.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Creates a Children component with pre-allocated capacity.
    ///
    /// Use this when you know approximately how many children the entity will have.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of children to pre-allocate space for
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Children;
    ///
    /// let children = Children::with_capacity(100);
    /// assert!(children.is_empty()); // No children yet
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            children: Vec::with_capacity(capacity),
        }
    }

    /// Creates a Children component from a slice of entities.
    ///
    /// # Arguments
    ///
    /// * `children` - Slice of child entities
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let entities = vec![Entity::new(1, 1), Entity::new(2, 1)];
    /// let children = Children::from_slice(&entities);
    ///
    /// assert_eq!(children.len(), 2);
    /// ```
    #[inline]
    pub fn from_slice(children: &[Entity]) -> Self {
        Self {
            children: children.to_vec(),
        }
    }

    /// Returns the number of children.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    /// assert_eq!(children.len(), 2);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns `true` if there are no children.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Children;
    ///
    /// let children = Children::new();
    /// assert!(children.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Adds a child entity to the end of the children list.
    ///
    /// # Arguments
    ///
    /// * `child` - The child entity to add
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let mut children = Children::new();
    /// children.push(Entity::new(1, 1));
    /// assert_eq!(children.len(), 1);
    /// ```
    #[inline]
    pub fn push(&mut self, child: Entity) {
        self.children.push(child);
    }

    /// Inserts a child entity at a specific index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to insert at
    /// * `child` - The child entity to insert
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let mut children = Children::new();
    /// children.push(Entity::new(1, 1));
    /// children.push(Entity::new(3, 1));
    /// children.insert(1, Entity::new(2, 1)); // Insert at index 1
    ///
    /// assert_eq!(children.get(1), Some(Entity::new(2, 1)));
    /// ```
    #[inline]
    pub fn insert(&mut self, index: usize, child: Entity) {
        self.children.insert(index, child);
    }

    /// Removes and returns the child at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the child to remove
    ///
    /// # Returns
    ///
    /// The removed child entity.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let mut children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    /// let removed = children.remove(0);
    ///
    /// assert_eq!(removed, Entity::new(1, 1));
    /// assert_eq!(children.len(), 1);
    /// ```
    #[inline]
    pub fn remove(&mut self, index: usize) -> Entity {
        self.children.remove(index)
    }

    /// Removes a child entity if it exists, preserving order.
    ///
    /// # Arguments
    ///
    /// * `child` - The child entity to remove
    ///
    /// # Returns
    ///
    /// `true` if the child was found and removed, `false` otherwise.
    ///
    /// # Performance
    ///
    /// This is O(n) as it must search for the child and shift elements.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let mut children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    ///
    /// assert!(children.remove_child(Entity::new(1, 1)));
    /// assert!(!children.remove_child(Entity::new(1, 1))); // Already removed
    /// ```
    pub fn remove_child(&mut self, child: Entity) -> bool {
        if let Some(index) = self.children.iter().position(|&e| e == child) {
            self.children.remove(index);
            true
        } else {
            false
        }
    }

    /// Removes a child entity using swap-remove (faster but doesn't preserve order).
    ///
    /// # Arguments
    ///
    /// * `child` - The child entity to remove
    ///
    /// # Returns
    ///
    /// `true` if the child was found and removed, `false` otherwise.
    ///
    /// # Performance
    ///
    /// This is O(n) for the search but O(1) for the actual removal.
    /// Use this when child order doesn't matter.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let mut children = Children::from_slice(&[
    ///     Entity::new(1, 1),
    ///     Entity::new(2, 1),
    ///     Entity::new(3, 1),
    /// ]);
    ///
    /// children.swap_remove_child(Entity::new(1, 1));
    /// // Order may have changed, but length is now 2
    /// assert_eq!(children.len(), 2);
    /// ```
    pub fn swap_remove_child(&mut self, child: Entity) -> bool {
        if let Some(index) = self.children.iter().position(|&e| e == child) {
            self.children.swap_remove(index);
            true
        } else {
            false
        }
    }

    /// Returns `true` if the given entity is a child.
    ///
    /// # Arguments
    ///
    /// * `child` - The entity to check
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let children = Children::from_slice(&[Entity::new(1, 1)]);
    ///
    /// assert!(children.contains(Entity::new(1, 1)));
    /// assert!(!children.contains(Entity::new(2, 1)));
    /// ```
    #[inline]
    pub fn contains(&self, child: Entity) -> bool {
        self.children.contains(&child)
    }

    /// Returns the child at the given index, if any.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the child to get
    ///
    /// # Returns
    ///
    /// `Some(entity)` if the index is valid, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    ///
    /// assert_eq!(children.get(0), Some(Entity::new(1, 1)));
    /// assert_eq!(children.get(10), None);
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> Option<Entity> {
        self.children.get(index).copied()
    }

    /// Returns the first child, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    /// assert_eq!(children.first(), Some(Entity::new(1, 1)));
    ///
    /// let empty = Children::new();
    /// assert_eq!(empty.first(), None);
    /// ```
    #[inline]
    pub fn first(&self) -> Option<Entity> {
        self.children.first().copied()
    }

    /// Returns the last child, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    /// assert_eq!(children.last(), Some(Entity::new(2, 1)));
    /// ```
    #[inline]
    pub fn last(&self) -> Option<Entity> {
        self.children.last().copied()
    }

    /// Returns an iterator over the children.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    ///
    /// for child in children.iter() {
    ///     println!("Child: {:?}", child);
    /// }
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.children.iter()
    }

    /// Returns the index of a child entity, if it exists.
    ///
    /// # Arguments
    ///
    /// * `child` - The entity to find
    ///
    /// # Returns
    ///
    /// `Some(index)` if found, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    ///
    /// assert_eq!(children.index_of(Entity::new(2, 1)), Some(1));
    /// assert_eq!(children.index_of(Entity::new(99, 1)), None);
    /// ```
    #[inline]
    pub fn index_of(&self, child: Entity) -> Option<usize> {
        self.children.iter().position(|&e| e == child)
    }

    /// Removes all children.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let mut children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    /// children.clear();
    ///
    /// assert!(children.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.children.clear();
    }

    /// Returns the children as a slice.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
    /// let slice = children.as_slice();
    ///
    /// assert_eq!(slice.len(), 2);
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[Entity] {
        &self.children
    }

    /// Retains only the children that satisfy the predicate.
    ///
    /// # Arguments
    ///
    /// * `f` - The predicate function
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let mut children = Children::from_slice(&[
    ///     Entity::new(1, 1),
    ///     Entity::new(2, 1),
    ///     Entity::new(3, 1),
    /// ]);
    ///
    /// // Keep only entities with even indices
    /// children.retain(|e| e.index() % 2 == 0);
    ///
    /// assert_eq!(children.len(), 1);
    /// assert!(children.contains(Entity::new(2, 1)));
    /// ```
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Entity) -> bool,
    {
        self.children.retain(f);
    }

    /// Sorts children by their entity index.
    ///
    /// This can be useful for deterministic ordering.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let mut children = Children::from_slice(&[
    ///     Entity::new(3, 1),
    ///     Entity::new(1, 1),
    ///     Entity::new(2, 1),
    /// ]);
    ///
    /// children.sort_by_index();
    ///
    /// assert_eq!(children.get(0), Some(Entity::new(1, 1)));
    /// assert_eq!(children.get(1), Some(Entity::new(2, 1)));
    /// assert_eq!(children.get(2), Some(Entity::new(3, 1)));
    /// ```
    pub fn sort_by_index(&mut self) {
        self.children.sort_by_key(|e| e.index());
    }

    /// Sorts children using a custom comparison function.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::Entity;
    /// use goud_engine::ecs::components::Children;
    ///
    /// let mut children = Children::from_slice(&[
    ///     Entity::new(1, 1),
    ///     Entity::new(2, 1),
    ///     Entity::new(3, 1),
    /// ]);
    ///
    /// // Sort in reverse order
    /// children.sort_by(|a, b| b.index().cmp(&a.index()));
    ///
    /// assert_eq!(children.get(0), Some(Entity::new(3, 1)));
    /// ```
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&Entity, &Entity) -> std::cmp::Ordering,
    {
        self.children.sort_by(compare);
    }
}

impl Default for Children {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Children {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Children")
            .field("count", &self.children.len())
            .field("children", &self.children)
            .finish()
    }
}

impl fmt::Display for Children {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Children({})", self.children.len())
    }
}

impl<'a> IntoIterator for &'a Children {
    type Item = &'a Entity;
    type IntoIter = std::slice::Iter<'a, Entity>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.children.iter()
    }
}

impl IntoIterator for Children {
    type Item = Entity;
    type IntoIter = std::vec::IntoIter<Entity>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}

impl From<Vec<Entity>> for Children {
    #[inline]
    fn from(children: Vec<Entity>) -> Self {
        Self { children }
    }
}

impl From<&[Entity]> for Children {
    #[inline]
    fn from(children: &[Entity]) -> Self {
        Self::from_slice(children)
    }
}

impl From<Children> for Vec<Entity> {
    #[inline]
    fn from(children: Children) -> Self {
        children.children
    }
}

// Implement Component trait
impl Component for Children {}

// =============================================================================
// Name Component
// =============================================================================

/// Component providing a human-readable name for an entity.
///
/// Names are useful for:
/// - Debugging and logging
/// - Editor integration and scene hierarchies
/// - Scripting references (find entity by name)
/// - UI display
///
/// # Performance
///
/// Names use Rust's `String` type internally. For performance-critical code,
/// consider using entity IDs directly rather than string lookups.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::components::Name;
///
/// let name = Name::new("Player");
/// assert_eq!(name.as_str(), "Player");
///
/// let mut name = Name::new("Enemy");
/// name.set("Boss");
/// assert_eq!(name.as_str(), "Boss");
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Name {
    /// The name string.
    name: String,
}

impl Name {
    /// Creates a new Name component with the given string.
    ///
    /// # Arguments
    ///
    /// * `name` - The name string (anything that can be converted to String)
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Name;
    ///
    /// let name = Name::new("Player");
    /// let name2 = Name::new(String::from("Enemy"));
    /// ```
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Returns the name as a string slice.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Name;
    ///
    /// let name = Name::new("Player");
    /// assert_eq!(name.as_str(), "Player");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// Sets a new name.
    ///
    /// # Arguments
    ///
    /// * `name` - The new name string
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Name;
    ///
    /// let mut name = Name::new("Player");
    /// name.set("Player_renamed");
    /// assert_eq!(name.as_str(), "Player_renamed");
    /// ```
    #[inline]
    pub fn set(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Returns the length of the name in bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Name;
    ///
    /// let name = Name::new("Test");
    /// assert_eq!(name.len(), 4);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.name.len()
    }

    /// Returns `true` if the name is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Name;
    ///
    /// let name = Name::new("");
    /// assert!(name.is_empty());
    ///
    /// let name2 = Name::new("Player");
    /// assert!(!name2.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    /// Consumes the Name and returns the inner String.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Name;
    ///
    /// let name = Name::new("Player");
    /// let string: String = name.into_string();
    /// assert_eq!(string, "Player");
    /// ```
    #[inline]
    pub fn into_string(self) -> String {
        self.name
    }

    /// Returns `true` if the name contains the given pattern.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Name;
    ///
    /// let name = Name::new("Player_01");
    /// assert!(name.contains("Player"));
    /// assert!(name.contains("01"));
    /// assert!(!name.contains("Enemy"));
    /// ```
    #[inline]
    pub fn contains(&self, pattern: &str) -> bool {
        self.name.contains(pattern)
    }

    /// Returns `true` if the name starts with the given prefix.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Name;
    ///
    /// let name = Name::new("Player_01");
    /// assert!(name.starts_with("Player"));
    /// assert!(!name.starts_with("Enemy"));
    /// ```
    #[inline]
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.name.starts_with(prefix)
    }

    /// Returns `true` if the name ends with the given suffix.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::components::Name;
    ///
    /// let name = Name::new("Player_01");
    /// assert!(name.ends_with("01"));
    /// assert!(!name.ends_with("Player"));
    /// ```
    #[inline]
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.name.ends_with(suffix)
    }
}

impl Default for Name {
    /// Returns a Name with an empty string.
    #[inline]
    fn default() -> Self {
        Self {
            name: String::new(),
        }
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name({:?})", self.name)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<&str> for Name {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Name {
    #[inline]
    fn from(s: String) -> Self {
        Self { name: s }
    }
}

impl From<Name> for String {
    #[inline]
    fn from(name: Name) -> Self {
        name.name
    }
}

impl AsRef<str> for Name {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl std::borrow::Borrow<str> for Name {
    #[inline]
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq<str> for Name {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}

impl PartialEq<&str> for Name {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.name == *other
    }
}

impl PartialEq<String> for Name {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        &self.name == other
    }
}

// Implement Component trait
impl Component for Name {}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Parent Tests
    // =========================================================================

    mod parent_tests {
        use super::*;

        #[test]
        fn test_parent_new() {
            let entity = Entity::new(42, 1);
            let parent = Parent::new(entity);
            assert_eq!(parent.get(), entity);
        }

        #[test]
        fn test_parent_get() {
            let entity = Entity::new(10, 2);
            let parent = Parent::new(entity);
            assert_eq!(parent.get().index(), 10);
            assert_eq!(parent.get().generation(), 2);
        }

        #[test]
        fn test_parent_set() {
            let mut parent = Parent::new(Entity::new(0, 1));
            parent.set(Entity::new(5, 3));
            assert_eq!(parent.get(), Entity::new(5, 3));
        }

        #[test]
        fn test_parent_default() {
            let parent = Parent::default();
            assert!(parent.get().is_placeholder());
        }

        #[test]
        fn test_parent_from_entity() {
            let entity = Entity::new(7, 2);
            let parent: Parent = entity.into();
            assert_eq!(parent.get(), entity);
        }

        #[test]
        fn test_entity_from_parent() {
            let parent = Parent::new(Entity::new(7, 2));
            let entity: Entity = parent.into();
            assert_eq!(entity, Entity::new(7, 2));
        }

        #[test]
        fn test_parent_clone_copy() {
            let parent = Parent::new(Entity::new(1, 1));
            let cloned = parent.clone();
            let copied = parent;
            assert_eq!(parent, cloned);
            assert_eq!(parent, copied);
        }

        #[test]
        fn test_parent_eq() {
            let p1 = Parent::new(Entity::new(1, 1));
            let p2 = Parent::new(Entity::new(1, 1));
            let p3 = Parent::new(Entity::new(2, 1));
            assert_eq!(p1, p2);
            assert_ne!(p1, p3);
        }

        #[test]
        fn test_parent_hash() {
            use std::collections::HashSet;
            let mut set = HashSet::new();
            set.insert(Parent::new(Entity::new(1, 1)));
            assert!(set.contains(&Parent::new(Entity::new(1, 1))));
            assert!(!set.contains(&Parent::new(Entity::new(2, 1))));
        }

        #[test]
        fn test_parent_debug() {
            let parent = Parent::new(Entity::new(42, 3));
            let debug = format!("{:?}", parent);
            assert!(debug.contains("Parent"));
            assert!(debug.contains("42"));
        }

        #[test]
        fn test_parent_display() {
            let parent = Parent::new(Entity::new(42, 3));
            let display = format!("{}", parent);
            assert!(display.contains("Parent"));
        }

        #[test]
        fn test_parent_is_component() {
            fn assert_component<T: Component>() {}
            assert_component::<Parent>();
        }

        #[test]
        fn test_parent_is_send_sync() {
            fn assert_send_sync<T: Send + Sync>() {}
            assert_send_sync::<Parent>();
        }

        #[test]
        fn test_parent_size() {
            // Parent wraps Entity, so should be same size
            assert_eq!(std::mem::size_of::<Parent>(), std::mem::size_of::<Entity>());
            assert_eq!(std::mem::size_of::<Parent>(), 8);
        }
    }

    // =========================================================================
    // Children Tests
    // =========================================================================

    mod children_tests {
        use super::*;

        #[test]
        fn test_children_new() {
            let children = Children::new();
            assert!(children.is_empty());
            assert_eq!(children.len(), 0);
        }

        #[test]
        fn test_children_with_capacity() {
            let children = Children::with_capacity(100);
            assert!(children.is_empty());
        }

        #[test]
        fn test_children_from_slice() {
            let entities = vec![Entity::new(1, 1), Entity::new(2, 1)];
            let children = Children::from_slice(&entities);
            assert_eq!(children.len(), 2);
        }

        #[test]
        fn test_children_push() {
            let mut children = Children::new();
            children.push(Entity::new(1, 1));
            children.push(Entity::new(2, 1));
            assert_eq!(children.len(), 2);
            assert_eq!(children.get(0), Some(Entity::new(1, 1)));
            assert_eq!(children.get(1), Some(Entity::new(2, 1)));
        }

        #[test]
        fn test_children_insert() {
            let mut children = Children::new();
            children.push(Entity::new(1, 1));
            children.push(Entity::new(3, 1));
            children.insert(1, Entity::new(2, 1));
            assert_eq!(children.get(1), Some(Entity::new(2, 1)));
            assert_eq!(children.len(), 3);
        }

        #[test]
        fn test_children_remove() {
            let mut children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            let removed = children.remove(0);
            assert_eq!(removed, Entity::new(1, 1));
            assert_eq!(children.len(), 1);
        }

        #[test]
        fn test_children_remove_child() {
            let mut children =
                Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1), Entity::new(3, 1)]);

            assert!(children.remove_child(Entity::new(2, 1)));
            assert_eq!(children.len(), 2);
            assert!(!children.contains(Entity::new(2, 1)));

            // Order preserved
            assert_eq!(children.get(0), Some(Entity::new(1, 1)));
            assert_eq!(children.get(1), Some(Entity::new(3, 1)));

            // Already removed
            assert!(!children.remove_child(Entity::new(2, 1)));
        }

        #[test]
        fn test_children_swap_remove_child() {
            let mut children =
                Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1), Entity::new(3, 1)]);

            assert!(children.swap_remove_child(Entity::new(1, 1)));
            assert_eq!(children.len(), 2);
            assert!(!children.contains(Entity::new(1, 1)));

            // Order NOT preserved (last element moved to removed position)
            assert!(children.contains(Entity::new(2, 1)));
            assert!(children.contains(Entity::new(3, 1)));
        }

        #[test]
        fn test_children_contains() {
            let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            assert!(children.contains(Entity::new(1, 1)));
            assert!(children.contains(Entity::new(2, 1)));
            assert!(!children.contains(Entity::new(3, 1)));
        }

        #[test]
        fn test_children_get() {
            let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            assert_eq!(children.get(0), Some(Entity::new(1, 1)));
            assert_eq!(children.get(1), Some(Entity::new(2, 1)));
            assert_eq!(children.get(2), None);
        }

        #[test]
        fn test_children_first_last() {
            let children =
                Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1), Entity::new(3, 1)]);
            assert_eq!(children.first(), Some(Entity::new(1, 1)));
            assert_eq!(children.last(), Some(Entity::new(3, 1)));

            let empty = Children::new();
            assert_eq!(empty.first(), None);
            assert_eq!(empty.last(), None);
        }

        #[test]
        fn test_children_iter() {
            let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            let collected: Vec<_> = children.iter().copied().collect();
            assert_eq!(collected, vec![Entity::new(1, 1), Entity::new(2, 1)]);
        }

        #[test]
        fn test_children_index_of() {
            let children =
                Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1), Entity::new(3, 1)]);
            assert_eq!(children.index_of(Entity::new(1, 1)), Some(0));
            assert_eq!(children.index_of(Entity::new(2, 1)), Some(1));
            assert_eq!(children.index_of(Entity::new(3, 1)), Some(2));
            assert_eq!(children.index_of(Entity::new(99, 1)), None);
        }

        #[test]
        fn test_children_clear() {
            let mut children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            children.clear();
            assert!(children.is_empty());
        }

        #[test]
        fn test_children_as_slice() {
            let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            let slice = children.as_slice();
            assert_eq!(slice.len(), 2);
            assert_eq!(slice[0], Entity::new(1, 1));
        }

        #[test]
        fn test_children_retain() {
            let mut children = Children::from_slice(&[
                Entity::new(1, 1),
                Entity::new(2, 1),
                Entity::new(3, 1),
                Entity::new(4, 1),
            ]);

            children.retain(|e| e.index() % 2 == 0);

            assert_eq!(children.len(), 2);
            assert!(children.contains(Entity::new(2, 1)));
            assert!(children.contains(Entity::new(4, 1)));
        }

        #[test]
        fn test_children_sort_by_index() {
            let mut children = Children::from_slice(&[
                Entity::new(3, 1),
                Entity::new(1, 1),
                Entity::new(2, 1),
            ]);

            children.sort_by_index();

            assert_eq!(children.get(0), Some(Entity::new(1, 1)));
            assert_eq!(children.get(1), Some(Entity::new(2, 1)));
            assert_eq!(children.get(2), Some(Entity::new(3, 1)));
        }

        #[test]
        fn test_children_sort_by() {
            let mut children = Children::from_slice(&[
                Entity::new(1, 1),
                Entity::new(2, 1),
                Entity::new(3, 1),
            ]);

            // Reverse sort
            children.sort_by(|a, b| b.index().cmp(&a.index()));

            assert_eq!(children.get(0), Some(Entity::new(3, 1)));
            assert_eq!(children.get(1), Some(Entity::new(2, 1)));
            assert_eq!(children.get(2), Some(Entity::new(1, 1)));
        }

        #[test]
        fn test_children_default() {
            let children: Children = Default::default();
            assert!(children.is_empty());
        }

        #[test]
        fn test_children_debug() {
            let children = Children::from_slice(&[Entity::new(1, 1)]);
            let debug = format!("{:?}", children);
            assert!(debug.contains("Children"));
            assert!(debug.contains("count"));
        }

        #[test]
        fn test_children_display() {
            let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            let display = format!("{}", children);
            assert!(display.contains("Children(2)"));
        }

        #[test]
        fn test_children_into_iter_ref() {
            let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            let collected: Vec<_> = (&children).into_iter().copied().collect();
            assert_eq!(collected, vec![Entity::new(1, 1), Entity::new(2, 1)]);
        }

        #[test]
        fn test_children_into_iter_owned() {
            let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            let collected: Vec<_> = children.into_iter().collect();
            assert_eq!(collected, vec![Entity::new(1, 1), Entity::new(2, 1)]);
        }

        #[test]
        fn test_children_from_vec() {
            let vec = vec![Entity::new(1, 1), Entity::new(2, 1)];
            let children: Children = vec.into();
            assert_eq!(children.len(), 2);
        }

        #[test]
        fn test_children_from_slice_trait() {
            let slice = &[Entity::new(1, 1), Entity::new(2, 1)][..];
            let children: Children = slice.into();
            assert_eq!(children.len(), 2);
        }

        #[test]
        fn test_vec_from_children() {
            let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            let vec: Vec<Entity> = children.into();
            assert_eq!(vec.len(), 2);
        }

        #[test]
        fn test_children_is_component() {
            fn assert_component<T: Component>() {}
            assert_component::<Children>();
        }

        #[test]
        fn test_children_is_send_sync() {
            fn assert_send_sync<T: Send + Sync>() {}
            assert_send_sync::<Children>();
        }

        #[test]
        fn test_children_clone() {
            let children = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            let cloned = children.clone();
            assert_eq!(children, cloned);
        }

        #[test]
        fn test_children_eq() {
            let c1 = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            let c2 = Children::from_slice(&[Entity::new(1, 1), Entity::new(2, 1)]);
            let c3 = Children::from_slice(&[Entity::new(2, 1), Entity::new(1, 1)]); // Different order

            assert_eq!(c1, c2);
            assert_ne!(c1, c3); // Order matters
        }

        #[test]
        fn test_children_many() {
            let mut children = Children::new();
            for i in 0..1000 {
                children.push(Entity::new(i, 1));
            }
            assert_eq!(children.len(), 1000);

            for i in 0..1000 {
                assert!(children.contains(Entity::new(i, 1)));
            }
        }
    }

    // =========================================================================
    // Name Tests
    // =========================================================================

    mod name_tests {
        use super::*;

        #[test]
        fn test_name_new() {
            let name = Name::new("Player");
            assert_eq!(name.as_str(), "Player");
        }

        #[test]
        fn test_name_new_string() {
            let name = Name::new(String::from("Enemy"));
            assert_eq!(name.as_str(), "Enemy");
        }

        #[test]
        fn test_name_as_str() {
            let name = Name::new("Test");
            assert_eq!(name.as_str(), "Test");
        }

        #[test]
        fn test_name_set() {
            let mut name = Name::new("Old");
            name.set("New");
            assert_eq!(name.as_str(), "New");
        }

        #[test]
        fn test_name_set_string() {
            let mut name = Name::new("Old");
            name.set(String::from("New"));
            assert_eq!(name.as_str(), "New");
        }

        #[test]
        fn test_name_len() {
            let name = Name::new("Test");
            assert_eq!(name.len(), 4);

            let empty = Name::new("");
            assert_eq!(empty.len(), 0);
        }

        #[test]
        fn test_name_is_empty() {
            let empty = Name::new("");
            assert!(empty.is_empty());

            let non_empty = Name::new("Player");
            assert!(!non_empty.is_empty());
        }

        #[test]
        fn test_name_into_string() {
            let name = Name::new("Player");
            let s: String = name.into_string();
            assert_eq!(s, "Player");
        }

        #[test]
        fn test_name_contains() {
            let name = Name::new("Player_01");
            assert!(name.contains("Player"));
            assert!(name.contains("01"));
            assert!(name.contains("_"));
            assert!(!name.contains("Enemy"));
        }

        #[test]
        fn test_name_starts_with() {
            let name = Name::new("Player_01");
            assert!(name.starts_with("Player"));
            assert!(name.starts_with("Play"));
            assert!(!name.starts_with("Enemy"));
        }

        #[test]
        fn test_name_ends_with() {
            let name = Name::new("Player_01");
            assert!(name.ends_with("01"));
            assert!(name.ends_with("_01"));
            assert!(!name.ends_with("Player"));
        }

        #[test]
        fn test_name_default() {
            let name: Name = Default::default();
            assert!(name.is_empty());
            assert_eq!(name.as_str(), "");
        }

        #[test]
        fn test_name_debug() {
            let name = Name::new("Test");
            let debug = format!("{:?}", name);
            assert!(debug.contains("Name"));
            assert!(debug.contains("Test"));
        }

        #[test]
        fn test_name_display() {
            let name = Name::new("Player");
            let display = format!("{}", name);
            assert_eq!(display, "Player");
        }

        #[test]
        fn test_name_from_str() {
            let name: Name = "Test".into();
            assert_eq!(name.as_str(), "Test");
        }

        #[test]
        fn test_name_from_string() {
            let name: Name = String::from("Test").into();
            assert_eq!(name.as_str(), "Test");
        }

        #[test]
        fn test_string_from_name() {
            let name = Name::new("Test");
            let s: String = name.into();
            assert_eq!(s, "Test");
        }

        #[test]
        fn test_name_as_ref() {
            let name = Name::new("Test");
            let s: &str = name.as_ref();
            assert_eq!(s, "Test");
        }

        #[test]
        fn test_name_borrow() {
            use std::borrow::Borrow;
            let name = Name::new("Test");
            let s: &str = name.borrow();
            assert_eq!(s, "Test");
        }

        #[test]
        fn test_name_eq_str() {
            let name = Name::new("Test");
            assert!(name == *"Test");
            assert!(name == "Test");
            assert!(name != *"Other");
        }

        #[test]
        fn test_name_eq_string() {
            let name = Name::new("Test");
            assert!(name == String::from("Test"));
            assert!(name != String::from("Other"));
        }

        #[test]
        fn test_name_clone() {
            let name = Name::new("Test");
            let cloned = name.clone();
            assert_eq!(name, cloned);
        }

        #[test]
        fn test_name_eq() {
            let n1 = Name::new("Test");
            let n2 = Name::new("Test");
            let n3 = Name::new("Other");

            assert_eq!(n1, n2);
            assert_ne!(n1, n3);
        }

        #[test]
        fn test_name_hash() {
            use std::collections::HashSet;
            let mut set = HashSet::new();
            set.insert(Name::new("Player"));
            assert!(set.contains(&Name::new("Player")));
            assert!(!set.contains(&Name::new("Enemy")));
        }

        #[test]
        fn test_name_is_component() {
            fn assert_component<T: Component>() {}
            assert_component::<Name>();
        }

        #[test]
        fn test_name_is_send_sync() {
            fn assert_send_sync<T: Send + Sync>() {}
            assert_send_sync::<Name>();
        }

        #[test]
        fn test_name_unicode() {
            let name = Name::new("玩家");
            assert_eq!(name.as_str(), "玩家");
            assert_eq!(name.len(), 6); // 2 characters, 3 bytes each in UTF-8
        }

        #[test]
        fn test_name_emoji() {
            let name = Name::new("Player 🎮");
            assert!(name.contains("🎮"));
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration_tests {
        use super::*;

        #[test]
        fn test_hierarchy_components_work_together() {
            // Simulate a simple hierarchy
            let parent_entity = Entity::new(0, 1);
            let child1 = Entity::new(1, 1);
            let child2 = Entity::new(2, 1);

            // Parent with children
            let children = Children::from_slice(&[child1, child2]);
            assert_eq!(children.len(), 2);

            // Children with parent
            let parent1 = Parent::new(parent_entity);
            let parent2 = Parent::new(parent_entity);
            assert_eq!(parent1.get(), parent_entity);
            assert_eq!(parent2.get(), parent_entity);

            // Names for debugging
            let parent_name = Name::new("Root");
            let child1_name = Name::new("Child_A");
            let child2_name = Name::new("Child_B");

            assert_eq!(parent_name.as_str(), "Root");
            assert!(child1_name.starts_with("Child"));
            assert!(child2_name.starts_with("Child"));
        }

        #[test]
        fn test_hierarchy_mutation() {
            let parent_entity = Entity::new(0, 1);
            let child1 = Entity::new(1, 1);
            let child2 = Entity::new(2, 1);
            let new_parent = Entity::new(3, 1);

            // Start with one child
            let mut children = Children::new();
            children.push(child1);

            // Child has parent
            let mut parent_comp = Parent::new(parent_entity);

            // Add another child
            children.push(child2);
            assert_eq!(children.len(), 2);

            // Reparent the child
            parent_comp.set(new_parent);
            assert_eq!(parent_comp.get(), new_parent);

            // Remove from old parent's children
            children.remove_child(child1);
            assert_eq!(children.len(), 1);
            assert!(!children.contains(child1));
        }

        #[test]
        fn test_components_are_distinct() {
            // Verify these are distinct component types
            use crate::ecs::ComponentId;

            let parent_id = ComponentId::of::<Parent>();
            let children_id = ComponentId::of::<Children>();
            let name_id = ComponentId::of::<Name>();

            assert_ne!(parent_id, children_id);
            assert_ne!(parent_id, name_id);
            assert_ne!(children_id, name_id);
        }
    }
}
