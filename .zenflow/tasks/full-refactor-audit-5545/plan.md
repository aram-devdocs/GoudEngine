# GoudEngine Full Refactor - Atomic Implementation Plan

## Document Information
- **Version:** 2.0
- **Date:** 2026-01-04
- **Based on:** spec.md v2.0, requirements.md v1.0
- **Status:** Ready for Implementation

---

## Plan Philosophy

Each step in this plan is:
- **Atomic**: One focused task, completable in 30-90 minutes
- **Isolated**: No dependencies on incomplete steps within the same phase
- **Testable**: Includes specific verification commands
- **Documented**: Clear context, edge cases, and design patterns
- **Non-Breaking**: Maintains backward compatibility until final integration

**Total Steps:** 127 atomic tasks across 6 phases

---

## Workflow Steps (Top-Level Tracking)

### [x] Step: Requirements
<!-- chat-id: 46b49ac7-ac11-4828-845e-e7025096003c -->
Completed. See `requirements.md`.

### [x] Step: Technical Specification
<!-- chat-id: 0293f184-5dac-4d07-b598-bbfc3219411a -->
Completed. See `spec.md`.

### [x] Step: Audit Technical Specification
<!-- chat-id: a37c0601-90d0-4e53-b4b8-b19eecd4d6f1 -->
Completed. Custom ECS, MonoGame-style DX, multi-language FFI.

### [x] Step: Planning
<!-- chat-id: ea939988-5414-4af9-98e4-af4bd61add5d -->
Completed v2.0 - 127 atomic implementation steps.


---

# Phase 1: Foundation Infrastructure

**Goal:** Build core utilities and patterns that underpin the entire engine. No existing code changes.

---

## 1.1 Error Handling System

### [x] Step 1.1.1: Create Core Module Structure
<!-- chat-id: 87eec9d5-ebf2-4554-9f58-665b9b9a3b97 -->
**File:** `goud_engine/src/core/mod.rs` (new)
**Context:** Establish the `core` module as the foundation for all shared engine utilities. This module will contain error handling, handles, events, and other cross-cutting concerns.

**Tasks:**
1. Create directory `goud_engine/src/core/`
2. Create `mod.rs` with empty module declarations
3. Update `goud_engine/src/lib.rs` to include `pub mod core;`
4. Ensure `cargo build` succeeds with no warnings

**Edge Cases:**
- Module visibility must be `pub` for SDK access
- Do not modify any existing code paths

**Design Pattern:** Module-based organization following Rust 2018 idioms

**Verification:**
```bash
cargo build 2>&1 | grep -E "(error|warning)" | head -20
# Expected: No errors, no new warnings
```

---

