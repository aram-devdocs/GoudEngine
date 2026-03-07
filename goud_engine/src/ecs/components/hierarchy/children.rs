//! [`Children`] component for listing child entities.

use crate::ecs::entity::Entity;
use crate::ecs::Component;
use std::fmt;

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
#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Children {
    /// The list of child entities in order.
    pub(crate) children: Vec<Entity>,
}

impl Children {
    /// Creates an empty Children component.
    #[inline]
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Creates a Children component with pre-allocated capacity.
    ///
    /// Use this when you know approximately how many children the entity will have.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            children: Vec::with_capacity(capacity),
        }
    }

    /// Creates a Children component from a slice of entities.
    #[inline]
    pub fn from_slice(children: &[Entity]) -> Self {
        Self {
            children: children.to_vec(),
        }
    }

    /// Returns the number of children.
    #[inline]
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns `true` if there are no children.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Adds a child entity to the end of the children list.
    #[inline]
    pub fn push(&mut self, child: Entity) {
        self.children.push(child);
    }

    /// Inserts a child entity at a specific index.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[inline]
    pub fn insert(&mut self, index: usize, child: Entity) {
        self.children.insert(index, child);
    }

    /// Removes and returns the child at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    pub fn remove(&mut self, index: usize) -> Entity {
        self.children.remove(index)
    }

    /// Removes a child entity if it exists, preserving order.
    ///
    /// Returns `true` if the child was found and removed, `false` otherwise.
    /// This is O(n) as it must search for the child and shift elements.
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
    /// Returns `true` if the child was found and removed, `false` otherwise.
    /// O(n) for the search but O(1) for the actual removal.
    pub fn swap_remove_child(&mut self, child: Entity) -> bool {
        if let Some(index) = self.children.iter().position(|&e| e == child) {
            self.children.swap_remove(index);
            true
        } else {
            false
        }
    }

    /// Returns `true` if the given entity is a child.
    #[inline]
    pub fn contains(&self, child: Entity) -> bool {
        self.children.contains(&child)
    }

    /// Returns the child at the given index, if any.
    #[inline]
    pub fn get(&self, index: usize) -> Option<Entity> {
        self.children.get(index).copied()
    }

    /// Returns the first child, if any.
    #[inline]
    pub fn first(&self) -> Option<Entity> {
        self.children.first().copied()
    }

    /// Returns the last child, if any.
    #[inline]
    pub fn last(&self) -> Option<Entity> {
        self.children.last().copied()
    }

    /// Returns an iterator over the children.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.children.iter()
    }

    /// Returns the index of a child entity, if it exists.
    ///
    /// Returns `Some(index)` if found, `None` otherwise.
    #[inline]
    pub fn index_of(&self, child: Entity) -> Option<usize> {
        self.children.iter().position(|&e| e == child)
    }

    /// Removes all children.
    #[inline]
    pub fn clear(&mut self) {
        self.children.clear();
    }

    /// Returns the children as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[Entity] {
        &self.children
    }

    /// Retains only the children that satisfy the predicate.
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

    /// Sorts children by their entity index for deterministic ordering.
    pub fn sort_by_index(&mut self) {
        self.children.sort_by_key(|e| e.index());
    }

    /// Sorts children using a custom comparison function.
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
