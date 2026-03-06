//! Double-buffered event queue storage.

use crate::core::event::traits::Event;

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
    /// This is exposed for use by [`crate::core::event::EventReader`] to access events without draining.
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