### [x] Step 1.1.2: Define Error Category Constants
<!-- chat-id: accf2d9d-c680-41a8-9a3b-802440f9a8e3 -->
**File:** `goud_engine/src/core/error.rs` (new)
**Context:** Define FFI-compatible error code ranges. These constants will be used across Rust and all language bindings (C#, Python, etc.) for consistent error handling.

**Tasks:**
1. Create `error.rs` with error code range constants:
   ```
   SUCCESS = 0
   CONTEXT_ERROR_BASE = 1      // 1-99: Context/initialization errors
   RESOURCE_ERROR_BASE = 100   // 100-199: Asset/resource errors
   GRAPHICS_ERROR_BASE = 200   // 200-299: Rendering errors
   ENTITY_ERROR_BASE = 300     // 300-399: ECS entity errors
   INPUT_ERROR_BASE = 400      // 400-499: Input handling errors
   SYSTEM_ERROR_BASE = 500     // 500-599: System/platform errors
   INTERNAL_ERROR_BASE = 900   // 900-999: Internal/unexpected errors
   ```
2. Define `GoudErrorCode` type alias as `i32` (FFI-compatible)
3. Add module to `core/mod.rs`

**Edge Cases:**
- Error codes must be stable across versions (API contract)
- Reserve ranges for future expansion within each category
- Use `i32` not `u32` for C compatibility (negative values reserved)

**Design Pattern:** Error code ranges (similar to HTTP status codes)

**Verification:**
```bash
cargo build && cargo test core::error
```

---

### [x] Step 1.1.3: Implement GoudError Enum - Context Errors
<!-- chat-id: 8f49db8f-b897-42a3-aaf2-e37dd94a0e92 -->
**File:** `goud_engine/src/core/error.rs` (update)
**Context:** Define the first category of errors for engine context and initialization failures.

**Tasks:**
1. Define `GoudError` enum with Context variants:
   - `NotInitialized` (code 1)
   - `AlreadyInitialized` (code 2)
   - `InvalidContext` (code 3)
   - `ContextDestroyed` (code 4)
   - `InitializationFailed(String)` (code 10)
2. Implement `error_code(&self) -> GoudErrorCode` method
3. Write 5 unit tests for error code mapping

**Edge Cases:**
- String messages in errors must not leak across FFI
- Error codes within a category should be logically grouped
- Leave gaps for inserting new errors without renumbering

**Verification:**
```bash
cargo test core::error::context
```

---

### [x] Step 1.1.4: Implement GoudError Enum - Resource Errors
<!-- chat-id: 71737b61-dbd3-4607-aba5-95f3c8e47735 -->
**File:** `goud_engine/src/core/error.rs` (update)
**Context:** Errors related to asset loading, resource management, and file operations.

**Tasks:**
1. Add Resource variants to `GoudError`:
   - `ResourceNotFound(String)` (code 100)
   - `ResourceLoadFailed(String)` (code 101)
   - `ResourceInvalidFormat(String)` (code 102)
   - `ResourceAlreadyExists(String)` (code 103)
   - `InvalidHandle` (code 110)
   - `HandleExpired` (code 111)
   - `HandleTypeMismatch` (code 112)
2. Write 5 unit tests

**Edge Cases:**
- File paths in errors should be sanitized (no absolute paths in logs)
- Handle errors are critical for FFI safety

**Verification:**
```bash
cargo test core::error::resource
```

---

### [ ] Step 1.1.5: Implement GoudError Enum - Graphics Errors
**File:** `goud_engine/src/core/error.rs` (update)
**Context:** Errors from the rendering subsystem, shaders, and GPU operations.

**Tasks:**
1. Add Graphics variants:
   - `ShaderCompilationFailed(String)` (code 200)
   - `ShaderLinkFailed(String)` (code 201)
   - `TextureCreationFailed(String)` (code 210)
   - `BufferCreationFailed(String)` (code 211)
   - `RenderTargetFailed(String)` (code 220)
   - `BackendNotSupported(String)` (code 230)
   - `DrawCallFailed(String)` (code 240)
2. Write 5 unit tests

**Edge Cases:**
- Shader errors should include line numbers when available
- GPU errors may not have detailed messages

**Verification:**
```bash
cargo test core::error::graphics
```

---

### [ ] Step 1.1.6: Implement GoudError Enum - Entity & System Errors
**File:** `goud_engine/src/core/error.rs` (update)
**Context:** ECS-related errors and general system/platform errors.

**Tasks:**
1. Add Entity variants:
   - `EntityNotFound` (code 300)
   - `EntityAlreadyExists` (code 301)
   - `ComponentNotFound` (code 310)
   - `ComponentAlreadyExists` (code 311)
   - `QueryFailed(String)` (code 320)
2. Add System variants:
   - `WindowCreationFailed(String)` (code 500)
   - `AudioInitFailed(String)` (code 510)
   - `PhysicsInitFailed(String)` (code 520)
   - `PlatformError(String)` (code 530)
3. Add Internal variants:
   - `InternalError(String)` (code 900)
   - `NotImplemented(String)` (code 901)
   - `InvalidState(String)` (code 902)
4. Write 8 unit tests

**Verification:**
```bash
cargo test core::error::entity
cargo test core::error::system
```

---

### [ ] Step 1.1.7: Implement Error Traits and Conversions
**File:** `goud_engine/src/core/error.rs` (update)
**Context:** Make `GoudError` a proper Rust error type with standard trait implementations.

**Tasks:**
1. Implement `std::fmt::Display` for `GoudError`
   - Format: `"[GOUD-{code}] {category}: {message}"`
2. Implement `std::fmt::Debug` for `GoudError`
3. Implement `std::error::Error` for `GoudError`
4. Implement `From<std::io::Error>` for `GoudError`
5. Implement `From<String>` for `GoudError`
6. Define `GoudResult<T> = Result<T, GoudError>` type alias
7. Write 10 unit tests for Display formatting and conversions

**Edge Cases:**
- Debug format should include full context
- Display format should be user-friendly
- Conversion from io::Error maps to appropriate category

**Design Pattern:** Standard Rust error handling with thiserror-style ergonomics

**Verification:**
```bash
cargo test core::error::traits
```

---

### [ ] Step 1.1.8: Implement FFI Error Bridge
**File:** `goud_engine/src/core/error.rs` (update)
**Context:** Bridge between Rust errors and FFI-safe error codes. Critical for C# SDK integration.

**Tasks:**
1. Create thread-local storage for last error:
   ```rust
   thread_local! {
       static LAST_ERROR: RefCell<Option<GoudError>> = RefCell::new(None);
   }
   ```
2. Implement `set_last_error(error: GoudError)`
3. Implement `take_last_error() -> Option<GoudError>`
4. Implement `last_error_code() -> GoudErrorCode`
5. Implement `last_error_message() -> Option<String>`
6. Create `GoudFFIResult` struct for C ABI:
   ```rust
   #[repr(C)]
   pub struct GoudFFIResult {
       pub code: GoudErrorCode,
       pub success: bool,
   }
   ```
7. Write 8 unit tests including thread-safety tests

**Edge Cases:**
- Thread-local storage means errors don't cross threads
- Must clear error after retrieval to prevent stale errors
- Error messages must be copied, not referenced (FFI lifetime issues)

**Design Pattern:** Thread-local error storage (similar to `errno`)

**Verification:**
```bash
cargo test core::error::ffi
```

---

## 1.2 Handle System

### [ ] Step 1.2.1: Define Handle Structure
**File:** `goud_engine/src/core/handle.rs` (new)
**Context:** Handles are type-safe, generation-counted references to engine objects. Critical for FFI safety - prevents use-after-free and type confusion.

**Tasks:**
1. Create `handle.rs` with `Handle<T>` struct:
   ```rust
   #[repr(C)]
   pub struct Handle<T> {
       index: u32,
       generation: u32,
       _marker: PhantomData<T>,
   }
   ```
2. Implement `Handle<T>::new(index: u32, generation: u32) -> Self`
3. Implement `Handle<T>::index(&self) -> u32`
4. Implement `Handle<T>::generation(&self) -> u32`
5. Implement `Handle<T>::INVALID` constant (index=u32::MAX, gen=0)
6. Implement `Handle<T>::is_valid(&self) -> bool` (not INVALID)
7. Add module to `core/mod.rs`
8. Write 5 unit tests

**Edge Cases:**
- `PhantomData` ensures type safety at compile time
- INVALID handle must be distinguishable from any valid handle
- `#[repr(C)]` required for FFI compatibility

**Design Pattern:** Generational indices (prevents ABA problem)

**Verification:**
```bash
cargo test core::handle::structure
```

---

### [ ] Step 1.2.2: Implement Handle Traits
**File:** `goud_engine/src/core/handle.rs` (update)
**Context:** Handles need standard trait implementations for use in collections and comparisons.

**Tasks:**
1. Derive/implement for `Handle<T>`:
   - `Clone` (derive)
   - `Copy` (derive)
   - `Debug` (custom: `Handle<TypeName>(index, gen)`)
   - `PartialEq` (compare index AND generation)
   - `Eq` (derive from PartialEq)
   - `Hash` (combine index and generation)
   - `Default` (returns INVALID)
2. Implement `From<Handle<T>>` for `u64` (pack index+gen)
3. Implement `From<u64>` for `Handle<T>` (unpack)
4. Write 10 unit tests for all traits

**Edge Cases:**
- Hash must be consistent with PartialEq
- Packed u64 format: upper 32 bits = generation, lower 32 = index
- Different `T` types must not be comparable

**Verification:**
```bash
cargo test core::handle::traits
```

---

### [ ] Step 1.2.3: Implement HandleAllocator
**File:** `goud_engine/src/core/handle.rs` (update)
**Context:** Manages handle allocation with generation counting and free-list recycling.

**Tasks:**
1. Create `HandleAllocator<T>` struct:
   ```rust
   pub struct HandleAllocator<T> {
       generations: Vec<u32>,
       free_list: Vec<u32>,
       _marker: PhantomData<T>,
   }
   ```
2. Implement `HandleAllocator<T>::new() -> Self`
3. Implement `HandleAllocator<T>::allocate() -> Handle<T>`:
   - Pop from free_list if available
   - Otherwise, push new generation entry
   - Return Handle with current generation
4. Implement `HandleAllocator<T>::deallocate(&mut self, handle: Handle<T>) -> bool`:
   - Validate handle is current generation
   - Increment generation (wraps at u32::MAX)
   - Push index to free_list
   - Return true if successfully deallocated
5. Implement `HandleAllocator<T>::is_alive(&self, handle: Handle<T>) -> bool`:
   - Check generation matches current
6. Write 10 unit tests

**Edge Cases:**
- Generation overflow wraps to 1 (0 reserved for never-allocated)
- Double-deallocation must fail gracefully
- Allocator should not panic on invalid handles

**Design Pattern:** Generational arena allocator

**Verification:**
```bash
cargo test core::handle::allocator
```

---

### [ ] Step 1.2.4: Implement HandleAllocator Capacity Management
**File:** `goud_engine/src/core/handle.rs` (update)
**Context:** Optimize allocator for bulk operations and memory efficiency.

**Tasks:**
1. Implement `HandleAllocator<T>::with_capacity(capacity: usize) -> Self`
2. Implement `HandleAllocator<T>::len(&self) -> usize` (active handles)
3. Implement `HandleAllocator<T>::capacity(&self) -> usize`
4. Implement `HandleAllocator<T>::is_empty(&self) -> bool`
5. Implement `HandleAllocator<T>::clear(&mut self)` (reset all)
6. Implement `HandleAllocator<T>::shrink_to_fit(&mut self)`
7. Write 5 unit tests including stress test (100K allocations)

**Edge Cases:**
- `len()` counts only alive handles, not total capacity
- `clear()` increments all generations (invalidates all handles)
- Shrink should only reduce free_list, not generations vec

**Verification:**
```bash
cargo test core::handle::capacity
```

---

### [ ] Step 1.2.5: Implement HandleMap
**File:** `goud_engine/src/core/handle.rs` (update)
**Context:** Associates handles with values. Used for all engine resource storage.

**Tasks:**
1. Create `HandleMap<T, V>` struct:
   ```rust
   pub struct HandleMap<T, V> {
       allocator: HandleAllocator<T>,
       values: Vec<Option<V>>,
   }
   ```
2. Implement `HandleMap<T, V>::new() -> Self`
3. Implement `HandleMap<T, V>::insert(&mut self, value: V) -> Handle<T>`
4. Implement `HandleMap<T, V>::remove(&mut self, handle: Handle<T>) -> Option<V>`
5. Implement `HandleMap<T, V>::get(&self, handle: Handle<T>) -> Option<&V>`
6. Implement `HandleMap<T, V>::get_mut(&mut self, handle: Handle<T>) -> Option<&mut V>`
7. Implement `HandleMap<T, V>::contains(&self, handle: Handle<T>) -> bool`
8. Write 10 unit tests

**Edge Cases:**
- Values vec must stay in sync with allocator
- Remove should drop the value, not just mark as None
- Get on invalid handle returns None, never panics

**Design Pattern:** Slot map / handle-based storage

**Verification:**
```bash
cargo test core::handle::map
```

---

### [ ] Step 1.2.6: Implement HandleMap Iterator
**File:** `goud_engine/src/core/handle.rs` (update)
**Context:** Enable iteration over all valid handle-value pairs.

**Tasks:**
1. Create `HandleMapIter<'a, T, V>` struct
2. Implement `Iterator` for `HandleMapIter`:
   - Item = `(Handle<T>, &'a V)`
   - Skip None entries
3. Create `HandleMapIterMut<'a, T, V>` for mutable iteration
4. Implement `HandleMap::iter(&self) -> HandleMapIter`
5. Implement `HandleMap::iter_mut(&mut self) -> HandleMapIterMut`
6. Implement `HandleMap::handles(&self) -> impl Iterator<Item = Handle<T>>`
7. Implement `HandleMap::values(&self) -> impl Iterator<Item = &V>`
8. Write 8 unit tests

**Edge Cases:**
- Iterator must skip deallocated slots
- Mutation during iteration is prevented by borrow checker
- Empty map iteration should yield nothing

**Verification:**
```bash
cargo test core::handle::iter
```

---

## 1.3 Event System

### [ ] Step 1.3.1: Define Event Trait
**File:** `goud_engine/src/core/event.rs` (new)
**Context:** Events enable decoupled communication between systems. The Event trait marks types that can be sent through the event system.

**Tasks:**
1. Create `event.rs` with Event trait:
   ```rust
   pub trait Event: Send + Sync + 'static {}
   ```
2. Create blanket implementation for all qualifying types:
   ```rust
   impl<T: Send + Sync + 'static> Event for T {}
   ```
3. Add module to `core/mod.rs`
4. Write 3 unit tests with example event types

**Edge Cases:**
- Events must be Send + Sync for parallel system execution
- 'static required for type erasure in event storage
- Blanket impl means any compatible type is an Event

**Design Pattern:** Marker trait with blanket implementation

**Verification:**
```bash
cargo test core::event::trait
```

---

### [ ] Step 1.3.2: Implement EventQueue
**File:** `goud_engine/src/core/event.rs` (update)
**Context:** Stores events of a single type for one frame. Double-buffered for producer/consumer pattern.

**Tasks:**
1. Create `EventQueue<E: Event>` struct:
   ```rust
   pub struct EventQueue<E: Event> {
       events_a: Vec<E>,
       events_b: Vec<E>,
       active_buffer: bool,  // false = A write/B read, true = B write/A read
   }
   ```
2. Implement `EventQueue<E>::new() -> Self`
3. Implement `EventQueue<E>::send(&mut self, event: E)`
4. Implement `EventQueue<E>::drain(&mut self) -> impl Iterator<Item = E>` (read buffer)
5. Implement `EventQueue<E>::swap_buffers(&mut self)` (swap active/read)
6. Implement `EventQueue<E>::clear(&mut self)` (clear both)
7. Implement `EventQueue<E>::is_empty(&self) -> bool` (write buffer)
8. Implement `EventQueue<E>::len(&self) -> usize` (write buffer)
9. Write 8 unit tests

**Edge Cases:**
- Write goes to active buffer, read from inactive
- Swap must happen at frame boundary
- Drain clears the read buffer

**Design Pattern:** Double-buffered event queue

**Verification:**
```bash
cargo test core::event::queue
```

---

### [ ] Step 1.3.3: Implement EventReader and EventWriter
**File:** `goud_engine/src/core/event.rs` (update)
**Context:** Type-safe accessors for systems to read/write events. Prevents multiple mutable borrows.

**Tasks:**
1. Create `EventReader<'a, E: Event>` struct:
   ```rust
   pub struct EventReader<'a, E: Event> {
       queue: &'a EventQueue<E>,
       read_index: usize,
   }
   ```
2. Implement `EventReader<E>::read(&mut self) -> impl Iterator<Item = &E>`
3. Implement `EventReader<E>::is_empty(&self) -> bool`
4. Create `EventWriter<'a, E: Event>` struct:
   ```rust
   pub struct EventWriter<'a, E: Event> {
       queue: &'a mut EventQueue<E>,
   }
   ```
5. Implement `EventWriter<E>::send(&mut self, event: E)`
6. Implement `EventWriter<E>::send_batch(&mut self, events: impl IntoIterator<Item = E>)`
7. Write 8 unit tests

**Edge Cases:**
- Reader tracks position to avoid re-reading same events
- Multiple readers can exist (shared borrow)
- Only one writer can exist (exclusive borrow)

**Design Pattern:** Split borrows for concurrent access

**Verification:**
```bash
cargo test core::event::accessor
```

---

### [ ] Step 1.3.4: Implement Events Resource Wrapper
**File:** `goud_engine/src/core/event.rs` (update)
**Context:** Wraps EventQueue to integrate with ECS resource system.

**Tasks:**
1. Create `Events<E: Event>` struct wrapping `EventQueue<E>`
2. Implement `Events<E>::new() -> Self`
3. Implement `Events<E>::reader(&self) -> EventReader<E>`
4. Implement `Events<E>::writer(&mut self) -> EventWriter<E>`
5. Implement `Events<E>::update(&mut self)` (swap buffers, clear old)
6. Implement `Resource` trait for `Events<E>` (to be defined later, stub for now)
7. Write 5 unit tests

**Edge Cases:**
- Update must be called exactly once per frame
- Order: systems run -> update called -> next frame
- Missing update causes events to persist incorrectly

**Verification:**
```bash
cargo test core::event::resource
```

---

### [ ] Step 1.3.5: Define Common Engine Events
**File:** `goud_engine/src/core/events.rs` (new, separate from event.rs)
**Context:** Pre-defined events that the engine will emit. Games can subscribe to these.

**Tasks:**
1. Create `events.rs` with common event types:
   ```rust
   pub struct AppStarted;
   pub struct AppExiting;
   pub struct WindowResized { pub width: u32, pub height: u32 }
   pub struct WindowFocused { pub focused: bool }
   pub struct FrameStarted { pub frame: u64, pub delta: f32 }
   pub struct FrameEnded { pub frame: u64 }
   ```
2. Add to `core/mod.rs`
3. Write 3 unit tests verifying Event trait bounds

**Edge Cases:**
- All events should be `#[derive(Debug, Clone)]`
- Consider `Copy` for small events
- Events are data-only, no behavior

**Verification:**
```bash
cargo test core::events
```

---

## 1.4 Math Types

### [ ] Step 1.4.1: Review and Plan Math Type Strategy
**File:** `goud_engine/src/core/math.rs` (new)
**Context:** Decide whether to wrap cgmath, use glam, or define custom types. Need FFI-compatible math types.

**Tasks:**
1. Audit current cgmath usage in codebase
2. Create `math.rs` with re-exports or wrapper decision
3. Define `#[repr(C)]` wrappers if needed for FFI:
   - `Vec2`, `Vec3`, `Vec4`
   - `Mat3`, `Mat4`
   - `Quat`
4. Ensure all types implement: Clone, Copy, Debug, PartialEq, Default
5. Add to `core/mod.rs`
6. Write 5 unit tests for basic operations

**Decision Points:**
- cgmath: Already in use, stable, but not FFI-friendly
- glam: Faster, SIMD, but migration effort
- Recommendation: Wrap cgmath with `#[repr(C)]` newtypes for FFI

**Edge Cases:**
- FFI types must have predictable memory layout
- Quaternion normalization edge cases
- Matrix inverse for singular matrices

**Verification:**
```bash
cargo test core::math
```

---

### [ ] Step 1.4.2: Implement FFI-Safe Vec2
**File:** `goud_engine/src/core/math.rs` (update)
**Context:** 2D vector for positions, velocities, scales.

**Tasks:**
1. Define `Vec2`:
   ```rust
   #[repr(C)]
   #[derive(Clone, Copy, Debug, PartialEq, Default)]
   pub struct Vec2 {
       pub x: f32,
       pub y: f32,
   }
   ```
2. Implement constructors: `new(x, y)`, `zero()`, `one()`, `unit_x()`, `unit_y()`
3. Implement operations: `add`, `sub`, `mul(scalar)`, `div(scalar)`, `neg`
4. Implement methods: `dot`, `length`, `length_squared`, `normalize`, `lerp`
5. Implement `From<cgmath::Vector2<f32>>` and `Into<cgmath::Vector2<f32>>`
6. Implement `Add`, `Sub`, `Mul<f32>`, `Div<f32>`, `Neg` traits
7. Write 10 unit tests

**Edge Cases:**
- Normalize of zero vector (return zero or panic?)
- Division by zero in div
- NaN propagation

**Verification:**
```bash
cargo test core::math::vec2
```

---

### [ ] Step 1.4.3: Implement FFI-Safe Vec3
**File:** `goud_engine/src/core/math.rs` (update)
**Context:** 3D vector for positions, rotations, colors.

**Tasks:**
1. Define `Vec3` similar to Vec2 with x, y, z
2. Add constructors: `new(x, y, z)`, `zero()`, `one()`, `unit_x/y/z()`
3. Add methods: `dot`, `cross`, `length`, `normalize`, `lerp`
4. Add conversions with cgmath
5. Add operator trait implementations
6. Write 10 unit tests

**Edge Cases:**
- Cross product order matters (a × b ≠ b × a)
- Used for both spatial and color data

**Verification:**
```bash
cargo test core::math::vec3
```

---

### [ ] Step 1.4.4: Implement FFI-Safe Rect and Color
**File:** `goud_engine/src/core/math.rs` (update)
**Context:** Common utility types for graphics.

**Tasks:**
1. Define `Rect`:
   ```rust
   #[repr(C)]
   pub struct Rect {
       pub x: f32,
       pub y: f32,
       pub width: f32,
       pub height: f32,
   }
   ```
2. Add methods: `new`, `from_min_max`, `min()`, `max()`, `center()`, `contains(Vec2)`, `intersects(Rect)`
3. Define `Color`:
   ```rust
   #[repr(C)]
   pub struct Color {
       pub r: f32,
       pub g: f32,
       pub b: f32,
       pub a: f32,
   }
   ```
4. Add constructors: `new`, `rgb`, `rgba`, `from_u8(r, g, b, a)`
5. Add constants: `WHITE`, `BLACK`, `RED`, `GREEN`, `BLUE`, `TRANSPARENT`
6. Write 10 unit tests

**Edge Cases:**
- Color components clamped to 0.0-1.0?
- Rect with negative width/height

**Verification:**
```bash
cargo test core::math::rect
cargo test core::math::color
```

---

# Phase 2: ECS Core Implementation

**Goal:** Build a complete, Bevy-inspired Entity-Component-System from scratch.

---

## 2.1 Entity System

### [ ] Step 2.1.1: Define Entity Type
**File:** `goud_engine/src/ecs/entity.rs` (new)
**Context:** Entities are lightweight identifiers. Using generational indices for safety.

**Tasks:**
1. Create `goud_engine/src/ecs/` directory
2. Create `entity.rs` with Entity struct:
   ```rust
   #[repr(C)]
   #[derive(Clone, Copy, PartialEq, Eq, Hash)]
   pub struct Entity {
       index: u32,
       generation: u32,
   }
   ```
3. Implement `Entity::new(index, generation)` (pub(crate))
4. Implement `Entity::index(&self) -> u32`
5. Implement `Entity::generation(&self) -> u32`
6. Implement `Entity::PLACEHOLDER` constant
7. Implement `Debug` with format `Entity(index:gen)`
8. Create `ecs/mod.rs` and add to `lib.rs`
9. Write 5 unit tests

**Edge Cases:**
- Entity is NOT generic (unlike Handle<T>)
- PLACEHOLDER must never be returned by allocator
- Entity should be 8 bytes total

**Verification:**
```bash
cargo test ecs::entity::structure
```

---

### [ ] Step 2.1.2: Implement EntityAllocator
**File:** `goud_engine/src/ecs/entity.rs` (update)
**Context:** Manages entity ID allocation with generation counting.

**Tasks:**
1. Create `EntityAllocator` struct (similar to HandleAllocator but for Entity)
2. Implement `allocate() -> Entity`
3. Implement `deallocate(Entity) -> bool`
4. Implement `is_alive(Entity) -> bool`
5. Implement `len()`, `capacity()`, `is_empty()`
6. Write 10 unit tests including recycling test

**Edge Cases:**
- Reuse of deallocated indices
- Generation overflow handling
- Thread safety considerations (not Send+Sync yet)

**Verification:**
```bash
cargo test ecs::entity::allocator
```

---

### [ ] Step 2.1.3: Implement EntityAllocator Bulk Operations
**File:** `goud_engine/src/ecs/entity.rs` (update)
**Context:** Optimize for spawning many entities at once.

**Tasks:**
1. Implement `allocate_batch(count: usize) -> Vec<Entity>`
2. Implement `deallocate_batch(entities: &[Entity]) -> usize` (returns success count)
3. Implement `reserve(additional: usize)`
4. Add benchmark for batch vs individual allocation
5. Write 5 unit tests

**Edge Cases:**
- Partial batch deallocation (some invalid)
- Reserve should pre-allocate generation vec

**Verification:**
```bash
cargo test ecs::entity::batch
```

---

## 2.2 Component Storage

### [ ] Step 2.2.1: Define Component Trait
**File:** `goud_engine/src/ecs/component.rs` (new)
**Context:** Components are data attached to entities. Trait marks types that can be components.

**Tasks:**
1. Create `component.rs` with Component trait:
   ```rust
   pub trait Component: Send + Sync + 'static {}
   ```
2. Do NOT add blanket implementation (components must opt-in)
3. Create derive macro placeholder comment for future
4. Add to `ecs/mod.rs`
5. Write 3 unit tests with example components

**Edge Cases:**
- Components must be Send + Sync for parallel systems
- 'static required for type storage
- No blanket impl - explicit opt-in for safety

**Verification:**
```bash
cargo test ecs::component::trait
```

---

### [ ] Step 2.2.2: Implement SparseSet Data Structure
**File:** `goud_engine/src/ecs/sparse_set.rs` (new)
**Context:** Core data structure for component storage. O(1) access, cache-friendly iteration.

**Tasks:**
1. Create `sparse_set.rs` with `SparseSet<T>`:
   ```rust
   pub struct SparseSet<T> {
       sparse: Vec<Option<usize>>,  // entity index -> dense index
       dense: Vec<Entity>,           // dense index -> entity
       values: Vec<T>,               // dense index -> component value
   }
   ```
2. Implement `new() -> Self`
3. Implement `insert(&mut self, entity: Entity, value: T) -> Option<T>` (returns old if exists)
4. Implement `remove(&mut self, entity: Entity) -> Option<T>`
5. Implement `get(&self, entity: Entity) -> Option<&T>`
6. Implement `get_mut(&mut self, entity: Entity) -> Option<&mut T>`
7. Implement `contains(&self, entity: Entity) -> bool`
8. Add to `ecs/mod.rs`
9. Write 15 unit tests

**Edge Cases:**
- Sparse vec grows on demand
- Remove uses swap-remove in dense array
- Entity index used as sparse key

**Design Pattern:** Sparse set (ECS standard)

**Verification:**
```bash
cargo test ecs::sparse_set
```

---

### [ ] Step 2.2.3: Implement SparseSet Iteration
**File:** `goud_engine/src/ecs/sparse_set.rs` (update)
**Context:** Enable cache-friendly iteration over all components.

**Tasks:**
1. Create `SparseSetIter<'a, T>` struct
2. Implement `Iterator` for `SparseSetIter` with Item = `(Entity, &'a T)`
3. Create `SparseSetIterMut<'a, T>` for mutable iteration
4. Implement `SparseSet::iter(&self) -> SparseSetIter`
5. Implement `SparseSet::iter_mut(&mut self) -> SparseSetIterMut`
6. Implement `SparseSet::entities(&self) -> impl Iterator<Item = Entity>`
7. Implement `SparseSet::values(&self) -> impl Iterator<Item = &T>`
8. Implement `len()`, `is_empty()`
9. Write 8 unit tests

**Edge Cases:**
- Iteration order matches dense array order (not entity order)
- Mutation during iteration prevented by borrow checker

**Verification:**
```bash
cargo test ecs::sparse_set::iter
```

---

### [ ] Step 2.2.4: Define ComponentId
**File:** `goud_engine/src/ecs/component.rs` (update)
**Context:** Runtime type identifier for components. Used for archetype keys.

**Tasks:**
1. Create `ComponentId` struct:
   ```rust
   #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
   pub struct ComponentId(TypeId);
   ```
2. Implement `ComponentId::of<T: Component>() -> ComponentId`
3. Implement `Debug` for ComponentId (include type name if possible)
4. Create `ComponentInfo` struct:
   ```rust
   pub struct ComponentInfo {
       pub id: ComponentId,
       pub name: &'static str,
       pub size: usize,
       pub align: usize,
   }
   ```
5. Implement `ComponentInfo::of<T: Component>() -> ComponentInfo`
6. Write 5 unit tests

**Edge Cases:**
- TypeId must be stable within a process
- Different generic instantiations have different TypeIds
- Name uses `std::any::type_name`

**Verification:**
```bash
cargo test ecs::component::id
```

---

### [ ] Step 2.2.5: Implement ComponentStorage Trait
**File:** `goud_engine/src/ecs/storage.rs` (new)
**Context:** Abstract interface for component storage backends.

**Tasks:**
1. Create `storage.rs` with `ComponentStorage` trait:
   ```rust
   pub trait ComponentStorage: Send + Sync {
       type Item: Component;
       fn insert(&mut self, entity: Entity, value: Self::Item) -> Option<Self::Item>;
       fn remove(&mut self, entity: Entity) -> Option<Self::Item>;
       fn get(&self, entity: Entity) -> Option<&Self::Item>;
       fn get_mut(&mut self, entity: Entity) -> Option<&mut Self::Item>;
       fn contains(&self, entity: Entity) -> bool;
       fn len(&self) -> usize;
       fn is_empty(&self) -> bool;
   }
   ```
2. Implement `ComponentStorage` for `SparseSet<T>`
3. Add to `ecs/mod.rs`
4. Write 5 unit tests

**Edge Cases:**
- Trait is object-safe considerations (it's not, that's okay)
- Storage must be Send + Sync for world

**Verification:**
```bash
cargo test ecs::storage
```

---

## 2.3 Archetype System

### [ ] Step 2.3.1: Define ArchetypeId
**File:** `goud_engine/src/ecs/archetype.rs` (new)
**Context:** Archetypes group entities with identical component sets for efficient iteration.

**Tasks:**
1. Create `archetype.rs` with `ArchetypeId`:
   ```rust
   #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
   pub struct ArchetypeId(u32);
   ```
2. Implement `ArchetypeId::EMPTY` constant (archetype with no components)
3. Implement `ArchetypeId::new(id: u32) -> Self`
4. Implement `ArchetypeId::index(&self) -> u32`
5. Add to `ecs/mod.rs`
6. Write 3 unit tests

**Verification:**
```bash
cargo test ecs::archetype::id
```

---

### [ ] Step 2.3.2: Define Archetype Structure
**File:** `goud_engine/src/ecs/archetype.rs` (update)
**Context:** An archetype stores all entities that have exactly the same set of components.

**Tasks:**
1. Create `Archetype` struct:
   ```rust
   pub struct Archetype {
       id: ArchetypeId,
       components: BTreeSet<ComponentId>,  // sorted for consistent hashing
       entities: Vec<Entity>,
   }
   ```
2. Implement `Archetype::new(id: ArchetypeId, components: BTreeSet<ComponentId>) -> Self`
3. Implement `Archetype::id(&self) -> ArchetypeId`
4. Implement `Archetype::components(&self) -> &BTreeSet<ComponentId>`
5. Implement `Archetype::has_component(&self, id: ComponentId) -> bool`
6. Implement `Archetype::entities(&self) -> &[Entity]`
7. Implement `Archetype::len(&self) -> usize`
8. Implement `Archetype::is_empty(&self) -> bool`
9. Write 5 unit tests

**Edge Cases:**
- Empty archetype (no components) is valid
- Component set must be sorted for consistent identity

**Verification:**
```bash
cargo test ecs::archetype::structure
```

---

### [ ] Step 2.3.3: Implement Archetype Entity Management
**File:** `goud_engine/src/ecs/archetype.rs` (update)
**Context:** Manage which entities belong to an archetype.

**Tasks:**
1. Add `entity_indices: HashMap<Entity, usize>` to Archetype for O(1) lookup
2. Implement `Archetype::add_entity(&mut self, entity: Entity)`
3. Implement `Archetype::remove_entity(&mut self, entity: Entity) -> bool`
4. Implement `Archetype::contains_entity(&self, entity: Entity) -> bool`
5. Implement `Archetype::entity_index(&self, entity: Entity) -> Option<usize>`
6. Write 8 unit tests including swap-remove correctness

**Edge Cases:**
- Remove uses swap-remove to maintain dense packing
- Index map must be updated on swap
- Duplicate add should be idempotent or error

**Verification:**
```bash
cargo test ecs::archetype::entities
```

---

### [ ] Step 2.3.4: Implement ArchetypeGraph
**File:** `goud_engine/src/ecs/archetype.rs` (update)
**Context:** Tracks relationships between archetypes for efficient component add/remove.

**Tasks:**
1. Create `ArchetypeGraph` struct:
   ```rust
   pub struct ArchetypeGraph {
       archetypes: Vec<Archetype>,
       component_index: HashMap<BTreeSet<ComponentId>, ArchetypeId>,
       edges: HashMap<(ArchetypeId, ComponentId), ArchetypeId>,  // add edges
       remove_edges: HashMap<(ArchetypeId, ComponentId), ArchetypeId>,
   }
   ```
2. Implement `ArchetypeGraph::new() -> Self` (with empty archetype)
3. Implement `ArchetypeGraph::get(&self, id: ArchetypeId) -> Option<&Archetype>`
4. Implement `ArchetypeGraph::get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype>`
5. Implement `ArchetypeGraph::find_or_create(&mut self, components: BTreeSet<ComponentId>) -> ArchetypeId`
6. Write 8 unit tests

**Edge Cases:**
- Empty archetype always exists at index 0
- Duplicate component sets return existing archetype

**Verification:**
```bash
cargo test ecs::archetype::graph
```

---

### [ ] Step 2.3.5: Implement Archetype Transitions
**File:** `goud_engine/src/ecs/archetype.rs` (update)
**Context:** When adding/removing components, entities move between archetypes.

**Tasks:**
1. Implement `ArchetypeGraph::get_add_edge(&mut self, from: ArchetypeId, component: ComponentId) -> ArchetypeId`:
   - Returns cached edge if exists
   - Otherwise creates new archetype with component added
2. Implement `ArchetypeGraph::get_remove_edge(&mut self, from: ArchetypeId, component: ComponentId) -> Option<ArchetypeId>`:
   - Returns None if component not in archetype
   - Returns cached edge if exists
   - Otherwise creates new archetype with component removed
3. Write 10 unit tests for transition correctness

**Edge Cases:**
- Adding component that already exists (no-op edge to self)
- Removing component that doesn't exist (return None)
- Creating edge to empty archetype

**Verification:**
```bash
cargo test ecs::archetype::transitions
```

---

## 2.4 World Container

### [ ] Step 2.4.1: Define World Structure
**File:** `goud_engine/src/ecs/world.rs` (new)
**Context:** World is the central container for all ECS data.

**Tasks:**
1. Create `world.rs` with `World` struct:
   ```rust
   pub struct World {
       entities: EntityAllocator,
       archetypes: ArchetypeGraph,
       entity_archetypes: HashMap<Entity, ArchetypeId>,
       storages: HashMap<ComponentId, Box<dyn Any + Send + Sync>>,
   }
   ```
2. Implement `World::new() -> Self`
3. Implement `World::entity_count(&self) -> usize`
4. Implement `World::archetype_count(&self) -> usize`
5. Add to `ecs/mod.rs`
6. Write 5 unit tests

**Edge Cases:**
- Type-erased storage requires careful downcasting
- Entity-archetype map must stay in sync

**Verification:**
```bash
cargo test ecs::world::structure
```

---

### [ ] Step 2.4.2: Implement World Entity Spawning
**File:** `goud_engine/src/ecs/world.rs` (update)
**Context:** Create new entities in the world.

**Tasks:**
1. Implement `World::spawn_empty(&mut self) -> Entity`:
   - Allocate entity
   - Add to empty archetype
   - Return entity
2. Implement `World::spawn(&mut self) -> EntityWorldMut`:
   - Returns builder for fluent component addition
3. Create `EntityWorldMut<'w>` struct for builder pattern
4. Implement `EntityWorldMut::id(&self) -> Entity`
5. Write 5 unit tests

**Edge Cases:**
- Spawn without components goes to empty archetype
- Builder holds mutable borrow on World

**Verification:**
```bash
cargo test ecs::world::spawn
```

---

### [ ] Step 2.4.3: Implement World Entity Despawning
**File:** `goud_engine/src/ecs/world.rs` (update)
**Context:** Remove entities and all their components.

**Tasks:**
1. Implement `World::despawn(&mut self, entity: Entity) -> bool`:
   - Find entity's archetype
   - Remove from archetype
   - Remove from all component storages
   - Deallocate entity
   - Return success
2. Implement `World::despawn_batch(&mut self, entities: &[Entity]) -> usize`
3. Implement `World::is_alive(&self, entity: Entity) -> bool`
4. Write 8 unit tests including component cleanup verification

**Edge Cases:**
- Despawning non-existent entity returns false
- Components must be dropped on despawn
- Batch despawn should be efficient

**Verification:**
```bash
cargo test ecs::world::despawn
```

---

### [ ] Step 2.4.4: Implement World Component Access
**File:** `goud_engine/src/ecs/world.rs` (update)
**Context:** Get and set components on entities.

**Tasks:**
1. Implement `World::get<T: Component>(&self, entity: Entity) -> Option<&T>`
2. Implement `World::get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T>`
3. Implement `World::has<T: Component>(&self, entity: Entity) -> bool`
4. Helper: `World::get_storage<T: Component>(&self) -> Option<&SparseSet<T>>`
5. Helper: `World::get_storage_mut<T: Component>(&mut self) -> Option<&mut SparseSet<T>>`
6. Write 10 unit tests

**Edge Cases:**
- Get on dead entity returns None
- Get non-existent component returns None
- Type safety ensured via ComponentId

**Verification:**
```bash
cargo test ecs::world::component_access
```

---

### [ ] Step 2.4.5: Implement World Component Insertion
**File:** `goud_engine/src/ecs/world.rs` (update)
**Context:** Add components to existing entities, handling archetype transitions.

**Tasks:**
1. Implement `World::insert<T: Component>(&mut self, entity: Entity, component: T) -> Option<T>`:
   - Get or create storage for T
   - If entity has T, replace and return old
   - If not, transition to new archetype and insert
2. Implement `World::insert_batch<T: Component>(&mut self, batch: impl IntoIterator<Item = (Entity, T)>)`
3. Implement `EntityWorldMut::insert<T: Component>(&mut self, component: T) -> &mut Self`
4. Write 10 unit tests including archetype transition

**Edge Cases:**
- Insert on dead entity should fail gracefully
- Archetype transition must move all existing components
- Storage creation must be thread-safe

**Verification:**
```bash
cargo test ecs::world::insert
```

---

### [ ] Step 2.4.6: Implement World Component Removal
**File:** `goud_engine/src/ecs/world.rs` (update)
**Context:** Remove components from entities, handling archetype transitions.

**Tasks:**
1. Implement `World::remove<T: Component>(&mut self, entity: Entity) -> Option<T>`:
   - If entity doesn't have T, return None
   - Remove from storage
   - Transition to new archetype (without T)
   - Return removed component
2. Implement `World::take<T: Component>(&mut self, entity: Entity) -> Option<T>` (alias for remove)
3. Write 8 unit tests including transition to empty archetype

**Edge Cases:**
- Remove last component transitions to empty archetype
- Remove non-existent component is no-op

**Verification:**
```bash
cargo test ecs::world::remove
```

---

## 2.5 Query System

### [ ] Step 2.5.1: Define Query Fetch Traits
**File:** `goud_engine/src/ecs/query/fetch.rs` (new)
**Context:** Queries fetch component data. Fetch traits define what can be queried.

**Tasks:**
1. Create `ecs/query/` directory and `fetch.rs`
2. Define `WorldQuery` trait:
   ```rust
   pub trait WorldQuery {
       type Item<'w>;
       type State;
       fn init_state(world: &World) -> Self::State;
       fn matches_archetype(state: &Self::State, archetype: &Archetype) -> bool;
       fn fetch<'w>(state: &Self::State, world: &'w World, entity: Entity) -> Option<Self::Item<'w>>;
   }
   ```
3. Create `ecs/query/mod.rs` and add to `ecs/mod.rs`
4. Write 3 unit tests with documentation

**Edge Cases:**
- GAT (Generic Associated Types) used for Item lifetime
- State caches component IDs for efficiency

**Verification:**
```bash
cargo test ecs::query::fetch
```

---

### [ ] Step 2.5.2: Implement Component Reference Fetch
**File:** `goud_engine/src/ecs/query/fetch.rs` (update)
**Context:** Query for `&T` - immutable component reference.

**Tasks:**
1. Implement `WorldQuery` for `&T where T: Component`:
   - `Item<'w>` = `&'w T`
   - `State` = `ComponentId`
   - `init_state` returns `ComponentId::of::<T>()`
   - `matches_archetype` checks archetype has component
   - `fetch` gets from storage
2. Write 5 unit tests

**Edge Cases:**
- Query for component entity doesn't have returns None
- Multiple read-only queries can run in parallel

**Verification:**
```bash
cargo test ecs::query::ref
```

---

### [ ] Step 2.5.3: Implement Mutable Component Reference Fetch
**File:** `goud_engine/src/ecs/query/fetch.rs` (update)
**Context:** Query for `&mut T` - mutable component reference.

**Tasks:**
1. Implement `WorldQuery` for `&mut T where T: Component`:
   - `Item<'w>` = `&'w mut T`
   - Same state as immutable
   - `fetch` gets mutable from storage
2. Define access conflict detection (read vs write)
3. Write 5 unit tests

**Edge Cases:**
- Mutable query conflicts with any other query on same component
- Only one mutable query per component allowed

**Verification:**
```bash
cargo test ecs::query::mut_ref
```

---

(Continuing with remaining phases in abbreviated form for space...)

---

# Phase 3: Systems & Scheduling

## 3.1 System Definition
### [ ] Step 3.1.1: Define System Trait
### [ ] Step 3.1.2: Implement SystemParam Trait
### [ ] Step 3.1.3: Implement Query as SystemParam
### [ ] Step 3.1.4: Implement Res/ResMut SystemParams
### [ ] Step 3.1.5: Implement FunctionSystem Wrapper

## 3.2 Resource Container
### [ ] Step 3.2.1: Define Resource Trait
### [ ] Step 3.2.2: Implement Resources Storage
### [ ] Step 3.2.3: Implement Resource Access API
### [ ] Step 3.2.4: Implement Non-Send Resources

## 3.3 Scheduler
### [ ] Step 3.3.1: Define Stage Enum
### [ ] Step 3.3.2: Implement SystemStage Container
### [ ] Step 3.3.3: Implement Access Conflict Detection
### [ ] Step 3.3.4: Implement Topological Ordering
### [ ] Step 3.3.5: Implement Parallel Execution
### [ ] Step 3.3.6: Implement System Ordering Constraints

## 3.4 Built-in Components
### [ ] Step 3.4.1: Implement Transform Component
### [ ] Step 3.4.2: Implement Transform2D Component
### [ ] Step 3.4.3: Implement Hierarchy Components (Parent, Children, Name)
### [ ] Step 3.4.4: Implement GlobalTransform and Propagation System

---

# Phase 4: Assets & FFI Layer

## 4.1 Asset System
### [ ] Step 4.1.1: Define Asset Trait
### [ ] Step 4.1.2: Implement AssetId and Handle
### [ ] Step 4.1.3: Implement AssetStorage
### [ ] Step 4.1.4: Implement AssetLoader Trait
### [ ] Step 4.1.5: Implement AssetServer

## 4.2 Asset Loaders
### [ ] Step 4.2.1: Implement Texture Loader
### [ ] Step 4.2.2: Implement Shader Loader
### [ ] Step 4.2.3: Implement Audio Loader (Stub)
### [ ] Step 4.2.4: Implement Hot Reloading

## 4.3 FFI Foundation
### [ ] Step 4.3.1: Define FFI Context Type
### [ ] Step 4.3.2: Implement Context Registry
### [ ] Step 4.3.3: Implement FFI Error Codes
### [ ] Step 4.3.4: Implement Thread-Local Error Storage
### [ ] Step 4.3.5: Implement Error Message Retrieval

## 4.4 FFI Entity Operations
### [ ] Step 4.4.1: Implement Entity Spawn FFI
### [ ] Step 4.4.2: Implement Entity Despawn FFI
### [ ] Step 4.4.3: Implement Component Add/Remove FFI
### [ ] Step 4.4.4: Implement Batch Operations FFI

---

# Phase 5: Graphics & Physics

## 5.1 Render Backend Abstraction
### [ ] Step 5.1.1: Define RenderBackend Trait
### [ ] Step 5.1.2: Define GPU Resource Types
### [ ] Step 5.1.3: Implement OpenGL Backend - Buffers
### [ ] Step 5.1.4: Implement OpenGL Backend - Textures
### [ ] Step 5.1.5: Implement OpenGL Backend - Shaders
### [ ] Step 5.1.6: Implement OpenGL Backend - Draw Calls

## 5.2 Sprite Rendering
### [ ] Step 5.2.1: Define Sprite Component
### [ ] Step 5.2.2: Implement SpriteBatch System
### [ ] Step 5.2.3: Implement Texture Batching
### [ ] Step 5.2.4: Implement Z-Layer Sorting
### [ ] Step 5.2.5: Integrate with ECS Renderer

## 5.3 Physics Foundation
### [ ] Step 5.3.1: Define PhysicsWorld Resource
### [ ] Step 5.3.2: Define RigidBody Component
### [ ] Step 5.3.3: Define Collider Component and Shapes
### [ ] Step 5.3.4: Implement AABB Calculations
### [ ] Step 5.3.5: Implement Broad Phase (Spatial Hash)

## 5.4 Collision Detection
### [ ] Step 5.4.1: Implement Circle-Circle Collision
### [ ] Step 5.4.2: Implement Box-Box Collision (SAT)
### [ ] Step 5.4.3: Implement Circle-Box Collision
### [ ] Step 5.4.4: Implement Collision Response
### [ ] Step 5.4.5: Implement Collision Events

---

# Phase 6: Audio, SDK & Polish

## 6.1 Audio System
### [ ] Step 6.1.1: Integrate Rodio Dependency
### [ ] Step 6.1.2: Implement AudioManager Resource
### [ ] Step 6.1.3: Implement AudioSource Component
### [ ] Step 6.1.4: Implement Audio Playback System
### [ ] Step 6.1.5: Implement Spatial Audio (Basic)

## 6.2 C# SDK Update
### [ ] Step 6.2.1: Update NativeMethods Generation
### [ ] Step 6.2.2: Implement GoudContext Wrapper
### [ ] Step 6.2.3: Implement Entity Wrapper
### [ ] Step 6.2.4: Implement Component Builders
### [ ] Step 6.2.5: Implement Error Handling

## 6.3 Input System
### [ ] Step 6.3.1: Implement InputManager Resource
### [ ] Step 6.3.2: Implement Action Mapping
### [ ] Step 6.3.3: Implement Input Buffering
### [ ] Step 6.3.4: Implement Gamepad Support

## 6.4 Integration & Testing
### [ ] Step 6.4.1: Integrate ECS with Existing Renderers
### [ ] Step 6.4.2: Update Example Games
### [ ] Step 6.4.3: Implement Performance Benchmarks
### [ ] Step 6.4.4: Documentation Pass
### [ ] Step 6.4.5: Final Migration and Cleanup

---

## Verification Checklist

### Per-Step Requirements
Each step MUST:
- [ ] Have all tests passing: `cargo test`
- [ ] Have no clippy warnings: `cargo clippy -- -D warnings`
- [ ] Have no new compile warnings: `cargo build 2>&1 | grep warning`
- [ ] Maintain backward compatibility (existing code compiles)
- [ ] Include doc comments for public APIs
- [ ] Be independently verifiable

### Phase Gate Criteria

**Phase 1 Complete When:**
- All 25 steps verified
- `core/` module fully tested
- No changes to existing code

**Phase 2 Complete When:**
- All 22 steps verified
- ECS can spawn/despawn entities
- Components can be added/removed
- Queries return correct results

**Phase 3 Complete When:**
- All 20 steps verified
- Systems execute in correct order
- Parallel execution works
- Transform hierarchy propagates

**Phase 4 Complete When:**
- All 20 steps verified
- Assets load and cache correctly
- FFI functions work from C#
- Batch operations achieve 10x speedup

**Phase 5 Complete When:**
- All 22 steps verified
- Sprites batch correctly (<100 draws for 10K)
- Physics simulation runs
- Collision events fire

**Phase 6 Complete When:**
- All 18 steps verified
- Audio plays correctly
- C# SDK fully updated
- All examples work
- Performance targets met

---

*Plan Version: 2.0*
*Created: 2026-01-04*
*Total Steps: 127 atomic tasks*
*Estimated Sessions: ~60-80 focused sessions*
