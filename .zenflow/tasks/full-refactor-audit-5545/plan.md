# GoudEngine Full Refactor - Implementation Plan

## Document Information
- **Version:** 1.0
- **Date:** 2026-01-04
- **Based on:** spec.md v2.0, requirements.md v1.0
- **Status:** Ready for Implementation

---

## Implementation Overview

This plan breaks down the GoudEngine refactor into **6 phases** with **atomic, session-scoped steps**. Each step is designed to be:
- Completable in a single focused session (2-4 hours)
- Independently testable
- Non-breaking to existing functionality until integration
- Fully documented with verification criteria

**Total Estimated Steps:** 54 atomic tasks across 6 phases

---

## Phase 1: Foundation - Core Infrastructure (Steps 1.1 - 1.12)

**Goal:** Establish foundational patterns without breaking existing code. Build the new infrastructure alongside existing code.

### [ ] Step 1.1: Error System Foundation
**Files:** `goud_engine/src/core/error.rs` (new)
**Tasks:**
1. Create `src/core/` directory structure
2. Define `GoudError` enum with categories:
   - Context errors (1-99)
   - Resource errors (100-199)
   - Graphics errors (200-299)
   - Entity errors (300-399)
   - Input errors (400-499)
   - System errors (500-599)
   - Internal errors (900-999)
3. Implement `std::error::Error` and `Display` traits
4. Create FFI-safe error code representation (`GoudResult`)
5. Write 10+ unit tests for error conversion and display

**Verification:**
```bash
cargo test core::error
```

---

### [ ] Step 1.2: Handle System Foundation
**Files:** `goud_engine/src/core/handle.rs` (new)
**Tasks:**
1. Define `Handle<T>` generic type with:
   - 32-bit index + 32-bit generation
   - Type marker (PhantomData)
2. Create `HandleAllocator<T>` for generation tracking
3. Implement `From`, `Into`, `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`
4. Add `is_valid()` check with allocator
5. Write 15+ unit tests for allocation, deallocation, generation wraparound

**Verification:**
```bash
cargo test core::handle
```

---

### [ ] Step 1.3: Event System Foundation
**Files:** `goud_engine/src/core/event.rs` (new)
**Tasks:**
1. Define `Event` trait marker
2. Create `EventQueue<E: Event>` with:
   - `push(event)`, `drain()`, `clear()`, `is_empty()`
3. Create `Events<E>` resource wrapper for system access
4. Create `EventWriter<E>` and `EventReader<E>` for controlled access
5. Add double-buffering for frame-boundary event handling
6. Write 10+ unit tests for event lifecycle

**Verification:**
```bash
cargo test core::event
```

---

### [ ] Step 1.4: Core Module Organization
**Files:** `goud_engine/src/core/mod.rs` (new), update `lib.rs`
**Tasks:**
1. Create `src/core/mod.rs` exporting:
   - `error`, `handle`, `event`
