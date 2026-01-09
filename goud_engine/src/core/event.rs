//! Event system for decoupled communication between engine systems.
//!
//! Events enable systems to communicate without direct dependencies. The event
//! system uses a double-buffered queue pattern where events are written during
//! one frame and read during the next.
//!
//! # Design
//!
//! The event system consists of:
//! - [`Event`]: Marker trait for types that can be sent as events
//! - [`EventQueue<E>`]: Double-buffered storage for events of type E
//! - [`EventReader<E>`]: Read-only accessor for consuming events
//! - [`EventWriter<E>`]: Write-only accessor for sending events
//! - [`Events<E>`]: ECS resource wrapper that manages the full event lifecycle
//!
//! # Usage
//!
//! Any type that is `Send + Sync + 'static` automatically implements `Event`:
//!
//! ```rust
//! use goud_engine::core::event::Event;
//!
//! // Custom event types
//! #[derive(Debug, Clone)]
//! struct PlayerDied {
//!     player_id: u32,
//!     cause: String,
//! }
//!
//! // PlayerDied automatically implements Event
//! fn send_event<E: Event>(event: E) {
//!     // ...
//! }
//!
//! let event = PlayerDied {
//!     player_id: 1,
//!     cause: "fell into lava".to_string(),
//! };
//! send_event(event);
//! ```
//!
//! # Thread Safety
//!
//! Events must be `Send + Sync` to support parallel system execution. The
//! `'static` bound ensures events don't contain borrowed data, which would
//! complicate lifetime management across frame boundaries.

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

/// Double-buffered event queue for storing events of a single type.
///
/// EventQueue uses a double-buffer pattern where events are written to the
/// active buffer and read from the inactive buffer. At the end of each frame,
/// `swap_buffers` is called to switch which buffer is active.
///
/// # Double-Buffer Pattern
///
/// ```text
/// Frame N:
///   - Systems write new events to Buffer A (active write buffer)
///   - Systems read events from Buffer B (read buffer from Frame N-1)
///
/// End of Frame N: swap_buffers()
///
/// Frame N+1:
///   - Systems write new events to Buffer B (now active write buffer)
///   - Systems read events from Buffer A (read buffer from Frame N)
/// ```
///
/// This pattern ensures:
/// - Writers and readers never access the same buffer simultaneously
/// - Events persist for exactly one frame after being written
/// - No locking required within a single-threaded frame
///
/// # Example
///
/// ```rust
/// use goud_engine::core::event::EventQueue;
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct DamageEvent { amount: u32 }
///
/// let mut queue: EventQueue<DamageEvent> = EventQueue::new();
///
/// // Write to active buffer
/// queue.send(DamageEvent { amount: 10 });
/// queue.send(DamageEvent { amount: 25 });
///
/// // Read buffer is empty (events are in write buffer)
/// assert!(queue.drain().collect::<Vec<_>>().is_empty());
///
/// // Swap buffers at frame boundary
/// queue.swap_buffers();
///
/// // Now we can read the events
/// let events: Vec<_> = queue.drain().collect();
/// assert_eq!(events.len(), 2);
/// assert_eq!(events[0].amount, 10);
/// assert_eq!(events[1].amount, 25);
/// ```
pub struct EventQueue<E: Event> {
    /// First buffer for events
    events_a: Vec<E>,
    /// Second buffer for events
    events_b: Vec<E>,
    /// Which buffer is currently the write buffer.
    /// - false: Buffer A is write, Buffer B is read
    /// - true: Buffer B is write, Buffer A is read
    active_buffer: bool,
}

