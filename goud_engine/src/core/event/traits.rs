//! The [`Event`] marker trait and its blanket implementation.

/// Marker trait for types that can be sent through the event system.
///
/// This trait is automatically implemented for any type that satisfies
/// `Send + Sync + 'static`. These bounds ensure:
///
/// - `Send`: Events can be transferred between threads
/// - `Sync`: Event references can be shared between threads
/// - `'static`: Events don't contain borrowed data
///
/// # Blanket Implementation
///
/// You don't need to manually implement this trait. Any type meeting the
/// bounds automatically qualifies:
///
/// ```rust
/// use goud_engine::core::event::Event;
///
/// struct MyEvent {
///     data: i32,
/// }
///
/// // This compiles because MyEvent is Send + Sync + 'static
/// fn accepts_event<E: Event>(_: E) {}
/// accepts_event(MyEvent { data: 42 });
/// ```
///
/// # Non-Qualifying Types
///
/// Types with non-static lifetimes or non-thread-safe internals won't
/// implement Event:
///
/// ```compile_fail
/// use std::rc::Rc;
/// use goud_engine::core::event::Event;
///
/// struct BadEvent {
///     data: Rc<i32>, // Rc is not Send
/// }
///
/// fn accepts_event<E: Event>(_: E) {}
/// accepts_event(BadEvent { data: Rc::new(42) }); // Won't compile
/// ```
pub trait Event: Send + Sync + 'static {}

/// Blanket implementation of Event for all qualifying types.
///
/// This ensures any `Send + Sync + 'static` type can be used as an event
/// without explicit implementation.
impl<T: Send + Sync + 'static> Event for T {}
