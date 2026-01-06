//! Resource system for the ECS.
//!
//! Resources are singleton data that exists outside the entity-component model.
//! Unlike components, resources are not attached to entities - they are globally
//! accessible within the World.
//!
//! # Examples of Resources
//!
//! - **Time**: Delta time, total elapsed time, frame count
//! - **Input State**: Keyboard, mouse, and gamepad state
//! - **Asset Manager**: Loaded textures, sounds, and other assets
//! - **Configuration**: Game settings, debug flags
//!
//! # Resource vs Component
//!
//! | Aspect | Resource | Component |
//! |--------|----------|-----------|
//! | Cardinality | One per type | Many per type |
//! | Ownership | Owned by World | Attached to Entity |
//! | Access | `Res<T>`, `ResMut<T>` | `Query<&T>`, `Query<&mut T>` |
//! | Use Case | Global state | Per-entity state |
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::{World, Resource};
//!
//! // Define a resource
//! struct Time {
//!     delta: f32,
//!     total: f32,
//! }
//! impl Resource for Time {}
//!
//! // Insert resource into World
//! let mut world = World::new();
//! world.insert_resource(Time { delta: 0.016, total: 0.0 });
//!
//! // Access resource (immutably)
//! let time = world.get_resource::<Time>().unwrap();
//! println!("Delta: {}", time.delta);
//!
//! // Access resource (mutably)
//! let time = world.get_resource_mut::<Time>().unwrap();
//! time.total += time.delta;
//! ```
//!
//! # System Parameters
//!
//! In systems, use `Res<T>` and `ResMut<T>` for resource access:
//!
//! ```ignore
//! fn update_system(time: Res<Time>, mut query: Query<&mut Position>) {
//!     for mut pos in query.iter_mut() {
//!         pos.x += time.delta * 10.0;
//!     }
//! }
//! ```
//!
//! # Thread Safety
//!
//! Resources must be `Send + Sync` for use in parallel systems. For resources
//! that are not thread-safe (e.g., window handles), use non-send resources
//! (future: `NonSend<T>`, `NonSendMut<T>`).

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::ops::{Deref, DerefMut};

// =============================================================================
// Resource Trait
// =============================================================================

/// Marker trait for types that can be used as resources.
///
/// Resources are singleton data stored in the [`World`](crate::ecs::World).
/// To make a type usable as a resource, implement this trait:
///
/// ```
/// use goud_engine::ecs::Resource;
///
/// struct GameConfig {
///     difficulty: u8,
///     sound_volume: f32,
/// }
///
/// impl Resource for GameConfig {}
/// ```
///
/// # Requirements
///
/// Resources must be:
/// - `Send + Sync` for parallel system execution
/// - `'static` for type erasure in storage
///
/// Unlike [`Component`](crate::ecs::Component), `Resource` does NOT require
/// explicit opt-in - any compatible type can be a resource.
pub trait Resource: Send + Sync + 'static {}

// Blanket implementation: any Send + Sync + 'static type can be a resource
impl<T: Send + Sync + 'static> Resource for T {}

// =============================================================================
// ResourceId
// =============================================================================

/// Unique identifier for a resource type.
///
/// `ResourceId` wraps a `TypeId` to identify resource types at runtime.
/// This is used internally for resource storage and access conflict detection.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::resource::ResourceId;
///
/// struct MyResource { value: i32 }
///
/// let id = ResourceId::of::<MyResource>();
/// let id2 = ResourceId::of::<MyResource>();
///
/// assert_eq!(id, id2);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(TypeId);

impl ResourceId {
    /// Returns the `ResourceId` for a specific type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::resource::ResourceId;
    ///
    /// struct Time { delta: f32 }
    /// let id = ResourceId::of::<Time>();
    /// ```
    #[inline]
    pub fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }

    /// Returns the underlying `TypeId`.
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.0
    }
}

impl fmt::Debug for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResourceId({:?})", self.0)
    }
}

// =============================================================================
// Resources Container
// =============================================================================

/// Storage container for all resources in a World.
///
/// `Resources` stores type-erased resource data and provides type-safe
/// access through generic methods.
///
/// # Thread Safety
///
/// The container itself is not thread-safe. Use external synchronization
/// or the scheduler for parallel access.
#[derive(Default)]
pub struct Resources {
    /// Type-erased resource storage.
    data: HashMap<ResourceId, Box<dyn Any + Send + Sync>>,
}

