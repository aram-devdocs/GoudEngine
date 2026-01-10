//! Component trait and utilities for the ECS.
//!
//! Components are data types that can be attached to entities. This module defines
//! the [`Component`] trait that marks types as valid components, along with
//! component identification utilities.
//!
//! # Design Philosophy
//!
//! Unlike some ECS implementations that use blanket implementations, GoudEngine
//! requires explicit opt-in for component types. This provides:
//!
//! - **Safety**: Only intentionally marked types become components
//! - **Clarity**: Clear distinction between data and component types
//! - **Control**: Future derive macro can add custom behavior
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::Component;
//!
//! // Define a position component
//! #[derive(Debug, Clone, Copy)]
//! struct Position {
//!     x: f32,
//!     y: f32,
//! }
//!
//! // Explicitly implement Component
//! impl Component for Position {}
//!
//! // Define a velocity component
//! #[derive(Debug, Clone, Copy)]
//! struct Velocity {
//!     x: f32,
//!     y: f32,
//! }
//!
//! impl Component for Velocity {}
//! ```
//!
//! # Future: Derive Macro
//!
//! In the future, a derive macro will simplify component definition:
//!
//! ```ignore
//! #[derive(Component)]
//! struct Position { x: f32, y: f32 }
//! ```
//!
//! The derive macro will automatically implement the Component trait and
//! potentially add reflection, serialization, or other capabilities.

use std::any::TypeId;

/// Marker trait for types that can be used as ECS components.
///
/// Components must be:
/// - `Send`: Can be transferred between threads
/// - `Sync`: Can be shared between threads via references
/// - `'static`: No borrowed data (required for type erasure and storage)
///
/// # Thread Safety
///
/// The `Send + Sync` bounds enable parallel system execution. Systems can
/// safely access components from multiple threads when access patterns
/// don't conflict (multiple readers or single writer).
///
/// # Implementation
///
/// Components require explicit opt-in via implementation:
///
/// ```
/// use goud_engine::ecs::Component;
///
/// struct Health(pub f32);
/// impl Component for Health {}
/// ```
///
/// This is intentional - not all `Send + Sync + 'static` types should
/// automatically be components. The explicit implementation:
///
/// 1. Documents intent that this type is meant for ECS use
/// 2. Allows future derive macro to add behavior
/// 3. Prevents accidental use of inappropriate types as components
///
/// # What Makes a Good Component?
///
/// - **Data-only**: Components should be pure data, no behavior
/// - **Small**: Prefer many small components over few large ones
/// - **Focused**: Each component represents one aspect of an entity
///
/// Good:
/// ```
/// # use goud_engine::ecs::Component;
/// struct Position { x: f32, y: f32, z: f32 }
/// impl Component for Position {}
///
/// struct Velocity { x: f32, y: f32, z: f32 }
/// impl Component for Velocity {}
/// ```
///
/// Avoid:
/// ```ignore
/// // Too large - combines unrelated data
/// struct Entity {
///     position: (f32, f32, f32),
///     velocity: (f32, f32, f32),
///     health: f32,
///     name: String,
/// }
/// ```
pub trait Component: Send + Sync + 'static {}

// NOTE: We intentionally do NOT provide a blanket implementation.
// Components must be explicitly opted-in. This differs from the Event trait
// which uses a blanket impl because events are more transient and any
// Send + Sync + 'static type is a reasonable event.
//
// For components, explicit opt-in provides:
// 1. Documentation that the type is intended as a component
// 2. Future extensibility for derive macros
// 3. Prevention of accidental component use

/// Unique identifier for a component type at runtime.
///
/// `ComponentId` wraps a [`TypeId`] and provides component-specific operations.
/// It's used as a key in archetype definitions and storage lookups.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{Component, ComponentId};
///
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// let id = ComponentId::of::<Position>();
/// println!("Position component ID: {:?}", id);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentId(TypeId);

impl ComponentId {
    /// Creates a `ComponentId` for the given component type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Component, ComponentId};
    ///
    /// struct Health(f32);
    /// impl Component for Health {}
    ///
    /// let id = ComponentId::of::<Health>();
    /// ```
    #[inline]
    pub fn of<T: Component>() -> Self {
        ComponentId(TypeId::of::<T>())
    }

