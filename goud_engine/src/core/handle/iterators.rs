//! Iterator types for [`HandleMap`](super::HandleMap).

use std::marker::PhantomData;

use super::allocator::HandleAllocator;
use super::handle_type::Handle;

/// An iterator over handle-value pairs in a [`HandleMap`](super::HandleMap).
///
/// This struct is created by the [`iter`](super::HandleMap::iter) method on `HandleMap`.
/// It yields `(Handle<T>, &V)` pairs for all live entries.
///
/// # Iteration Order
///
/// Iteration order is based on the internal storage order (by slot index),
/// which corresponds to allocation order for slots that haven't been recycled.
/// Do not rely on any specific ordering as it may change with insertions and removals.
///
/// # Example
///
/// ```
/// use goud_engine::core::handle::HandleMap;
///
/// struct Texture;
///
/// let mut map: HandleMap<Texture, &str> = HandleMap::new();
/// map.insert("first");
/// map.insert("second");
/// map.insert("third");
///
/// for (handle, value) in map.iter() {
///     println!("Handle {:?} => {}", handle, value);
/// }
/// ```
pub struct HandleMapIter<'a, T, V> {
    /// Reference to the allocator for generation checking.
    pub(crate) allocator: &'a HandleAllocator<T>,

    /// Iterator over the values vector with indices.
    pub(crate) values_iter: std::iter::Enumerate<std::slice::Iter<'a, Option<V>>>,
}

