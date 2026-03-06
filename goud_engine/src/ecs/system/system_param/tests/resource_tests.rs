//! Tests for resource system parameters: Res, ResMut, conflict detection.

use crate::ecs::query::Access;
use crate::ecs::resource::{Res, ResMut, ResourceId};
use crate::ecs::system::{
    ReadOnlySystemParam, ResMutState, ResState, SystemParam, SystemParamState,
};
use crate::ecs::World;

// =========================================================================
// Res<T> SystemParam Tests
// =========================================================================

mod res_param {
    use super::*;

    #[derive(Debug)]
    struct Time {
        delta: f32,
        total: f32,
    }

    #[derive(Debug)]
    struct Score(u32);

    #[test]
    fn test_res_state_init() {
        let mut world = World::new();
        let _state: ResState<Time> = ResState::init(&mut world);
    }

    #[test]
    fn test_res_update_access() {
        let mut world = World::new();
        let state: ResState<Time> = ResState::init(&mut world);

        let mut access = Access::new();
        Res::<Time>::update_access(&state, &mut access);

        assert!(access
            .resource_reads()
            .any(|&id| id == ResourceId::of::<Time>()));
        assert!(access.is_read_only());
    }

    #[test]
    fn test_res_get_param() {
        let mut world = World::new();
        world.insert_resource(Time {
            delta: 0.016,
            total: 1.0,
        });

        let mut state: ResState<Time> = ResState::init(&mut world);
        let time: Res<Time> = Res::get_param(&mut state, &world);

        assert_eq!(time.delta, 0.016);
        assert_eq!(time.total, 1.0);
    }

    #[test]
    fn test_res_get_param_mut() {
        let mut world = World::new();
        world.insert_resource(Time {
            delta: 0.016,
            total: 1.0,
        });

        let mut state: ResState<Time> = ResState::init(&mut world);
        let time: Res<Time> = Res::get_param_mut(&mut state, &mut world);

        assert_eq!(time.delta, 0.016);
    }

    #[test]
    #[should_panic(expected = "Resource does not exist")]
    fn test_res_get_param_missing_resource() {
        let mut world = World::new();
        let mut state: ResState<Time> = ResState::init(&mut world);
        let _time: Res<Time> = Res::get_param(&mut state, &world);
    }

    #[test]
    fn test_res_implements_system_param() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<Res<Time>>();
        requires_system_param::<Res<Score>>();
    }

    #[test]
    fn test_res_implements_read_only() {
        fn requires_read_only<T: ReadOnlySystemParam>() {}
        requires_read_only::<Res<Time>>();
        requires_read_only::<Res<Score>>();
    }

    #[test]
    fn test_res_multiple_resources() {
        let mut world = World::new();
        world.insert_resource(Time {
            delta: 0.016,
            total: 0.0,
        });
        world.insert_resource(Score(100));

        let mut time_state: ResState<Time> = ResState::init(&mut world);
        let mut score_state: ResState<Score> = ResState::init(&mut world);

        let time: Res<Time> = Res::get_param(&mut time_state, &world);
        let score: Res<Score> = Res::get_param(&mut score_state, &world);

        assert_eq!(time.delta, 0.016);
        assert_eq!(score.0, 100);
    }

    #[test]
    fn test_res_access_no_conflict() {
        let mut world = World::new();

        let state1: ResState<Time> = ResState::init(&mut world);
        let state2: ResState<Time> = ResState::init(&mut world);

        let mut access1 = Access::new();
        Res::<Time>::update_access(&state1, &mut access1);

        let mut access2 = Access::new();
        Res::<Time>::update_access(&state2, &mut access2);

        assert!(!access1.conflicts_with(&access2));
    }
}

// =========================================================================
// ResMut<T> SystemParam Tests
// =========================================================================

mod res_mut_param {
    use super::*;

    #[derive(Debug)]
    struct Time {
        delta: f32,
        total: f32,
    }

    #[derive(Debug)]
    struct Score(u32);

    #[test]
    fn test_res_mut_state_init() {
        let mut world = World::new();
        let _state: ResMutState<Time> = ResMutState::init(&mut world);
    }

    #[test]
    fn test_res_mut_update_access() {
        let mut world = World::new();
        let state: ResMutState<Time> = ResMutState::init(&mut world);

        let mut access = Access::new();
        ResMut::<Time>::update_access(&state, &mut access);

        assert!(access.resource_writes().contains(&ResourceId::of::<Time>()));
        assert!(!access.is_read_only());
    }

    #[test]
    fn test_res_mut_get_param_mut() {
        let mut world = World::new();
        world.insert_resource(Time {
            delta: 0.016,
            total: 1.0,
        });

        let mut state: ResMutState<Time> = ResMutState::init(&mut world);
        let mut time: ResMut<Time> = ResMut::get_param_mut(&mut state, &mut world);

        assert_eq!(time.delta, 0.016);
        time.total += time.delta;
        assert_eq!(time.total, 1.016);
    }

    #[test]
    #[should_panic(expected = "ResMut<T> requires mutable world access")]
    fn test_res_mut_get_param_panics() {
        let mut world = World::new();
        world.insert_resource(Time {
            delta: 0.016,
            total: 1.0,
        });

        let mut state: ResMutState<Time> = ResMutState::init(&mut world);
        let _time: ResMut<Time> = ResMut::get_param(&mut state, &world);
    }

    #[test]
    #[should_panic(expected = "Resource does not exist")]
    fn test_res_mut_get_param_mut_missing_resource() {
        let mut world = World::new();
        let mut state: ResMutState<Time> = ResMutState::init(&mut world);
        let _time: ResMut<Time> = ResMut::get_param_mut(&mut state, &mut world);
    }

