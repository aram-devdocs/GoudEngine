//! Tests for core system parameter traits, tuples, static params, and custom params.

use crate::ecs::query::Access;
use crate::ecs::system::{
    ReadOnlySystemParam, StaticSystemParam, StaticSystemParamState, SystemParam, SystemParamState,
};
use crate::ecs::World;

// Test components
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Position {
    pub x: f32,
    pub y: f32,
}
impl crate::ecs::Component for Position {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Velocity {
    pub x: f32,
    pub y: f32,
}
impl crate::ecs::Component for Velocity {}

// =========================================================================
// SystemParamState Tests
// =========================================================================

mod system_param_state {
    use super::*;

    #[test]
    fn test_unit_state_init() {
        let mut world = World::new();
        let _state: () = <()>::init(&mut world);
        // Unit state is just ()
    }

    #[test]
    fn test_unit_state_apply() {
        let mut world = World::new();
        let mut state: () = <()>::init(&mut world);
        state.apply(&mut world);
        // Should not panic
    }

    #[test]
    fn test_state_is_send_sync() {
        fn requires_send_sync<T: Send + Sync>() {}
        requires_send_sync::<()>();
    }

    #[test]
    fn test_tuple_state_init() {
        let mut world = World::new();
        let _state: ((), ()) = <((), ())>::init(&mut world);
    }

    #[test]
    fn test_tuple_state_apply() {
        let mut world = World::new();
        let mut state: ((), (), ()) = <((), (), ())>::init(&mut world);
        state.apply(&mut world);
    }
}

// =========================================================================
// SystemParam Tests
// =========================================================================

mod system_param {
    use super::*;

    #[test]
    fn test_unit_param_update_access() {
        let state = ();
        let mut access = Access::new();
        <()>::update_access(&state, &mut access);

        assert!(access.is_read_only());
        assert_eq!(access.writes().len(), 0);
    }

    #[test]
    fn test_unit_param_get_param() {
        let world = World::new();
        let mut state = ();
        let _result: () = <()>::get_param(&mut state, &world);
    }

    #[test]
    fn test_unit_param_get_param_mut() {
        let mut world = World::new();
        let mut state = ();
        let _result: () = <()>::get_param_mut(&mut state, &mut world);
    }

    #[test]
    fn test_unit_implements_system_param() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<()>();
    }

    #[test]
    fn test_unit_implements_read_only() {
        fn requires_read_only<T: ReadOnlySystemParam>() {}
        requires_read_only::<()>();
    }
}

// =========================================================================
// Tuple SystemParam Tests
// =========================================================================

mod tuple_param {
    use super::*;

    #[test]
    fn test_single_tuple_param() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<((),)>();
    }

    #[test]
    fn test_double_tuple_param() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<((), ())>();
    }

    #[test]
    fn test_triple_tuple_param() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<((), (), ())>();
    }

    #[test]
    fn test_tuple_get_param() {
        let world = World::new();
        let mut state: ((), ()) = ((), ());
        let (a, b): ((), ()) = <((), ())>::get_param(&mut state, &world);
        assert_eq!(a, ());
        assert_eq!(b, ());
    }

    #[test]
    fn test_tuple_update_access() {
        let state: ((), ()) = ((), ());
        let mut access = Access::new();
        <((), ())>::update_access(&state, &mut access);
        assert!(access.is_read_only());
    }

    #[test]
    fn test_nested_tuple_param() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<(((), ()), ())>();
    }

    #[test]
    fn test_large_tuple_param() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<(
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
        )>();
    }

    #[test]
    fn test_tuple_read_only() {
        fn requires_read_only<T: ReadOnlySystemParam>() {}
        requires_read_only::<((), ())>();
        requires_read_only::<((), (), ())>();
    }
}

// =========================================================================
// Custom SystemParam Tests
// =========================================================================

mod custom_param {
    use super::*;

    struct EntityCount(usize);
    struct EntityCountState;

    impl SystemParamState for EntityCountState {
        fn init(_world: &mut World) -> Self {
            EntityCountState
        }
    }

    impl SystemParam for EntityCount {
        type State = EntityCountState;
        type Item<'w, 's> = EntityCount;

        fn update_access(_state: &Self::State, _access: &mut Access) {}

        fn get_param<'w, 's>(_state: &'s mut Self::State, world: &'w World) -> Self::Item<'w, 's> {
            EntityCount(world.entity_count())
        }
    }

    impl ReadOnlySystemParam for EntityCount {}

    #[test]
    fn test_custom_param_implements_trait() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<EntityCount>();
    }

    #[test]
    fn test_custom_param_state_init() {
        let mut world = World::new();
        let _state = EntityCountState::init(&mut world);
    }

    #[test]
    fn test_custom_param_get_param() {
        let mut world = World::new();
        world.spawn_empty();
        world.spawn_empty();
        world.spawn_empty();

        let mut state = EntityCountState;
        let count = EntityCount::get_param(&mut state, &world);
        assert_eq!(count.0, 3);
    }

    #[test]
    fn test_custom_param_access() {
        let state = EntityCountState;
        let mut access = Access::new();
        EntityCount::update_access(&state, &mut access);
        assert!(access.is_read_only());
    }

    #[test]
    fn test_custom_param_read_only() {
        fn requires_read_only<T: ReadOnlySystemParam>() {}
        requires_read_only::<EntityCount>();
    }
}

