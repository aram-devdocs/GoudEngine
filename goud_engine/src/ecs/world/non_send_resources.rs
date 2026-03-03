use super::super::resource::{NonSend, NonSendMut, NonSendResource};
use super::World;

impl World {
    // =========================================================================
    // Non-Send Resources
    // =========================================================================

    /// Inserts a non-send resource into the world.
    ///
    /// Non-send resources are resources that cannot be safely sent between threads.
    /// Unlike regular resources, each non-send resource type can only have one instance
    /// and must only be accessed from the main thread.
    ///
    /// If a non-send resource of this type already exists, it is replaced and the old
    /// value is returned.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type (must implement `NonSendResource`)
    ///
    /// # Arguments
    ///
    /// * `resource` - The non-send resource value to insert
    ///
    /// # Returns
    ///
    /// - `Some(old_resource)` if a non-send resource of this type was replaced
    /// - `None` if this is a new non-send resource type
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    ///
    /// let handle = world.get_non_send_resource::<WindowHandle>().unwrap();
    /// assert_eq!(*handle.id, 42);
    /// ```
    #[inline]
    pub fn insert_non_send_resource<T: NonSendResource>(&mut self, resource: T) -> Option<T> {
        self.non_send_resources.insert(resource)
    }

    /// Removes a non-send resource from the world and returns it.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to remove
    ///
    /// # Returns
    ///
    /// - `Some(resource)` if the non-send resource existed
    /// - `None` if no non-send resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    ///
    /// let handle = world.remove_non_send_resource::<WindowHandle>();
    /// assert!(handle.is_some());
    /// assert!(world.get_non_send_resource::<WindowHandle>().is_none());
    /// ```
    #[inline]
    pub fn remove_non_send_resource<T: NonSendResource>(&mut self) -> Option<T> {
        self.non_send_resources.remove()
    }

    /// Returns an immutable reference to a non-send resource.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(&T)` if the non-send resource exists
    /// - `None` if no non-send resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    ///
    /// if let Some(handle) = world.get_non_send_resource::<WindowHandle>() {
    ///     println!("Window ID: {}", handle.id);
    /// }
    /// ```
    #[inline]
    pub fn get_non_send_resource<T: NonSendResource>(&self) -> Option<&T> {
        self.non_send_resources.get()
    }

    /// Returns a mutable reference to a non-send resource.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(&mut T)` if the non-send resource exists
    /// - `None` if no non-send resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// struct Counter { value: Rc<RefCell<u32>> }
    /// impl NonSendResource for Counter {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(Counter { value: Rc::new(RefCell::new(0)) });
    ///
    /// if let Some(counter) = world.get_non_send_resource_mut::<Counter>() {
    ///     *counter.value.borrow_mut() += 1;
    /// }
    /// ```
    #[inline]
    pub fn get_non_send_resource_mut<T: NonSendResource>(&mut self) -> Option<&mut T> {
        self.non_send_resources.get_mut()
    }

    /// Returns an immutable [`NonSend`] wrapper for a non-send resource.
    ///
    /// This is the primary way to access non-send resources in systems. The `NonSend<T>`
    /// wrapper implements `Deref`, allowing direct access to the resource.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(NonSend<T>)` if the non-send resource exists
    /// - `None` if no non-send resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    ///
    /// let handle = world.non_send_resource::<WindowHandle>().unwrap();
    /// println!("Window ID: {}", handle.id);
    /// ```
    #[inline]
    pub fn non_send_resource<T: NonSendResource>(&self) -> Option<NonSend<'_, T>> {
        self.non_send_resources.get::<T>().map(NonSend::new)
    }

    /// Returns a mutable [`NonSendMut`] wrapper for a non-send resource.
    ///
    /// This is the primary way to mutably access non-send resources in systems. The
    /// `NonSendMut<T>` wrapper implements `Deref` and `DerefMut`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(NonSendMut<T>)` if the non-send resource exists
    /// - `None` if no non-send resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    ///
    /// struct Counter { value: Rc<RefCell<u32>> }
    /// impl NonSendResource for Counter {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(Counter { value: Rc::new(RefCell::new(0)) });
    ///
    /// {
    ///     let mut counter = world.non_send_resource_mut::<Counter>().unwrap();
    ///     *counter.value.borrow_mut() += 1;
    /// }
    /// ```
    #[inline]
    pub fn non_send_resource_mut<T: NonSendResource>(&mut self) -> Option<NonSendMut<'_, T>> {
        self.non_send_resources.get_mut::<T>().map(NonSendMut::new)
    }

    /// Returns `true` if a non-send resource of the specified type exists.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The non-send resource type to check
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// assert!(!world.contains_non_send_resource::<WindowHandle>());
    ///
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    /// assert!(world.contains_non_send_resource::<WindowHandle>());
    /// ```
    #[inline]
    pub fn contains_non_send_resource<T: NonSendResource>(&self) -> bool {
        self.non_send_resources.contains::<T>()
    }

    /// Returns the number of non-send resources in the world.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// struct OpenGLContext { ctx: Rc<u32> }
    /// impl NonSendResource for OpenGLContext {}
    ///
    /// let mut world = World::new();
    /// assert_eq!(world.non_send_resource_count(), 0);
    ///
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(1) });
    /// world.insert_non_send_resource(OpenGLContext { ctx: Rc::new(2) });
    /// assert_eq!(world.non_send_resource_count(), 2);
    /// ```
    #[inline]
    pub fn non_send_resource_count(&self) -> usize {
        self.non_send_resources.len()
    }

    /// Clears all non-send resources from the world.
    ///
    /// This removes all non-send resources but leaves entities, components,
    /// and regular resources intact.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, resource::NonSendResource};
    /// use std::rc::Rc;
    ///
    /// struct WindowHandle { id: Rc<u32> }
    /// impl NonSendResource for WindowHandle {}
    ///
    /// let mut world = World::new();
    /// world.insert_non_send_resource(WindowHandle { id: Rc::new(42) });
    ///
    /// world.clear_non_send_resources();
    /// assert_eq!(world.non_send_resource_count(), 0);
    /// ```
    #[inline]
    pub fn clear_non_send_resources(&mut self) {
        self.non_send_resources.clear();
    }
}
