//! Iterator types for [`SparseSet`].
//!
//! Provides immutable and mutable iterators over `(Entity, &T)` and
//! `(Entity, &mut T)` pairs, as well as `IntoIterator` implementations
//! for `&SparseSet<T>` and `&mut SparseSet<T>`.

use super::super::Entity;
use super::core::SparseSet;

// =============================================================================
// Iterator Types
// =============================================================================

/// An iterator over `(Entity, &T)` pairs in a sparse set.
///
/// Created by [`SparseSet::iter()`].
#[derive(Debug)]
pub struct SparseSetIter<'a, T> {
    pub(super) dense: std::slice::Iter<'a, Entity>,
    pub(super) values: std::slice::Iter<'a, T>,
}

impl<'a, T> Iterator for SparseSetIter<'a, T> {
    type Item = (Entity, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let entity = *self.dense.next()?;
        let value = self.values.next()?;
        Some((entity, value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.dense.size_hint()
    }
}

impl<T> ExactSizeIterator for SparseSetIter<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.dense.len()
    }
}

impl<T> std::iter::FusedIterator for SparseSetIter<'_, T> {}

/// A mutable iterator over `(Entity, &mut T)` pairs in a sparse set.
///
/// Created by [`SparseSet::iter_mut()`].
#[derive(Debug)]
pub struct SparseSetIterMut<'a, T> {
    pub(super) dense: std::slice::Iter<'a, Entity>,
    pub(super) values: std::slice::IterMut<'a, T>,
}

impl<'a, T> Iterator for SparseSetIterMut<'a, T> {
    type Item = (Entity, &'a mut T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let entity = *self.dense.next()?;
        let value = self.values.next()?;
        Some((entity, value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.dense.size_hint()
    }
}

impl<T> ExactSizeIterator for SparseSetIterMut<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.dense.len()
    }
}

impl<T> std::iter::FusedIterator for SparseSetIterMut<'_, T> {}

// =============================================================================
// IntoIterator Implementations
// =============================================================================

impl<'a, T> IntoIterator for &'a SparseSet<T> {
    type Item = (Entity, &'a T);
    type IntoIter = SparseSetIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SparseSet<T> {
    type Item = (Entity, &'a mut T);
    type IntoIter = SparseSetIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
