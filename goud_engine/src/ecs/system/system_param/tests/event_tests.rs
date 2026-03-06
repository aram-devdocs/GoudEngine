//! Tests for event system parameters: EcsEventReader, EcsEventWriter.

use crate::core::event::Events;
use crate::ecs::query::Access;
use crate::ecs::resource::ResourceId;
use crate::ecs::system::system_param::event_params::{EcsEventReader, EcsEventWriter};
use crate::ecs::system::{
    EcsEventReaderState, EcsEventWriterState, ReadOnlySystemParam, SystemParam, SystemParamState,
};
use crate::ecs::World;

// =========================================================================
// Test event types
// =========================================================================

#[derive(Debug, Clone, PartialEq)]
struct DamageEvent {
    amount: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct HealEvent {
    amount: u32,
}

// =========================================================================
// EcsEventReader Tests
// =========================================================================

mod reader {
    use super::*;

    #[test]
    fn test_reader_reads_events_from_world() {
        let mut world = World::new();
        let mut events = Events::<DamageEvent>::new();
        events.send(DamageEvent { amount: 10 });
        events.send(DamageEvent { amount: 25 });
        events.update();
        world.insert_resource(events);

        let mut state = EcsEventReaderState::<DamageEvent>::init(&mut world);
        let mut reader = EcsEventReader::<DamageEvent>::get_param(&mut state, &world);

        let received: Vec<_> = reader.read().collect();
        assert_eq!(received.len(), 2);
        assert_eq!(received[0].amount, 10);
        assert_eq!(received[1].amount, 25);
    }

    #[test]
    fn test_reader_cursor_tracks_position() {
        let mut world = World::new();
        let mut events = Events::<DamageEvent>::new();
        events.send(DamageEvent { amount: 10 });
        events.update();
        world.insert_resource(events);

        let mut state = EcsEventReaderState::<DamageEvent>::init(&mut world);

        // First read consumes the event
        {
            let mut reader = EcsEventReader::<DamageEvent>::get_param(&mut state, &world);
            let received: Vec<_> = reader.read().collect();
            assert_eq!(received.len(), 1);
        }

        // Second read returns nothing -- cursor already advanced
        {
            let mut reader = EcsEventReader::<DamageEvent>::get_param(&mut state, &world);
            let received: Vec<_> = reader.read().collect();
            assert_eq!(received.len(), 0);
        }
    }

    #[test]
    fn test_reader_is_empty_and_len() {
        let mut world = World::new();
        let mut events = Events::<DamageEvent>::new();
        events.send(DamageEvent { amount: 5 });
        events.send(DamageEvent { amount: 15 });
        events.update();
        world.insert_resource(events);

        let mut state = EcsEventReaderState::<DamageEvent>::init(&mut world);
        let mut reader = EcsEventReader::<DamageEvent>::get_param(&mut state, &world);

        assert!(!reader.is_empty());
        assert_eq!(reader.len(), 2);

        let _ = reader.read().count();

        assert!(reader.is_empty());
        assert_eq!(reader.len(), 0);
    }

    #[test]
    fn test_reader_access_is_read() {
        let mut world = World::new();
        let state = EcsEventReaderState::<DamageEvent>::init(&mut world);

        let mut access = Access::new();
        EcsEventReader::<DamageEvent>::update_access(&state, &mut access);

        assert!(access
            .resource_reads()
            .any(|&id| id == ResourceId::of::<Events<DamageEvent>>()));
        assert!(access.is_read_only());
    }

    #[test]
    fn test_reader_implements_read_only() {
        fn requires_read_only<T: ReadOnlySystemParam>() {}
        requires_read_only::<EcsEventReader<DamageEvent>>();
    }

    #[test]
    fn test_reader_implements_system_param() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<EcsEventReader<DamageEvent>>();
    }

    #[test]
    fn test_multiple_readers_independent_cursors() {
        let mut world = World::new();
        let mut events = Events::<DamageEvent>::new();
        events.send(DamageEvent { amount: 10 });
        events.send(DamageEvent { amount: 20 });
        events.update();
        world.insert_resource(events);

        let mut state_a = EcsEventReaderState::<DamageEvent>::init(&mut world);
        let mut state_b = EcsEventReaderState::<DamageEvent>::init(&mut world);

        // Reader A consumes both events
        {
            let mut reader_a = EcsEventReader::<DamageEvent>::get_param(&mut state_a, &world);
            assert_eq!(reader_a.read().count(), 2);
        }

        // Reader B still sees both events (independent cursor)
        {
            let mut reader_b = EcsEventReader::<DamageEvent>::get_param(&mut state_b, &world);
            let received: Vec<_> = reader_b.read().collect();
            assert_eq!(received.len(), 2);
            assert_eq!(received[0].amount, 10);
            assert_eq!(received[1].amount, 20);
        }

        // Reader A sees nothing on second read
        {
            let mut reader_a = EcsEventReader::<DamageEvent>::get_param(&mut state_a, &world);
            assert_eq!(reader_a.read().count(), 0);
        }
    }

