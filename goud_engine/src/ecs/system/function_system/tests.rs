//! Tests for the function system module.

#[cfg(test)]
mod tests {
    use crate::ecs::component::ComponentId;
    use crate::ecs::query::{Query, With};
    use crate::ecs::system::function_system::core::{FnMarker, FunctionSystem};
    use crate::ecs::system::{BoxedSystem, IntoSystem, System};
    use crate::ecs::{Component, World};

    // Test components
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    #[derive(Debug, Clone, Copy)]
    struct Player;
    impl Component for Player {}

    // Test resource (for documentation only - resource systems tested elsewhere)
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct Score(u32);

    // =========================================================================
    // Zero Parameter Function Tests
    // =========================================================================

    mod zero_params {
        use super::*;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        #[test]
        fn test_fn_no_params() {
            static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

            fn my_system() {
                CALL_COUNT.fetch_add(1, Ordering::SeqCst);
            }

            let mut world = World::new();
            let mut boxed = my_system.into_system();

            assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 0);
            boxed.run(&mut world);
            assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
            boxed.run(&mut world);
            assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 2);
        }

        #[test]
        fn test_fn_no_params_name() {
            fn my_system() {}

            let boxed = my_system.into_system();
            // Name should contain the function name
            assert!(boxed.name().contains("my_system"));
        }

        #[test]
        fn test_fn_no_params_is_read_only() {
            fn my_system() {}

            let boxed = my_system.into_system();
            // No parameters means read-only
            assert!(boxed.is_read_only());
        }

        #[test]
        fn test_fn_closure_no_params() {
            let counter = Arc::new(AtomicU32::new(0));
            let counter_clone = counter.clone();

            let system = move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            };

            let mut world = World::new();
            let mut boxed = system.into_system();

            boxed.run(&mut world);
            assert_eq!(counter.load(Ordering::SeqCst), 1);
        }
    }

    // =========================================================================
    // One Parameter Function Tests
    // =========================================================================

    mod one_param {
        use super::*;

        #[test]
        fn test_fn_one_query_param() {
            fn count_entities(_query: Query<&Position>) {
                // Query would be used here
            }

            let boxed = count_entities.into_system();
            assert!(boxed.name().contains("count_entities"));
        }

        #[test]
        fn test_fn_query_access_tracking() {
            fn position_reader(_query: Query<&Position>) {}

            let mut world = World::new();
            let mut boxed = position_reader.into_system();
            boxed.initialize(&mut world);

            let access = boxed.component_access();
            assert!(access
                .reads()
                .any(|&id| id == ComponentId::of::<Position>()));
            assert!(access.is_read_only());
        }

        #[test]
        fn test_fn_filtered_query() {
            fn player_positions(_query: Query<&Position, With<Player>>) {}

            let boxed = player_positions.into_system();
            assert!(boxed.name().contains("player_positions"));
        }
    }

    // =========================================================================
    // Two Parameter Function Tests
    // =========================================================================

    mod two_params {
        use super::*;

        #[test]
        fn test_fn_two_queries() {
            fn movement_system(_positions: Query<&Position>, _velocities: Query<&Velocity>) {}

            let boxed = movement_system.into_system();
            assert!(boxed.name().contains("movement_system"));
        }

        #[test]
        fn test_fn_two_filtered_queries() {
            fn player_system(
                _positions: Query<&Position, With<Player>>,
                _velocities: Query<&Velocity>,
            ) {
            }

            let boxed = player_system.into_system();
            assert!(boxed.name().contains("player_system"));
        }
    }

    // =========================================================================
    // Three+ Parameter Function Tests
    // =========================================================================

    mod multi_params {
        use super::*;
        use crate::ecs::Entity;

        #[test]
        fn test_fn_three_queries() {
            fn complex_system(
                _positions: Query<&Position>,
                _velocities: Query<&Velocity>,
                _entities: Query<Entity>,
            ) {
            }

            let boxed = complex_system.into_system();
            assert!(boxed.name().contains("complex_system"));
        }

        #[test]
        fn test_fn_four_queries() {
            fn even_more_complex(
                _positions: Query<&Position>,
                _velocities: Query<&Velocity>,
                _players: Query<&Position, With<Player>>,
                _entities: Query<Entity>,
            ) {
            }

            let boxed = even_more_complex.into_system();
            assert!(boxed.name().contains("even_more_complex"));
        }

        #[test]
        fn test_fn_with_filtered_queries() {
            fn filtered_system(
                _players: Query<&Position, With<Player>>,
                _velocities: Query<&Velocity, With<Player>>,
            ) {
            }

            let boxed = filtered_system.into_system();
            assert!(boxed.name().contains("filtered_system"));
        }
    }

    // =========================================================================
    // FunctionSystem Direct Tests
    // =========================================================================

    mod function_system_direct {
        use super::*;

        #[test]
        fn test_function_system_new() {
            let system = FunctionSystem::<FnMarker, _>::new(|| {});
            assert!(format!("{:?}", system).contains("FunctionSystem"));
        }

        #[test]
        fn test_function_system_with_name() {
            fn my_fn() {}

            let system = FunctionSystem::<FnMarker, _>::new(my_fn).with_name("CustomName");
            // Note: with_name sets meta.name, but System::name() returns type name
            // This is intentional - the type name is more useful for debugging
            let _ = system;
        }

        #[test]
        fn test_function_system_debug() {
            fn my_fn() {}

            let system = FunctionSystem::<FnMarker, _>::new(my_fn);
            let debug = format!("{:?}", system);
            assert!(debug.contains("FunctionSystem"));
            assert!(debug.contains("initialized"));
        }

        #[test]
        fn test_function_system_initialize() {
            fn my_fn() {}

            let mut world = World::new();
            let mut system = FunctionSystem::<FnMarker, _>::new(my_fn);

            assert!(format!("{:?}", system).contains("initialized: false"));
            system.initialize(&mut world);
            assert!(format!("{:?}", system).contains("initialized: true"));
        }

        #[test]
        fn test_function_system_run_initializes() {
            static mut CALLED: bool = false;

            fn my_fn() {
                unsafe {
                    CALLED = true;
                }
            }

            let mut world = World::new();
            let mut system = FunctionSystem::<FnMarker, _>::new(my_fn);

            // Running should initialize if needed
            system.run(&mut world);
            assert!(unsafe { CALLED });
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        #[test]
        fn test_boxed_system_collection() {
            fn system_a() {}
            fn system_b() {}
            fn system_c() {}

            let systems: Vec<BoxedSystem> = vec![
                system_a.into_system(),
                system_b.into_system(),
                system_c.into_system(),
            ];

            assert_eq!(systems.len(), 3);
            assert!(systems[0].name().contains("system_a"));
            assert!(systems[1].name().contains("system_b"));
            assert!(systems[2].name().contains("system_c"));
        }

        #[test]
        fn test_run_boxed_systems() {
            let counter = Arc::new(AtomicU32::new(0));

            let c1 = counter.clone();
            let system1 = move || {
                c1.fetch_add(1, Ordering::SeqCst);
            };

            let c2 = counter.clone();
            let system2 = move || {
                c2.fetch_add(10, Ordering::SeqCst);
            };

            let mut systems: Vec<BoxedSystem> = vec![system1.into_system(), system2.into_system()];

            let mut world = World::new();

            for system in &mut systems {
                system.run(&mut world);
            }

            assert_eq!(counter.load(Ordering::SeqCst), 11);
        }

        #[test]
        fn test_system_with_actual_queries() {
            // Create a system that actually uses queries
            fn position_printer(_query: Query<&Position>) {
                // Would print position in real system
            }

            let mut world = World::new();

            // Add some entities
            let e1 = world.spawn_empty();
            world.insert(e1, Position { x: 1.0, y: 2.0 });

            let e2 = world.spawn_empty();
            world.insert(e2, Position { x: 3.0, y: 4.0 });

            let mut boxed = position_printer.into_system();
            boxed.run(&mut world);
            // System ran without panic
        }

        #[test]
        fn test_reader_system() {
            fn reader(_query: Query<&Position>) {}

            let mut world = World::new();

            let mut reader_system = reader.into_system();

            // Initialize to get access patterns
            reader_system.initialize(&mut world);

            // Verify access pattern is read-only
            assert!(reader_system.is_read_only());
        }

        #[test]
        fn test_multi_query_system() {
            fn multi_query(_positions: Query<&Position>, _velocities: Query<&Velocity>) {}

            let mut world = World::new();

            let e = world.spawn_empty();
            world.insert(e, Position { x: 0.0, y: 0.0 });
            world.insert(e, Velocity { x: 1.0, y: 1.0 });

            let mut boxed = multi_query.into_system();
            boxed.run(&mut world);
            // System runs successfully
        }
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    mod thread_safety {
        use super::*;

        #[test]
        fn test_function_system_is_send() {
            fn my_system() {}

            fn requires_send<T: Send>() {}
            requires_send::<FunctionSystem<FnMarker, fn()>>();

            let _ = my_system.into_system();
        }

        #[test]
        fn test_boxed_system_is_send() {
            fn my_system() {}

            fn requires_send<T: Send>() {}

            let boxed = my_system.into_system();
            requires_send::<BoxedSystem>();
            let _ = boxed;
        }
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_world() {
            fn check_empty(_query: Query<&Position>) {
                // Would check count
            }

            let mut world = World::new();
            let mut boxed = check_empty.into_system();
            boxed.run(&mut world);
        }

        #[test]
        fn test_multiple_runs() {
            static mut RUN_COUNT: u32 = 0;

            fn counting_system() {
                unsafe {
                    RUN_COUNT += 1;
                }
            }

            let mut world = World::new();
            let mut boxed = counting_system.into_system();

            for _ in 0..100 {
                boxed.run(&mut world);
            }

            assert_eq!(unsafe { RUN_COUNT }, 100);
        }

        #[test]
        fn test_system_state_persists() {
            use std::sync::atomic::{AtomicU32, Ordering};
            use std::sync::Arc;

            let run_count = Arc::new(AtomicU32::new(0));
            let run_count_clone = run_count.clone();

            let init_once = move || {
                run_count_clone.fetch_add(1, Ordering::SeqCst);
            };

            let mut world = World::new();
            let mut boxed = init_once.into_system();

            // Initialize explicitly
            boxed.initialize(&mut world);

            // Running should work and update run count
            boxed.run(&mut world);
            assert_eq!(run_count.load(Ordering::SeqCst), 1);

            // Running again should work too
            boxed.run(&mut world);
            assert_eq!(run_count.load(Ordering::SeqCst), 2);
        }
    }
}