impl<E: Event> EventQueue<E> {
    /// Creates a new empty EventQueue.
    ///
    /// Both buffers start empty, and Buffer A is the initial write buffer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::EventQueue;
    ///
    /// struct MyEvent { data: i32 }
    /// let queue: EventQueue<MyEvent> = EventQueue::new();
    /// assert!(queue.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            events_a: Vec::new(),
            events_b: Vec::new(),
            active_buffer: false,
        }
    }

    /// Sends an event to the write buffer.
    ///
    /// The event will be available for reading after the next `swap_buffers` call.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::EventQueue;
    ///
    /// struct ScoreEvent { points: i32 }
    /// let mut queue: EventQueue<ScoreEvent> = EventQueue::new();
    ///
    /// queue.send(ScoreEvent { points: 100 });
    /// queue.send(ScoreEvent { points: 50 });
    ///
    /// assert_eq!(queue.len(), 2);
    /// ```
    pub fn send(&mut self, event: E) {
        self.write_buffer_mut().push(event);
    }

    /// Drains all events from the read buffer, returning an iterator.
    ///
    /// After draining, the read buffer will be empty. This is the primary
    /// way to consume events during a frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::EventQueue;
    ///
    /// #[derive(Debug)]
    /// struct Event { id: u32 }
    ///
    /// let mut queue: EventQueue<Event> = EventQueue::new();
    /// queue.send(Event { id: 1 });
    /// queue.send(Event { id: 2 });
    /// queue.swap_buffers();
    ///
    /// for event in queue.drain() {
    ///     println!("Processing event: {:?}", event);
    /// }
    /// ```
    pub fn drain(&mut self) -> impl Iterator<Item = E> + '_ {
        self.read_buffer_mut().drain(..)
    }

    /// Swaps the active and read buffers.
    ///
    /// This should be called exactly once per frame, typically at the frame
    /// boundary. After swapping:
    /// - The old write buffer becomes the new read buffer
    /// - The old read buffer (now cleared) becomes the new write buffer
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::EventQueue;
    ///
    /// struct FrameEvent;
    /// let mut queue: EventQueue<FrameEvent> = EventQueue::new();
    ///
    /// // Frame 1: send events
    /// queue.send(FrameEvent);
    ///
    /// // End of Frame 1
    /// queue.swap_buffers();
    ///
    /// // Frame 2: events from Frame 1 are now readable
    /// let count = queue.drain().count();
    /// assert_eq!(count, 1);
    /// ```
    pub fn swap_buffers(&mut self) {
        // Clear the read buffer before it becomes the write buffer
        self.read_buffer_mut().clear();
        // Swap which buffer is active
        self.active_buffer = !self.active_buffer;
    }

    /// Clears both buffers, removing all events.
    ///
    /// This is useful for resetting the event system, such as when
    /// transitioning between game states.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::EventQueue;
    ///
    /// struct Event;
    /// let mut queue: EventQueue<Event> = EventQueue::new();
    ///
    /// queue.send(Event);
    /// queue.swap_buffers();
    /// queue.send(Event);
    ///
    /// queue.clear();
    ///
    /// assert!(queue.is_empty());
    /// assert!(queue.drain().next().is_none());
    /// ```
    pub fn clear(&mut self) {
        self.events_a.clear();
        self.events_b.clear();
    }

    /// Returns `true` if the write buffer has no pending events.
    ///
    /// Note: This only checks the write buffer. The read buffer may still
    /// contain events from the previous frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::EventQueue;
    ///
    /// struct Event;
    /// let mut queue: EventQueue<Event> = EventQueue::new();
    ///
    /// assert!(queue.is_empty());
    ///
    /// queue.send(Event);
    /// assert!(!queue.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.write_buffer().is_empty()
    }

    /// Returns the number of events in the write buffer.
    ///
    /// Note: This only counts events in the write buffer (pending events).
    /// Use `read_len()` to count events available for reading.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::EventQueue;
    ///
    /// struct Event;
    /// let mut queue: EventQueue<Event> = EventQueue::new();
    ///
    /// assert_eq!(queue.len(), 0);
    ///
    /// queue.send(Event);
    /// queue.send(Event);
    /// queue.send(Event);
    ///
    /// assert_eq!(queue.len(), 3);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.write_buffer().len()
    }

    /// Returns the number of events available for reading.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::EventQueue;
    ///
    /// struct Event;
    /// let mut queue: EventQueue<Event> = EventQueue::new();
    ///
    /// queue.send(Event);
    /// queue.send(Event);
    ///
    /// // Events are in write buffer, not yet readable
    /// assert_eq!(queue.read_len(), 0);
    ///
    /// queue.swap_buffers();
    ///
    /// // Now they're in the read buffer
    /// assert_eq!(queue.read_len(), 2);
    /// ```
    #[must_use]
    pub fn read_len(&self) -> usize {
        self.read_buffer().len()
    }

    /// Returns a reference to the write buffer.
    #[inline]
    fn write_buffer(&self) -> &Vec<E> {
        if self.active_buffer {
            &self.events_b
        } else {
            &self.events_a
        }
    }

    /// Returns a mutable reference to the write buffer.
    #[inline]
    fn write_buffer_mut(&mut self) -> &mut Vec<E> {
        if self.active_buffer {
            &mut self.events_b
        } else {
            &mut self.events_a
        }
    }

    /// Returns a reference to the read buffer.
    ///
    /// This is exposed for use by [`EventReader`] to access events without draining.
    #[inline]
    pub fn read_buffer(&self) -> &Vec<E> {
        if self.active_buffer {
            &self.events_a
        } else {
            &self.events_b
        }
    }

    /// Returns a mutable reference to the read buffer.
    #[inline]
    fn read_buffer_mut(&mut self) -> &mut Vec<E> {
        if self.active_buffer {
            &mut self.events_a
        } else {
            &mut self.events_b
        }
    }
}

impl<E: Event> Default for EventQueue<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Read-only accessor for consuming events from an [`EventQueue`].
///
/// `EventReader` provides a way for systems to read events without consuming
/// them immediately. It tracks a read index to ensure events are not read
/// multiple times within the same frame by the same reader.
///
/// # Multiple Readers
///
/// Multiple `EventReader` instances can exist for the same queue (shared borrow).
/// Each reader maintains its own read position, allowing different systems to
/// read the same events independently.
///
/// # Read Tracking
///
/// The reader tracks which events have been read via an internal index. When
/// `read()` is called, it returns only unread events and advances the index.
/// This prevents double-processing of events within a single system.
///
/// # Example
///
/// ```rust
/// use goud_engine::core::event::{EventQueue, EventReader};
///
/// #[derive(Debug, Clone)]
/// struct DamageEvent { amount: u32 }
///
/// let mut queue: EventQueue<DamageEvent> = EventQueue::new();
/// queue.send(DamageEvent { amount: 10 });
/// queue.send(DamageEvent { amount: 25 });
/// queue.swap_buffers();
///
/// // Create a reader
/// let mut reader = EventReader::new(&queue);
///
/// // First read gets both events
/// let events: Vec<_> = reader.read().collect();
/// assert_eq!(events.len(), 2);
///
/// // Second read gets nothing (already read)
/// let events: Vec<_> = reader.read().collect();
/// assert!(events.is_empty());
/// ```
pub struct EventReader<'a, E: Event> {
    /// Reference to the event queue
    queue: &'a EventQueue<E>,
    /// Index of the next unread event in the read buffer
    read_index: usize,
}

impl<'a, E: Event> EventReader<'a, E> {
    /// Creates a new `EventReader` for the given queue.
    ///
    /// The reader starts at index 0, meaning it will read all available events
    /// in the read buffer on the first call to `read()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::{EventQueue, EventReader};
    ///
    /// struct MyEvent { data: i32 }
    ///
    /// let queue: EventQueue<MyEvent> = EventQueue::new();
    /// let reader = EventReader::new(&queue);
    /// assert!(reader.is_empty());
    /// ```
    #[must_use]
    pub fn new(queue: &'a EventQueue<E>) -> Self {
        Self {
            queue,
            read_index: 0,
        }
    }

    /// Returns an iterator over unread events.
    ///
    /// Each call to `read()` returns only events that haven't been read by
    /// this reader instance yet. The read index is advanced after iteration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::{EventQueue, EventReader};
    ///
    /// #[derive(Debug)]
    /// struct ScoreEvent { points: i32 }
    ///
    /// let mut queue: EventQueue<ScoreEvent> = EventQueue::new();
    /// queue.send(ScoreEvent { points: 100 });
    /// queue.send(ScoreEvent { points: 50 });
    /// queue.swap_buffers();
    ///
    /// let mut reader = EventReader::new(&queue);
    ///
    /// // Read all events
    /// for event in reader.read() {
    ///     println!("Score: {}", event.points);
    /// }
    /// ```
    pub fn read(&mut self) -> EventReaderIter<'_, 'a, E> {
        EventReaderIter { reader: self }
    }