    #[test]
    fn test_reader_state_is_send_sync() {
        fn requires_send_sync<T: Send + Sync>() {}
        requires_send_sync::<EcsEventReaderState<DamageEvent>>();
    }
}

// =========================================================================
// EcsEventWriter Tests
// =========================================================================

mod writer {
    use super::*;

    #[test]
    fn test_writer_sends_events() {
        let mut world = World::new();
        world.insert_resource(Events::<DamageEvent>::new());

        let mut state = EcsEventWriterState::<DamageEvent>::init(&mut world);
        {
            let mut writer = EcsEventWriter::<DamageEvent>::get_param_mut(&mut state, &mut world);
            writer.send(DamageEvent { amount: 42 });
            writer.send(DamageEvent { amount: 99 });
        }

        // Swap buffers so events become readable
        {
            let mut events = world.resource_mut::<Events<DamageEvent>>().unwrap();
            events.update();
        }

        let events = world.resource::<Events<DamageEvent>>().unwrap();
        assert_eq!(events.read_len(), 2);
        let buffer = events.read_buffer();
        assert_eq!(buffer[0].amount, 42);
        assert_eq!(buffer[1].amount, 99);
    }

    #[test]
    fn test_writer_send_batch() {
        let mut world = World::new();
        world.insert_resource(Events::<DamageEvent>::new());

        let mut state = EcsEventWriterState::<DamageEvent>::init(&mut world);
        {
            let mut writer = EcsEventWriter::<DamageEvent>::get_param_mut(&mut state, &mut world);
            writer.send_batch(vec![
                DamageEvent { amount: 1 },
                DamageEvent { amount: 2 },
                DamageEvent { amount: 3 },
            ]);
        }

        let events = world.resource::<Events<DamageEvent>>().unwrap();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_writer_access_is_write() {
        let mut world = World::new();
        let state = EcsEventWriterState::<DamageEvent>::init(&mut world);

        let mut access = Access::new();
        EcsEventWriter::<DamageEvent>::update_access(&state, &mut access);

        assert!(access
            .resource_writes()
            .contains(&ResourceId::of::<Events<DamageEvent>>()));
        assert!(!access.is_read_only());
    }

    #[test]
    #[should_panic(expected = "requires mutable world access")]
    fn test_writer_get_param_panics() {
        let mut world = World::new();
        world.insert_resource(Events::<DamageEvent>::new());

        let mut state = EcsEventWriterState::<DamageEvent>::init(&mut world);
        let _writer = EcsEventWriter::<DamageEvent>::get_param(&mut state, &world);
    }

    #[test]
    fn test_writer_implements_system_param() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<EcsEventWriter<DamageEvent>>();
    }

    #[test]
    fn test_writer_state_is_send_sync() {
        fn requires_send_sync<T: Send + Sync>() {}
        requires_send_sync::<EcsEventWriterState<DamageEvent>>();
    }
}

// =========================================================================
// Reader/Writer Conflict Tests
// =========================================================================

mod conflict {
    use super::*;

    #[test]
    fn test_reader_reader_no_conflict() {
        let mut world = World::new();
        let state_a = EcsEventReaderState::<DamageEvent>::init(&mut world);
        let state_b = EcsEventReaderState::<DamageEvent>::init(&mut world);

        let mut access_a = Access::new();
        EcsEventReader::<DamageEvent>::update_access(&state_a, &mut access_a);

        let mut access_b = Access::new();
        EcsEventReader::<DamageEvent>::update_access(&state_b, &mut access_b);

        assert!(!access_a.conflicts_with(&access_b));
    }

    #[test]
    fn test_reader_writer_conflict() {
        let mut world = World::new();
        let reader_state = EcsEventReaderState::<DamageEvent>::init(&mut world);
        let writer_state = EcsEventWriterState::<DamageEvent>::init(&mut world);

        let mut read_access = Access::new();
        EcsEventReader::<DamageEvent>::update_access(&reader_state, &mut read_access);

        let mut write_access = Access::new();
        EcsEventWriter::<DamageEvent>::update_access(&writer_state, &mut write_access);

        assert!(read_access.conflicts_with(&write_access));
        assert!(write_access.conflicts_with(&read_access));
    }

    #[test]
    fn test_different_event_types_no_conflict() {
        let mut world = World::new();
        let damage_state = EcsEventReaderState::<DamageEvent>::init(&mut world);
        let heal_state = EcsEventWriterState::<HealEvent>::init(&mut world);

        let mut damage_access = Access::new();
        EcsEventReader::<DamageEvent>::update_access(&damage_state, &mut damage_access);

        let mut heal_access = Access::new();
        EcsEventWriter::<HealEvent>::update_access(&heal_state, &mut heal_access);

        assert!(!damage_access.conflicts_with(&heal_access));
    }
}
