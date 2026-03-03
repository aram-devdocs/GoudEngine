//! The Stage trait for system execution containers.

use crate::ecs::World;

/// Trait for stage implementations that can contain and run systems.
///
/// A stage is a container for systems that run together in a specific order.
/// Different stage implementations can provide different execution strategies
/// (sequential, parallel, etc.).
///
/// # Implementation
///
/// Stages must be `Send + Sync` for thread safety when stored in schedules.
pub trait Stage: Send + Sync {
    /// Returns the name of this stage.
    fn name(&self) -> &str;

    /// Runs all systems in this stage on the given world.
    fn run(&mut self, world: &mut World);

    /// Initializes the stage and all its systems.
    ///
    /// Called once when the stage is first added to a schedule.
    fn initialize(&mut self, _world: &mut World) {
        // Default: no initialization needed
    }

    /// Returns the number of systems in this stage.
    fn system_count(&self) -> usize;

    /// Returns true if this stage has no systems.
    fn is_empty(&self) -> bool {
        self.system_count() == 0
    }
}
