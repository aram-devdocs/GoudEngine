use super::super::resource::{Res, ResMut, Resource};
use super::World;

impl World {
    // =========================================================================
    // Resource Management
    // =========================================================================

    /// Inserts a resource into the world.
    ///
    /// Resources are singleton data that exists outside the entity-component
    /// model. Unlike components, each resource type can only have one instance.
    ///
    /// If a resource of this type already exists, it is replaced and the old
    /// value is returned.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type (must implement `Send + Sync + 'static`)
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource value to insert
    ///
    /// # Returns
    ///
    /// - `Some(old_resource)` if a resource of this type was replaced
    /// - `None` if this is a new resource type
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Time { delta: f32, total: f32 }
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Time { delta: 0.016, total: 0.0 });
    ///
    /// let time = world.get_resource::<Time>().unwrap();
    /// assert_eq!(time.delta, 0.016);
    /// ```
    #[inline]
    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> Option<T> {
        self.resources.insert(resource)
    }

    /// Removes a resource from the world and returns it.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to remove
    ///
    /// # Returns
    ///
    /// - `Some(resource)` if the resource existed
    /// - `None` if no resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Config { debug: bool }
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Config { debug: true });
    ///
    /// let config = world.remove_resource::<Config>();
    /// assert!(config.is_some());
    /// assert!(world.get_resource::<Config>().is_none());
    /// ```
    #[inline]
    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.resources.remove()
    }

    /// Returns an immutable reference to a resource.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(&T)` if the resource exists
    /// - `None` if no resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Score(100));
    ///
    /// if let Some(score) = world.get_resource::<Score>() {
    ///     println!("Score: {}", score.0);
    /// }
    /// ```
    #[inline]
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.resources.get()
    }

    /// Returns a mutable reference to a resource.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(&mut T)` if the resource exists
    /// - `None` if no resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Score(100));
    ///
    /// if let Some(score) = world.get_resource_mut::<Score>() {
    ///     score.0 += 50;
    /// }
    ///
    /// assert_eq!(world.get_resource::<Score>().unwrap().0, 150);
    /// ```
    #[inline]
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resources.get_mut()
    }

    /// Returns an immutable [`Res`] wrapper for a resource.
    ///
    /// This is the primary way to access resources in systems. The `Res<T>`
    /// wrapper provides convenient access via `Deref`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(Res<T>)` if the resource exists
    /// - `None` if no resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Time { delta: f32 }
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Time { delta: 0.016 });
    ///
    /// let time = world.resource::<Time>().unwrap();
    /// assert_eq!(time.delta, 0.016);
    /// ```
    #[inline]
    pub fn resource<T: Resource>(&self) -> Option<Res<'_, T>> {
        self.resources.get::<T>().map(Res::new)
    }

    /// Returns a mutable [`ResMut`] wrapper for a resource.
    ///
    /// This is the primary way to mutably access resources in systems. The
    /// `ResMut<T>` wrapper provides convenient access via `Deref` and `DerefMut`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to access
    ///
    /// # Returns
    ///
    /// - `Some(ResMut<T>)` if the resource exists
    /// - `None` if no resource of this type exists
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Score(100));
    ///
    /// {
    ///     let mut score = world.resource_mut::<Score>().unwrap();
    ///     score.0 += 50;
    /// }
    ///
    /// assert_eq!(world.get_resource::<Score>().unwrap().0, 150);
    /// ```
    #[inline]
    pub fn resource_mut<T: Resource>(&mut self) -> Option<ResMut<'_, T>> {
        self.resources.get_mut::<T>().map(ResMut::new)
    }

    /// Returns `true` if a resource of the specified type exists.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The resource type to check
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    ///
    /// let mut world = World::new();
    /// assert!(!world.contains_resource::<Score>());
    ///
    /// world.insert_resource(Score(100));
    /// assert!(world.contains_resource::<Score>());
    /// ```
    #[inline]
    pub fn contains_resource<T: Resource>(&self) -> bool {
        self.resources.contains::<T>()
    }

    /// Returns the number of resources in the world.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    /// struct Time { delta: f32 }
    ///
    /// let mut world = World::new();
    /// assert_eq!(world.resource_count(), 0);
    ///
    /// world.insert_resource(Score(100));
    /// world.insert_resource(Time { delta: 0.016 });
    /// assert_eq!(world.resource_count(), 2);
    /// ```
    #[inline]
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// Clears all resources from the world.
    ///
    /// This removes all resources but leaves entities and components intact.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// struct Score(u32);
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Score(100));
    ///
    /// world.clear_resources();
    /// assert_eq!(world.resource_count(), 0);
    /// ```
    #[inline]
    pub fn clear_resources(&mut self) {
        self.resources.clear();
    }
}
