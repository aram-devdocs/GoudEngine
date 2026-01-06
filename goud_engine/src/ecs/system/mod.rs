//! System module for the ECS.
//!
//! Systems are functions or structs that operate on components and resources.
//! They form the behavior layer of the Entity-Component-System pattern,
//! processing entities based on their component composition.
//!
//! # Architecture
//!
//! The system module is built around several key concepts:
//!
//! - **System**: Trait for any type that can run on a [`World`](crate::ecs::World)
//! - **IntoSystem**: Trait for converting functions into systems
//! - **SystemMeta**: Metadata about a system (name, access patterns)
//! - **SystemParam**: Trait for types that can be extracted from World as function parameters
//! - **FunctionSystem**: Wrapper that converts functions with `SystemParam` parameters into systems
//!
//! # Design Philosophy
//!
//! GoudEngine's system design follows these principles:
//!
//! 1. **Function-First**: Regular Rust functions are the primary way to define systems
//! 2. **Type Safety**: System parameters are checked at compile time
//! 3. **Parallel-Ready**: Access patterns are tracked for safe parallel execution
//! 4. **Flexible**: Both functions and structs can be systems
//!
//! # Basic Usage
//!
//! ## Struct Systems
//!
//! ```
//! use goud_engine::ecs::{World, Component};
//! use goud_engine::ecs::system::{System, BoxedSystem};
//!
//! // Define a component
//! #[derive(Debug, Clone, Copy)]
//! struct Position { x: f32, y: f32 }
//! impl Component for Position {}
//!
//! // A simple system that operates on the world
//! struct PrintPositionCount;
//!
//! impl System for PrintPositionCount {
//!     fn name(&self) -> &'static str {
//!         "PrintPositionCount"
//!     }
//!
//!     fn run(&mut self, world: &mut World) {
//!         // Systems can access world data
//!         println!("Entities: {}", world.entity_count());
//!     }
//! }
//!
//! // Create and run the system
//! let mut world = World::new();
//! let mut system = PrintPositionCount;
//! system.run(&mut world);
//! ```
//!
//! ## Function Systems
//!
//! Functions with valid system parameters are automatically convertible to systems:
//!
//! ```
//! use goud_engine::ecs::{World, Component};
//! use goud_engine::ecs::query::Query;
//! use goud_engine::ecs::system::{IntoSystem, BoxedSystem};
//!
//! #[derive(Debug, Clone, Copy)]
//! struct Position { x: f32, y: f32 }
//! impl Component for Position {}
//!
//! // Define a system as a regular function
//! fn print_positions(query: Query<&Position>) {
//!     // System logic here
//! }
//!
//! // Convert to BoxedSystem
//! let mut boxed: BoxedSystem = print_positions.into_system();
//!
//! // Run it
//! let mut world = World::new();
//! boxed.run(&mut world);
//! ```
//!
//! # System Types
//!
//! ## Struct Systems
//!
//! Implement the [`System`] trait directly for custom behavior:
//!
//! ```
//! use goud_engine::ecs::{World};
//! use goud_engine::ecs::system::System;
//!
//! struct MySystem {
//!     counter: u32,
//! }
//!
//! impl System for MySystem {
//!     fn name(&self) -> &'static str { "MySystem" }
//!
//!     fn run(&mut self, world: &mut World) {
//!         self.counter += 1;
//!     }
//! }
//! ```
//!
//! ## Function Systems
//!
//! Functions with valid system parameters are automatically convertible via [`IntoSystem`]:
//!
//! ```
//! use goud_engine::ecs::{World, Component};
//! use goud_engine::ecs::query::Query;
//! use goud_engine::ecs::resource::ResMut;
//! use goud_engine::ecs::system::IntoSystem;
//!
//! #[derive(Debug, Clone, Copy)]
//! struct Position { x: f32, y: f32 }
//! impl Component for Position {}
//!
//! #[derive(Debug, Clone, Copy)]
//! struct Velocity { x: f32, y: f32 }
//! impl Component for Velocity {}
//!
//! struct Time { delta: f32 }
//!
//! fn movement_system(
//!     mut positions: Query<&mut Position>,
//!     velocities: Query<&Velocity>,
//! ) {
//!     // Movement logic would go here
//! }
//!
//! // Convert to a BoxedSystem
//! let system = movement_system.into_system();
//! ```
//!
//! # System Parameters
//!
//! System parameters are types that can be extracted from the World when a
//! function system runs. They implement the [`SystemParam`] trait.
//!
//! Built-in parameters:
//! - `Query<Q, F>` - Queries for entities with specific components
//! - `Res<T>` - Immutable resource access
//! - `ResMut<T>` - Mutable resource access
//! - Tuples of system parameters (up to 16 elements)
//!
//! # Access Tracking
//!
//! Systems track their component and resource access for:
//!
//! 1. **Conflict Detection**: Prevents parallel execution of conflicting systems
//! 2. **Ordering Validation**: Ensures explicit ordering for dependent systems
//! 3. **Debug Information**: Helps developers understand system behavior
//!
//! # Thread Safety
//!
//! Systems must be `Send` to support parallel scheduling (future feature).
//! The [`System`] trait has a `Send` bound for this purpose.

mod function_system;
mod system_param;
mod system_trait;

// Re-export function system types
pub use function_system::{FunctionSystem, SystemParamFunction};

// Re-export system parameter types
pub use system_param::{
    ParamSet, ReadOnlySystemParam, ResMutState, ResState, StaticSystemParam,
    StaticSystemParamState, SystemParam, SystemParamState,
};

// Re-export core system types
pub use system_trait::{BoxedSystem, IntoSystem, System, SystemId, SystemMeta};