impl<'a, T, V> Iterator for HandleMapIter<'a, T, V> {
    type Item = (Handle<T>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate through values, skipping None entries
        loop {
            match self.values_iter.next() {
                Some((index, Some(value))) => {
                    // Reconstruct the handle from index and current generation
                    let index = index as u32;
                    // Get the current generation for this slot
                    if let Some(&generation) = self.allocator.generations.get(index as usize) {
                        let handle = Handle::new(index, generation);
                        // Only yield if the handle is alive (matches current generation)
                        if self.allocator.is_alive(handle) {
                            return Some((handle, value));
                        }
                    }
                    // Generation mismatch or out of bounds - skip this entry
                    // This shouldn't happen if the map is consistent, but handle gracefully
                }
                Some((_, None)) => {
                    // Empty slot, continue to next
                    continue;
                }
                None => {
                    // End of iteration
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Lower bound is 0 (all remaining could be None)
        // Upper bound is remaining elements in the iterator
        let (_, upper) = self.values_iter.size_hint();
        (0, upper)
    }
}

/// A mutable iterator over handle-value pairs in a [`HandleMap`](super::HandleMap).
///
/// This struct is created by the [`iter_mut`](super::HandleMap::iter_mut) method on `HandleMap`.
/// It yields `(Handle<T>, &mut V)` pairs for all live entries.
///
/// # Example
///
/// ```
/// use goud_engine::core::handle::HandleMap;
///
/// struct Counter;
///
/// let mut map: HandleMap<Counter, i32> = HandleMap::new();
/// map.insert(1);
/// map.insert(2);
/// map.insert(3);
///
/// for (handle, value) in map.iter_mut() {
///     *value *= 2;
/// }
///
/// // All values are now doubled
/// ```
pub struct HandleMapIterMut<'a, T, V> {
    /// Pointer to the allocator for generation checking.
    /// We use a raw pointer because we can't hold a reference while iterating mutably.
    pub(crate) allocator_ptr: *const HandleAllocator<T>,

    /// Iterator over the values vector with indices.
    pub(crate) values_iter: std::iter::Enumerate<std::slice::IterMut<'a, Option<V>>>,

    /// Phantom marker for lifetime.
    pub(crate) _marker: PhantomData<&'a HandleAllocator<T>>,
}

impl<'a, T, V> Iterator for HandleMapIterMut<'a, T, V> {
    type Item = (Handle<T>, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate through values, skipping None entries
        loop {
            match self.values_iter.next() {
                Some((index, Some(value))) => {
                    // Reconstruct the handle from index and current generation
                    let index = index as u32;
                    // SAFETY: allocator_ptr is valid for the lifetime 'a
                    let allocator = unsafe { &*self.allocator_ptr };
                    if let Some(&generation) = allocator.generations.get(index as usize) {
                        let handle = Handle::new(index, generation);
                        if allocator.is_alive(handle) {
                            return Some((handle, value));
                        }
                    }
                }
                Some((_, None)) => {
                    continue;
                }
                None => {
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.values_iter.size_hint();
        (0, upper)
    }
}

/// An iterator over handles in a [`HandleMap`](super::HandleMap).
///
/// This struct is created by the [`handles`](super::HandleMap::handles) method on `HandleMap`.
/// It yields `Handle<T>` for all live entries.
///
/// # Example
///
/// ```
/// use goud_engine::core::handle::HandleMap;
///
/// struct Entity;
///
/// let mut map: HandleMap<Entity, String> = HandleMap::new();
/// let h1 = map.insert("entity1".to_string());
/// let h2 = map.insert("entity2".to_string());
///
/// let handles: Vec<_> = map.handles().collect();
/// assert!(handles.contains(&h1));
/// assert!(handles.contains(&h2));
/// ```
pub struct HandleMapHandles<'a, T, V> {
    /// The underlying iterator.
    pub(crate) inner: HandleMapIter<'a, T, V>,
}

impl<'a, T, V> Iterator for HandleMapHandles<'a, T, V> {
    type Item = Handle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(handle, _)| handle)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// An iterator over values in a [`HandleMap`](super::HandleMap).
///
/// This struct is created by the [`values`](super::HandleMap::values) method on `HandleMap`.
/// It yields `&V` for all live entries.
///
/// # Example
///
/// ```
/// use goud_engine::core::handle::HandleMap;
///
/// struct Score;
///
/// let mut map: HandleMap<Score, i32> = HandleMap::new();
/// map.insert(100);
/// map.insert(200);
/// map.insert(300);
///
/// let sum: i32 = map.values().sum();
/// assert_eq!(sum, 600);
/// ```
pub struct HandleMapValues<'a, T, V> {
    /// The underlying iterator.
    pub(crate) inner: HandleMapIter<'a, T, V>,
}

impl<'a, T, V> Iterator for HandleMapValues<'a, T, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A mutable iterator over values in a [`HandleMap`](super::HandleMap).
///
/// This struct is created by the [`values_mut`](super::HandleMap::values_mut) method
/// on `HandleMap`. It yields `&mut V` for all live entries.
pub struct HandleMapValuesMut<'a, T, V> {
    /// The underlying iterator.
    pub(crate) inner: HandleMapIterMut<'a, T, V>,
}

impl<'a, T, V> Iterator for HandleMapValuesMut<'a, T, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

// =============================================================================
// HandleMap iterator factory methods and IntoIterator impls
// =============================================================================

use super::map::HandleMap;

impl<T, V> HandleMap<T, V> {
    /// Returns an iterator over handle-value pairs.
    ///
    /// The iterator yields `(Handle<T>, &V)` for each live entry in the map.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Item;
    ///
    /// let mut map: HandleMap<Item, &str> = HandleMap::new();
    /// map.insert("a");
    /// map.insert("b");
    /// map.insert("c");
    ///
    /// let count = map.iter().count();
    /// assert_eq!(count, 3);
    /// ```
    #[inline]
    pub fn iter(&self) -> HandleMapIter<'_, T, V> {
        HandleMapIter {
            allocator: &self.allocator,
            values_iter: self.values.iter().enumerate(),
        }
    }

    /// Returns a mutable iterator over handle-value pairs.
    ///
    /// The iterator yields `(Handle<T>, &mut V)` for each live entry in the map.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Counter;
    ///
    /// let mut map: HandleMap<Counter, i32> = HandleMap::new();
    /// map.insert(1);
    /// map.insert(2);
    ///
    /// for (_, value) in map.iter_mut() {
    ///     *value *= 10;
    /// }
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> HandleMapIterMut<'_, T, V> {
        HandleMapIterMut {
            allocator_ptr: &self.allocator as *const HandleAllocator<T>,
            values_iter: self.values.iter_mut().enumerate(),
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over handles only.
    ///
    /// This is more efficient than `iter().map(|(h, _)| h)` if you only need handles.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Entity;
    ///
    /// let mut map: HandleMap<Entity, ()> = HandleMap::new();
    /// map.insert(());
    /// map.insert(());
    ///
    /// let handles: Vec<_> = map.handles().collect();
    /// assert_eq!(handles.len(), 2);
    /// ```
    #[inline]
    pub fn handles(&self) -> HandleMapHandles<'_, T, V> {
        HandleMapHandles { inner: self.iter() }
    }

    /// Returns an iterator over values only.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Score;
    ///
    /// let mut map: HandleMap<Score, i32> = HandleMap::new();
    /// map.insert(10);
    /// map.insert(20);
    /// map.insert(30);
    ///
    /// let total: i32 = map.values().sum();
    /// assert_eq!(total, 60);
    /// ```
    #[inline]
    pub fn values(&self) -> HandleMapValues<'_, T, V> {
        HandleMapValues { inner: self.iter() }
    }

    /// Returns a mutable iterator over values only.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Data;
    ///
    /// let mut map: HandleMap<Data, i32> = HandleMap::new();
    /// map.insert(1);
    /// map.insert(2);
    ///
    /// for value in map.values_mut() {
    ///     *value += 100;
    /// }
    /// ```
    #[inline]
    pub fn values_mut(&mut self) -> HandleMapValuesMut<'_, T, V> {
        HandleMapValuesMut {
            inner: self.iter_mut(),
        }
    }
}

/// Allows iterating over a `HandleMap` with a for loop.
impl<'a, T, V> IntoIterator for &'a HandleMap<T, V> {
    type Item = (Handle<T>, &'a V);
    type IntoIter = HandleMapIter<'a, T, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Allows mutably iterating over a `HandleMap` with a for loop.
impl<'a, T, V> IntoIterator for &'a mut HandleMap<T, V> {
    type Item = (Handle<T>, &'a mut V);
    type IntoIter = HandleMapIterMut<'a, T, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
