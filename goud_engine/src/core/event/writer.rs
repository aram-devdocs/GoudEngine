//! Event writer type for sending events to a queue.

use crate::core::event::{queue::EventQueue, traits::Event};

/// Write-only accessor for sending events to an [`EventQueue`].
///
/// `EventWriter` provides exclusive (mutable) access to send events to a queue.
/// Only one writer can exist at a time due to Rust's borrowing rules, which
/// prevents data races.
///
/// # Usage Pattern
///
/// In an ECS context, systems that need to send events request an `EventWriter`
/// as a system parameter. The borrow checker ensures no other system can
/// simultaneously write to the same event queue.
///
/// # Example
///
/// ```rust
/// use goud_engine::core::event::{EventQueue, EventWriter};
///
/// #[derive(Debug)]
/// struct ExplosionEvent {
///     x: f32,
///     y: f32,
///     radius: f32,
/// }
///
/// let mut queue: EventQueue<ExplosionEvent> = EventQueue::new();
///
/// // Create a writer
/// let mut writer = EventWriter::new(&mut queue);
///
/// // Send events
/// writer.send(ExplosionEvent { x: 10.0, y: 20.0, radius: 5.0 });
/// writer.send(ExplosionEvent { x: 30.0, y: 40.0, radius: 3.0 });
///
/// // Writer is dropped here, releasing the mutable borrow
///
/// // Now we can swap and read
/// queue.swap_buffers();
/// ```
pub struct EventWriter<'a, E: Event> {
    /// Mutable reference to the event queue
    queue: &'a mut EventQueue<E>,
}

impl<'a, E: Event> EventWriter<'a, E> {
    /// Creates a new `EventWriter` for the given queue.
    ///
    /// This takes a mutable reference to the queue, ensuring exclusive access.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::{EventQueue, EventWriter};
    ///
    /// struct MyEvent { data: i32 }
    ///
    /// let mut queue: EventQueue<MyEvent> = EventQueue::new();
    /// let mut writer = EventWriter::new(&mut queue);
    ///
    /// writer.send(MyEvent { data: 42 });
    /// ```
    #[must_use]
    pub fn new(queue: &'a mut EventQueue<E>) -> Self {
        Self { queue }
    }

    /// Sends a single event to the queue.
    ///
    /// The event is written to the write buffer and will be available for
    /// reading after the next `swap_buffers()` call.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::{EventQueue, EventWriter};
    ///
    /// struct InputEvent { key: char }
    ///
    /// let mut queue: EventQueue<InputEvent> = EventQueue::new();
    /// let mut writer = EventWriter::new(&mut queue);
    ///
    /// writer.send(InputEvent { key: 'w' });
    /// writer.send(InputEvent { key: 'a' });
    /// writer.send(InputEvent { key: 's' });
    /// writer.send(InputEvent { key: 'd' });
    /// ```
    pub fn send(&mut self, event: E) {
        self.queue.send(event);
    }

    /// Sends multiple events to the queue in batch.
    ///
    /// This is more efficient than calling `send()` multiple times when you
    /// have a collection of events to send.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::{EventQueue, EventWriter};
    ///
    /// #[derive(Clone)]
    /// struct SpawnEvent { entity_id: u32 }
    ///
    /// let mut queue: EventQueue<SpawnEvent> = EventQueue::new();
    /// let mut writer = EventWriter::new(&mut queue);
    ///
    /// // Send a batch of events
    /// let spawns = vec![
    ///     SpawnEvent { entity_id: 1 },
    ///     SpawnEvent { entity_id: 2 },
    ///     SpawnEvent { entity_id: 3 },
    /// ];
    /// writer.send_batch(spawns);
    /// ```
    pub fn send_batch(&mut self, events: impl IntoIterator<Item = E>) {
        for event in events {
            self.queue.send(event);
        }
    }

    /// Returns `true` if no events have been written in the current frame.
    ///
    /// Note: This only checks events written since the last `swap_buffers()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::{EventQueue, EventWriter};
    ///
    /// struct Event;
    ///
    /// let mut queue: EventQueue<Event> = EventQueue::new();
    /// let mut writer = EventWriter::new(&mut queue);
    ///
    /// assert!(writer.is_empty());
    ///
    /// writer.send(Event);
    /// assert!(!writer.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the number of events written in the current frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::{EventQueue, EventWriter};
    ///
    /// struct Event;
    ///
    /// let mut queue: EventQueue<Event> = EventQueue::new();
    /// let mut writer = EventWriter::new(&mut queue);
    ///
    /// writer.send(Event);
    /// writer.send(Event);
    ///
    /// assert_eq!(writer.len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}