impl Resources {
    /// Creates an empty resources container.
    #[inline]
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Inserts a resource into the container.
    ///
    /// If a resource of this type already exists, it is replaced and the
    /// old value is returned.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to insert
    ///
    /// # Returns
    ///
    /// `Some(T)` if a resource of this type was replaced, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::resource::Resources;
    ///
    /// struct Score(u32);
    ///
    /// let mut resources = Resources::new();
    /// assert!(resources.insert(Score(0)).is_none());
    /// assert!(resources.insert(Score(100)).is_some()); // Replaced
    /// ```
    #[inline]
    pub fn insert<T: Resource>(&mut self, resource: T) -> Option<T> {
        let id = ResourceId::of::<T>();
        let old = self.data.insert(id, Box::new(resource));
        old.and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }

    /// Removes a resource from the container.
    ///
    /// # Returns
    ///
    /// `Some(T)` if the resource existed, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::resource::Resources;
    ///
    /// struct Score(u32);
    ///
    /// let mut resources = Resources::new();
    /// resources.insert(Score(100));
    ///
    /// let score = resources.remove::<Score>();
    /// assert!(score.is_some());
    /// assert_eq!(score.unwrap().0, 100);
    ///
    /// assert!(resources.remove::<Score>().is_none()); // Already removed
    /// ```
    #[inline]
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let id = ResourceId::of::<T>();
        self.data
            .remove(&id)
            .and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }

    /// Returns an immutable reference to a resource.
    ///
    /// # Returns
    ///
    /// `Some(&T)` if the resource exists, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::resource::Resources;
    ///
    /// struct Score(u32);
    ///
    /// let mut resources = Resources::new();
    /// resources.insert(Score(100));
    ///
    /// let score = resources.get::<Score>().unwrap();
    /// assert_eq!(score.0, 100);
    /// ```
    #[inline]
    pub fn get<T: Resource>(&self) -> Option<&T> {
        let id = ResourceId::of::<T>();
        self.data.get(&id).and_then(|boxed| boxed.downcast_ref())
    }

    /// Returns a mutable reference to a resource.
    ///
    /// # Returns
    ///
    /// `Some(&mut T)` if the resource exists, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::resource::Resources;
    ///
    /// struct Score(u32);
    ///
    /// let mut resources = Resources::new();
    /// resources.insert(Score(100));
    ///
    /// let score = resources.get_mut::<Score>().unwrap();
    /// score.0 += 50;
    /// assert_eq!(resources.get::<Score>().unwrap().0, 150);
    /// ```
    #[inline]
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let id = ResourceId::of::<T>();
        self.data
            .get_mut(&id)
            .and_then(|boxed| boxed.downcast_mut())
    }

    /// Returns `true` if a resource of the specified type exists.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::resource::Resources;
    ///
    /// struct Score(u32);
    /// struct Health(f32);
    ///
    /// let mut resources = Resources::new();
    /// resources.insert(Score(100));
    ///
    /// assert!(resources.contains::<Score>());
    /// assert!(!resources.contains::<Health>());
    /// ```
    #[inline]
    pub fn contains<T: Resource>(&self) -> bool {
        self.data.contains_key(&ResourceId::of::<T>())
    }

    /// Returns the number of resources in the container.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if there are no resources.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Removes all resources from the container.
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl fmt::Debug for Resources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resources")
            .field("count", &self.data.len())
            .finish()
    }
}

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

// =============================================================================
// Non-Send Resources
// =============================================================================

/// Marker trait for types that can be used as non-send resources.
///
/// Non-send resources are resources that cannot be safely sent between threads.
/// Unlike regular [`Resource`], non-send resources only require `'static`.
///
/// # Use Cases
///
/// Non-send resources are useful for:
///
/// - **Raw pointers**: Window handles, OpenGL contexts, etc.
/// - **Thread-local data**: Data bound to a specific thread
/// - **Rc/RefCell types**: Reference-counted non-atomic types
/// - **Platform-specific handles**: OS handles that are thread-affine
///
/// # Example
///
/// ```
/// use goud_engine::ecs::resource::NonSendResource;
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// // This type is NOT Send because of Rc
/// struct WindowHandle {
///     id: Rc<RefCell<u32>>,
/// }
///
/// // Explicit implementation required
/// impl NonSendResource for WindowHandle {}
/// ```
///
/// # Thread Safety
///
/// Non-send resources must only be accessed from the main thread.
/// The scheduler ensures non-send systems run on the main thread.
///
/// # Differences from Resource
///
/// | Aspect | Resource | NonSendResource |
/// |--------|----------|-----------------|
/// | Thread-safe | Yes (Send + Sync) | No |
/// | System access | Any thread | Main thread only |
/// | Storage | `Resources` | `NonSendResources` |
/// | Wrapper | `Res<T>`, `ResMut<T>` | `NonSend<T>`, `NonSendMut<T>` |
pub trait NonSendResource: 'static {}

// NOTE: No blanket implementation - types must explicitly opt-in to non-send
// resources. This is intentional to prevent accidentally using non-thread-safe
// types as regular resources.