    /// Returns `true` if there are no unread events.
    ///
    /// This checks if all events in the read buffer have been consumed by
    /// this reader.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::{EventQueue, EventReader};
    ///
    /// struct Event;
    ///
    /// let mut queue: EventQueue<Event> = EventQueue::new();
    /// queue.send(Event);
    /// queue.swap_buffers();
    ///
    /// let mut reader = EventReader::new(&queue);
    /// assert!(!reader.is_empty());
    ///
    /// // Consume all events
    /// let _ = reader.read().count();
    /// assert!(reader.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.read_index >= self.queue.read_len()
    }

    /// Returns the number of unread events.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::{EventQueue, EventReader};
    ///
    /// struct Event;
    ///
    /// let mut queue: EventQueue<Event> = EventQueue::new();
    /// queue.send(Event);
    /// queue.send(Event);
    /// queue.send(Event);
    /// queue.swap_buffers();
    ///
    /// let mut reader = EventReader::new(&queue);
    /// assert_eq!(reader.len(), 3);
    ///
    /// // Read one event
    /// let _ = reader.read().next();
    /// assert_eq!(reader.len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.read_len().saturating_sub(self.read_index)
    }

    /// Clears the reader's position, allowing events to be re-read.
    ///
    /// This resets the read index to 0. The next call to `read()` will
    /// return all events in the read buffer again.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::{EventQueue, EventReader};
    ///
    /// #[derive(Debug, Clone)]
    /// struct Event { id: u32 }
    ///
    /// let mut queue: EventQueue<Event> = EventQueue::new();
    /// queue.send(Event { id: 1 });
    /// queue.swap_buffers();
    ///
    /// let mut reader = EventReader::new(&queue);
    ///
    /// // Read all events
    /// let count1 = reader.read().count();
    /// assert_eq!(count1, 1);
    ///
    /// // Nothing left to read
    /// let count2 = reader.read().count();
    /// assert_eq!(count2, 0);
    ///
    /// // Reset and read again
    /// reader.clear();
    /// let count3 = reader.read().count();
    /// assert_eq!(count3, 1);
    /// ```
    pub fn clear(&mut self) {
        self.read_index = 0;
    }
}

/// Iterator over unread events from an [`EventReader`].
///
/// This iterator is returned by [`EventReader::read()`] and yields references
/// to events that haven't been read yet by this reader.
pub struct EventReaderIter<'r, 'a, E: Event> {
    reader: &'r mut EventReader<'a, E>,
}

impl<'r, 'a, E: Event> Iterator for EventReaderIter<'r, 'a, E> {
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        let buffer = self.reader.queue.read_buffer();
        if self.reader.read_index < buffer.len() {
            let event = &buffer[self.reader.read_index];
            self.reader.read_index += 1;
            Some(event)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.reader.len();
        (remaining, Some(remaining))
    }
}

impl<'r, 'a, E: Event> ExactSizeIterator for EventReaderIter<'r, 'a, E> {}

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

/// ECS-compatible resource wrapper for event queues.
///
/// `Events<E>` wraps an [`EventQueue<E>`] and provides a high-level API
/// suitable for use as an ECS resource. It manages the event lifecycle
/// automatically and provides convenient accessor methods.
///
/// # Resource Integration
///
/// In a typical ECS architecture, `Events<E>` is stored as a resource in the
/// World. Systems that need to send or receive events request it via the
/// resource system.
///
/// # Frame Lifecycle
///
/// The `update()` method must be called once per frame (typically at the start
/// or end of the frame) to:
/// 1. Swap the double buffers
/// 2. Clear stale events from the previous frame
///
/// This ensures events persist for exactly one frame after being sent.
///
/// # Example
///
/// ```rust
/// use goud_engine::core::event::Events;
///
/// #[derive(Debug, Clone)]
/// struct DamageEvent {
///     entity_id: u32,
///     amount: i32,
/// }
///
/// // Create the events resource
/// let mut events: Events<DamageEvent> = Events::new();
///
/// // Systems can send events
/// events.send(DamageEvent { entity_id: 1, amount: 50 });
/// events.send(DamageEvent { entity_id: 2, amount: 25 });
///
/// // At frame boundary, call update
/// events.update();
///
/// // Now systems can read the events
/// let mut reader = events.reader();
/// for event in reader.read() {
///     println!("Entity {} took {} damage", event.entity_id, event.amount);
/// }
/// ```
///
/// # Thread Safety
///
/// `Events<E>` is `Send + Sync` when `E` is `Send + Sync`, enabling safe
/// use in multi-threaded ECS systems.
pub struct Events<E: Event> {
    /// The underlying double-buffered event queue
    queue: EventQueue<E>,
}

