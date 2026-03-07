//! Application framework for organizing ECS systems and plugins.
//!
//! The [`App`] struct is the top-level container that holds a [`World`], manages
//! system stages, and supports plugin-based composition.
//!
//! # Example
//!
//! ```rust
//! use goud_engine::ecs::app::App;
//! use goud_engine::ecs::schedule::CoreStage;
//! use goud_engine::ecs::system::System;
//! use goud_engine::ecs::World;
//!
//! struct MySystem;
//! impl System for MySystem {
//!     fn name(&self) -> &'static str { "MySystem" }
//!     fn run(&mut self, _world: &mut World) {}
//! }
//!
//! let mut app = App::new();
//! app.add_system(CoreStage::Update, MySystem);
//! app.run_once();
//! ```

pub mod builtin_plugins;
pub mod physics_plugins;
pub mod plugin;

pub use builtin_plugins::{DefaultPlugins, TransformPropagationPlugin};
pub use physics_plugins::{PhysicsPlugin2D, PhysicsPlugin3D};
pub use plugin::{Plugin, PluginGroup};

use std::any::TypeId;
use std::collections::HashSet;

use crate::ecs::resource::{NonSendResource, Resource};
use crate::ecs::schedule::{CoreStage, Stage, SystemSetConfig, SystemStage};
use crate::ecs::system::IntoSystem;
use crate::ecs::World;

/// The main application container for ECS-based games.
///
/// `App` holds a [`World`] and a set of [`SystemStage`]s organized by
/// [`CoreStage`]. Systems are added to stages and executed in stage order.
///
/// # Stage Execution Order
///
/// Stages run in [`CoreStage`] order: PreUpdate, Update, PostUpdate,
/// PreRender, Render, PostRender.
pub struct App {
    /// The ECS world containing all entities, components, and resources.
    world: World,
    /// Stages indexed by CoreStage variant (in execution order).
    stages: Vec<(CoreStage, SystemStage)>,
    /// Tracks which plugins have been initialized (by TypeId).
    initialized_plugins: HashSet<TypeId>,
}

impl App {
    /// Creates a new App with default stages and all [`DefaultPlugins`] applied.
    pub fn new_with_defaults() -> Self {
        let mut app = Self::new();
        app.add_plugin_group(DefaultPlugins);
        app
    }

    /// Creates a new App with default stages for each [`CoreStage`] variant.
    pub fn new() -> Self {
        let stages = CoreStage::all()
            .iter()
            .map(|&stage| (stage, SystemStage::from_core(stage)))
            .collect();

        Self {
            world: World::new(),
            stages,
            initialized_plugins: HashSet::new(),
        }
    }

    /// Adds a plugin to the app.
    ///
    /// Each plugin type can only be added once. Duplicate additions are
    /// silently ignored.
    ///
    /// # Panics
    ///
    /// Panics if the plugin declares dependencies that have not been added.
    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        let plugin_type_id = TypeId::of::<P>();

        if self.initialized_plugins.contains(&plugin_type_id) {
            log::warn!(
                "Plugin '{}' already added, skipping duplicate",
                plugin.name()
            );
            return self;
        }

        // Check dependencies
        for dep in plugin.dependencies() {
            assert!(
                self.initialized_plugins.contains(&dep),
                "Plugin '{}' has an unmet dependency (TypeId: {:?}). \
                 Add the dependency plugin first.",
                plugin.name(),
                dep
            );
        }

        plugin.build(self);
        self.initialized_plugins.insert(plugin_type_id);
        self
    }

    /// Adds a plugin group to the app.
    pub fn add_plugin_group<G: PluginGroup>(&mut self, group: G) -> &mut Self {
        group.build(self);
        self
    }

    /// Inserts a non-send resource into the world.
    pub fn insert_non_send_resource<T: NonSendResource>(&mut self, resource: T) -> &mut Self {
        self.world.insert_non_send_resource(resource);
        self
    }

    /// Adds a system to the specified stage.
    pub fn add_system<S, Marker>(&mut self, stage: CoreStage, system: S) -> &mut Self
    where
        S: IntoSystem<Marker>,
    {
        for (core_stage, system_stage) in &mut self.stages {
            if *core_stage == stage {
                system_stage.add_system(system);
                return self;
            }
        }
        self
    }

    /// Inserts a resource into the world.
    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    /// Returns an immutable reference to the world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Returns a mutable reference to the world.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Executes all stages once in order.
    ///
    /// This runs every stage's systems on the world, from PreUpdate
    /// through PostRender.
    pub fn run_once(&mut self) {
        for (_stage_label, stage) in &mut self.stages {
            stage.run(&mut self.world);
        }
    }

    /// Executes all stages once, designed for per-frame use.
    ///
    /// Currently identical to [`run_once`](Self::run_once). In the future,
    /// this may include frame-specific logic such as delta time updates.
    pub fn update(&mut self) {
        self.run_once();
    }

    // =====================================================================
    // Named System Sets API
    // =====================================================================

    /// Registers a named system set in the specified stage.
    pub fn register_set(&mut self, stage: CoreStage, name: &str) -> &mut Self {
        for (core_stage, system_stage) in &mut self.stages {
            if *core_stage == stage {
                system_stage.register_set(name);
                return self;
            }
        }
        self
    }

    /// Adds a system to a named set, returning its [`SystemId`].
    ///
    /// The system is added to the stage and simultaneously placed in the
    /// named set.
    pub fn add_system_to_set<S, Marker>(
        &mut self,
        stage: CoreStage,
        set_name: &str,
        system: S,
    ) -> &mut Self
    where
        S: IntoSystem<Marker>,
    {
        for (core_stage, system_stage) in &mut self.stages {
            if *core_stage == stage {
                assert!(
                    system_stage.get_set(set_name).is_some(),
                    "System set '{set_name}' is not registered in stage {stage:?}"
                );
                let id = system_stage.add_system(system);
                system_stage.add_system_to_set(set_name, id);
                return self;
            }
        }
        self
    }

    /// Configures ordering for a named set in the specified stage.
    pub fn configure_set(
        &mut self,
        stage: CoreStage,
        name: &str,
        config: SystemSetConfig,
    ) -> &mut Self {
        for (core_stage, system_stage) in &mut self.stages {
            if *core_stage == stage {
                system_stage.configure_named_set(name, config);
                return self;
            }
        }
        self
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total_systems: usize = self.stages.iter().map(|(_, s)| s.system_count()).sum();
        f.debug_struct("App")
            .field("stage_count", &self.stages.len())
            .field("total_systems", &total_systems)
            .field("plugin_count", &self.initialized_plugins.len())
            .finish()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests_extended.rs"]
mod tests_extended;
