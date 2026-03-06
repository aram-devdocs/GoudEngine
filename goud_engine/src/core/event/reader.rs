//! Event reader types for consuming events from a queue.

use crate::core::event::{queue::EventQueue, traits::Event};

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
    pub(crate) reader: &'r mut EventReader<'a, E>,
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