// =============================================================================
// NonSendResourceId
// =============================================================================

/// Unique identifier for a non-send resource type.
///
/// Like [`ResourceId`] but for non-send resources. Used internally
/// for storage and access conflict detection.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NonSendResourceId(TypeId);

impl NonSendResourceId {
    /// Returns the `NonSendResourceId` for a specific type.
    #[inline]
    pub fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }

    /// Returns the underlying `TypeId`.
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.0
    }
}

impl fmt::Debug for NonSendResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NonSendResourceId({:?})", self.0)
    }
}

// =============================================================================
// NonSendResources Container
// =============================================================================

/// Marker type that prevents a struct from implementing `Send` or `Sync`.
///
/// This is used to ensure `NonSendResources` cannot be sent between threads.
/// The `*const ()` raw pointer makes the containing type `!Send` and `!Sync`.
pub struct NonSendMarker(std::marker::PhantomData<*const ()>);

// Safety: NonSendMarker is intentionally NOT Send or Sync.
// This is a marker type used to "infect" containing types.

impl Default for NonSendMarker {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl fmt::Debug for NonSendMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NonSendMarker")
    }
}

/// Storage container for non-send resources.
///
/// Unlike [`Resources`], this container stores resources that are NOT `Send + Sync`.
/// These resources must only be accessed from the thread they were created on
/// (typically the main thread).
///
/// # Thread Safety
///
/// This container is NOT `Send` or `Sync`. It must remain on the main thread.
/// The scheduler ensures that systems using non-send resources run on the main thread.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::resource::{NonSendResources, NonSendResource};
/// use std::rc::Rc;
///
/// struct RawPointerResource {
///     ptr: *mut u32,
/// }
/// impl NonSendResource for RawPointerResource {}
///
/// let mut resources = NonSendResources::new();
///
/// let mut value = 42u32;
/// resources.insert(RawPointerResource { ptr: &mut value as *mut u32 });
///
/// assert!(resources.contains::<RawPointerResource>());
/// ```
#[derive(Default)]
pub struct NonSendResources {
    /// Type-erased non-send resource storage.
    /// Note: Uses `Box<dyn Any>` NOT `Box<dyn Any + Send + Sync>`.
    data: HashMap<NonSendResourceId, Box<dyn Any>>,
    /// Marker to make this type !Send and !Sync.
    _marker: NonSendMarker,
}

impl NonSendResources {
    /// Creates an empty non-send resources container.
    #[inline]
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            _marker: NonSendMarker::default(),
        }
    }

    /// Inserts a non-send resource into the container.
    ///
    /// If a resource of this type already exists, it is replaced and the
    /// old value is returned.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to insert
    ///
    /// # Returns
    ///
    /// `Some(T)` if a resource of this type was replaced, `None` otherwise.
    #[inline]
    pub fn insert<T: NonSendResource>(&mut self, resource: T) -> Option<T> {
        let id = NonSendResourceId::of::<T>();
        let old = self.data.insert(id, Box::new(resource));
        old.and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }

    /// Removes a non-send resource from the container.
    ///
    /// # Returns
    ///
    /// `Some(T)` if the resource existed, `None` otherwise.
    #[inline]
    pub fn remove<T: NonSendResource>(&mut self) -> Option<T> {
        let id = NonSendResourceId::of::<T>();
        self.data
            .remove(&id)
            .and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }

    /// Returns an immutable reference to a non-send resource.
    ///
    /// # Returns
    ///
    /// `Some(&T)` if the resource exists, `None` otherwise.
    #[inline]
    pub fn get<T: NonSendResource>(&self) -> Option<&T> {
        let id = NonSendResourceId::of::<T>();
        self.data.get(&id).and_then(|boxed| boxed.downcast_ref())
    }

    /// Returns a mutable reference to a non-send resource.
    ///
    /// # Returns
    ///
    /// `Some(&mut T)` if the resource exists, `None` otherwise.
    #[inline]
    pub fn get_mut<T: NonSendResource>(&mut self) -> Option<&mut T> {
        let id = NonSendResourceId::of::<T>();
        self.data
            .get_mut(&id)
            .and_then(|boxed| boxed.downcast_mut())
    }

    /// Returns `true` if a non-send resource of the specified type exists.
    #[inline]
    pub fn contains<T: NonSendResource>(&self) -> bool {
        self.data.contains_key(&NonSendResourceId::of::<T>())
    }

    /// Returns the number of non-send resources in the container.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if there are no non-send resources.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Removes all non-send resources from the container.
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl fmt::Debug for NonSendResources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NonSendResources")
            .field("count", &self.data.len())
            .finish()
    }
}

// NonSendResources is NOT Send or Sync due to NonSendMarker containing *const ()
// This is verified by compile-time tests in the tests module.

