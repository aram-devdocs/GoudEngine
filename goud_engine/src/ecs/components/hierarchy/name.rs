//! [`Name`] component for human-readable entity identification.

use crate::ecs::Component;
use std::fmt;

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
#[derive(Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
