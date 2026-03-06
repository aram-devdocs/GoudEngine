//! Tests for [`PhysicsWorld`].

#[cfg(test)]
mod tests {
    use crate::core::math::Vec2;
    use crate::ecs::physics_world::PhysicsWorld;

    // =========================================================================
    // Construction Tests
    // =========================================================================

    #[test]
    fn test_new() {
        let physics = PhysicsWorld::new();
        assert_eq!(physics.gravity(), Vec2::new(0.0, -980.0));
        assert_eq!(physics.timestep(), 1.0 / 60.0);
        assert_eq!(physics.velocity_iterations(), 8);
        assert_eq!(physics.position_iterations(), 3);
        assert_eq!(physics.time_scale(), 1.0);
        assert!(!physics.is_paused());
        assert!(physics.is_sleep_enabled());
    }

    #[test]
    fn test_default() {
        let physics = PhysicsWorld::default();
        assert_eq!(physics.timestep(), 1.0 / 60.0);
    }

    #[test]
    fn test_zero_gravity() {
        let physics = PhysicsWorld::zero_gravity();
        assert_eq!(physics.gravity(), Vec2::zero());
    }

    // =========================================================================
    // Builder Pattern Tests
    // =========================================================================

    #[test]
    fn test_with_gravity() {
        let physics = PhysicsWorld::new().with_gravity(Vec2::new(0.0, -490.0));
        assert_eq!(physics.gravity(), Vec2::new(0.0, -490.0));
    }

    #[test]
    fn test_with_timestep() {
        let physics = PhysicsWorld::new().with_timestep(1.0 / 120.0);
        assert_eq!(physics.timestep(), 1.0 / 120.0);
    }

    #[test]
    #[should_panic(expected = "Timestep must be positive and finite")]
    fn test_with_timestep_invalid() {
        let _ = PhysicsWorld::new().with_timestep(0.0);
    }

    #[test]
    fn test_with_iterations() {
        let physics = PhysicsWorld::new().with_iterations(10, 4);
        assert_eq!(physics.velocity_iterations(), 10);
        assert_eq!(physics.position_iterations(), 4);
    }

    #[test]
    fn test_with_time_scale() {
        let physics = PhysicsWorld::new().with_time_scale(0.5);
        assert_eq!(physics.time_scale(), 0.5);
    }

    #[test]
    #[should_panic(expected = "Time scale must be in range [0.0, 10.0]")]
    fn test_with_time_scale_too_high() {
        let _ = PhysicsWorld::new().with_time_scale(11.0);
    }

    #[test]
    fn test_with_sleep_config() {
        let physics = PhysicsWorld::new().with_sleep_config(false, 2.0, 0.05, 1.0);
        assert!(!physics.is_sleep_enabled());
        assert_eq!(physics.sleep_linear_threshold(), 2.0);
        assert_eq!(physics.sleep_angular_threshold(), 0.05);
        assert_eq!(physics.sleep_time_threshold(), 1.0);
    }

    #[test]
    fn test_builder_chaining() {
        let physics = PhysicsWorld::new()
            .with_gravity(Vec2::new(0.0, -490.0))
            .with_timestep(1.0 / 120.0)
            .with_iterations(10, 4)
            .with_time_scale(2.0);

        assert_eq!(physics.gravity(), Vec2::new(0.0, -490.0));
        assert_eq!(physics.timestep(), 1.0 / 120.0);
        assert_eq!(physics.velocity_iterations(), 10);
        assert_eq!(physics.time_scale(), 2.0);
    }

    // =========================================================================
    // Mutator Tests
    // =========================================================================

    #[test]
    fn test_set_gravity() {
        let mut physics = PhysicsWorld::new();
        physics.set_gravity(Vec2::new(10.0, 0.0));
        assert_eq!(physics.gravity(), Vec2::new(10.0, 0.0));
    }

    #[test]
    fn test_pause_resume() {
        let mut physics = PhysicsWorld::new();
        assert!(!physics.is_paused());

        physics.pause();
        assert!(physics.is_paused());

        physics.resume();
        assert!(!physics.is_paused());
    }

    #[test]
    fn test_set_time_scale() {
        let mut physics = PhysicsWorld::new();
        physics.set_time_scale(0.5);
        assert_eq!(physics.time_scale(), 0.5);
    }

    // =========================================================================
    // Simulation Control Tests
    // =========================================================================

    #[test]
    fn test_advance_single_step() {
        let mut physics = PhysicsWorld::new();
        let steps = physics.advance(1.0 / 60.0);
        assert_eq!(steps, 1);
        assert!(physics.should_step());
    }

    #[test]
    fn test_advance_multiple_steps() {
        let mut physics = PhysicsWorld::new();
        let steps = physics.advance(0.1); // 100ms = ~6 steps at 60 Hz
        assert!(steps >= 6);
    }

    #[test]
    fn test_advance_no_steps() {
        let mut physics = PhysicsWorld::new();
        let steps = physics.advance(0.001); // 1ms < timestep
        assert_eq!(steps, 0);
        assert!(!physics.should_step());
    }

