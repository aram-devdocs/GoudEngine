//! Integration and thread safety tests for system parameters.

use crate::ecs::component::ComponentId;
use crate::ecs::query::Access;
use crate::ecs::system::{
    StaticSystemParam, StaticSystemParamState, SystemParam, SystemParamState,
};
use crate::ecs::World;

// Shared test components
#[derive(Debug, Clone, Copy, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
impl crate::ecs::Component for Position {}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
}
impl crate::ecs::Component for Velocity {}

// =========================================================================
// Integration Tests (core)
// =========================================================================

mod integration {
    use super::*;

    #[test]
    fn test_param_with_entity_spawn() {
        struct EntitySpawner;
        struct EntitySpawnerState;

        impl SystemParamState for EntitySpawnerState {
            fn init(_world: &mut World) -> Self {
                EntitySpawnerState
            }
        }

        impl SystemParam for EntitySpawner {
            type State = EntitySpawnerState;
            type Item<'w, 's> = EntitySpawner;

            fn update_access(_state: &Self::State, _access: &mut Access) {}

            fn get_param<'w, 's>(
                _state: &'s mut Self::State,
                _world: &'w World,
            ) -> Self::Item<'w, 's> {
                EntitySpawner
            }
        }

        let mut world = World::new();
        let mut state = EntitySpawnerState::init(&mut world);
        let _spawner1 = EntitySpawner::get_param(&mut state, &world);
        let _spawner2 = EntitySpawner::get_param(&mut state, &world);
    }

    #[test]
    fn test_combined_access_tracking() {
        struct ReaderA;
        struct WriterB;

        struct ReaderAState(ComponentId);
        struct WriterBState(ComponentId);

        impl SystemParamState for ReaderAState {
            fn init(_world: &mut World) -> Self {
                Self(ComponentId::of::<Position>())
            }
        }

        impl SystemParamState for WriterBState {
            fn init(_world: &mut World) -> Self {
                Self(ComponentId::of::<Velocity>())
            }
        }

        impl SystemParam for ReaderA {
            type State = ReaderAState;
            type Item<'w, 's> = ReaderA;

            fn update_access(state: &Self::State, access: &mut Access) {
                access.add_read(state.0);
            }

            fn get_param<'w, 's>(
                _state: &'s mut Self::State,
                _world: &'w World,
            ) -> Self::Item<'w, 's> {
                ReaderA
            }
        }

        impl SystemParam for WriterB {
            type State = WriterBState;
            type Item<'w, 's> = WriterB;

            fn update_access(state: &Self::State, access: &mut Access) {
                access.add_write(state.0);
            }

            fn get_param<'w, 's>(
                _state: &'s mut Self::State,
                _world: &'w World,
            ) -> Self::Item<'w, 's> {
                WriterB
            }
        }

        let mut world = World::new();
        let state: (ReaderAState, WriterBState) = <(ReaderAState, WriterBState)>::init(&mut world);

        let mut access = Access::new();
        <(ReaderA, WriterB)>::update_access(&state, &mut access);

        assert!(!access.is_read_only());
        assert!(access.writes().contains(&ComponentId::of::<Velocity>()));
        assert!(access
            .reads()
            .any(|&id| id == ComponentId::of::<Position>()));
    }
}

// =========================================================================
// Thread Safety Tests
// =========================================================================

mod thread_safety {
    use super::*;

    #[test]
    fn test_system_param_state_is_send() {
        fn requires_send<T: Send>() {}
        requires_send::<()>();
    }

    #[test]
    fn test_system_param_state_is_sync() {
        fn requires_sync<T: Sync>() {}
        requires_sync::<()>();
    }

    #[test]
    fn test_tuple_state_is_send_sync() {
        fn requires_send_sync<T: Send + Sync>() {}
        requires_send_sync::<((), ())>();
        requires_send_sync::<((), (), ())>();
    }

    #[test]
    fn test_static_param_state_is_send_sync() {
        fn requires_send_sync<T: Send + Sync>() {}

        #[derive(Debug)]
        struct SendSyncState;
        impl SystemParamState for SendSyncState {
            fn init(_world: &mut World) -> Self {
                SendSyncState
            }
        }

        requires_send_sync::<StaticSystemParamState<SendSyncState>>();
    }
}