impl<E: Event> Events<E> {
    /// Creates a new empty `Events` resource.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::Events;
    ///
    /// struct MyEvent { data: i32 }
    /// let events: Events<MyEvent> = Events::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: EventQueue::new(),
        }
    }

    /// Creates an `EventReader` for reading events from this resource.
    ///
    /// Multiple readers can be created simultaneously, as they only require
    /// shared access to the underlying queue.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::Events;
    ///
    /// #[derive(Debug)]
    /// struct MyEvent { id: u32 }
    ///
    /// let mut events: Events<MyEvent> = Events::new();
    /// events.send(MyEvent { id: 1 });
    /// events.update();
    ///
    /// let mut reader = events.reader();
    /// for event in reader.read() {
    ///     println!("Got event: {:?}", event);
    /// }
    /// ```
    #[must_use]
    pub fn reader(&self) -> EventReader<'_, E> {
        EventReader::new(&self.queue)
    }

    /// Creates an `EventWriter` for sending events to this resource.
    ///
    /// Only one writer can exist at a time due to the mutable borrow requirement.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::Events;
    ///
    /// struct SpawnEvent { entity_type: String }
    ///
    /// let mut events: Events<SpawnEvent> = Events::new();
    ///
    /// {
    ///     let mut writer = events.writer();
    ///     writer.send(SpawnEvent { entity_type: "enemy".to_string() });
    ///     writer.send(SpawnEvent { entity_type: "item".to_string() });
    /// }
    /// ```
    #[must_use]
    pub fn writer(&mut self) -> EventWriter<'_, E> {
        EventWriter::new(&mut self.queue)
    }

    /// Sends an event directly to the write buffer.
    ///
    /// This is a convenience method equivalent to creating a writer and
    /// calling `send()` on it.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::Events;
    ///
    /// struct InputEvent { key: char }
    ///
    /// let mut events: Events<InputEvent> = Events::new();
    /// events.send(InputEvent { key: 'w' });
    /// events.send(InputEvent { key: 'a' });
    /// ```
    pub fn send(&mut self, event: E) {
        self.queue.send(event);
    }

    /// Sends multiple events to the write buffer in batch.
    ///
    /// More efficient than calling `send()` multiple times when you have
    /// a collection of events.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::Events;
    ///
    /// #[derive(Clone)]
    /// struct ParticleEvent { x: f32, y: f32 }
    ///
    /// let mut events: Events<ParticleEvent> = Events::new();
    /// let particles = vec![
    ///     ParticleEvent { x: 0.0, y: 0.0 },
    ///     ParticleEvent { x: 1.0, y: 1.0 },
    ///     ParticleEvent { x: 2.0, y: 2.0 },
    /// ];
    /// events.send_batch(particles);
    /// ```
    pub fn send_batch(&mut self, events: impl IntoIterator<Item = E>) {
        for event in events {
            self.queue.send(event);
        }
    }

    /// Updates the event system at frame boundary.
    ///
    /// This method MUST be called exactly once per frame, typically at the
    /// start or end of the game loop. It:
    /// 1. Clears events from the previous read buffer (they've had their chance)
    /// 2. Swaps the buffers so newly written events become readable
    ///
    /// # Frame Timing
    ///
    /// ```text
    /// Frame N:
    ///   1. update() called - events from Frame N-1 become readable
    ///   2. Systems read events
    ///   3. Systems write new events
    ///
    /// Frame N+1:
    ///   1. update() called - Frame N-1 events cleared, Frame N events readable
    ///   ... and so on
    /// ```
    ///
    /// # Important
    ///
    /// - Calling `update()` more than once per frame will cause events to be
    ///   lost (the write buffer gets swapped to read before systems can write)
    /// - Not calling `update()` will cause events to accumulate and never
    ///   become readable
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::Events;
    ///
    /// struct GameEvent;
    ///
    /// let mut events: Events<GameEvent> = Events::new();
    ///
    /// // Game loop
    /// for frame in 0..3 {
    ///     // Start of frame: update events
    ///     events.update();
    ///
    ///     // Read events from previous frame
    ///     let mut reader = events.reader();
    ///     for _ in reader.read() {
    ///         // Process events...
    ///     }
    ///
    ///     // Systems write new events
    ///     events.send(GameEvent);
    /// }
    /// ```
    pub fn update(&mut self) {
        self.queue.swap_buffers();
    }

    /// Drains all events from the read buffer, consuming them.
    ///
    /// Unlike using a reader, this removes the events entirely from the buffer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::Events;
    ///
    /// struct Event { id: u32 }
    ///
    /// let mut events: Events<Event> = Events::new();
    /// events.send(Event { id: 1 });
    /// events.send(Event { id: 2 });
    /// events.update();
    ///
    /// let collected: Vec<Event> = events.drain().collect();
    /// assert_eq!(collected.len(), 2);
    /// ```
    pub fn drain(&mut self) -> impl Iterator<Item = E> + '_ {
        self.queue.drain()
    }

    /// Clears all events from both buffers.
    ///
    /// Use this when transitioning between game states or when you need
    /// to discard all pending events.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::Events;
    ///
    /// struct Event;
    ///
    /// let mut events: Events<Event> = Events::new();
    /// events.send(Event);
    /// events.update();
    /// events.send(Event);
    ///
    /// events.clear();
    /// assert!(events.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// Returns `true` if no events are pending in the write buffer.
    ///
    /// Note: The read buffer may still contain events from the previous frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::Events;
    ///
    /// struct Event;
    ///
    /// let mut events: Events<Event> = Events::new();
    /// assert!(events.is_empty());
    ///
    /// events.send(Event);
    /// assert!(!events.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the number of events in the write buffer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::Events;
    ///
    /// struct Event;
    ///
    /// let mut events: Events<Event> = Events::new();
    /// events.send(Event);
    /// events.send(Event);
    ///
    /// assert_eq!(events.len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns the number of events available for reading.
    ///
    /// # Example
    ///
    /// ```rust
    /// use goud_engine::core::event::Events;
    ///
    /// struct Event;
    ///
    /// let mut events: Events<Event> = Events::new();
    /// events.send(Event);
    /// events.send(Event);
    ///
    /// // Events are in write buffer
    /// assert_eq!(events.read_len(), 0);
    ///
    /// events.update();
    ///
    /// // Now in read buffer
    /// assert_eq!(events.read_len(), 2);
    /// ```
    #[must_use]
    pub fn read_len(&self) -> usize {
        self.queue.read_len()
    }
}

impl<E: Event> Default for Events<E> {
    fn default() -> Self {
        Self::new()
    }
}