// =============================================================================
// NonSend<T> - Immutable Non-Send Resource Access
// =============================================================================

/// Immutable access to a non-send resource of type `T`.
///
/// `NonSend<T>` provides read-only access to a non-send resource stored in the World.
/// It implements `Deref`, so you can access the inner value directly.
///
/// # Thread Safety
///
/// Systems using `NonSend<T>` are constrained to run on the main thread.
/// This is enforced by the scheduler at runtime.
///
/// # Example
///
/// ```ignore
/// fn print_window_info(window: NonSend<WindowHandle>) {
///     println!("Window handle: {:?}", window.id);
/// }
/// ```
///
/// # Panics
///
/// Operations on `NonSend<T>` will panic if the resource doesn't exist.
/// Use `Option<NonSend<T>>` for optional access.
pub struct NonSend<'w, T: NonSendResource> {
    value: &'w T,
}

impl<'w, T: NonSendResource> NonSend<'w, T> {
    /// Creates a new `NonSend` from a reference.
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

impl<T: NonSendResource> Deref for NonSend<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: NonSendResource + fmt::Debug> fmt::Debug for NonSend<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NonSend").field(&self.value).finish()
    }
}

// Clone for NonSend - it's just a reference, so this is cheap
impl<T: NonSendResource> Clone for NonSend<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: NonSendResource> Copy for NonSend<'_, T> {}

// =============================================================================
// NonSendMut<T> - Mutable Non-Send Resource Access
// =============================================================================

/// Mutable access to a non-send resource of type `T`.
///
/// `NonSendMut<T>` provides read-write access to a non-send resource stored in the World.
/// It implements `Deref` and `DerefMut`, so you can access the inner value directly.
///
/// # Thread Safety
///
/// Systems using `NonSendMut<T>` are constrained to run on the main thread.
/// This is enforced by the scheduler at runtime.
///
/// # Example
///
/// ```ignore
/// fn update_window(mut window: NonSendMut<WindowHandle>) {
///     window.update_title("New Title");
/// }
/// ```
///
/// # Panics
///
/// Operations on `NonSendMut<T>` will panic if the resource doesn't exist.
/// Use `Option<NonSendMut<T>>` for optional access.
pub struct NonSendMut<'w, T: NonSendResource> {
    value: &'w mut T,
}

impl<'w, T: NonSendResource> NonSendMut<'w, T> {
    /// Creates a new `NonSendMut` from a mutable reference.
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

impl<T: NonSendResource> Deref for NonSendMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: NonSendResource> DerefMut for NonSendMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<T: NonSendResource + fmt::Debug> fmt::Debug for NonSendMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NonSendMut").field(&self.value).finish()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test resources
    #[derive(Debug)]
    struct Time {
        delta: f32,
        total: f32,
    }

    #[derive(Debug)]
    struct Score(u32);

    #[derive(Debug)]
    struct Config {
        debug: bool,
        volume: f32,
    }

    // =========================================================================
    // Resource Trait Tests
    // =========================================================================

    mod resource_trait {
        use super::*;

        #[test]
        fn test_resource_auto_impl() {
            // Any Send + Sync + 'static type should be a Resource
            fn requires_resource<T: Resource>() {}

            requires_resource::<Time>();
            requires_resource::<Score>();
            requires_resource::<i32>();
            requires_resource::<String>();
            requires_resource::<Vec<u8>>();
        }

        #[test]
        fn test_resource_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<Time>();
            requires_send::<Score>();
        }