    #[test]
    fn test_advance_with_time_scale() {
        let mut physics = PhysicsWorld::new().with_time_scale(2.0);

        let steps = physics.advance(1.0 / 120.0); // 8.33ms * 2 = 16.67ms
        assert_eq!(steps, 1);
    }

    #[test]
    fn test_advance_paused() {
        let mut physics = PhysicsWorld::new();
        physics.pause();

        let steps = physics.advance(1.0);
        assert_eq!(steps, 0);
        assert!(!physics.should_step());
    }

    #[test]
    fn test_step() {
        let mut physics = PhysicsWorld::new();
        physics.advance(0.1);

        let initial_steps = physics.step_count();
        physics.step();
        assert_eq!(physics.step_count(), initial_steps + 1);
        assert!(physics.simulation_time() > 0.0);
    }

    #[test]
    fn test_step_loop() {
        let mut physics = PhysicsWorld::new();
        physics.advance(0.1); // 100ms

        let mut step_count = 0;
        while physics.should_step() {
            physics.step();
            step_count += 1;

            // Safety: prevent infinite loop
            if step_count > 100 {
                break;
            }
        }

        assert!(step_count >= 6); // ~6 steps at 60 Hz
        assert!(!physics.should_step());
    }

    #[test]
    fn test_interpolation_alpha() {
        let mut physics = PhysicsWorld::new();

        // Half a step
        physics.advance(1.0 / 120.0);
        let alpha = physics.interpolation_alpha();
        assert!(alpha > 0.4 && alpha < 0.6);

        // Full step
        physics.advance(1.0 / 120.0);
        let alpha = physics.interpolation_alpha();
        assert!(alpha > 0.9);
    }

    #[test]
    fn test_reset() {
        let mut physics = PhysicsWorld::new();
        physics.advance(1.0);
        while physics.should_step() {
            physics.step();
        }

        assert!(physics.step_count() > 0);
        assert!(physics.simulation_time() > 0.0);

        physics.reset();
        assert_eq!(physics.step_count(), 0);
        assert_eq!(physics.simulation_time(), 0.0);
        assert_eq!(physics.time_accumulator(), 0.0);
    }

    // =========================================================================
    // Utility Tests
    // =========================================================================

    #[test]
    fn test_frequency() {
        let physics = PhysicsWorld::new();
        let freq = physics.frequency();
        assert!((freq - 60.0).abs() < 0.01);

        let physics = PhysicsWorld::new().with_timestep(1.0 / 120.0);
        let freq = physics.frequency();
        assert!((freq - 120.0).abs() < 0.01);
    }

    #[test]
    fn test_timestep_duration() {
        let physics = PhysicsWorld::new();
        let duration = physics.timestep_duration();
        assert_eq!(duration.as_millis(), 16); // ~16.67ms
    }

    #[test]
    fn test_stats() {
        let mut physics = PhysicsWorld::new();
        physics.advance(1.0);
        while physics.should_step() {
            physics.step();
        }

        let stats = physics.stats();
        assert!(stats.contains("PhysicsWorld Stats"));
        assert!(stats.contains("Timestep"));
        assert!(stats.contains("Steps"));
        assert!(stats.contains("Gravity"));
    }

    // =========================================================================
    // Clone and Debug Tests
    // =========================================================================

    #[test]
    fn test_clone() {
        let physics1 = PhysicsWorld::new()
            .with_gravity(Vec2::new(0.0, -490.0))
            .with_timestep(1.0 / 120.0);

        let physics2 = physics1.clone();
        assert_eq!(physics2.gravity(), physics1.gravity());
        assert_eq!(physics2.timestep(), physics1.timestep());
    }

    #[test]
    fn test_debug() {
        let physics = PhysicsWorld::new();
        let debug_str = format!("{:?}", physics);
        assert!(debug_str.contains("PhysicsWorld"));
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    #[test]
    fn test_is_send() {
        fn requires_send<T: Send>() {}
        requires_send::<PhysicsWorld>();
    }

    #[test]
    fn test_is_sync() {
        fn requires_sync<T: Sync>() {}
        requires_sync::<PhysicsWorld>();
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    fn test_fixed_timestep_pattern() {
        let mut physics = PhysicsWorld::new();

        // Simulate variable frame rates
        let frame_times = vec![0.016, 0.020, 0.012, 0.033];
        let mut total_steps = 0;

        for dt in frame_times {
            physics.advance(dt);

            while physics.should_step() {
                physics.step();
                total_steps += 1;
            }
        }

        assert_eq!(physics.step_count(), total_steps as u64);
    }

    #[test]
    fn test_slow_motion() {
        let mut physics = PhysicsWorld::new().with_time_scale(0.5);

        // 100ms real time = 50ms simulated time at 0.5x
        physics.advance(0.1);

        // At 60 Hz with 0.5x time scale:
        // 100ms * 0.5 = 50ms simulated time
        // 50ms / 16.67ms per step = ~3 steps
        let mut actual_steps = 0;

        while physics.should_step() {
            physics.step();
            actual_steps += 1;
        }

        // Should be around 3 steps (allow for floating point precision)
        assert!(actual_steps >= 2 && actual_steps <= 4);
    }
}