// Note: Events<E> is automatically Send + Sync because EventQueue<E> is Send + Sync
// when E is Send + Sync, which is guaranteed by the Event trait bound.
// This is enforced by the test `test_events_is_send_sync`.

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that simple struct implements Event
    #[test]
    fn test_simple_struct_is_event() {
        #[derive(Debug, Clone)]
        struct SimpleEvent {
            value: i32,
        }

        fn accepts_event<E: Event>(_: E) {}

        let event = SimpleEvent { value: 42 };
        accepts_event(event);
    }

    /// Test that struct with String field implements Event
    #[test]
    fn test_event_with_string() {
        #[derive(Debug, Clone)]
        struct MessageEvent {
            message: String,
            priority: u8,
        }

        fn accepts_event<E: Event>(_: E) {}

        let event = MessageEvent {
            message: "hello".to_string(),
            priority: 1,
        };
        accepts_event(event);
    }

    /// Test that unit struct implements Event
    #[test]
    fn test_unit_struct_event() {
        struct UnitEvent;

        fn accepts_event<E: Event>(_: E) {}
        accepts_event(UnitEvent);
    }

    /// Test that tuple struct implements Event
    #[test]
    fn test_tuple_struct_event() {
        struct TupleEvent(i32, String);

        fn accepts_event<E: Event>(_: E) {}
        accepts_event(TupleEvent(1, "test".to_string()));
    }

    /// Test that primitive types implement Event
    #[test]
    fn test_primitive_types_are_events() {
        fn accepts_event<E: Event>(_: E) {}

        accepts_event(42i32);
        accepts_event(3.14f64);
        accepts_event(true);
        accepts_event("static string");
        accepts_event(String::from("owned string"));
    }

    /// Test that Vec of events is itself an Event
    #[test]
    fn test_container_of_events() {
        #[derive(Clone)]
        struct InnerEvent {
            id: u32,
        }

        fn accepts_event<E: Event>(_: E) {}

        let events = vec![InnerEvent { id: 1 }, InnerEvent { id: 2 }];
        accepts_event(events);
    }

    /// Test that Arc-wrapped data is an Event
    #[test]
    fn test_arc_wrapped_event() {
        use std::sync::Arc;

        struct SharedData {
            data: i32,
        }

        fn accepts_event<E: Event>(_: E) {}

        let shared = Arc::new(SharedData { data: 42 });
        accepts_event(shared);
    }

    /// Test Event trait bounds are correct
    #[test]
    fn test_event_trait_bounds() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        fn assert_static<T: 'static>() {}

        struct TestEvent;

        // Event requires all these bounds
        assert_send::<TestEvent>();
        assert_sync::<TestEvent>();
        assert_static::<TestEvent>();
    }

    // =========================================================================
    // EventQueue Tests
    // =========================================================================

    /// Test EventQueue creation and default state
    #[test]
    fn test_event_queue_new() {
        #[derive(Debug)]
        struct TestEvent {
            id: u32,
        }

        let queue: EventQueue<TestEvent> = EventQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.read_len(), 0);
    }

    /// Test EventQueue Default trait implementation
    #[test]
    fn test_event_queue_default() {
        #[derive(Debug)]
        struct TestEvent;

        let queue: EventQueue<TestEvent> = EventQueue::default();
        assert!(queue.is_empty());
    }

    /// Test sending events to write buffer
    #[test]
    fn test_event_queue_send() {
        #[derive(Debug, PartialEq)]
        struct CountEvent {
            count: i32,
        }

        let mut queue: EventQueue<CountEvent> = EventQueue::new();

        queue.send(CountEvent { count: 1 });
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());

        queue.send(CountEvent { count: 2 });
        queue.send(CountEvent { count: 3 });
        assert_eq!(queue.len(), 3);

        // Read buffer should still be empty
        assert_eq!(queue.read_len(), 0);
    }

    /// Test buffer swapping moves events from write to read buffer
    #[test]
    fn test_event_queue_swap_buffers() {
        #[derive(Debug, PartialEq)]
        struct SwapEvent {
            value: i32,
        }

        let mut queue: EventQueue<SwapEvent> = EventQueue::new();

        // Send to write buffer
        queue.send(SwapEvent { value: 10 });
        queue.send(SwapEvent { value: 20 });

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.read_len(), 0);

        // Swap buffers
        queue.swap_buffers();

        // Now events are in read buffer
        assert_eq!(queue.len(), 0); // Write buffer is now empty
        assert_eq!(queue.read_len(), 2); // Read buffer has events
    }

    /// Test draining events from read buffer
    #[test]
    fn test_event_queue_drain() {
        #[derive(Debug, PartialEq, Clone)]
        struct DrainEvent {
            data: String,
        }

        let mut queue: EventQueue<DrainEvent> = EventQueue::new();

        queue.send(DrainEvent {
            data: "first".to_string(),
        });
        queue.send(DrainEvent {
            data: "second".to_string(),
        });
        queue.send(DrainEvent {
            data: "third".to_string(),
        });

        queue.swap_buffers();

        // Drain and collect events
        let events: Vec<DrainEvent> = queue.drain().collect();

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].data, "first");
        assert_eq!(events[1].data, "second");
        assert_eq!(events[2].data, "third");

        // Read buffer should now be empty
        assert_eq!(queue.read_len(), 0);
    }

    /// Test that drain on empty read buffer yields nothing
    #[test]
    fn test_event_queue_drain_empty() {
        #[derive(Debug)]
        struct EmptyEvent;

        let mut queue: EventQueue<EmptyEvent> = EventQueue::new();

        // Drain without sending anything
        let events: Vec<EmptyEvent> = queue.drain().collect();
        assert!(events.is_empty());

        // Send to write buffer but don't swap
        queue.send(EmptyEvent);

        // Drain should still be empty (events in write buffer)
        let events: Vec<EmptyEvent> = queue.drain().collect();
        assert!(events.is_empty());
    }

    /// Test clearing both buffers
    #[test]
    fn test_event_queue_clear() {
        #[derive(Debug)]
        struct ClearEvent {
            id: u32,
        }

        let mut queue: EventQueue<ClearEvent> = EventQueue::new();

        // Add events to write buffer
        queue.send(ClearEvent { id: 1 });
        queue.send(ClearEvent { id: 2 });

        // Swap so some events are in read buffer
        queue.swap_buffers();

        // Add more to new write buffer
        queue.send(ClearEvent { id: 3 });

        assert_eq!(queue.len(), 1); // Write buffer
        assert_eq!(queue.read_len(), 2); // Read buffer

        // Clear everything
        queue.clear();

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.read_len(), 0);
        assert!(queue.drain().next().is_none());
    }

    /// Test multi-frame event lifecycle
    #[test]
    fn test_event_queue_multi_frame_lifecycle() {
        #[derive(Debug, PartialEq)]
        struct FrameEvent {
            frame: u32,
        }

        let mut queue: EventQueue<FrameEvent> = EventQueue::new();

        // Frame 1: Send event
        queue.send(FrameEvent { frame: 1 });
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.read_len(), 0);

        // End of Frame 1
        queue.swap_buffers();

        // Frame 2: Event from frame 1 is now readable
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.read_len(), 1);

        // Read it
        let events: Vec<FrameEvent> = queue.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].frame, 1);

        // Send new event in frame 2
        queue.send(FrameEvent { frame: 2 });

        // End of Frame 2
        queue.swap_buffers();

        // Frame 3: Event from frame 2 is readable, frame 1 event is gone
        let events: Vec<FrameEvent> = queue.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].frame, 2);

        // End of Frame 3 (no new events)
        queue.swap_buffers();

        // Frame 4: No events
        let events: Vec<FrameEvent> = queue.drain().collect();
        assert!(events.is_empty());
    }

    /// Test that events preserve order
    #[test]
    fn test_event_queue_preserves_order() {
        #[derive(Debug, PartialEq)]
        struct OrderedEvent {
            sequence: usize,
        }

        let mut queue: EventQueue<OrderedEvent> = EventQueue::new();

        // Send events in order
        for i in 0..100 {
            queue.send(OrderedEvent { sequence: i });
        }

        queue.swap_buffers();

        // Verify order is preserved
        let events: Vec<OrderedEvent> = queue.drain().collect();
        assert_eq!(events.len(), 100);

        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.sequence, i, "Event order mismatch at index {}", i);
        }
    }

    // =========================================================================
    // EventReader Tests
    // =========================================================================

    /// Test EventReader creation
    #[test]
    fn test_event_reader_new() {
        #[derive(Debug)]
        struct TestEvent {
            id: u32,
        }

        let queue: EventQueue<TestEvent> = EventQueue::new();
        let reader = EventReader::new(&queue);

        assert!(reader.is_empty());
        assert_eq!(reader.len(), 0);
    }

    /// Test EventReader reads all events
    #[test]
    fn test_event_reader_reads_all() {
        #[derive(Debug, PartialEq)]
        struct TestEvent {
            value: i32,
        }

        let mut queue: EventQueue<TestEvent> = EventQueue::new();
        queue.send(TestEvent { value: 1 });
        queue.send(TestEvent { value: 2 });
        queue.send(TestEvent { value: 3 });
        queue.swap_buffers();

        let mut reader = EventReader::new(&queue);

        let events: Vec<&TestEvent> = reader.read().collect();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].value, 1);
        assert_eq!(events[1].value, 2);
        assert_eq!(events[2].value, 3);
    }

    /// Test EventReader tracks read position
    #[test]
    fn test_event_reader_tracks_position() {
        #[derive(Debug)]
        struct TestEvent {
            id: u32,
        }

        let mut queue: EventQueue<TestEvent> = EventQueue::new();
        queue.send(TestEvent { id: 1 });
        queue.send(TestEvent { id: 2 });
        queue.send(TestEvent { id: 3 });
        queue.swap_buffers();

        let mut reader = EventReader::new(&queue);
        assert_eq!(reader.len(), 3);

        // Read first event
        let first = reader.read().next();
        assert!(first.is_some());
        assert_eq!(first.unwrap().id, 1);
        assert_eq!(reader.len(), 2);

        // Read second event
        let second = reader.read().next();
        assert!(second.is_some());
        assert_eq!(second.unwrap().id, 2);
        assert_eq!(reader.len(), 1);

        // Read third event
        let third = reader.read().next();
        assert!(third.is_some());
        assert_eq!(third.unwrap().id, 3);
        assert_eq!(reader.len(), 0);

        // No more events
        assert!(reader.is_empty());
        assert!(reader.read().next().is_none());
    }

    /// Test EventReader doesn't re-read events
    #[test]
    fn test_event_reader_no_rereading() {
        #[derive(Debug)]
        struct TestEvent;

        let mut queue: EventQueue<TestEvent> = EventQueue::new();
        queue.send(TestEvent);
        queue.send(TestEvent);
        queue.swap_buffers();

        let mut reader = EventReader::new(&queue);

        // First read gets both
        let count1 = reader.read().count();
        assert_eq!(count1, 2);

        // Second read gets nothing
        let count2 = reader.read().count();
        assert_eq!(count2, 0);

        // Third read still gets nothing
        let count3 = reader.read().count();
        assert_eq!(count3, 0);
    }

    /// Test EventReader clear resets position
    #[test]
    fn test_event_reader_clear() {
        #[derive(Debug)]
        struct TestEvent {
            id: u32,
        }

        let mut queue: EventQueue<TestEvent> = EventQueue::new();
        queue.send(TestEvent { id: 1 });
        queue.send(TestEvent { id: 2 });
        queue.swap_buffers();

        let mut reader = EventReader::new(&queue);

        // Read all events
        let count1 = reader.read().count();
        assert_eq!(count1, 2);
        assert!(reader.is_empty());

        // Clear and read again
        reader.clear();
        assert!(!reader.is_empty());
        assert_eq!(reader.len(), 2);

        let count2 = reader.read().count();
        assert_eq!(count2, 2);
    }

    /// Test multiple EventReaders are independent
    #[test]
    fn test_event_reader_multiple_independent() {
        #[derive(Debug, PartialEq)]
        struct TestEvent {
            value: i32,
        }

        let mut queue: EventQueue<TestEvent> = EventQueue::new();
        queue.send(TestEvent { value: 10 });
        queue.send(TestEvent { value: 20 });
        queue.swap_buffers();

        // Create two readers
        let mut reader1 = EventReader::new(&queue);
        let mut reader2 = EventReader::new(&queue);

        // Reader 1 reads one event
        let event1 = reader1.read().next().unwrap();
        assert_eq!(event1.value, 10);

        // Reader 2 still sees both events
        let events2: Vec<_> = reader2.read().collect();
        assert_eq!(events2.len(), 2);
        assert_eq!(events2[0].value, 10);
        assert_eq!(events2[1].value, 20);

        // Reader 1 continues from where it left off
        let event1_second = reader1.read().next().unwrap();
        assert_eq!(event1_second.value, 20);
    }

    /// Test EventReaderIter is ExactSizeIterator
    #[test]
    fn test_event_reader_iter_exact_size() {
        #[derive(Debug)]
        struct TestEvent;

        let mut queue: EventQueue<TestEvent> = EventQueue::new();
        queue.send(TestEvent);
        queue.send(TestEvent);
        queue.send(TestEvent);
        queue.swap_buffers();

        let mut reader = EventReader::new(&queue);
        let iter = reader.read();

        assert_eq!(iter.len(), 3);
    }

    /// Test EventReader with empty queue
    #[test]
    fn test_event_reader_empty_queue() {
        #[derive(Debug)]
        struct TestEvent;

        let queue: EventQueue<TestEvent> = EventQueue::new();
        let mut reader = EventReader::new(&queue);

        assert!(reader.is_empty());
        assert_eq!(reader.len(), 0);
        assert!(reader.read().next().is_none());
    }

    // =========================================================================
    // EventWriter Tests
    // =========================================================================

    /// Test EventWriter creation
    #[test]
    fn test_event_writer_new() {
        #[derive(Debug)]
        struct TestEvent;

        let mut queue: EventQueue<TestEvent> = EventQueue::new();
        let writer = EventWriter::new(&mut queue);

        assert!(writer.is_empty());
        assert_eq!(writer.len(), 0);
    }

    /// Test EventWriter sends events
    #[test]
    fn test_event_writer_send() {
        #[derive(Debug, PartialEq)]
        struct TestEvent {
            value: i32,
        }

        let mut queue: EventQueue<TestEvent> = EventQueue::new();

        {
            let mut writer = EventWriter::new(&mut queue);
            writer.send(TestEvent { value: 100 });
            writer.send(TestEvent { value: 200 });

            assert_eq!(writer.len(), 2);
            assert!(!writer.is_empty());
        }

        // Verify events were written
        assert_eq!(queue.len(), 2);
        queue.swap_buffers();

        let events: Vec<TestEvent> = queue.drain().collect();
        assert_eq!(events[0].value, 100);
        assert_eq!(events[1].value, 200);
    }

    /// Test EventWriter send_batch
    #[test]
    fn test_event_writer_send_batch() {
        #[derive(Debug, PartialEq, Clone)]
        struct TestEvent {
            id: u32,
        }

        let mut queue: EventQueue<TestEvent> = EventQueue::new();

        {
            let mut writer = EventWriter::new(&mut queue);
            let batch = vec![
                TestEvent { id: 1 },
                TestEvent { id: 2 },
                TestEvent { id: 3 },
                TestEvent { id: 4 },
                TestEvent { id: 5 },
            ];
            writer.send_batch(batch);

            assert_eq!(writer.len(), 5);
        }

        queue.swap_buffers();
        let events: Vec<TestEvent> = queue.drain().collect();
        assert_eq!(events.len(), 5);

        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.id, (i + 1) as u32);
        }
    }

    /// Test EventWriter send_batch with iterator
    #[test]
    fn test_event_writer_send_batch_iterator() {
        #[derive(Debug, PartialEq)]
        struct TestEvent {
            value: usize,
        }

        let mut queue: EventQueue<TestEvent> = EventQueue::new();

        {
            let mut writer = EventWriter::new(&mut queue);
            // Send using a range iterator
            writer.send_batch((0..10).map(|i| TestEvent { value: i }));
            assert_eq!(writer.len(), 10);
        }

        queue.swap_buffers();
        let events: Vec<TestEvent> = queue.drain().collect();
        assert_eq!(events.len(), 10);

        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.value, i);
        }
    }

    /// Test EventWriter exclusive access
    #[test]
    fn test_event_writer_exclusive_access() {
        #[derive(Debug)]
        struct TestEvent {
            id: u32,
        }

        let mut queue: EventQueue<TestEvent> = EventQueue::new();

        // Writer takes mutable borrow
        let mut writer = EventWriter::new(&mut queue);
        writer.send(TestEvent { id: 1 });

        // Can't access queue while writer exists (this is compile-time enforced)
        // After writer is dropped, queue is accessible again
        drop(writer);

        // Now we can access the queue
        assert_eq!(queue.len(), 1);
    }

    /// Test EventWriter with reader pattern
    #[test]
    fn test_event_writer_reader_integration() {
        #[derive(Debug, PartialEq)]
        struct TestEvent {
            msg: String,
        }

        let mut queue: EventQueue<TestEvent> = EventQueue::new();

        // Write phase
        {
            let mut writer = EventWriter::new(&mut queue);
            writer.send(TestEvent {
                msg: "hello".to_string(),
            });
            writer.send(TestEvent {
                msg: "world".to_string(),
            });
        }

        // Swap buffers
        queue.swap_buffers();

        // Read phase
        {
            let mut reader = EventReader::new(&queue);
            let events: Vec<&TestEvent> = reader.read().collect();

            assert_eq!(events.len(), 2);
            assert_eq!(events[0].msg, "hello");
            assert_eq!(events[1].msg, "world");
        }
    }

    /// Test EventWriter empty batch
    #[test]
    fn test_event_writer_empty_batch() {
        #[derive(Debug)]
        struct TestEvent;

        let mut queue: EventQueue<TestEvent> = EventQueue::new();

        {
            let mut writer = EventWriter::new(&mut queue);
            let empty: Vec<TestEvent> = vec![];
            writer.send_batch(empty);

            assert!(writer.is_empty());
            assert_eq!(writer.len(), 0);
        }

        queue.swap_buffers();
        assert_eq!(queue.read_len(), 0);
    }

    /// Test EventWriter preserves order
    #[test]
    fn test_event_writer_preserves_order() {
        #[derive(Debug, PartialEq)]
        struct TestEvent {
            sequence: usize,
        }

        let mut queue: EventQueue<TestEvent> = EventQueue::new();

        {
            let mut writer = EventWriter::new(&mut queue);
            for i in 0..50 {
                writer.send(TestEvent { sequence: i });
            }
        }

        queue.swap_buffers();

        let mut reader = EventReader::new(&queue);
        let events: Vec<&TestEvent> = reader.read().collect();

        assert_eq!(events.len(), 50);
        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.sequence, i, "Order mismatch at index {}", i);
        }
    }

    // =========================================================================
    // Events Resource Wrapper Tests
    // =========================================================================

    /// Test Events resource creation
    #[test]
    fn test_events_new() {
        #[derive(Debug)]
        struct TestEvent {
            id: u32,
        }

        let events: Events<TestEvent> = Events::new();
        assert!(events.is_empty());
        assert_eq!(events.len(), 0);
    }

    /// Test Events resource default
    #[test]
    fn test_events_default() {
        #[derive(Debug)]
        struct TestEvent;

        let events: Events<TestEvent> = Events::default();
        assert!(events.is_empty());
    }

    /// Test Events send and read cycle
    #[test]
    fn test_events_send_read_cycle() {
        #[derive(Debug, PartialEq)]
        struct TestEvent {
            value: i32,
        }

        let mut events: Events<TestEvent> = Events::new();

        // Send events
        events.send(TestEvent { value: 10 });
        events.send(TestEvent { value: 20 });
        events.send(TestEvent { value: 30 });

        assert_eq!(events.len(), 3);

        // Call update to swap buffers
        events.update();

        // Now read the events
        let mut reader = events.reader();
        let received: Vec<&TestEvent> = reader.read().collect();

        assert_eq!(received.len(), 3);
        assert_eq!(received[0].value, 10);
        assert_eq!(received[1].value, 20);
        assert_eq!(received[2].value, 30);
    }

    /// Test Events writer access
    #[test]
    fn test_events_writer() {
        #[derive(Debug, PartialEq)]
        struct TestEvent {
            msg: String,
        }

        let mut events: Events<TestEvent> = Events::new();

        {
            let mut writer = events.writer();
            writer.send(TestEvent {
                msg: "hello".to_string(),
            });
            writer.send(TestEvent {
                msg: "world".to_string(),
            });
        }

        assert_eq!(events.len(), 2);

        events.update();

        let mut reader = events.reader();
        let received: Vec<_> = reader.read().collect();
        assert_eq!(received.len(), 2);
    }

    /// Test Events update clears old events
    #[test]
    fn test_events_update_clears_old() {
        #[derive(Debug)]
        struct TestEvent {
            frame: u32,
        }

        let mut events: Events<TestEvent> = Events::new();

        // Frame 1: send event
        events.send(TestEvent { frame: 1 });
        events.update();

        // Frame 2: read event from frame 1
        {
            let mut reader = events.reader();
            let count = reader.read().count();
            assert_eq!(count, 1);
        }

        // Send new event in frame 2
        events.send(TestEvent { frame: 2 });
        events.update();

        // Frame 3: should only see event from frame 2
        {
            let mut reader = events.reader();
            let received: Vec<_> = reader.read().collect();
            assert_eq!(received.len(), 1);
            assert_eq!(received[0].frame, 2);
        }

        // No new events
        events.update();

        // Frame 4: no events
        {
            let mut reader = events.reader();
            let count = reader.read().count();
            assert_eq!(count, 0);
        }
    }

    /// Test Events send_batch
    #[test]
    fn test_events_send_batch() {
        #[derive(Debug, PartialEq, Clone)]
        struct TestEvent {
            id: u32,
        }

        let mut events: Events<TestEvent> = Events::new();

        let batch = vec![
            TestEvent { id: 1 },
            TestEvent { id: 2 },
            TestEvent { id: 3 },
        ];
        events.send_batch(batch);

        assert_eq!(events.len(), 3);

        events.update();

        let mut reader = events.reader();
        let received: Vec<_> = reader.read().collect();
        assert_eq!(received.len(), 3);
    }

    /// Test Events clear
    #[test]
    fn test_events_clear() {
        #[derive(Debug)]
        struct TestEvent;

        let mut events: Events<TestEvent> = Events::new();

        events.send(TestEvent);
        events.update();
        events.send(TestEvent);

        events.clear();

        assert!(events.is_empty());

        let mut reader = events.reader();
        assert!(reader.read().next().is_none());
    }

    /// Test Events is Send + Sync
    #[test]
    fn test_events_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        #[derive(Debug)]
        struct TestEvent {
            data: i32,
        }

        assert_send::<Events<TestEvent>>();
        assert_sync::<Events<TestEvent>>();
    }

    /// Test Events drain for consuming events
    #[test]
    fn test_events_drain() {
        #[derive(Debug, PartialEq)]
        struct TestEvent {
            value: i32,
        }

        let mut events: Events<TestEvent> = Events::new();

        events.send(TestEvent { value: 1 });
        events.send(TestEvent { value: 2 });
        events.update();

        // Drain consumes the events
        let drained: Vec<_> = events.drain().collect();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].value, 1);
        assert_eq!(drained[1].value, 2);

        // Events are gone
        let mut reader = events.reader();
        assert!(reader.read().next().is_none());
    }

    /// Test Events multiple readers independent
    #[test]
    fn test_events_multiple_readers() {
        #[derive(Debug, PartialEq)]
        struct TestEvent {
            id: u32,
        }

        let mut events: Events<TestEvent> = Events::new();
        events.send(TestEvent { id: 1 });
        events.send(TestEvent { id: 2 });
        events.update();

        // Create two readers
        let mut reader1 = events.reader();
        let mut reader2 = events.reader();

        // Both see all events
        assert_eq!(reader1.read().count(), 2);
        assert_eq!(reader2.read().count(), 2);
    }
}
