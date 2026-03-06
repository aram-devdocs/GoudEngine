//! Tests for the Events ECS resource wrapper.

use crate::core::event::Events;

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