2. Add `context.rs` stub (empty for now)
3. Update `lib.rs` to include `mod core` (but don't use yet)
4. Add `prelude.rs` stub at root
5. Ensure existing code still compiles and tests pass

**Verification:**
```bash
cargo build && cargo test
```

---

### [ ] Step 1.5: ECS Entity Implementation
**Files:** `goud_engine/src/ecs/entity.rs` (new)
**Tasks:**
1. Define `Entity` struct (64-bit: 32-bit index + 32-bit generation)
2. Create `EntityAllocator` with free-list recycling
3. Implement methods:
   - `allocate() -> Entity`
   - `deallocate(Entity) -> bool`
   - `is_alive(Entity) -> bool`
4. Add `Entity::PLACEHOLDER` constant
5. Write 15+ unit tests including stress test with 100K allocations

**Verification:**
```bash
cargo test ecs::entity
```

---

### [ ] Step 1.6: ECS Component Trait and Storage Interface
**Files:** `goud_engine/src/ecs/component.rs` (new)
**Tasks:**
1. Define `Component` trait marker:
   ```rust
   pub trait Component: 'static + Send + Sync {}
   ```
2. Define `ComponentStorage` trait with:
   - `insert(&mut self, entity: Entity, component: T)`
   - `remove(&mut self, entity: Entity) -> Option<T>`
   - `get(&self, entity: Entity) -> Option<&T>`
   - `get_mut(&mut self, entity: Entity) -> Option<&mut T>`
   - `contains(&self, entity: Entity) -> bool`
3. Create `SparseSet<T>` implementation of `ComponentStorage`
4. Write 20+ unit tests for insert, remove, iteration

**Verification:**
```bash
cargo test ecs::component
```

---

### [ ] Step 1.7: ECS Archetype Foundation
**Files:** `goud_engine/src/ecs/archetype.rs` (new)
**Tasks:**
1. Define `ComponentId` (TypeId wrapper with ordering)
2. Define `ArchetypeId` (unique identifier)
3. Define `Archetype` struct containing:
   - Set of `ComponentId`s
   - Entity list for this archetype
   - Dense storage pointers per component type
4. Create `ArchetypeGraph` for archetype relationships:
   - `add_component(archetype, component_type) -> new_archetype`
   - `remove_component(archetype, component_type) -> new_archetype`
5. Write 10+ unit tests for archetype creation and transitions

**Verification:**
```bash
cargo test ecs::archetype
```

---

### [ ] Step 1.8: ECS Query System Foundation
**Files:** `goud_engine/src/ecs/query.rs` (new)
**Tasks:**
1. Define `WorldQuery` trait for query types
2. Define `QueryState<Q: WorldQuery>` caching matched archetypes
3. Implement query parameter types:
   - `&T` (read-only component)
   - `&mut T` (mutable component)
   - `Option<&T>` (optional component)
4. Add `With<T>` and `Without<T>` filters
5. Write 15+ unit tests for query iteration

**Verification:**
```bash
cargo test ecs::query
```

---

### [ ] Step 1.9: ECS Resource System
**Files:** `goud_engine/src/ecs/resource.rs` (new)
**Tasks:**
1. Define `Resource` trait marker:
   ```rust
   pub trait Resource: 'static + Send + Sync {}
   ```
2. Create `Resources` container:
   - `insert<R: Resource>(resource: R)`
   - `get<R: Resource>() -> Option<&R>`
   - `get_mut<R: Resource>() -> Option<&mut R>`
   - `remove<R: Resource>() -> Option<R>`
3. Use `TypeId`-keyed `HashMap` with type erasure
4. Implement `Res<R>` and `ResMut<R>` system parameters
5. Write 10+ unit tests

**Verification:**
```bash
cargo test ecs::resource
```

---

### [ ] Step 1.10: ECS World Container
**Files:** `goud_engine/src/ecs/world.rs` (new)
**Tasks:**
1. Define `World` struct combining:
   - `EntityAllocator`
   - `ArchetypeGraph`
   - `Resources`
   - Component storages
2. Implement core methods:
   - `spawn() -> EntityBuilder`
   - `despawn(Entity)`
   - `get::<T>(Entity) -> Option<&T>`
   - `get_mut::<T>(Entity) -> Option<&mut T>`
   - `insert_component::<T>(Entity, T)`
   - `remove_component::<T>(Entity) -> Option<T>`
3. Write 20+ unit tests including multi-component entities

**Verification:**
```bash
cargo test ecs::world
```

---

### [ ] Step 1.11: ECS System Traits
**Files:** `goud_engine/src/ecs/system.rs` (new)
**Tasks:**
1. Define `System` trait:
   ```rust
   pub trait System: Send + Sync {
       fn run(&mut self, world: &mut World);
       fn component_access(&self) -> Access;
   }
   ```
2. Define `Access` struct tracking read/write component sets
3. Create `IntoSystem` trait for function conversion
4. Implement `FunctionSystem<F>` wrapper for closures
5. Write 10+ unit tests for system execution

**Verification:**
```bash
cargo test ecs::system
```

---

### [ ] Step 1.12: ECS Commands (Deferred Operations)
**Files:** `goud_engine/src/ecs/commands.rs` (new)
**Tasks:**
1. Define `Command` trait for deferred operations
2. Create `Commands` buffer:
   - `spawn() -> EntityCommands`
   - `despawn(Entity)`
   - `insert_component::<T>(Entity, T)`
   - `remove_component::<T>(Entity)`
3. Create `EntityCommands` builder for spawn operations
4. Implement command queue flushing to World
5. Write 15+ unit tests

**Verification:**
```bash
cargo test ecs::commands
```

---

## Phase 2: Core Systems Integration (Steps 2.1 - 2.10)

**Goal:** Build system scheduler, integrate built-in components, and establish transform hierarchy.

### [ ] Step 2.1: System Scheduler - Stage Definition
**Files:** `goud_engine/src/ecs/schedule.rs` (new)
**Tasks:**
1. Define `Stage` enum:
   - `PreUpdate`, `Update`, `PostUpdate`
   - `PreRender`, `Render`, `PostRender`
2. Create `SystemStage` container holding systems for a stage
3. Add `add_system()` and `run()` methods
4. Write 5+ unit tests for stage execution order

**Verification:**
```bash
cargo test ecs::schedule::stage
```

---

### [ ] Step 2.2: System Scheduler - Dependency Analysis
**Files:** `goud_engine/src/ecs/schedule.rs` (update)
**Tasks:**
1. Implement `Schedule` combining all stages
2. Add dependency analysis based on `Access`:
   - Read-only systems can parallelize
   - Write conflicts require ordering
3. Create topological sort for system ordering
4. Add explicit ordering: `before()`, `after()`, `chain()`
5. Write 10+ unit tests for conflict detection and ordering

**Verification:**
```bash
cargo test ecs::schedule::dependency
```

---

### [ ] Step 2.3: System Scheduler - Parallel Execution
**Files:** `goud_engine/src/ecs/schedule.rs` (update)
**Tasks:**
1. Identify parallel groups (non-conflicting systems)
2. Integrate `rayon` for parallel iteration
3. Add feature flag `parallel` (default on)
4. Implement `run_parallel()` for parallel groups
5. Write 5+ unit tests verifying parallel execution
6. Add benchmark comparing sequential vs parallel

**Verification:**
```bash
cargo test ecs::schedule::parallel
cargo bench schedule_parallel
```

---

### [ ] Step 2.4: Built-in Components - Transform
**Files:** `goud_engine/src/components/transform.rs` (new)
**Tasks:**
1. Define `Transform` component:
   - `position: Vec3`, `rotation: Quat`, `scale: Vec3`
2. Define `Transform2D` component:
   - `position: Vec2`, `rotation: f32`, `scale: Vec2`, `z_layer: i32`
3. Define `GlobalTransform` (computed):
   - `matrix: Mat4`
4. Implement `Component` trait for all
5. Add builder methods and defaults
6. Write 15+ unit tests for transformations

**Verification:**
```bash
cargo test components::transform
```

---

### [ ] Step 2.5: Built-in Components - Hierarchy
**Files:** `goud_engine/src/components/hierarchy.rs` (new)
**Tasks:**
1. Define `Parent` component: `entity: Entity`
2. Define `Children` component: `entities: SmallVec<[Entity; 8]>`
3. Define `Name` component: `name: String`
4. Add helper methods for hierarchy traversal
5. Write 10+ unit tests for parent-child relationships

**Verification:**
```bash
cargo test components::hierarchy
```

---

### [ ] Step 2.6: Built-in Components - Rendering
**Files:** `goud_engine/src/components/rendering.rs` (new)
**Tasks:**
1. Define `Sprite` component:
   - `texture: Handle<Texture>`, `source_rect`, `color`, `flip`
2. Define `Mesh` component:
   - `mesh: Handle<Mesh>`, `material: Handle<Material>`
3. Define `Camera` component:
   - `projection`, `viewport`, `order`
4. Define `Light` component:
   - `kind`, `color`, `intensity`, `shadows`
5. Write 10+ unit tests

**Verification:**
```bash
cargo test components::rendering
```

---

### [ ] Step 2.7: Built-in Components - Physics
**Files:** `goud_engine/src/components/physics.rs` (new)
**Tasks:**
1. Define `RigidBody` component:
   - `body_type`, `mass`, `damping`
2. Define `Collider` component:
   - `shape`, `is_sensor`, `friction`, `restitution`
3. Define `Velocity` component:
   - `linear: Vec3`, `angular: Vec3`
4. Define `ColliderShape` enum (Box, Circle, Capsule, etc.)
5. Write 10+ unit tests

**Verification:**
```bash
cargo test components::physics
```

---

### [ ] Step 2.8: Built-in Components - Audio
**Files:** `goud_engine/src/components/audio.rs` (new)
**Tasks:**
1. Define `AudioSource` component:
   - `audio: Handle<AudioClip>`, `volume`, `pitch`, `looping`, `spatial`
2. Define `AudioListener` marker component
3. Define `AudioState` enum (Playing, Paused, Stopped)
4. Write 5+ unit tests

**Verification:**
```bash
cargo test components::audio
```

---

### [ ] Step 2.9: Transform Propagation System
**Files:** `goud_engine/src/systems/transform.rs` (new)
**Tasks:**
1. Implement `transform_propagation_system`:
   - Query entities with `Transform` and `Parent`
   - Compute `GlobalTransform` from hierarchy
2. Add dirty flag optimization (optional for v1)
3. Register in `PostUpdate` stage
4. Write 10+ unit tests with nested hierarchies

**Verification:**
```bash
cargo test systems::transform
```

---

### [ ] Step 2.10: Hierarchy Maintenance System
**Files:** `goud_engine/src/systems/hierarchy.rs` (new)
**Tasks:**
1. Implement `hierarchy_maintenance_system`:
   - Sync `Parent`/`Children` relationships
   - Clean up orphaned children on parent despawn
2. Register in `PostUpdate` stage (before transform propagation)
3. Write 10+ unit tests

**Verification:**
```bash
cargo test systems::hierarchy
```

---

## Phase 3: Resource Management & FFI (Steps 3.1 - 3.8)

**Goal:** Build asset management and modernize FFI layer.

### [ ] Step 3.1: Asset Handle System
**Files:** `goud_engine/src/assets/handle.rs` (new)
**Tasks:**
1. Create `Handle<A>` for assets (distinct from ECS handles)
2. Create `AssetId` with generation counter
3. Implement reference counting via `Arc`
4. Add `HandleState` enum (Loading, Loaded, Failed)
5. Write 10+ unit tests

**Verification:**
```bash
cargo test assets::handle
```

---

### [ ] Step 3.2: Asset Manager Core
**Files:** `goud_engine/src/assets/manager.rs` (new)
**Tasks:**
1. Create `AssetManager` with:
   - `load<A: Asset>(path) -> Handle<A>`
   - `get<A: Asset>(handle) -> Option<&A>`
   - `unload(handle)`
2. Define `Asset` trait
3. Implement asset caching by path
4. Add reference counting cleanup
5. Write 10+ unit tests

**Verification:**
```bash
cargo test assets::manager
```

---

### [ ] Step 3.3: Async Asset Loading
**Files:** `goud_engine/src/assets/loader.rs` (new)
**Tasks:**
1. Create `AssetLoader` trait for type-specific loading
2. Implement async loading pipeline using channels
3. Add loading state tracking
4. Create `AssetServer` coordinating loaders
5. Write 10+ unit tests with mock loaders

**Verification:**
```bash
cargo test assets::loader
```

---

### [ ] Step 3.4: FFI Context and Lifecycle
**Files:** `goud_engine/src/ffi/context.rs` (new)
**Tasks:**
1. Define `GoudContext` opaque handle type
2. Create `goud_context_create()` returning handle
3. Create `goud_context_destroy(handle)`
4. Implement context registry with validation
5. Add version query: `goud_version() -> *const c_char`
6. Write FFI tests

**Verification:**
```bash
cargo test ffi::context
```

---

### [ ] Step 3.5: FFI Error Handling
**Files:** `goud_engine/src/ffi/error.rs` (new)
**Tasks:**
1. Define `GoudResult` as i32 error code
2. Define all error code constants (per spec ranges)
3. Create `goud_error_message(code) -> *const c_char`
4. Implement thread-local last error storage
5. Create `goud_get_last_error() -> GoudResult`
6. Write comprehensive error tests

**Verification:**
```bash
cargo test ffi::error
```

---

### [ ] Step 3.6: FFI Entity Operations
**Files:** `goud_engine/src/ffi/entities.rs` (new)
**Tasks:**
1. Define `GoudEntity` opaque handle type
2. Create entity FFI functions:
   - `goud_entity_spawn(ctx, out_entity) -> GoudResult`
   - `goud_entity_despawn(ctx, entity) -> GoudResult`
   - `goud_entity_is_alive(ctx, entity) -> bool`
3. Write FFI tests

**Verification:**
```bash
cargo test ffi::entities
```

---

### [ ] Step 3.7: FFI Batch Operations
**Files:** `goud_engine/src/ffi/batch.rs` (new)
**Tasks:**
1. Create batch spawn: `goud_entities_spawn_batch(ctx, count, out_entities)`
2. Create batch despawn: `goud_entities_despawn_batch(ctx, entities, count)`
3. Create batch transform update: `goud_transforms_update_batch(ctx, data, count)`
4. Measure and document 10x speedup target
5. Write performance benchmark

**Verification:**
```bash
cargo test ffi::batch
cargo bench ffi_batch
```

---

### [ ] Step 3.8: FFI Module Organization
**Files:** `goud_engine/src/ffi/mod.rs` (new)
**Tasks:**
1. Create `src/ffi/mod.rs` organizing all FFI modules
2. Add utility functions in `ffi/utils.rs`:
   - String conversion helpers
   - Null pointer checks
3. Create `ffi/components.rs` stub for component access
4. Create `ffi/resources.rs` stub for asset access
5. Ensure all FFI functions are `#[no_mangle] pub extern "C"`

**Verification:**
```bash
cargo build --features ffi
```

---

## Phase 4: Graphics Enhancement (Steps 4.1 - 4.8)

**Goal:** Abstract graphics backend, implement batching, enhance materials.

### [ ] Step 4.1: Render Backend Trait
**Files:** `goud_engine/src/graphics/backend/mod.rs` (new)
**Tasks:**
1. Define `RenderBackend` trait:
   - `create_buffer()`, `destroy_buffer()`
   - `create_texture()`, `destroy_texture()`
   - `create_shader()`, `destroy_shader()`
   - `begin_frame()`, `end_frame()`
   - `submit_draw_call()`
2. Define `DrawCall` struct with all draw parameters
3. Define `BufferId`, `TextureId`, `ShaderId` types
4. Write trait documentation

**Verification:**
```bash
cargo doc --open
```

---

### [ ] Step 4.2: OpenGL Backend Implementation
**Files:** `goud_engine/src/graphics/backend/opengl/mod.rs` (new)
**Tasks:**
1. Implement `RenderBackend` for `OpenGLBackend`
2. Migrate existing OpenGL code to new structure:
   - Buffer creation/binding
   - Texture creation/binding
   - Shader compilation
3. Keep existing rendering working
4. Write 10+ integration tests

**Verification:**
```bash
cargo test graphics::backend::opengl
```

---

### [ ] Step 4.3: Sprite Batching System
**Files:** `goud_engine/src/graphics/batch.rs` (new)
**Tasks:**
1. Create `SpriteBatch` struct:
   - Collects sprites per texture
   - Builds vertex buffer for batch
2. Implement batching algorithm:
   - Sort by texture, then z-layer
   - Generate instance data
3. Target: <100 draw calls for 10K same-texture sprites
4. Write benchmark test

**Verification:**
```bash
cargo bench sprite_batch
```

---

### [ ] Step 4.4: Material System Foundation
**Files:** `goud_engine/src/graphics/material.rs` (new)
**Tasks:**
1. Define `Material` struct:
   - `shader: Handle<Shader>`
   - `textures: Vec<Handle<Texture>>`
   - `properties: HashMap<String, MaterialProperty>`
2. Define `MaterialProperty` enum (Float, Vec2, Vec3, Color, etc.)
3. Create `MaterialBuilder` for construction
4. Write 10+ unit tests

**Verification:**
```bash
cargo test graphics::material
```

---

### [ ] Step 4.5: Shader Management Enhancement
**Files:** `goud_engine/src/graphics/shader.rs` (update)
**Tasks:**
1. Add external shader file loading
2. Implement shader hot-reload (dev mode)
3. Add `#include` preprocessor support
4. Create shader reflection for uniform discovery
5. Write 10+ tests

**Verification:**
```bash
cargo test graphics::shader
```

---

### [ ] Step 4.6: Camera System Unification
**Files:** `goud_engine/src/graphics/camera.rs` (new)
**Tasks:**
1. Create unified `CameraController`:
   - Orthographic mode (2D)
   - Perspective mode (3D)
2. Add camera effects:
   - Screen shake
   - Smooth follow
   - Zoom interpolation
3. Support multiple cameras with render order
4. Write 10+ tests

**Verification:**
```bash
cargo test graphics::camera
```

---

### [ ] Step 4.7: Render Systems
**Files:** `goud_engine/src/systems/rendering.rs` (new)
**Tasks:**
1. Implement `sprite_extraction_system` (PreRender)
2. Implement `sprite_render_system` (Render)
3. Implement `camera_update_system` (PreRender)
4. Register all in appropriate stages
5. Write integration tests with mock backend

**Verification:**
```bash
cargo test systems::rendering
```

---

### [ ] Step 4.8: Lighting System Enhancement
**Files:** `goud_engine/src/graphics/light.rs` (update)
**Tasks:**
1. Increase max lights (16 configurable)
2. Add light culling per camera frustum
3. Implement `light_update_system`
4. Add global ambient light resource
5. Write 5+ tests

**Verification:**
```bash
cargo test graphics::light
```

---

## Phase 5: Physics & Audio (Steps 5.1 - 5.7)

**Goal:** Implement custom physics engine and integrate audio.

### [ ] Step 5.1: Physics World Foundation
**Files:** `goud_engine/src/physics/world.rs` (new)
**Tasks:**
1. Create `PhysicsWorld` resource:
   - Gravity setting
   - Body registry
   - Simulation parameters
2. Define `PhysicsConfig` for world settings
3. Implement basic stepping
4. Write 5+ unit tests

**Verification:**
```bash
cargo test physics::world
```

---

### [ ] Step 5.2: Rigid Body Implementation
**Files:** `goud_engine/src/physics/body.rs` (new)
**Tasks:**
1. Create `Body` struct:
   - Position, velocity, acceleration
   - Mass, inverse mass
   - Body type (Dynamic, Static, Kinematic)
2. Implement force application
3. Implement integration (Euler/Verlet)
4. Write 15+ unit tests

**Verification:**
```bash
cargo test physics::body
```

---

### [ ] Step 5.3: Collision Shapes
**Files:** `goud_engine/src/physics/collider.rs` (new)
**Tasks:**
1. Define `Shape` enum:
   - Circle/Sphere
   - Box/Cuboid
   - Capsule
2. Implement AABB for each shape
3. Implement shape-to-AABB conversion
4. Write 10+ unit tests

**Verification:**
```bash
cargo test physics::collider
```

---

### [ ] Step 5.4: Broad Phase Collision
**Files:** `goud_engine/src/physics/broad_phase.rs` (new)
**Tasks:**
1. Implement spatial hash grid
2. Create `BroadPhase` trait
3. Implement `potential_collisions() -> Vec<(Entity, Entity)>`
4. Write 10+ tests with many bodies

**Verification:**
```bash
cargo test physics::broad_phase
```

---

### [ ] Step 5.5: Narrow Phase Collision
**Files:** `goud_engine/src/physics/narrow_phase.rs` (new)
**Tasks:**
1. Implement GJK algorithm for convex shapes
2. Implement circle-circle collision
3. Implement box-box collision (SAT)
4. Generate contact points and normals
5. Write 15+ tests for each collision pair

**Verification:**
```bash
cargo test physics::narrow_phase
```

---

### [ ] Step 5.6: Physics Systems Integration
**Files:** `goud_engine/src/systems/physics.rs` (new)
**Tasks:**
1. Implement `physics_sync_to_world_system` (PreUpdate)
2. Implement `physics_step_system` (Update)
3. Implement `physics_sync_from_world_system` (PostUpdate)
4. Implement `collision_events_system` (PostUpdate)
5. Define collision events
6. Write 10+ integration tests

**Verification:**
```bash
cargo test systems::physics
```

---

### [ ] Step 5.7: Audio System Integration
**Files:** `goud_engine/src/audio/` (new directory)
**Tasks:**
1. Integrate `rodio` for audio playback
2. Create `AudioManager` resource
3. Implement `AudioClip` asset type
4. Implement `audio_playback_system`
5. Implement `spatial_audio_update_system`
6. Write 5+ tests (with mock audio)

**Verification:**
```bash
cargo test audio
```

---

## Phase 6: Developer Experience & Polish (Steps 6.1 - 6.9)

**Goal:** C# SDK update, documentation, examples, and performance optimization.

### [ ] Step 6.1: C# SDK - Core Refactor
**Files:** `sdks/GoudEngine/Core/` (update)
**Tasks:**
1. Update `GoudContext` to use new FFI
2. Create `World` C# wrapper
3. Create `Entity` C# wrapper with component access
4. Update error handling to use new error codes
5. Write 10+ C# unit tests

**Verification:**
```bash
dotnet test sdks/GoudEngine.Tests
```

---

### [ ] Step 6.2: C# SDK - Component Builders
**Files:** `sdks/GoudEngine/Builders/` (new)
**Tasks:**
1. Create `SpriteBuilder` with fluent API
2. Create `LightBuilder` with fluent API
3. Create `BodyBuilder` for physics
4. Create `MaterialBuilder`
5. Write 10+ tests for builders

**Verification:**
```bash
dotnet test sdks/GoudEngine.Tests
```

---

### [ ] Step 6.3: C# SDK - Event System
**Files:** `sdks/GoudEngine/Events/` (new)
**Tasks:**
1. Create `EventBus` for C# event subscription
2. Create `CollisionEventArgs`
3. Implement FFI callbacks for events
4. Write 5+ event tests

**Verification:**
```bash
dotnet test sdks/GoudEngine.Tests
```

---

### [ ] Step 6.4: Input System Enhancement
**Files:** `goud_engine/src/input/` (new)
**Tasks:**
1. Create `InputManager` resource
2. Implement `ActionMap` for action-based input
3. Add input buffering (2-frame)
4. Add gamepad support
5. Create `input_update_system`
6. Write 10+ tests

**Verification:**
```bash
cargo test input
```

---

### [ ] Step 6.5: Integration - Connect New ECS to Existing Renderers
**Files:** Multiple
**Tasks:**
1. Bridge new ECS World to existing Renderer2D/3D
2. Migrate sprite rendering to ECS-based
3. Ensure existing examples still work
4. Write integration tests

**Verification:**
```bash
cargo test --test integration
./dev.sh --game flappy_goud
```

---

### [ ] Step 6.6: Documentation - API Reference
**Files:** All public APIs
**Tasks:**
1. Add comprehensive `///` doc comments to all public types
2. Add `//!` module documentation
3. Add code examples in doc comments
4. Generate and verify rustdoc

**Verification:**
```bash
cargo doc --no-deps --open
```

---

### [ ] Step 6.7: Example Game - Updated Flappy Goud
**Files:** `examples/flappy_goud/`
**Tasks:**
1. Update to use new ECS patterns
2. Use builder APIs
3. Add physics-based collision
4. Add audio effects
5. Document as tutorial

**Verification:**
```bash
./dev.sh --game flappy_goud
```

---

### [ ] Step 6.8: Performance Benchmarks
**Files:** `goud_engine/benches/` (new)
**Tasks:**
1. Create ECS benchmarks (entity spawn, queries)
2. Create rendering benchmarks (batching)
3. Create FFI benchmarks (single vs batch)
4. Set up CI benchmark tracking
5. Document performance targets met

**Verification:**
```bash
cargo bench
```

---

### [ ] Step 6.9: Final Integration & Migration
**Files:** Multiple
**Tasks:**
1. Remove old ECS code (after new ECS proven)
2. Update all FFI to new patterns
3. Update C# SDK completely
4. Verify all examples work
5. Create migration guide document
6. Tag v0.1.0 release

**Verification:**
```bash
cargo test
dotnet test
./dev.sh --game flappy_goud
./dev.sh --game 3d_cube
./dev.sh --game goud_jumper
```

---

## Verification Checklist

### Per-Step Requirements
Each step MUST:
- [ ] Have all tests passing
- [ ] Have no new clippy warnings
- [ ] Maintain existing functionality (no regressions)
- [ ] Include documentation for public APIs
- [ ] Be completable in isolation

### Phase Gate Criteria

**Phase 1 Complete When:**
- [ ] 80%+ test coverage on new `core/` and `ecs/` modules
- [ ] All 12 steps verified
- [ ] Existing code still compiles

**Phase 2 Complete When:**
- [ ] System scheduler working with parallel execution
- [ ] All built-in components implemented
- [ ] Transform hierarchy propagation working
- [ ] All 10 steps verified

**Phase 3 Complete When:**
- [ ] Asset loading working
- [ ] New FFI layer operational
- [ ] Batch operations showing 10x improvement
- [ ] All 8 steps verified

**Phase 4 Complete When:**
- [ ] Render backend abstraction complete
- [ ] Sprite batching achieving <100 draw calls for 10K sprites
- [ ] All 8 steps verified

**Phase 5 Complete When:**
- [ ] Physics simulation working
- [ ] Audio playback working
- [ ] Collision events firing
- [ ] All 7 steps verified

**Phase 6 Complete When:**
- [ ] C# SDK fully updated
- [ ] All examples working
- [ ] Documentation complete
- [ ] All 9 steps verified
- [ ] Performance targets met

---

## Workflow Steps

### [x] Step: Requirements
<!-- chat-id: 46b49ac7-ac11-4828-845e-e7025096003c -->

Create a Product Requirements Document (PRD) based on the feature description.

1. Review existing codebase to understand current architecture and patterns
2. Analyze the feature definition and identify unclear aspects
3. Ask the user for clarifications on aspects that significantly impact scope or user experience
4. Make reasonable decisions for minor details based on context and conventions
5. If user can't clarify, make a decision, state the assumption, and continue

Save the PRD to `{@artifacts_path}/requirements.md`.

### [x] Step: Technical Specification
<!-- chat-id: 0293f184-5dac-4d07-b598-bbfc3219411a -->

Create a technical specification based on the PRD in `{@artifacts_path}/requirements.md`.

1. Review existing codebase architecture and identify reusable components
2. Define the implementation approach

Save to `{@artifacts_path}/spec.md` with:
- Technical context (language, dependencies)
- Implementation approach referencing existing code patterns
- Source code structure changes
- Data model / API / interface changes
- Delivery phases (incremental, testable milestones)
- Verification approach using project lint/test commands

### [x] Step: Audit Technical Specification
<!-- chat-id: a37c0601-90d0-4e53-b4b8-b19eecd4d6f1 -->

we want to use bevy, we want to build our own ecs. you can reverse engineer from bevy but it jhas to be our code and part of our codebase. update the spec codument to account for that. also be verbose, we want this to be like monogame but with the tools youd fine ind a modenr engin. make the scripting easy because the open source code of the game engin e is strong.
 `{@artifacts_path}/spec.md` is the file to audit and update. ensure high level system architere, leave actual code examples and scripts for actual docs.

**Completed Changes (v2.0):**
- Replaced bevy_ecs dependency with custom Bevy-inspired ECS architecture (fully owned code)
- Added comprehensive ECS documentation: entities, components, systems, archetypes, queries, scheduling
- Added high-level system architecture diagrams (6 layers from Platform to Language Bindings)
- Added data flow architecture showing game loop, events, world state
- Documented MonoGame-inspired developer experience features (SpriteBatch, Content Pipeline, Game class)
- Defined multi-language binding strategy (C#, Python, Lua, TypeScript, Go, Rust)
- Removed low-level code examples, focused on architectural concepts and design decisions
- Added custom physics engine (Rapier-inspired, no external dependency)
- Added detailed component categories and built-in systems tables
- Expanded FFI architecture with handle system and batch operations
- Added success metrics and risk mitigation sections
- Added glossary and target file structure

### [x] Step: Planning
<!-- chat-id: ea939988-5414-4af9-98e4-af4bd61add5d -->

Create a detailed implementation plan based on `{@artifacts_path}/spec.md`.

1. Break down the work into concrete tasks
2. Each task should reference relevant contracts and include verification steps
3. Replace the Implementation step below with the planned tasks

**Completed:** 54 atomic implementation steps organized into 6 phases:
- Phase 1: Foundation (12 steps) - ECS core, error handling, handles, events
- Phase 2: Core Systems (10 steps) - Scheduler, built-in components, hierarchy
- Phase 3: Resource Management & FFI (8 steps) - Assets, new FFI layer
- Phase 4: Graphics Enhancement (8 steps) - Backend abstraction, batching
- Phase 5: Physics & Audio (7 steps) - Custom physics, audio integration
- Phase 6: Developer Experience (9 steps) - C# SDK, docs, examples

### [ ] Step: Implementation - Phase 1
<!-- Each sub-step to be completed individually -->

See Phase 1 steps above (1.1 - 1.12). Start with Step 1.1: Error System Foundation.

### [ ] Step: Implementation - Phase 2

See Phase 2 steps above (2.1 - 2.10).

### [ ] Step: Implementation - Phase 3

See Phase 3 steps above (3.1 - 3.8).

### [ ] Step: Implementation - Phase 4

See Phase 4 steps above (4.1 - 4.8).

### [ ] Step: Implementation - Phase 5

See Phase 5 steps above (5.1 - 5.7).

### [ ] Step: Implementation - Phase 6

See Phase 6 steps above (6.1 - 6.9).

---

*Plan Version: 1.0*
*Created: 2026-01-04*
*Total Steps: 54 atomic tasks*
*Estimated Implementation: Large (per spec.md Phase estimates)*
