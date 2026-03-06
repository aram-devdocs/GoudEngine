//! ECS resource wrapper for the event system.

use crate::core::event::{
    queue::EventQueue, reader::EventReader, traits::Event, writer::EventWriter,
};

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

    /// Returns a slice of events available for reading.
    ///
    /// This provides direct access to the read buffer, enabling cursor-based
    /// reading patterns used by ECS system parameters.
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
    /// let buffer = events.read_buffer();
    /// assert_eq!(buffer.len(), 2);
    /// ```
    #[must_use]
    pub fn read_buffer(&self) -> &[E] {
        self.queue.read_buffer()
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
