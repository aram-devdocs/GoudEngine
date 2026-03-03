//! HandleMap: a slot-map that associates handles with values.

use super::allocator::HandleAllocator;
use super::handle_type::Handle;

/// A map that associates handles with values using generational indices.
///
/// `HandleMap` is a slot-map data structure that combines a `HandleAllocator`
/// with value storage. It provides O(1) insertion, lookup, and removal with
/// generational safety (stale handles return None, never wrong values).
///
/// # Type Parameters
///
/// - `T`: Marker type for the handle (provides type safety)
/// - `V`: The value type stored in the map
///
/// # Example
///
/// ```
/// use goud_engine::core::handle::HandleMap;
///
/// struct Texture;
///
/// let mut textures: HandleMap<Texture, String> = HandleMap::new();
/// let handle = textures.insert("player.png".to_string());
///
/// assert_eq!(textures.get(handle), Some(&"player.png".to_string()));
/// textures.remove(handle);
/// assert!(textures.get(handle).is_none());
/// ```
///
/// # Thread Safety
///
/// `HandleMap` is NOT thread-safe. For concurrent access, wrap in
/// appropriate synchronization primitives.
pub struct HandleMap<T, V> {
    /// The handle allocator managing index and generation tracking.
    pub(crate) allocator: HandleAllocator<T>,

    /// Storage for values, indexed by handle index.
    ///
    /// Entries are `Some(value)` for live handles, `None` for deallocated slots.
    /// The index in this vector corresponds to `handle.index()`.
    pub(crate) values: Vec<Option<V>>,
}

