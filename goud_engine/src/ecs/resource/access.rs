//! Immutable and mutable resource access wrappers: [`Res`] and [`ResMut`].

use std::fmt;
use std::ops::{Deref, DerefMut};

use super::types::Resource;

// =============================================================================
// Res<T> - Immutable Resource Access
// =============================================================================

/// Immutable access to a resource of type `T`.
///
/// `Res<T>` provides read-only access to a resource stored in the World.
/// It implements `Deref`, so you can access the inner value directly.
///
/// # Panics
///
/// Operations on `Res<T>` will panic if the resource doesn't exist.
/// Use `Option<Res<T>>` for optional access (future).
///
/// # Example
///
/// ```ignore
/// fn print_time(time: Res<Time>) {
///     println!("Delta: {}, Total: {}", time.delta, time.total);
/// }
/// ```
///
/// # Thread Safety
///
/// Multiple `Res<T>` instances can coexist, as they only provide read access.
/// They conflict with `ResMut<T>` on the same resource type.
pub struct Res<'w, T: Resource> {
    value: &'w T,
}

impl<'w, T: Resource> Res<'w, T> {
    /// Creates a new `Res` from a reference.
    ///
    /// This is primarily used internally by the system parameter infrastructure.
    #[inline]
    pub fn new(value: &'w T) -> Self {
        Self { value }
    }

    /// Returns a reference to the inner value.
    #[inline]
    pub fn into_inner(self) -> &'w T {
        self.value
    }
}

impl<T: Resource> Deref for Res<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: Resource + fmt::Debug> fmt::Debug for Res<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Res").field(&self.value).finish()
    }
}

// Clone for Res - it's just a reference, so this is cheap
impl<T: Resource> Clone for Res<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Resource> Copy for Res<'_, T> {}

// =============================================================================
// ResMut<T> - Mutable Resource Access
// =============================================================================

/// Mutable access to a resource of type `T`.
///
/// `ResMut<T>` provides read-write access to a resource stored in the World.
/// It implements `Deref` and `DerefMut`, so you can access the inner value directly.
///
/// # Panics
///
/// Operations on `ResMut<T>` will panic if the resource doesn't exist.
/// Use `Option<ResMut<T>>` for optional access (future).
///
/// # Example
///
/// ```ignore
/// fn update_time(mut time: ResMut<Time>, delta: f32) {
///     time.delta = delta;
///     time.total += delta;
/// }
/// ```
///
/// # Thread Safety
///
/// Only one `ResMut<T>` can exist at a time for a given resource type.
/// It conflicts with both `Res<T>` and `ResMut<T>` on the same type.
pub struct ResMut<'w, T: Resource> {
    value: &'w mut T,
}

impl<'w, T: Resource> ResMut<'w, T> {
    /// Creates a new `ResMut` from a mutable reference.
    ///
    /// This is primarily used internally by the system parameter infrastructure.
    #[inline]
    pub fn new(value: &'w mut T) -> Self {
        Self { value }
    }

    /// Returns a mutable reference to the inner value.
    #[inline]
    pub fn into_inner(self) -> &'w mut T {
        self.value
    }
}

impl<T: Resource> Deref for ResMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: Resource> DerefMut for ResMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<T: Resource + fmt::Debug> fmt::Debug for ResMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ResMut").field(&self.value).finish()
    }
}
