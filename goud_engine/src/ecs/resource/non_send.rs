//! Non-send resource types for thread-affine data.
//!
//! Non-send resources are resources that cannot be safely sent between threads.
//! They must be accessed only from the thread they were created on (typically
//! the main thread).

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::ops::{Deref, DerefMut};

// =============================================================================
// NonSendResource Trait
// =============================================================================

/// Marker trait for types that can be used as non-send resources.
///
/// Non-send resources are resources that cannot be safely sent between threads.
/// Unlike regular [`Resource`](super::types::Resource), non-send resources only require `'static`.
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
/// Like [`ResourceId`](super::types::ResourceId) but for non-send resources.
/// Used internally for storage and access conflict detection.
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
/// Unlike [`Resources`](super::types::Resources), this container stores resources
/// that are NOT `Send + Sync`. These resources must only be accessed from the
/// thread they were created on (typically the main thread).
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
pub struct NonSendResources {
    /// Type-erased non-send resource storage.
    /// Note: Uses `Box<dyn Any>` NOT `Box<dyn Any + Send + Sync>`.
    data: HashMap<NonSendResourceId, Box<dyn Any>>,
    /// Marker to make this type !Send and !Sync.
    _marker: NonSendMarker,
    /// The thread this container was created on. All access is validated against this.
    main_thread_id: std::thread::ThreadId,
}

impl Default for NonSendResources {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
            _marker: NonSendMarker::default(),
            main_thread_id: std::thread::current().id(),
        }
    }
}

impl NonSendResources {
    /// Creates an empty non-send resources container.
    ///
    /// Records the current thread as the "main" thread for access validation.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the thread ID this container was created on.
    #[inline]
    pub fn main_thread_id(&self) -> std::thread::ThreadId {
        self.main_thread_id
    }

    /// Panics if the current thread is not the main thread.
    #[inline]
    fn validate_main_thread(&self) {
        let current = std::thread::current().id();
        assert!(
            current == self.main_thread_id,
            "Non-send resource accessed from thread {current:?}, \
             but was created on thread {:?}. \
             Non-send resources can only be accessed from the main thread.",
            self.main_thread_id
        );
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
        self.validate_main_thread();
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
        self.validate_main_thread();
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
        self.validate_main_thread();
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
        self.validate_main_thread();
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
        self.data.clear()
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
