//! Tests for the Event trait, EventQueue, and EventReader.

use crate::core::event::{Event, EventQueue, EventReader};

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