// =========================================================================
// Custom Param with Component Access Tests
// =========================================================================

mod param_with_access {
    use crate::ecs::component::ComponentId;

    use super::*;

    struct PositionReader;

    struct PositionReaderState {
        component_id: ComponentId,
    }

    impl SystemParamState for PositionReaderState {
        fn init(_world: &mut World) -> Self {
            Self {
                component_id: ComponentId::of::<Position>(),
            }
        }
    }

    impl SystemParam for PositionReader {
        type State = PositionReaderState;
        type Item<'w, 's> = PositionReader;

        fn update_access(state: &Self::State, access: &mut Access) {
            access.add_read(state.component_id);
        }

        fn get_param<'w, 's>(_state: &'s mut Self::State, _world: &'w World) -> Self::Item<'w, 's> {
            PositionReader
        }
    }

    impl ReadOnlySystemParam for PositionReader {}

    struct PositionWriter;

    struct PositionWriterState {
        component_id: ComponentId,
    }

    impl SystemParamState for PositionWriterState {
        fn init(_world: &mut World) -> Self {
            Self {
                component_id: ComponentId::of::<Position>(),
            }
        }
    }

    impl SystemParam for PositionWriter {
        type State = PositionWriterState;
        type Item<'w, 's> = PositionWriter;

        fn update_access(state: &Self::State, access: &mut Access) {
            access.add_write(state.component_id);
        }

        fn get_param<'w, 's>(_state: &'s mut Self::State, _world: &'w World) -> Self::Item<'w, 's> {
            PositionWriter
        }
    }

    #[test]
    fn test_reader_is_read_only() {
        let mut world = World::new();
        let state = PositionReaderState::init(&mut world);
        let mut access = Access::new();
        PositionReader::update_access(&state, &mut access);
        assert!(access.is_read_only());
    }

    #[test]
    fn test_writer_is_not_read_only() {
        let mut world = World::new();
        let state = PositionWriterState::init(&mut world);
        let mut access = Access::new();
        PositionWriter::update_access(&state, &mut access);
        assert!(!access.is_read_only());
    }

    #[test]
    fn test_reader_writer_conflict() {
        let mut world = World::new();

        let reader_state = PositionReaderState::init(&mut world);
        let mut reader_access = Access::new();
        PositionReader::update_access(&reader_state, &mut reader_access);

        let writer_state = PositionWriterState::init(&mut world);
        let mut writer_access = Access::new();
        PositionWriter::update_access(&writer_state, &mut writer_access);

        assert!(reader_access.conflicts_with(&writer_access));
        assert!(writer_access.conflicts_with(&reader_access));
    }

    #[test]
    fn test_readers_dont_conflict() {
        let mut world = World::new();

        let state1 = PositionReaderState::init(&mut world);
        let mut access1 = Access::new();
        PositionReader::update_access(&state1, &mut access1);

        let state2 = PositionReaderState::init(&mut world);
        let mut access2 = Access::new();
        PositionReader::update_access(&state2, &mut access2);

        assert!(!access1.conflicts_with(&access2));
    }
}

// =========================================================================
// StaticSystemParam Tests
// =========================================================================

mod static_param {
    use super::*;

    #[derive(Debug, Default)]
    struct CounterState {
        count: u32,
    }

    impl SystemParamState for CounterState {
        fn init(_world: &mut World) -> Self {
            Self::default()
        }
    }

    #[test]
    fn test_static_param_state_init() {
        let mut world = World::new();
        let state: StaticSystemParamState<CounterState> =
            StaticSystemParamState::<CounterState>::init(&mut world);
        assert_eq!(state.get().count, 0);
    }

    #[test]
    fn test_static_param_state_get() {
        let mut world = World::new();
        let state: StaticSystemParamState<CounterState> =
            StaticSystemParamState::<CounterState>::init(&mut world);
        assert_eq!(state.get().count, 0);
    }

    #[test]
    fn test_static_param_state_get_mut() {
        let mut world = World::new();
        let mut state: StaticSystemParamState<CounterState> =
            StaticSystemParamState::<CounterState>::init(&mut world);
        state.get_mut().count = 42;
        assert_eq!(state.get().count, 42);
    }

    #[test]
    fn test_static_param_get_param() {
        let mut world = World::new();
        let mut state: StaticSystemParamState<CounterState> =
            StaticSystemParamState::<CounterState>::init(&mut world);

        let counter: &mut CounterState =
            StaticSystemParam::<CounterState>::get_param(&mut state, &world);

        counter.count = 100;
        assert_eq!(state.get().count, 100);
    }

    #[test]
    fn test_static_param_no_access() {
        let mut world = World::new();
        let state: StaticSystemParamState<CounterState> =
            StaticSystemParamState::<CounterState>::init(&mut world);

        let mut access = Access::new();
        StaticSystemParam::<CounterState>::update_access(&state, &mut access);

        assert!(access.is_read_only());
        assert_eq!(access.writes().len(), 0);
    }
}
