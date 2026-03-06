//! `SystemMeta` — metadata about a system (name, access patterns).

use std::borrow::Cow;

use crate::ecs::query::Access;

/// Metadata about a system.
///
/// `SystemMeta` stores information about a system that is useful for:
/// - Debugging and logging (name)
/// - Conflict detection (component access)
/// - Scheduling (dependencies, ordering)
///
/// # Example
///
/// ```
/// use goud_engine::ecs::system::SystemMeta;
///
/// let meta = SystemMeta::new("MySystem");
/// assert_eq!(meta.name(), "MySystem");
/// ```
#[derive(Debug, Clone)]
pub struct SystemMeta {
    /// Human-readable name of the system.
    name: Cow<'static, str>,
    /// Component and resource access patterns.
    component_access: Access,
}

impl SystemMeta {
    /// Creates new system metadata with the given name.
    #[inline]
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            component_access: Access::new(),
        }
    }

    /// Returns the system's name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the system's name.
    #[inline]
    pub fn set_name(&mut self, name: impl Into<Cow<'static, str>>) {
        self.name = name.into();
    }

    /// Returns the system's component access pattern.
    #[inline]
    pub fn component_access(&self) -> &Access {
        &self.component_access
    }

    /// Returns a mutable reference to the component access pattern.
    #[inline]
    pub fn component_access_mut(&mut self) -> &mut Access {
        &mut self.component_access
    }

    /// Sets the component access pattern.
    #[inline]
    pub fn set_component_access(&mut self, access: Access) {
        self.component_access = access;
    }

    /// Checks if this system's access conflicts with another.
    ///
    /// Two systems conflict if one writes to a component that the other
    /// reads or writes.
    #[inline]
    pub fn conflicts_with(&self, other: &SystemMeta) -> bool {
        self.component_access
            .conflicts_with(&other.component_access)
    }

    /// Returns true if this system only reads data (no writes).
    #[inline]
    pub fn is_read_only(&self) -> bool {
        self.component_access.is_read_only()
    }
}

impl Default for SystemMeta {
    fn default() -> Self {
        Self::new("UnnamedSystem")
    }
}