    #[test]
    fn test_res_mut_implements_system_param() {
        fn requires_system_param<T: SystemParam>() {}
        requires_system_param::<ResMut<Time>>();
        requires_system_param::<ResMut<Score>>();
    }

    #[test]
    fn test_res_mut_not_read_only() {
        let mut world = World::new();
        let state: ResMutState<Time> = ResMutState::init(&mut world);

        let mut access = Access::new();
        ResMut::<Time>::update_access(&state, &mut access);

        assert!(!access.is_read_only());
    }

    #[test]
    fn test_res_mut_modify_resource() {
        let mut world = World::new();
        world.insert_resource(Score(100));

        {
            let mut state: ResMutState<Score> = ResMutState::init(&mut world);
            let mut score: ResMut<Score> = ResMut::get_param_mut(&mut state, &mut world);
            score.0 += 50;
        }

        assert_eq!(world.get_resource::<Score>().unwrap().0, 150);
    }
}

// =========================================================================
// Res/ResMut Conflict Tests
// =========================================================================

mod res_conflict {
    use super::*;

    #[derive(Debug)]
    struct Time {
        delta: f32,
    }

    #[derive(Debug)]
    struct Score(u32);

    #[test]
    fn test_res_res_no_conflict() {
        let mut world = World::new();

        let state1: ResState<Time> = ResState::init(&mut world);
        let state2: ResState<Time> = ResState::init(&mut world);

        let mut access1 = Access::new();
        Res::<Time>::update_access(&state1, &mut access1);

        let mut access2 = Access::new();
        Res::<Time>::update_access(&state2, &mut access2);

        assert!(!access1.conflicts_with(&access2));
    }

    #[test]
    fn test_res_res_mut_conflict() {
        let mut world = World::new();

        let res_state: ResState<Time> = ResState::init(&mut world);
        let res_mut_state: ResMutState<Time> = ResMutState::init(&mut world);

        let mut read_access = Access::new();
        Res::<Time>::update_access(&res_state, &mut read_access);

        let mut write_access = Access::new();
        ResMut::<Time>::update_access(&res_mut_state, &mut write_access);

        assert!(read_access.conflicts_with(&write_access));
        assert!(write_access.conflicts_with(&read_access));
    }

    #[test]
    fn test_res_mut_res_mut_conflict() {
        let mut world = World::new();

        let state1: ResMutState<Time> = ResMutState::init(&mut world);
        let state2: ResMutState<Time> = ResMutState::init(&mut world);

        let mut access1 = Access::new();
        ResMut::<Time>::update_access(&state1, &mut access1);

        let mut access2 = Access::new();
        ResMut::<Time>::update_access(&state2, &mut access2);

        assert!(access1.conflicts_with(&access2));
    }

    #[test]
    fn test_different_resources_no_conflict() {
        let mut world = World::new();

        let time_state: ResState<Time> = ResState::init(&mut world);
        let score_state: ResMutState<Score> = ResMutState::init(&mut world);

        let mut time_access = Access::new();
        Res::<Time>::update_access(&time_state, &mut time_access);

        let mut score_access = Access::new();
        ResMut::<Score>::update_access(&score_state, &mut score_access);

        assert!(!time_access.conflicts_with(&score_access));
    }

    #[test]
    fn test_combined_access() {
        let mut world = World::new();

        let time_state: ResState<Time> = ResState::init(&mut world);
        let score_state: ResMutState<Score> = ResMutState::init(&mut world);

        let mut access = Access::new();
        Res::<Time>::update_access(&time_state, &mut access);
        ResMut::<Score>::update_access(&score_state, &mut access);

        assert!(access
            .resource_reads()
            .any(|&id| id == ResourceId::of::<Time>()));
        assert!(access
            .resource_writes()
            .contains(&ResourceId::of::<Score>()));
        assert!(!access.is_read_only());
    }
}

// =========================================================================
// Resource Integration Tests
// =========================================================================

mod res_integration {
    use super::*;
    use crate::ecs::Component;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone)]
    struct Time {
        delta: f32,
        total: f32,
    }

    #[derive(Debug, Clone)]
    struct Score(u32);

    #[test]
    fn test_res_with_component_query() {
        let mut world = World::new();
        world.insert_resource(Time {
            delta: 0.016,
            total: 0.0,
        });

        let e = world.spawn_empty();
        world.insert(e, Position { x: 0.0, y: 0.0 });

        let time = world.get_resource::<Time>().unwrap();
        let pos = world.get::<Position>(e).unwrap();

        assert_eq!(time.delta, 0.016);
        assert_eq!(pos.x, 0.0);
    }

    #[test]
    fn test_res_mut_modifies_world() {
        let mut world = World::new();
        world.insert_resource(Score(0));

        for _ in 0..10 {
            let mut state: ResMutState<Score> = ResMutState::init(&mut world);
            let mut score = ResMut::get_param_mut(&mut state, &mut world);
            score.0 += 10;
        }

        assert_eq!(world.get_resource::<Score>().unwrap().0, 100);
    }

    #[test]
    fn test_res_state_is_send_sync() {
        fn requires_send_sync<T: Send + Sync>() {}
        requires_send_sync::<ResState<Time>>();
        requires_send_sync::<ResMutState<Score>>();
    }

    #[test]
    fn test_res_state_is_clone() {
        let mut world = World::new();
        let state: ResState<Time> = ResState::init(&mut world);
        let _cloned = state.clone();
    }
}