    /// Returns the inner `TypeId`.
    ///
    /// Useful for advanced use cases or debugging.
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.0
    }
}

impl std::fmt::Debug for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TypeId doesn't provide type name, but Debug shows enough for debugging
        write!(f, "ComponentId({:?})", self.0)
    }
}

/// Metadata about a component type.
///
/// Provides runtime information about a component including its name, size,
/// and alignment. Useful for debugging, reflection, and memory layout calculations.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{Component, ComponentInfo};
///
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// let info = ComponentInfo::of::<Position>();
/// println!("Component: {} (size: {}, align: {})", info.name, info.size, info.align);
/// ```
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    /// Unique identifier for this component type.
    pub id: ComponentId,
    /// Type name (from `std::any::type_name`).
    pub name: &'static str,
    /// Size in bytes.
    pub size: usize,
    /// Memory alignment in bytes.
    pub align: usize,
}

impl ComponentInfo {
    /// Creates `ComponentInfo` for the given component type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{Component, ComponentInfo};
    ///
    /// struct Velocity { x: f32, y: f32 }
    /// impl Component for Velocity {}
    ///
    /// let info = ComponentInfo::of::<Velocity>();
    /// assert_eq!(info.size, std::mem::size_of::<Velocity>());
    /// ```
    #[inline]
    pub fn of<T: Component>() -> Self {
        ComponentInfo {
            id: ComponentId::of::<T>(),
            name: std::any::type_name::<T>(),
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example components for testing
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    #[derive(Debug, Clone, PartialEq)]
    struct Name(String);
    impl Component for Name {}

    // Zero-sized component (marker/tag)
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Player;
    impl Component for Player {}

    // ==========================================================================
    // Component Trait Tests
    // ==========================================================================

    mod component_trait {
        use super::*;

        #[test]
        fn test_component_is_send() {
            fn assert_send<T: Send>() {}
            assert_send::<Position>();
            assert_send::<Velocity>();
            assert_send::<Name>();
            assert_send::<Player>();
        }

        #[test]
        fn test_component_is_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<Position>();
            assert_sync::<Velocity>();
            assert_sync::<Name>();
            assert_sync::<Player>();
        }

        #[test]
        fn test_component_is_static() {
            fn assert_static<T: 'static>() {}
            assert_static::<Position>();
            assert_static::<Velocity>();
            assert_static::<Name>();
            assert_static::<Player>();
        }

        #[test]
        fn test_component_can_be_boxed_as_any() {
            // Components can be type-erased and stored
            use std::any::Any;

            fn store_component<T: Component>(component: T) -> Box<dyn Any + Send + Sync> {
                Box::new(component)
            }

            let pos = Position { x: 1.0, y: 2.0 };
            let boxed = store_component(pos);

            // Can downcast back
            let recovered = boxed.downcast::<Position>().unwrap();
            assert_eq!(*recovered, pos);
        }
    }

    // ==========================================================================
    // ComponentId Tests
    // ==========================================================================

    mod component_id {
        use super::*;

        #[test]
        fn test_component_id_of_returns_same_id_for_same_type() {
            let id1 = ComponentId::of::<Position>();
            let id2 = ComponentId::of::<Position>();
            assert_eq!(id1, id2);
        }

        #[test]
        fn test_component_id_differs_between_types() {
            let pos_id = ComponentId::of::<Position>();
            let vel_id = ComponentId::of::<Velocity>();
            assert_ne!(pos_id, vel_id);
        }

        #[test]
        fn test_component_id_hash_consistency() {
            use std::collections::HashMap;

            let mut map: HashMap<ComponentId, &str> = HashMap::new();
            map.insert(ComponentId::of::<Position>(), "position");
            map.insert(ComponentId::of::<Velocity>(), "velocity");

            assert_eq!(map.get(&ComponentId::of::<Position>()), Some(&"position"));
            assert_eq!(map.get(&ComponentId::of::<Velocity>()), Some(&"velocity"));
        }

        #[test]
        fn test_component_id_ordering() {
            // ComponentId implements Ord for use in BTreeSet/BTreeMap
            use std::collections::BTreeSet;

            let mut set: BTreeSet<ComponentId> = BTreeSet::new();
            set.insert(ComponentId::of::<Position>());
            set.insert(ComponentId::of::<Velocity>());
            set.insert(ComponentId::of::<Name>());

            assert_eq!(set.len(), 3);

            // Inserting same ID again doesn't increase size
            set.insert(ComponentId::of::<Position>());
            assert_eq!(set.len(), 3);
        }

        #[test]
        fn test_component_id_debug_format() {
            let id = ComponentId::of::<Position>();
            let debug_str = format!("{id:?}");
            assert!(debug_str.contains("ComponentId"));
        }

        #[test]
        fn test_component_id_type_id() {
            let id = ComponentId::of::<Position>();
            assert_eq!(id.type_id(), TypeId::of::<Position>());
        }

        #[test]
        fn test_component_id_zero_sized_type() {
            // Zero-sized types (like marker components) should work
            let id = ComponentId::of::<Player>();
            assert_eq!(id, ComponentId::of::<Player>());
        }

        #[test]
        fn test_component_id_generic_types_differ() {
            // Generic instantiations should have different IDs
            #[derive(Debug)]
            struct Container<T>(T);
            impl<T: Send + Sync + 'static> Component for Container<T> {}

            let id_u32 = ComponentId::of::<Container<u32>>();
            let id_f32 = ComponentId::of::<Container<f32>>();
            assert_ne!(id_u32, id_f32);
        }
    }

    // ==========================================================================
    // ComponentInfo Tests
    // ==========================================================================

    mod component_info {
        use super::*;

        #[test]
        fn test_component_info_of() {
            let info = ComponentInfo::of::<Position>();

            assert_eq!(info.id, ComponentId::of::<Position>());
            assert!(info.name.contains("Position"));
            assert_eq!(info.size, std::mem::size_of::<Position>());
            assert_eq!(info.align, std::mem::align_of::<Position>());
        }

        #[test]
        fn test_component_info_size_and_align() {
            let pos_info = ComponentInfo::of::<Position>();
            // Position has 2 f32 fields = 8 bytes, align 4
            assert_eq!(pos_info.size, 8);
            assert_eq!(pos_info.align, 4);

            let name_info = ComponentInfo::of::<Name>();
            // Name contains String which is 24 bytes on 64-bit
            assert_eq!(name_info.size, std::mem::size_of::<Name>());
        }

        #[test]
        fn test_component_info_zero_sized() {
            let info = ComponentInfo::of::<Player>();
            assert_eq!(info.size, 0);
            assert_eq!(info.align, 1);
        }

        #[test]
        fn test_component_info_clone() {
            let info1 = ComponentInfo::of::<Position>();
            let info2 = info1.clone();

            assert_eq!(info1.id, info2.id);
            assert_eq!(info1.name, info2.name);
            assert_eq!(info1.size, info2.size);
            assert_eq!(info1.align, info2.align);
        }

        #[test]
        fn test_component_info_debug() {
            let info = ComponentInfo::of::<Position>();
            let debug_str = format!("{info:?}");

            assert!(debug_str.contains("ComponentInfo"));
            assert!(debug_str.contains("Position"));
        }
    }

    // ==========================================================================
    // No Blanket Implementation Tests
    // ==========================================================================

    mod no_blanket_impl {
        use super::*;

        // This type is Send + Sync + 'static but NOT a Component
        #[derive(Debug)]
        struct NotAComponent {
            data: i32,
        }

        // Compile-time test: This should NOT compile if uncommented,
        // proving there's no blanket implementation
        //
        // fn try_use_as_component<T: Component>(_: T) {}
        // fn test_not_a_component() {
        //     try_use_as_component(NotAComponent { data: 42 });
        // }

        #[test]
        fn test_explicit_impl_required() {
            // This test documents that Component requires explicit implementation.
            // The commented code above would fail to compile because NotAComponent
            // doesn't implement Component, even though it's Send + Sync + 'static.

            // We can still use NotAComponent normally, just not as a component
            let _not_component = NotAComponent { data: 42 };

            // But types that DO implement Component work fine
            fn use_component<T: Component>(_: T) {}
            use_component(Position { x: 0.0, y: 0.0 });
        }
    }
}