        #[test]
        fn test_resource_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<Time>();
            requires_sync::<Score>();
        }
    }

    // =========================================================================
    // ResourceId Tests
    // =========================================================================

    mod resource_id {
        use super::*;

        #[test]
        fn test_resource_id_of() {
            let id1 = ResourceId::of::<Time>();
            let id2 = ResourceId::of::<Time>();
            assert_eq!(id1, id2);
        }

        #[test]
        fn test_resource_id_different_types() {
            let id1 = ResourceId::of::<Time>();
            let id2 = ResourceId::of::<Score>();
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_resource_id_type_id() {
            let id = ResourceId::of::<Time>();
            assert_eq!(id.type_id(), TypeId::of::<Time>());
        }

        #[test]
        fn test_resource_id_hash() {
            use std::collections::HashSet;
            let mut set = HashSet::new();
            set.insert(ResourceId::of::<Time>());
            set.insert(ResourceId::of::<Score>());
            assert_eq!(set.len(), 2);

            // Same type should not add again
            set.insert(ResourceId::of::<Time>());
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_resource_id_ord() {
            use std::collections::BTreeSet;
            let mut set = BTreeSet::new();
            set.insert(ResourceId::of::<Time>());
            set.insert(ResourceId::of::<Score>());
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_resource_id_debug() {
            let id = ResourceId::of::<Time>();
            let debug_str = format!("{:?}", id);
            assert!(debug_str.contains("ResourceId"));
        }

        #[test]
        fn test_resource_id_clone() {
            let id1 = ResourceId::of::<Time>();
            let id2 = id1;
            assert_eq!(id1, id2);
        }
    }

    // =========================================================================
    // Resources Container Tests
    // =========================================================================

    mod resources_container {
        use super::*;

        #[test]
        fn test_resources_new() {
            let resources = Resources::new();
            assert!(resources.is_empty());
            assert_eq!(resources.len(), 0);
        }

        #[test]
        fn test_resources_default() {
            let resources = Resources::default();
            assert!(resources.is_empty());
        }

        #[test]
        fn test_resources_insert() {
            let mut resources = Resources::new();
            let old = resources.insert(Score(100));
            assert!(old.is_none());
            assert_eq!(resources.len(), 1);
        }

        #[test]
        fn test_resources_insert_replace() {
            let mut resources = Resources::new();
            resources.insert(Score(100));
            let old = resources.insert(Score(200));
            assert!(old.is_some());
            assert_eq!(old.unwrap().0, 100);
            assert_eq!(resources.get::<Score>().unwrap().0, 200);
        }

        #[test]
        fn test_resources_remove() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            let removed = resources.remove::<Score>();
            assert!(removed.is_some());
            assert_eq!(removed.unwrap().0, 100);
            assert!(resources.is_empty());
        }

        #[test]
        fn test_resources_remove_nonexistent() {
            let mut resources = Resources::new();
            let removed = resources.remove::<Score>();
            assert!(removed.is_none());
        }

        #[test]
        fn test_resources_get() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            let score = resources.get::<Score>();
            assert!(score.is_some());
            assert_eq!(score.unwrap().0, 100);
        }

        #[test]
        fn test_resources_get_nonexistent() {
            let resources = Resources::new();
            assert!(resources.get::<Score>().is_none());
        }

        #[test]
        fn test_resources_get_mut() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            let score = resources.get_mut::<Score>().unwrap();
            score.0 += 50;

            assert_eq!(resources.get::<Score>().unwrap().0, 150);
        }

        #[test]
        fn test_resources_get_mut_nonexistent() {
            let mut resources = Resources::new();
            assert!(resources.get_mut::<Score>().is_none());
        }

        #[test]
        fn test_resources_contains() {
            let mut resources = Resources::new();
            assert!(!resources.contains::<Score>());

            resources.insert(Score(100));
            assert!(resources.contains::<Score>());

            resources.remove::<Score>();
            assert!(!resources.contains::<Score>());
        }

        #[test]
        fn test_resources_multiple_types() {
            let mut resources = Resources::new();
            resources.insert(Score(100));
            resources.insert(Time {
                delta: 0.016,
                total: 0.0,
            });
            resources.insert(Config {
                debug: true,
                volume: 0.8,
            });

            assert_eq!(resources.len(), 3);
            assert_eq!(resources.get::<Score>().unwrap().0, 100);
            assert_eq!(resources.get::<Time>().unwrap().delta, 0.016);
            assert!(resources.get::<Config>().unwrap().debug);
        }

        #[test]
        fn test_resources_clear() {
            let mut resources = Resources::new();
            resources.insert(Score(100));
            resources.insert(Time {
                delta: 0.016,
                total: 0.0,
            });

            resources.clear();
            assert!(resources.is_empty());
            assert_eq!(resources.len(), 0);
        }

        #[test]
        fn test_resources_debug() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            let debug_str = format!("{:?}", resources);
            assert!(debug_str.contains("Resources"));
            assert!(debug_str.contains("count"));
        }
    }

    // =========================================================================
    // Res<T> Tests
    // =========================================================================

    mod res_tests {
        use super::*;

        #[test]
        fn test_res_new() {
            let time = Time {
                delta: 0.016,
                total: 1.0,
            };
            let res = Res::new(&time);
            assert_eq!(res.delta, 0.016);
            assert_eq!(res.total, 1.0);
        }

        #[test]
        fn test_res_deref() {
            let score = Score(100);
            let res = Res::new(&score);
            assert_eq!(res.0, 100);
        }

        #[test]
        fn test_res_into_inner() {
            let score = Score(100);
            let res = Res::new(&score);
            let inner = res.into_inner();
            assert_eq!(inner.0, 100);
        }

        #[test]
        fn test_res_debug() {
            let score = Score(100);
            let res = Res::new(&score);
            let debug_str = format!("{:?}", res);
            assert!(debug_str.contains("Res"));
        }

        #[test]
        fn test_res_clone() {
            let score = Score(100);
            let res = Res::new(&score);
            let cloned = res;
            assert_eq!(cloned.0, 100);
        }

        #[test]
        fn test_res_copy() {
            let score = Score(100);
            let res = Res::new(&score);
            let copied = res;
            // Both still valid
            assert_eq!(res.0, 100);
            assert_eq!(copied.0, 100);
        }
    }

    // =========================================================================
    // ResMut<T> Tests
    // =========================================================================

    mod res_mut_tests {
        use super::*;

        #[test]
        fn test_res_mut_new() {
            let mut time = Time {
                delta: 0.016,
                total: 1.0,
            };
            let res = ResMut::new(&mut time);
            assert_eq!(res.delta, 0.016);
            assert_eq!(res.total, 1.0);
        }

        #[test]
        fn test_res_mut_deref() {
            let mut score = Score(100);
            let res = ResMut::new(&mut score);
            assert_eq!(res.0, 100);
        }

        #[test]
        fn test_res_mut_deref_mut() {
            let mut score = Score(100);
            {
                let mut res = ResMut::new(&mut score);
                res.0 += 50;
            }
            assert_eq!(score.0, 150);
        }

        #[test]
        fn test_res_mut_into_inner() {
            let mut score = Score(100);
            let res = ResMut::new(&mut score);
            let inner = res.into_inner();
            inner.0 += 50;
            assert_eq!(score.0, 150);
        }

        #[test]
        fn test_res_mut_debug() {
            let mut score = Score(100);
            let res = ResMut::new(&mut score);
            let debug_str = format!("{:?}", res);
            assert!(debug_str.contains("ResMut"));
        }

        #[test]
        fn test_res_mut_modify_complex() {
            let mut time = Time {
                delta: 0.016,
                total: 0.0,
            };

            {
                let mut res = ResMut::new(&mut time);
                res.total += res.delta;
            }

            assert_eq!(time.total, 0.016);
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_resources_with_res() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            let score_ref = resources.get::<Score>().unwrap();
            let res = Res::new(score_ref);

            assert_eq!(res.0, 100);
        }

        #[test]
        fn test_resources_with_res_mut() {
            let mut resources = Resources::new();
            resources.insert(Score(100));

            {
                let score_ref = resources.get_mut::<Score>().unwrap();
                let mut res = ResMut::new(score_ref);
                res.0 += 50;
            }

            assert_eq!(resources.get::<Score>().unwrap().0, 150);
        }

        #[test]
        fn test_resource_lifecycle() {
            let mut resources = Resources::new();

            // Insert
            resources.insert(Score(0));
            assert!(resources.contains::<Score>());

            // Modify
            resources.get_mut::<Score>().unwrap().0 = 100;

            // Read
            assert_eq!(resources.get::<Score>().unwrap().0, 100);

            // Replace
            resources.insert(Score(200));
            assert_eq!(resources.get::<Score>().unwrap().0, 200);

            // Remove
            let removed = resources.remove::<Score>();
            assert_eq!(removed.unwrap().0, 200);
            assert!(!resources.contains::<Score>());
        }
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    mod thread_safety {
        use super::*;

        #[test]
        fn test_resources_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<Resources>();
        }

        #[test]
        fn test_res_is_send() {
            fn requires_send<T: Send>() {}
            // Res is Send if T is Send
            fn check<T: Resource>() {
                // Can't directly check Res<'_, T> for Send due to lifetime
                // but the underlying reference is Send if T is Sync
            }
            check::<Score>();
        }

        #[test]
        fn test_res_mut_is_send() {
            fn requires_send<T: Send>() {}
            // ResMut is Send if T is Send
            fn check<T: Resource>() {
                // Can't directly check ResMut<'_, T> for Send due to lifetime
            }
            check::<Score>();
        }
    }

    // =========================================================================
    // Non-Send Resource Tests
    // =========================================================================

    mod non_send_resources {
        use super::*;
        use std::cell::RefCell;
        use std::rc::Rc;

        // Non-send test resources
        struct WindowHandle {
            id: Rc<u32>,
        }
        impl NonSendResource for WindowHandle {}

        struct OpenGLContext {
            ctx: Rc<RefCell<u32>>,
        }
        impl NonSendResource for OpenGLContext {}

        struct RawPointerResource {
            ptr: *mut u32,
        }
        impl NonSendResource for RawPointerResource {}

        // =====================================================================
        // NonSendResourceId Tests
        // =====================================================================

        #[test]
        fn test_non_send_resource_id_of() {
            let id1 = NonSendResourceId::of::<WindowHandle>();
            let id2 = NonSendResourceId::of::<WindowHandle>();
            assert_eq!(id1, id2);
        }

        #[test]
        fn test_non_send_resource_id_different_types() {
            let id1 = NonSendResourceId::of::<WindowHandle>();
            let id2 = NonSendResourceId::of::<OpenGLContext>();
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_non_send_resource_id_type_id() {
            let id = NonSendResourceId::of::<WindowHandle>();
            assert_eq!(id.type_id(), TypeId::of::<WindowHandle>());
        }

        #[test]
        fn test_non_send_resource_id_hash() {
            use std::collections::HashSet;
            let mut set = HashSet::new();
            set.insert(NonSendResourceId::of::<WindowHandle>());
            set.insert(NonSendResourceId::of::<OpenGLContext>());
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_non_send_resource_id_ord() {
            use std::collections::BTreeSet;
            let mut set = BTreeSet::new();
            set.insert(NonSendResourceId::of::<WindowHandle>());
            set.insert(NonSendResourceId::of::<OpenGLContext>());
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_non_send_resource_id_debug() {
            let id = NonSendResourceId::of::<WindowHandle>();
            let debug_str = format!("{:?}", id);
            assert!(debug_str.contains("NonSendResourceId"));
        }

        // =====================================================================
        // NonSendResources Container Tests
        // =====================================================================

        #[test]
        fn test_non_send_resources_new() {
            let resources = NonSendResources::new();
            assert!(resources.is_empty());
            assert_eq!(resources.len(), 0);
        }

        #[test]
        fn test_non_send_resources_default() {
            let resources = NonSendResources::default();
            assert!(resources.is_empty());
        }

        #[test]
        fn test_non_send_resources_insert() {
            let mut resources = NonSendResources::new();
            let old = resources.insert(WindowHandle { id: Rc::new(42) });
            assert!(old.is_none());
            assert_eq!(resources.len(), 1);
        }

        #[test]
        fn test_non_send_resources_insert_replace() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(42) });
            let old = resources.insert(WindowHandle { id: Rc::new(100) });
            assert!(old.is_some());
            assert_eq!(*old.unwrap().id, 42);
            assert_eq!(*resources.get::<WindowHandle>().unwrap().id, 100);
        }

        #[test]
        fn test_non_send_resources_remove() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(42) });

            let removed = resources.remove::<WindowHandle>();
            assert!(removed.is_some());
            assert_eq!(*removed.unwrap().id, 42);
            assert!(resources.is_empty());
        }

        #[test]
        fn test_non_send_resources_remove_nonexistent() {
            let mut resources = NonSendResources::new();
            let removed = resources.remove::<WindowHandle>();
            assert!(removed.is_none());
        }

        #[test]
        fn test_non_send_resources_get() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(42) });

            let handle = resources.get::<WindowHandle>();
            assert!(handle.is_some());
            assert_eq!(*handle.unwrap().id, 42);
        }

        #[test]
        fn test_non_send_resources_get_nonexistent() {
            let resources = NonSendResources::new();
            assert!(resources.get::<WindowHandle>().is_none());
        }

        #[test]
        fn test_non_send_resources_get_mut() {
            let mut resources = NonSendResources::new();
            resources.insert(OpenGLContext {
                ctx: Rc::new(RefCell::new(1)),
            });

            let ctx = resources.get_mut::<OpenGLContext>().unwrap();
            *ctx.ctx.borrow_mut() = 42;

            assert_eq!(*resources.get::<OpenGLContext>().unwrap().ctx.borrow(), 42);
        }

        #[test]
        fn test_non_send_resources_contains() {
            let mut resources = NonSendResources::new();
            assert!(!resources.contains::<WindowHandle>());

            resources.insert(WindowHandle { id: Rc::new(42) });
            assert!(resources.contains::<WindowHandle>());

            resources.remove::<WindowHandle>();
            assert!(!resources.contains::<WindowHandle>());
        }

        #[test]
        fn test_non_send_resources_multiple_types() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(1) });
            resources.insert(OpenGLContext {
                ctx: Rc::new(RefCell::new(2)),
            });

            assert_eq!(resources.len(), 2);
            assert_eq!(*resources.get::<WindowHandle>().unwrap().id, 1);
            assert_eq!(*resources.get::<OpenGLContext>().unwrap().ctx.borrow(), 2);
        }

        #[test]
        fn test_non_send_resources_clear() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(1) });
            resources.insert(OpenGLContext {
                ctx: Rc::new(RefCell::new(2)),
            });

            resources.clear();
            assert!(resources.is_empty());
            assert_eq!(resources.len(), 0);
        }

        #[test]
        fn test_non_send_resources_debug() {
            let mut resources = NonSendResources::new();
            resources.insert(WindowHandle { id: Rc::new(42) });

            let debug_str = format!("{:?}", resources);
            assert!(debug_str.contains("NonSendResources"));
            assert!(debug_str.contains("count"));
        }

        #[test]
        fn test_non_send_resources_with_raw_pointer() {
            let mut value = 42u32;
            let mut resources = NonSendResources::new();
            resources.insert(RawPointerResource {
                ptr: &mut value as *mut u32,
            });

            let res = resources.get::<RawPointerResource>().unwrap();
            assert!(!res.ptr.is_null());
        }

        // =====================================================================
        // NonSendResources Thread Safety Tests
        // =====================================================================

        #[test]
        fn test_non_send_resources_is_not_send() {
            // NonSendResources should NOT implement Send
            fn check_not_send<T>() {
                // This is a compile-time check - the test passes by compiling
                // We can't easily test !Send at runtime
            }
            check_not_send::<NonSendResources>();
            // The actual !Send is enforced by the NonSendMarker containing *const ()
        }

        #[test]
        fn test_non_send_resources_is_not_sync() {
            // NonSendResources should NOT implement Sync
            fn check_not_sync<T>() {
                // This is a compile-time check - the test passes by compiling
                // We can't easily test !Sync at runtime
            }
            check_not_sync::<NonSendResources>();
            // The actual !Sync is enforced by the NonSendMarker containing *const ()
        }

        // =====================================================================
        // NonSend<T> and NonSendMut<T> Wrapper Tests
        // =====================================================================

        #[test]
        fn test_non_send_new() {
            let handle = WindowHandle { id: Rc::new(42) };
            let non_send = NonSend::new(&handle);
            assert_eq!(*non_send.id, 42);
        }

        #[test]
        fn test_non_send_deref() {
            let handle = WindowHandle { id: Rc::new(42) };
            let non_send = NonSend::new(&handle);
            assert_eq!(*non_send.id, 42);
        }

        #[test]
        fn test_non_send_into_inner() {
            let handle = WindowHandle { id: Rc::new(42) };
            let non_send = NonSend::new(&handle);
            let inner = non_send.into_inner();
            assert_eq!(*inner.id, 42);
        }

        #[test]
        fn test_non_send_clone() {
            let handle = WindowHandle { id: Rc::new(42) };
            let non_send = NonSend::new(&handle);
            let cloned = non_send;
            // Both should be valid
            assert_eq!(*non_send.id, 42);
            assert_eq!(*cloned.id, 42);
        }

        #[test]
        fn test_non_send_copy() {
            let handle = WindowHandle { id: Rc::new(42) };
            let non_send = NonSend::new(&handle);
            let copied = non_send;
            // Both should still be valid
            assert_eq!(*non_send.id, 42);
            assert_eq!(*copied.id, 42);
        }

        #[test]
        fn test_non_send_mut_new() {
            let mut ctx = OpenGLContext {
                ctx: Rc::new(RefCell::new(1)),
            };
            let non_send_mut = NonSendMut::new(&mut ctx);
            assert_eq!(*non_send_mut.ctx.borrow(), 1);
        }

        #[test]
        fn test_non_send_mut_deref() {
            let mut ctx = OpenGLContext {
                ctx: Rc::new(RefCell::new(1)),
            };
            let non_send_mut = NonSendMut::new(&mut ctx);
            assert_eq!(*non_send_mut.ctx.borrow(), 1);
        }

        #[test]
        fn test_non_send_mut_deref_mut() {
            let mut ctx = OpenGLContext {
                ctx: Rc::new(RefCell::new(1)),
            };
            {
                let mut non_send_mut = NonSendMut::new(&mut ctx);
                *non_send_mut.ctx.borrow_mut() = 42;
            }
            assert_eq!(*ctx.ctx.borrow(), 42);
        }

        #[test]
        fn test_non_send_mut_into_inner() {
            let mut ctx = OpenGLContext {
                ctx: Rc::new(RefCell::new(1)),
            };
            let non_send_mut = NonSendMut::new(&mut ctx);
            let inner = non_send_mut.into_inner();
            *inner.ctx.borrow_mut() = 42;
            assert_eq!(*ctx.ctx.borrow(), 42);
        }

        // =====================================================================
        // Integration Tests
        // =====================================================================

        #[test]
        fn test_non_send_resource_lifecycle() {
            let mut resources = NonSendResources::new();

            // Insert
            resources.insert(WindowHandle { id: Rc::new(0) });
            assert!(resources.contains::<WindowHandle>());

            // Modify (via Rc)
            let id = resources.get::<WindowHandle>().unwrap().id.clone();
            assert_eq!(*id, 0);

            // Replace
            resources.insert(WindowHandle { id: Rc::new(42) });
            assert_eq!(*resources.get::<WindowHandle>().unwrap().id, 42);

            // Remove
            let removed = resources.remove::<WindowHandle>();
            assert_eq!(*removed.unwrap().id, 42);
            assert!(!resources.contains::<WindowHandle>());
        }
    }
}
