//! Tests for EventWriter, Events resource wrapper, and EventReaderIter.

use crate::core::event::{Event, EventQueue, EventReader, EventWriter};

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

// =========================================================================
// EventReaderIter Tests
// =========================================================================

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
