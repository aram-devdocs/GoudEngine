//! Core resource types: [`Resource`] trait, [`ResourceId`], and [`Resources`] container.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;

// =============================================================================
// Resource Trait
// =============================================================================

/// Marker trait for types that can be used as resources.
///
/// Resources are singleton data stored in the [`World`](crate::ecs::World).
/// To make a type usable as a resource, implement this trait:
///
/// ```ignore
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