impl<T, V> HandleMap<T, V> {
    /// Creates a new, empty handle map.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Mesh;
    /// struct MeshData { vertex_count: usize }
    ///
    /// let map: HandleMap<Mesh, MeshData> = HandleMap::new();
    /// assert!(map.is_empty());
    /// assert_eq!(map.len(), 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            allocator: HandleAllocator::new(),
            values: Vec::new(),
        }
    }

    /// Creates a new handle map with pre-allocated capacity.
    ///
    /// This is useful when you know approximately how many entries you'll need,
    /// as it avoids repeated reallocations during bulk insertion.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of entries to pre-allocate space for
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Entity;
    /// struct EntityData { name: String }
    ///
    /// let map: HandleMap<Entity, EntityData> = HandleMap::with_capacity(1000);
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            allocator: HandleAllocator::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    /// Inserts a value into the map and returns a handle to it.
    ///
    /// The returned handle can be used to retrieve, modify, or remove the value.
    /// The handle remains valid until the value is removed.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to insert
    ///
    /// # Returns
    ///
    /// A handle that can be used to access the inserted value.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Shader;
    /// struct ShaderData { name: String }
    ///
    /// let mut map: HandleMap<Shader, ShaderData> = HandleMap::new();
    ///
    /// let h1 = map.insert(ShaderData { name: "basic".to_string() });
    /// let h2 = map.insert(ShaderData { name: "pbr".to_string() });
    ///
    /// assert_ne!(h1, h2);
    /// assert!(h1.is_valid());
    /// assert!(h2.is_valid());
    /// assert_eq!(map.len(), 2);
    /// ```
    pub fn insert(&mut self, value: V) -> Handle<T> {
        let handle = self.allocator.allocate();
        let index = handle.index() as usize;

        // Ensure values vec is large enough
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }

        self.values[index] = Some(value);
        handle
    }

    /// Removes a value from the map by its handle.
    ///
    /// If the handle is valid and points to an existing value, the value is
    /// removed and returned. The handle becomes stale after this operation.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle of the value to remove
    ///
    /// # Returns
    ///
    /// - `Some(value)` if the handle was valid and the value was removed
    /// - `None` if the handle was invalid, stale, or already removed
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Audio;
    /// struct AudioClip { duration: f32 }
    ///
    /// let mut map: HandleMap<Audio, AudioClip> = HandleMap::new();
    ///
    /// let handle = map.insert(AudioClip { duration: 5.5 });
    /// assert_eq!(map.len(), 1);
    ///
    /// let removed = map.remove(handle);
    /// assert!(removed.is_some());
    /// assert_eq!(removed.unwrap().duration, 5.5);
    /// assert_eq!(map.len(), 0);
    ///
    /// // Handle is now stale
    /// assert!(map.remove(handle).is_none());
    /// ```
    pub fn remove(&mut self, handle: Handle<T>) -> Option<V> {
        // Check if handle is alive
        if !self.allocator.is_alive(handle) {
            return None;
        }

        let index = handle.index() as usize;

        // Deallocate the handle
        if !self.allocator.deallocate(handle) {
            return None;
        }

        // Take the value from storage
        self.values.get_mut(index).and_then(|slot| slot.take())
    }

    /// Returns a reference to the value associated with a handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to look up
    ///
    /// # Returns
    ///
    /// - `Some(&V)` if the handle is valid and points to a value
    /// - `None` if the handle is invalid, stale, or the value was removed
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Sprite;
    /// struct SpriteData { x: f32, y: f32 }
    ///
    /// let mut map: HandleMap<Sprite, SpriteData> = HandleMap::new();
    ///
    /// let handle = map.insert(SpriteData { x: 100.0, y: 200.0 });
    ///
    /// if let Some(sprite) = map.get(handle) {
    ///     assert_eq!(sprite.x, 100.0);
    ///     assert_eq!(sprite.y, 200.0);
    /// }
    /// ```
    #[inline]
    pub fn get(&self, handle: Handle<T>) -> Option<&V> {
        if !self.allocator.is_alive(handle) {
            return None;
        }

        let index = handle.index() as usize;
        self.values.get(index).and_then(|slot| slot.as_ref())
    }

    /// Returns a mutable reference to the value associated with a handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to look up
    ///
    /// # Returns
    ///
    /// - `Some(&mut V)` if the handle is valid and points to a value
    /// - `None` if the handle is invalid, stale, or the value was removed
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Transform;
    /// struct TransformData { x: f32, y: f32 }
    ///
    /// let mut map: HandleMap<Transform, TransformData> = HandleMap::new();
    ///
    /// let handle = map.insert(TransformData { x: 0.0, y: 0.0 });
    ///
    /// if let Some(transform) = map.get_mut(handle) {
    ///     transform.x = 50.0;
    ///     transform.y = 100.0;
    /// }
    ///
    /// assert_eq!(map.get(handle).unwrap().x, 50.0);
    /// ```
    #[inline]
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut V> {
        if !self.allocator.is_alive(handle) {
            return None;
        }

        let index = handle.index() as usize;
        self.values.get_mut(index).and_then(|slot| slot.as_mut())
    }

    /// Checks if a handle is valid and points to a value in this map.
    ///
    /// This is equivalent to `self.get(handle).is_some()` but more efficient.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to check
    ///
    /// # Returns
    ///
    /// `true` if the handle is valid and the value exists, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Entity;
    /// struct EntityData { id: u32 }
    ///
    /// let mut map: HandleMap<Entity, EntityData> = HandleMap::new();
    ///
    /// let handle = map.insert(EntityData { id: 1 });
    /// assert!(map.contains(handle));
    ///
    /// map.remove(handle);
    /// assert!(!map.contains(handle));
    /// ```
    #[inline]
    pub fn contains(&self, handle: Handle<T>) -> bool {
        self.allocator.is_alive(handle)
    }

    /// Returns the number of values currently stored in the map.
    ///
    /// This counts only live entries, not removed slots.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Resource;
    ///
    /// let mut map: HandleMap<Resource, i32> = HandleMap::new();
    /// assert_eq!(map.len(), 0);
    ///
    /// let h1 = map.insert(10);
    /// let h2 = map.insert(20);
    /// assert_eq!(map.len(), 2);
    ///
    /// map.remove(h1);
    /// assert_eq!(map.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.allocator.len()
    }

    /// Returns `true` if the map contains no values.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Item;
    ///
    /// let mut map: HandleMap<Item, String> = HandleMap::new();
    /// assert!(map.is_empty());
    ///
    /// let handle = map.insert("item".to_string());
    /// assert!(!map.is_empty());
    ///
    /// map.remove(handle);
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.allocator.is_empty()
    }

    /// Returns the total capacity of the map.
    ///
    /// This is the number of slots allocated, including both live and removed entries.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Data;
    ///
    /// let mut map: HandleMap<Data, i32> = HandleMap::new();
    /// let h1 = map.insert(1);
    /// let h2 = map.insert(2);
    ///
    /// assert_eq!(map.capacity(), 2);
    ///
    /// map.remove(h1);
    /// // Capacity unchanged after removal
    /// assert_eq!(map.capacity(), 2);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.allocator.capacity()
    }

    /// Clears all values from the map, invalidating all handles.
    ///
    /// After this operation, all previously returned handles become stale.
    /// The capacity is retained but `len()` becomes 0.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Object;
    ///
    /// let mut map: HandleMap<Object, i32> = HandleMap::new();
    ///
    /// let h1 = map.insert(100);
    /// let h2 = map.insert(200);
    /// let h3 = map.insert(300);
    ///
    /// map.clear();
    ///
    /// assert!(map.is_empty());
    /// assert!(!map.contains(h1));
    /// assert!(!map.contains(h2));
    /// assert!(!map.contains(h3));
    ///
    /// // Capacity retained
    /// assert_eq!(map.capacity(), 3);
    /// ```
    pub fn clear(&mut self) {
        self.allocator.clear();

        // Clear all values but retain the vec capacity
        for slot in &mut self.values {
            *slot = None;
        }
    }

    /// Reserves capacity for at least `additional` more elements.
    ///
    /// # Arguments
    ///
    /// * `additional` - The number of additional entries to reserve space for
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Component;
    ///
    /// let mut map: HandleMap<Component, i32> = HandleMap::new();
    /// map.reserve(100);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.values.reserve(additional);
    }

    /// Shrinks the capacity of the map as much as possible.
    ///
    /// This reduces memory usage by releasing excess capacity in internal
    /// data structures.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::core::handle::HandleMap;
    ///
    /// struct Item;
    ///
    /// let mut map: HandleMap<Item, i32> = HandleMap::with_capacity(100);
    ///
    /// for i in 0..10 {
    ///     map.insert(i);
    /// }
    ///
    /// map.shrink_to_fit();
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.allocator.shrink_to_fit();
        self.values.shrink_to_fit();
    }
}

impl<T, V> Default for HandleMap<T, V> {
    /// Creates an empty `HandleMap` (same as `new()`).
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, V: std::fmt::Debug> std::fmt::Debug for HandleMap<T, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = std::any::type_name::<T>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);

        f.debug_struct(&format!("HandleMap<{}, ...>", short_name))
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .finish()
    }
}
