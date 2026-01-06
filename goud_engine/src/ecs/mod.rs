//! Entity-Component-System (ECS) for GoudEngine.
//!
//! This module provides a complete ECS implementation inspired by Bevy and other
//! modern game engines. The ECS pattern separates data (components) from behavior
//! (systems) using entities as lightweight identifiers.
//!
//! # Architecture Overview
//!
//! - **Entities**: Lightweight identifiers (index + generation)
//! - **Components**: Data attached to entities via sparse set storage
//! - **Sparse Sets**: O(1) component storage with cache-friendly iteration
//! - **Archetypes**: Groups of entities with identical component sets
//! - **World**: Central container managing all ECS state
//! - **Systems**: Functions that operate on components
//!
//! # Example
//!
//! ```
//! use goud_engine::ecs::{Entity, Component, SparseSet};
//!
//! // Define a component
//! struct Position { x: f32, y: f32 }
//! impl Component for Position {}
//!
//! // Create entities and store components
//! let e1 = Entity::new(0, 1);
//! let e2 = Entity::new(1, 1);
//!
//! let mut positions: SparseSet<Position> = SparseSet::new();
//! positions.insert(e1, Position { x: 0.0, y: 0.0 });
//! positions.insert(e2, Position { x: 10.0, y: 20.0 });
//!
//! // Cache-friendly iteration
//! for (entity, pos) in positions.iter() {
//!     println!("{}: ({}, {})", entity, pos.x, pos.y);
//! }
//! ```
//!
//! # Systems
//!
//! Systems define behavior that operates on entities and components:
//!
//! ```
//! use goud_engine::ecs::{World, Component};
//! use goud_engine::ecs::system::{System, BoxedSystem};
//!
//! // Define a component
//! struct Health(f32);
//! impl Component for Health {}
//!
//! // Define a system
//! struct PrintHealthSystem;
//!
//! impl System for PrintHealthSystem {
//!     fn name(&self) -> &'static str { "PrintHealthSystem" }
//!     fn run(&mut self, world: &mut World) {
//!         println!("Entity count: {}", world.entity_count());
//!     }
//! }
//!
//! // Run the system
//! let mut world = World::new();
//! let mut system = PrintHealthSystem;
//! system.run(&mut world);
//! ```

pub mod archetype;
pub mod broad_phase;
pub mod collision;
pub mod component;
pub mod components;
pub mod entity;
pub mod input_manager;
pub mod query;
pub mod resource;
pub mod schedule;
pub mod sparse_set;
pub mod storage;
pub mod system;
pub mod systems;
pub mod world;
pub mod physics_world;

// Re-export commonly used types
pub use archetype::{Archetype, ArchetypeGraph, ArchetypeId};
pub use broad_phase::{SpatialHash, SpatialHashStats};
pub use collision::{
    aabb_aabb_collision, box_box_collision, circle_aabb_collision, circle_circle_collision,
    circle_obb_collision, compute_position_correction, resolve_collision, CollisionEnded,
    CollisionResponse, CollisionStarted, Contact,
};
pub use component::{Component, ComponentId, ComponentInfo};
pub use entity::{Entity, EntityAllocator};
pub use input_manager::{InputBinding, InputManager};
pub use query::{Query, QueryIter, QueryIterMut, QuerySystemParamState};
pub use sparse_set::{SparseSet, SparseSetIter, SparseSetIterMut};
pub use resource::{Res, ResMut, Resource, ResourceId, Resources};
pub use schedule::{CoreStage, Stage, StageLabel, StageLabelId, StageOrder, StagePosition, SystemStage};
pub use storage::{AnyComponentStorage, ComponentStorage};
pub use systems::{SpriteRenderSystem, TransformPropagationSystem};
pub use world::{EntityWorldMut, World};
pub use physics_world::PhysicsWorld;
