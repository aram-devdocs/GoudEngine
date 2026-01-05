# GoudEngine Full Refactor - Technical Specification

## Document Information
- **Version:** 2.0
- **Date:** 2026-01-04
- **Based on:** requirements.md PRD v1.0
- **Status:** Approved for Implementation

---

## 1. Executive Summary

This specification defines the complete architectural overhaul of GoudEngine, transforming it from a prototype-level engine into a professional, hardened game engine with industry-standard patterns. The engine will feature:

- **Custom Bevy-Inspired ECS**: Our own Entity-Component-System implementation, reverse-engineered from Bevy's architecture but fully owned and integrated into our codebase
- **MonoGame-Like Developer Experience**: Easy scripting APIs with the depth of a modern engine
- **Universal Language Support**: Clean abstractions enabling bindings for C#, Python, TypeScript, Lua, Go, and Rust itself
- **Test-Driven Development**: Comprehensive TDD approach ensuring hardened, production-ready code

---

## 2. Design Philosophy

### 2.1 Core Principles

| Principle | Description |
|-----------|-------------|
| **Data-Oriented Design** | Components are pure data; systems transform data; cache-friendly memory layouts |
| **Composition Over Inheritance** | Entities are bags of components; behavior emerges from system combinations |
| **Zero-Cost Abstractions** | High-level APIs compile to optimal low-level code |
| **Language Agnostic Core** | Engine logic independent of scripting language; clean FFI boundaries |
| **Explicit Over Implicit** | No hidden behavior; developers control everything explicitly |
| **Fail Fast, Fail Loud** | Comprehensive error handling with clear, actionable messages |

### 2.2 Inspirations

| Engine | What We Take |
|--------|--------------|
| **Bevy** | ECS architecture, system scheduling, resource management, plugin patterns |
| **MonoGame/XNA** | Developer experience, content pipeline, SpriteBatch API, familiar abstractions |
| **Godot** | Node hierarchy concepts, signal/event patterns, scripting accessibility |
| **Unity** | Component model familiarity, inspector-style configuration, prefab concepts |

---

## 3. High-Level System Architecture

### 3.1 Architectural Layers

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                         LANGUAGE BINDING LAYER                                  ║
║  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌──────────┐ ║
║  │   C# SDK    │ │ Python SDK  │ │  Lua SDK    │ │    Go SDK   │ │ Rust API │ ║
║  │  (Primary)  │ │  (Planned)  │ │  (Planned)  │ │  (Planned)  │ │ (Native) │ ║
║  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └────┬─────┘ ║
║         │               │               │               │              │       ║
╠═════════╧═══════════════╧═══════════════╧═══════════════╧══════════════╧═══════╣
║                          FOREIGN FUNCTION INTERFACE                             ║
║  ┌─────────────────────────────────────────────────────────────────────────┐   ║
║  │                        C ABI Stable API Layer                            │   ║
║  │  • Versioned API with semantic versioning                                │   ║
║  │  • Handle-based resource management (no raw pointers)                    │   ║
║  │  • Comprehensive error codes with messages                               │   ║
║  │  • Batch operations for performance                                      │   ║
║  │  • Thread-safety annotations on all functions                            │   ║
║  └─────────────────────────────────────────────────────────────────────────┘   ║
║         │                                                                       ║
╠═════════╧═══════════════════════════════════════════════════════════════════════╣
║                           ENGINE CORE LAYER                                     ║
║  ┌─────────────────────────────────────────────────────────────────────────┐   ║
║  │                         Application Context                              │   ║
║  │  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐                  │   ║
║  │  │ World        │  │ Resources    │  │ Event Bus     │                  │   ║
║  │  │ (ECS State)  │  │ (Singletons) │  │ (Pub/Sub)     │                  │   ║
║  │  └──────────────┘  └──────────────┘  └───────────────┘                  │   ║
║  │  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐                  │   ║
║  │  │ System       │  │ Plugin       │  │ Asset         │                  │   ║
║  │  │ Scheduler    │  │ Registry     │  │ Server        │                  │   ║
║  │  └──────────────┘  └──────────────┘  └───────────────┘                  │   ║
║  └─────────────────────────────────────────────────────────────────────────┘   ║
║         │                                                                       ║
╠═════════╧═══════════════════════════════════════════════════════════════════════╣
║                          SUBSYSTEM LAYER                                        ║
║  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   ║
║  │    ECS     │ │  Graphics  │ │   Audio    │ │  Physics   │ │   Input    │   ║
║  │            │ │            │ │            │ │            │ │            │   ║
║  │ • World    │ │ • Renderer │ │ • Mixer    │ │ • World    │ │ • Actions  │   ║
║  │ • Entities │ │ • Materials│ │ • Sources  │ │ • Bodies   │ │ • Bindings │   ║
║  │ • Comps    │ │ • Cameras  │ │ • Spatial  │ │ • Colliders│ │ • Devices  │   ║
║  │ • Systems  │ │ • Lights   │ │ • Effects  │ │ • Joints   │ │ • Events   │   ║
║  │ • Queries  │ │ • Batching │ │            │ │ • Queries  │ │            │   ║
║  └────────────┘ └────────────┘ └────────────┘ └────────────┘ └────────────┘   ║
║         │             │             │             │             │             ║
╠═════════╧═════════════╧═════════════╧═════════════╧═════════════╧═════════════╣
║                         BACKEND IMPLEMENTATION LAYER                           ║
║  ┌─────────────────────────────────────────────────────────────────────────┐  ║
║  │ Graphics Backends      │ Audio Backends      │ Physics Backends        │  ║
║  │ ┌────────┐ ┌────────┐ │ ┌────────────────┐  │ ┌────────────────────┐  │  ║
║  │ │OpenGL  │ │ Vulkan │ │ │ rodio          │  │ │ Custom 2D/3D       │  │  ║
║  │ │ 3.3    │ │(future)│ │ │                │  │ │ (rapier-inspired)  │  │  ║
║  │ └────────┘ └────────┘ │ └────────────────┘  │ └────────────────────┘  │  ║
║  └─────────────────────────────────────────────────────────────────────────┘  ║
║         │                                                                      ║
╠═════════╧══════════════════════════════════════════════════════════════════════╣
║                            PLATFORM LAYER                                       ║
║  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   ║
║  │  Window    │ │   Input    │ │ Filesystem │ │ Threading  │ │   Time     │   ║
║  │  (GLFW)    │ │   (Raw)    │ │  (Async)   │ │  (Rayon)   │ │  (Instant) │   ║
║  └────────────┘ └────────────┘ └────────────┘ └────────────┘ └────────────┘   ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### 3.2 Data Flow Architecture

```
                              GAME LOOP
                                  │
    ┌─────────────────────────────┴─────────────────────────────┐
    │                                                           │
    ▼                                                           ▼
┌───────────┐    ┌───────────┐    ┌───────────┐    ┌───────────────┐
│   Input   │───▶│  Update   │───▶│  Render   │───▶│ Present Frame │
│  Polling  │    │  Systems  │    │  Systems  │    │   to Screen   │
└───────────┘    └───────────┘    └───────────┘    └───────────────┘
    │                  │                │
    ▼                  ▼                ▼
┌───────────┐    ┌───────────┐    ┌───────────┐
│  Events   │    │   World   │    │  Commands │
│   Queue   │    │   State   │    │   Queue   │
└───────────┘    └───────────┘    └───────────┘
                       │
           ┌───────────┴───────────┐
           ▼                       ▼
    ┌─────────────┐         ┌─────────────┐
    │  Entities   │         │  Resources  │
    │ Components  │         │ (Singletons)│
    └─────────────┘         └─────────────┘
```

---

## 4. Custom ECS Architecture (Bevy-Inspired)

### 4.1 Design Goals

Our ECS implementation is inspired by Bevy's architecture but will be fully owned code within the GoudEngine codebase. This gives us:

- **Full Control**: No external dependency constraints
- **Tailored Features**: Optimized for game engine needs
- **Learning Opportunity**: Deep understanding of ECS internals
- **Customization**: Can diverge from Bevy patterns where beneficial

### 4.2 Core ECS Concepts

#### 4.2.1 Entities

Entities are lightweight identifiers (IDs) that serve as keys into component storage.

**Design Decisions:**
- Entity = 64-bit identifier (32-bit index + 32-bit generation)
- Generation counter prevents use-after-free bugs
- Entity allocation uses a free-list for ID reuse
- No Entity struct methods beyond ID access

**Entity Lifecycle:**
```
Create → Spawn Components → Active → Despawn → Recycle ID
           │                   │          │
           ▼                   ▼          ▼
     Entity ID           Component    Free List
       Issued             Queries      Returns
```

#### 4.2.2 Components

Components are plain data containers with no behavior. They define what an entity "is" or "has".

**Design Decisions:**
- Components implement a `Component` trait marker
- Components are stored in archetype-based storage (SoA layout)
- Components can be added/removed dynamically (archetype migration)
- Built-in components for common patterns (Transform, Name, Parent, Children)

**Component Categories:**

| Category | Purpose | Examples |
|----------|---------|----------|
| **Spatial** | Position, rotation, scale | Transform, Transform2D, GlobalTransform |
| **Rendering** | Visual representation | Sprite, Mesh, Material, Camera |
| **Physics** | Physical simulation | RigidBody, Collider, Velocity |
| **Audio** | Sound emission | AudioSource, AudioListener |
| **Hierarchy** | Parent-child relationships | Parent, Children, Name |
| **Lifecycle** | Entity state | Enabled, Persistent, Prefab |

#### 4.2.3 Systems

Systems are functions that operate on component data. They define behavior.

**Design Decisions:**
- Systems are pure functions with explicit dependencies
- System parameters declare what data they read/write
- Scheduler automatically parallelizes non-conflicting systems
- Systems can be grouped into stages for ordering

**System Stages:**

```
┌─────────────┐
│  PreUpdate  │  Input processing, event handling
├─────────────┤
│   Update    │  Game logic, AI, physics step
├─────────────┤
│ PostUpdate  │  Cleanup, state synchronization
├─────────────┤
│  PreRender  │  Culling, LOD, batching
├─────────────┤
│   Render    │  Draw calls, GPU submission
├─────────────┤
│ PostRender  │  Frame cleanup, stats collection
└─────────────┘
```

#### 4.2.4 Resources

Resources are singleton data that exists outside the entity-component model.

**Examples:**
- Time (delta time, total time)
- Input state (keyboard, mouse, gamepads)
- Asset manager (loaded assets)
- Render context (current renderer)
- Configuration (engine settings)

#### 4.2.5 Events

Events enable decoupled communication between systems.

**Event Patterns:**
- Event producers write to event queues
- Event consumers read from event queues
- Events are cleared each frame (unless persistent)
- Supports both immediate and deferred event handling

### 4.3 Archetype-Based Storage

#### 4.3.1 What is an Archetype?

An archetype represents a unique combination of component types. All entities with the same set of components share an archetype.

**Example:**
- Archetype A: [Transform, Sprite] → All 2D sprites
- Archetype B: [Transform, Sprite, RigidBody] → Physics-enabled sprites
- Archetype C: [Transform, Mesh, Material] → 3D renderable objects

#### 4.3.2 Why Archetypes?

| Benefit | Description |
|---------|-------------|
| **Cache Efficiency** | Components stored contiguously per archetype |
| **Fast Iteration** | Systems iterate archetypes, not individual entities |
| **Parallel Safety** | Different archetypes can be processed in parallel |
| **Query Optimization** | Archetype matching is done once, not per-entity |

#### 4.3.3 Archetype Operations

**Adding Component:**
```
Entity in Archetype A [Transform, Sprite]
                ↓
        Add RigidBody
                ↓
Entity moves to Archetype B [Transform, Sprite, RigidBody]
```

**Removing Component:**
```
Entity in Archetype B [Transform, Sprite, RigidBody]
                ↓
       Remove RigidBody
                ↓
Entity moves to Archetype A [Transform, Sprite]
```

### 4.4 Query System

Queries are the primary way systems access component data.

**Query Capabilities:**
- **Read Access**: `Query<&Transform>` - read-only borrow
- **Write Access**: `Query<&mut Transform>` - mutable borrow
- **Multiple Components**: `Query<(&Transform, &Sprite)>` - tuples
- **Optional Components**: `Query<(&Transform, Option<&Sprite>)>` - may not exist
- **Filters**: `Query<&Transform, With<Sprite>>` - only matching entities
- **Exclusion**: `Query<&Transform, Without<RigidBody>>` - exclude matching

**Query Execution:**
1. Query identifies matching archetypes at registration
2. Each frame, iterates matching archetypes
3. Returns component references for each entity
4. Iteration can be parallelized across archetypes

### 4.5 System Scheduling

#### 4.5.1 Dependency Analysis

The scheduler analyzes system parameters to determine:
- Which components a system reads
- Which components a system writes
- Which resources a system accesses
- Whether systems can run in parallel

#### 4.5.2 Scheduling Algorithm

```
1. Group systems by stage
2. Within each stage:
   a. Build dependency graph from read/write conflicts
   b. Topologically sort respecting explicit orderings
   c. Identify parallel groups (non-conflicting systems)
3. Execute stages sequentially
4. Within stages, execute parallel groups concurrently
```

#### 4.5.3 Explicit Ordering

When automatic scheduling isn't sufficient:
- `system_a.before(system_b)` - A runs before B
- `system_a.after(system_b)` - A runs after B
- `system_a.chain(system_b)` - A then B, strictly ordered

### 4.6 Built-In Components

#### 4.6.1 Transform Components

**Transform (3D):**
- `position: Vec3` - World position
- `rotation: Quat` - Orientation as quaternion
- `scale: Vec3` - Non-uniform scale

**Transform2D:**
- `position: Vec2` - World position
- `rotation: f32` - Rotation in radians
- `scale: Vec2` - Non-uniform scale
- `z_layer: i32` - Depth ordering

**GlobalTransform:**
- Computed from local Transform + parent hierarchy
- Read-only (written by transform propagation system)
- Used by rendering and physics

#### 4.6.2 Hierarchy Components

**Parent:**
- `entity: Entity` - Parent entity ID

**Children:**
- `entities: Vec<Entity>` - Child entity IDs

**Name:**
- `name: String` - Human-readable identifier

#### 4.6.3 Rendering Components

**Sprite:**
- `texture: Handle<Texture>` - Texture asset handle
- `source_rect: Option<Rect>` - Source rectangle for atlases
- `color: Color` - Tint color
- `flip_x: bool` - Horizontal flip
- `flip_y: bool` - Vertical flip

**Mesh:**
- `mesh: Handle<Mesh>` - Mesh asset handle
- `material: Handle<Material>` - Material asset handle

**Camera:**
- `projection: Projection` - Orthographic or Perspective
- `viewport: Option<Viewport>` - Render target region
- `order: i32` - Render order for multiple cameras

**Light:**
- `kind: LightKind` - Point, Directional, Spot
- `color: Color` - Light color
- `intensity: f32` - Light strength
- `shadows: bool` - Cast shadows

#### 4.6.4 Physics Components

**RigidBody:**
- `body_type: BodyType` - Dynamic, Static, Kinematic
- `mass: f32` - Mass (dynamic only)
- `linear_damping: f32` - Velocity damping
- `angular_damping: f32` - Rotation damping

**Collider:**
- `shape: ColliderShape` - Box, Circle, Capsule, etc.
- `is_sensor: bool` - Trigger-only (no physics response)
- `friction: f32` - Surface friction
- `restitution: f32` - Bounciness

**Velocity:**
- `linear: Vec3` - Linear velocity
- `angular: Vec3` - Angular velocity

#### 4.6.5 Audio Components

**AudioSource:**
- `audio: Handle<AudioClip>` - Audio asset handle
- `volume: f32` - Playback volume
- `pitch: f32` - Playback speed/pitch
- `looping: bool` - Loop playback
- `spatial: bool` - 3D spatial audio

**AudioListener:**
- Marker component for the entity that "hears" audio
- Only one listener should exist at a time

### 4.7 Built-In Systems

#### 4.7.1 Core Systems

| System | Stage | Purpose |
|--------|-------|---------|
| `transform_propagation` | PostUpdate | Compute GlobalTransform from hierarchy |
| `hierarchy_maintenance` | PostUpdate | Sync Parent/Children relationships |
| `visibility_propagation` | PostUpdate | Inherit visibility from parents |

#### 4.7.2 Rendering Systems

| System | Stage | Purpose |
|--------|-------|---------|
| `sprite_extraction` | PreRender | Gather visible sprites for batching |
| `mesh_extraction` | PreRender | Gather visible meshes for rendering |
| `camera_update` | PreRender | Compute view/projection matrices |
| `light_update` | PreRender | Gather lights for rendering |
| `sprite_render` | Render | Draw 2D sprites |
| `mesh_render` | Render | Draw 3D meshes |

#### 4.7.3 Physics Systems

| System | Stage | Purpose |
|--------|-------|---------|
| `physics_sync_to_world` | PreUpdate | Sync transforms to physics world |
| `physics_step` | Update | Run physics simulation |
| `physics_sync_from_world` | PostUpdate | Sync physics results to transforms |
| `collision_events` | PostUpdate | Generate collision events |

#### 4.7.4 Audio Systems

| System | Stage | Purpose |
|--------|-------|---------|
| `audio_playback` | Update | Handle audio source playback |
| `spatial_audio_update` | Update | Update spatial audio positions |
| `audio_cleanup` | PostUpdate | Clean up finished audio sources |

---

## 5. Resource Management Architecture

### 5.1 Handle-Based Resources

All resources (textures, meshes, audio, etc.) are accessed through handles, never raw pointers.

**Handle Design:**
- 64-bit handle: 32-bit index + 32-bit generation
- Handles are type-safe: `Handle<Texture>` vs `Handle<Mesh>`
- Handles can be validated before use
- Stale handles (pointing to freed resources) are detected

### 5.2 Asset Loading Pipeline

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Raw Asset  │───▶│   Loader    │───▶│   Cache     │───▶│   Handle    │
│   (File)    │    │  (Async)    │    │  (Memory)   │    │  (Returned) │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                         │
                         ▼
                   ┌─────────────┐
                   │   GPU       │
                   │  Upload     │
                   │ (Textures)  │
                   └─────────────┘
```

### 5.3 Asset Manager Features

| Feature | Description |
|---------|-------------|
| **Async Loading** | Load assets without blocking game loop |
| **Reference Counting** | Automatic cleanup when no references remain |
| **Hot Reloading** | Detect file changes and reload (dev mode) |
| **Dependency Tracking** | Load dependent assets automatically |
| **Caching** | Avoid reloading already-loaded assets |
| **Progress Reporting** | Track loading progress for UI |

### 5.4 Supported Asset Types

| Type | File Formats | GPU Resource |
|------|--------------|--------------|
| **Texture** | PNG, JPG, BMP, TGA | Texture2D |
| **Mesh** | OBJ, glTF | VertexBuffer + IndexBuffer |
| **Audio** | WAV, OGG, MP3 | Audio Buffer |
| **Shader** | GLSL | Shader Program |
| **Font** | TTF, OTF | Texture Atlas |
| **TiledMap** | TMX, JSON | TileMap Data |
| **Animation** | Custom JSON | Animation Clips |

---

## 6. Rendering Architecture

### 6.1 Render Backend Abstraction

The rendering system is abstracted behind a `RenderBackend` trait, enabling multiple graphics API implementations.

**Supported Backends:**
- OpenGL 3.3 Core (current, primary)
- Vulkan (future)
- Metal (future, macOS)
- WebGPU (future, web)

### 6.2 Rendering Pipeline

#### 6.2.1 2D Rendering Pipeline

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Sprites   │───▶│   Sort by   │───▶│   Batch by  │───▶│   Draw      │
│   Query     │    │   Z-Layer   │    │   Texture   │    │   Calls     │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

**Batching Strategy:**
- Sprites with same texture batched into single draw call
- Z-layer ordering preserved
- Instancing for identical sprites
- Target: <100 draw calls for 10,000 sprites

#### 6.2.2 3D Rendering Pipeline

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Meshes    │───▶│   Frustum   │───▶│   Sort by   │───▶│   Draw      │
│   Query     │    │   Culling   │    │   Material  │    │   Calls     │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                                            │
                                            ▼
                                     ┌─────────────┐
                                     │   Lighting  │
                                     │   Pass      │
                                     └─────────────┘
```

### 6.3 Material System

Materials define how surfaces appear when rendered.

**Material Properties:**
- Base color (texture or solid)
- Normal map
- Metallic/Roughness (PBR)
- Emission
- Custom shader parameters

**Shader System:**
- External shader files (not embedded strings)
- Hot-reloading in development
- Shader preprocessor with #include
- Automatic uniform binding via reflection

### 6.4 Camera System

**Unified Camera Model:**
- Single Camera component works for 2D and 3D
- Projection mode: Orthographic or Perspective
- Viewport support for split-screen
- Multiple cameras with render ordering
- Built-in effects: shake, smooth follow

### 6.5 Lighting System

**Light Types:**
- Point Light: Position + range
- Directional Light: Direction (sun-like)
- Spot Light: Position + direction + cone angle

**Lighting Features:**
- Maximum 16 lights per scene (configurable)
- Light attenuation curves
- Shadow mapping (future)
- Global ambient light

---

## 7. Physics Architecture

### 7.1 Physics Engine Integration

We will implement our own physics engine inspired by Rapier's architecture, giving us full control and no external dependencies.

**Features:**
- 2D and 3D physics support
- Rigid body dynamics
- Collision detection (broad + narrow phase)
- Multiple collider shapes
- Constraint/joint system
- Ray casting and shape queries

### 7.2 Collision Shapes

| Shape | 2D | 3D | Use Case |
|-------|----|----|----------|
| Box/Cuboid | ✓ | ✓ | Rectangular objects |
| Circle/Sphere | ✓ | ✓ | Round objects |
| Capsule | ✓ | ✓ | Characters |
| Polygon/Convex Hull | ✓ | ✓ | Complex shapes |
| Heightfield | - | ✓ | Terrain |

### 7.3 Physics Configuration

**World Settings:**
- Gravity vector
- Simulation substeps
- Sleep thresholds
- Collision layers/masks

**Body Types:**
- Dynamic: Fully simulated
- Static: Never moves
- Kinematic: Moved by code, affects others

### 7.4 Collision Events

**Event Types:**
- `CollisionStarted(entity_a, entity_b)` - Bodies began touching
- `CollisionEnded(entity_a, entity_b)` - Bodies stopped touching
- `SensorEntered(sensor, entity)` - Entity entered sensor
- `SensorExited(sensor, entity)` - Entity exited sensor

---

## 8. Audio Architecture

### 8.1 Audio System Design

Built on `rodio` for cross-platform audio playback.

**Core Features:**
- Sound effects (one-shot)
- Background music (streaming, looping)
- 3D spatial audio
- Audio mixing and volume control
- Multiple audio channels/groups

### 8.2 Audio Concepts

**AudioClip:** Raw audio data loaded from file
**AudioSource:** Component that plays audio
**AudioListener:** Component that "hears" audio (camera usually)
**AudioChannel:** Grouping for volume control (music, sfx, voice)

### 8.3 Spatial Audio

When `AudioSource.spatial = true`:
- Volume attenuates with distance
- Stereo panning based on listener orientation
- Doppler effect (optional)

---

## 9. Input Architecture

### 9.1 Action-Based Input

Abstract input into game actions, not raw keys.

**Example Mappings:**
- "Jump" → Spacebar, Gamepad A
- "Fire" → Left Mouse, Gamepad Right Trigger
- "Move" → WASD, Left Stick

### 9.2 Input Features

| Feature | Description |
|---------|-------------|
| **Key State** | Pressed, JustPressed, JustReleased |
| **Mouse** | Position, delta, buttons, scroll |
| **Gamepad** | Buttons, axes, rumble |
| **Touch** | Touch points (future) |
| **Action Mapping** | Configurable bindings |
| **Input Buffering** | Frame-perfect input capture |

### 9.3 Input Events

Events generated for action-oriented systems:
- `ActionPressed(action_name)`
- `ActionReleased(action_name)`
- `AxisChanged(axis_name, value)`

---

## 10. FFI Architecture

### 10.1 C ABI Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Stability** | Versioned API, semantic versioning |
| **Safety** | Handle-based, no raw pointers in API |
| **Performance** | Batch operations, minimal copying |
| **Clarity** | Comprehensive error codes |
| **Portability** | C ABI compatible with all languages |

### 10.2 Handle System

All engine objects accessed through typed handles:

| Handle Type | C Type | Purpose |
|-------------|--------|---------|
| `ContextHandle` | `GoudContext` | Engine instance |
| `EntityHandle` | `GoudEntity` | ECS entity |
| `SpriteHandle` | `GoudSprite` | Sprite component wrapper |
| `TextureHandle` | `GoudTexture` | Texture asset |
| `MeshHandle` | `GoudMesh` | Mesh asset |
| `AudioHandle` | `GoudAudio` | Audio source |
| `BodyHandle` | `GoudBody` | Physics body |
| `ColliderHandle` | `GoudCollider` | Physics collider |

### 10.3 Error Handling

Comprehensive error codes returned from all FFI functions:

| Range | Category | Examples |
|-------|----------|----------|
| 0 | Success | Ok |
| 1-99 | Context | InvalidContext, NotInitialized |
| 100-199 | Resource | NotFound, LoadFailed, InvalidHandle |
| 200-299 | Graphics | ShaderFailed, TextureFailed |
| 300-399 | Entity | EntityNotFound, ComponentNotFound |
| 400-499 | Input | InvalidParameter, NullPointer |
| 500-599 | System | WindowFailed, AudioFailed, PhysicsFailed |
| 900-999 | Internal | InternalError, NotImplemented |

### 10.4 FFI Module Organization

```
src/ffi/
├── mod.rs              # FFI module root, exports
├── context.rs          # Context lifecycle
├── entities.rs         # Entity operations
├── components.rs       # Component access
├── resources.rs        # Asset loading/unloading
├── rendering.rs        # Camera, lights
├── physics.rs          # Physics operations
├── audio.rs            # Audio operations
├── input.rs            # Input queries
├── batch.rs            # Batch operations
└── utils.rs            # String conversion, helpers
```

### 10.5 Batch Operations

For performance-critical operations, batch APIs reduce FFI overhead:

**Batch Create:**
- Create multiple entities in single call
- Attach multiple components in single call

**Batch Update:**
- Update multiple transforms in single call
- Update multiple sprite properties in single call

**Batch Query:**
- Query multiple entity positions in single call
- Query multiple collision states in single call

**Performance Target:** 10x speedup vs individual calls

---

## 11. C# SDK Architecture

### 11.1 SDK Design Goals

Make C# feel like first-class citizen, similar to MonoGame/XNA experience.

**Design Principles:**
- Familiar C# idioms (properties, events, LINQ)
- Builder patterns for complex objects
- Automatic resource management (IDisposable)
- Strong typing (no raw uints)
- Exception-based error handling
- Comprehensive XML documentation

### 11.2 SDK Structure

```
GoudEngine/
├── Core/
│   ├── GoudContext.cs      # Main entry point
│   ├── World.cs            # ECS world access
│   ├── Entity.cs           # Entity operations
│   └── Handle.cs           # Type-safe handles
├── Components/
│   ├── Transform.cs        # Transform component
│   ├── Sprite.cs           # Sprite component
│   ├── Camera.cs           # Camera component
│   ├── Light.cs            # Light component
│   ├── RigidBody.cs        # Physics body
│   ├── Collider.cs         # Physics collider
│   └── AudioSource.cs      # Audio component
├── Assets/
│   ├── AssetManager.cs     # Asset loading
│   ├── Texture.cs          # Texture wrapper
│   ├── Mesh.cs             # Mesh wrapper
│   └── AudioClip.cs        # Audio wrapper
├── Input/
│   ├── InputManager.cs     # Input access
│   ├── ActionMap.cs        # Action bindings
│   └── Keys.cs             # Key enum
├── Math/
│   ├── Vector2.cs          # 2D vector
│   ├── Vector3.cs          # 3D vector
│   ├── Quaternion.cs       # Rotation
│   ├── Matrix4.cs          # 4x4 matrix
│   ├── Color.cs            # RGBA color
│   └── Rectangle.cs        # Rectangle
├── Builders/
│   ├── SpriteBuilder.cs    # Sprite creation
│   ├── LightBuilder.cs     # Light creation
│   ├── BodyBuilder.cs      # Physics body
│   └── MaterialBuilder.cs  # Material creation
├── Events/
│   ├── EventBus.cs         # Event subscription
│   └── CollisionEvents.cs  # Physics events
└── Native/
    ├── NativeMethods.g.cs  # Auto-generated bindings
    └── NativeUtils.cs      # Marshaling helpers
```

### 11.3 API Examples (Conceptual)

**Basic Game Loop:**
```
// Initialization
context.Initialize()
context.Run()  // Internally: while !ShouldClose { BeginFrame; Update; EndFrame }

// Entity creation with builder
entity = world.Spawn()
    .With(Transform.At(100, 200))
    .With(Sprite.From(texture).Color(Color.White))
    .Build()

// System registration
world.AddSystem<MovementSystem>()
world.AddSystem<RenderingSystem>()
```

**Resource Loading:**
```
// Async loading with progress
task = assets.LoadAsync<Texture>("player.png")
await task

// Immediate loading
texture = assets.Load<Texture>("player.png")
```

### 11.4 MonoGame-Inspired Features

| Feature | Implementation |
|---------|----------------|
| **SpriteBatch-style API** | Batch drawing with Begin/End |
| **Content Pipeline** | Asset preprocessing and packaging |
| **Game Class Pattern** | Familiar Initialize/Update/Draw lifecycle |
| **Vector/Matrix Types** | Matching API to XNA math types |
| **Rectangle Helpers** | Intersects, Contains, etc. |

---

## 12. Multi-Language Binding Strategy

### 12.1 Binding Generation

**C Header (cbindgen):**
- Automatically generate goud_api.h from Rust FFI
- Version number in header
- All types and functions documented

**C# (csbindgen):**
- Current approach, already working
- Enhanced with more type safety

**Python (future):**
- Use ctypes or cffi
- Pythonic wrappers over C API

**Lua (future):**
- Use LuaJIT FFI or C bindings
- Game scripting focus

**TypeScript/Node (future):**
- Use node-ffi or N-API
- Editor tooling focus

**Go (future):**
- Use cgo
- Server-side game logic

### 12.2 Language Binding Levels

| Level | Description | Languages |
|-------|-------------|-----------|
| **L0: C ABI** | Raw FFI functions | All |
| **L1: Safe Wrapper** | Handle validation, error conversion | C#, Python |
| **L2: Idiomatic API** | Language-native patterns | C#, Rust |
| **L3: High-Level DSL** | Game-specific abstractions | Lua |

---

## 13. Testing Strategy

### 13.1 Test Categories

| Category | Coverage Target | Tools |
|----------|-----------------|-------|
| **Unit Tests** | >80% | cargo test |
| **Integration Tests** | Key paths | cargo test --test integration |
| **FFI Tests** | All functions | C# test project |
| **Rendering Tests** | Visual regression | Snapshot comparison |
| **Performance Tests** | Benchmark tracking | criterion |
| **Fuzz Tests** | FFI inputs | cargo-fuzz |

### 13.2 TDD Workflow

For each new feature:
1. Write failing test that specifies behavior
2. Implement minimum code to pass
3. Refactor while tests stay green
4. Document with examples

### 13.3 Test Infrastructure

**Headless Testing:**
- Mock OpenGL context for CI
- Offscreen rendering for visual tests

**Benchmark Tracking:**
- Store benchmark results in CI
- Alert on performance regressions

**Coverage Reporting:**
- cargo tarpaulin for Rust
- Report uploaded to coverage service

---

## 14. Development Phases

### Phase 1: Foundation (Estimated Effort: Large)

**Objectives:**
- Implement custom ECS core
- Refactor FFI layer with handle system
- Implement comprehensive error handling
- Establish TDD infrastructure

**Deliverables:**
- Entity/Component/System core
- Archetype storage
- Query system
- New FFI module structure
- Error types and propagation
- 80%+ test coverage on new code

### Phase 2: Core Systems (Estimated Effort: Large)

**Objectives:**
- Integrate built-in components
- Implement system scheduler
- Add resource management
- Implement transform hierarchy

**Deliverables:**
- All built-in components
- Parallel system execution
- Asset loading pipeline
- Parent/child transforms

### Phase 3: Graphics Enhancement (Estimated Effort: Medium)

**Objectives:**
- Implement render backend abstraction
- Add sprite batching
- Implement material system
- Enhance camera system

**Deliverables:**
- RenderBackend trait + OpenGL impl
- <100 draw calls for 10K sprites
- Material/shader system
- Camera effects

### Phase 4: Physics & Audio (Estimated Effort: Medium)

**Objectives:**
- Implement physics engine
- Integrate audio system
- Add collision events
- Implement spatial audio

**Deliverables:**
- 2D/3D physics
- Audio playback
- Collision detection events
- 3D audio positioning

### Phase 5: Developer Experience (Estimated Effort: Medium)

**Objectives:**
- Implement builder patterns
- Create comprehensive docs
- Update C# SDK
- Create example games

**Deliverables:**
- Fluent APIs everywhere
- Full API documentation
- Updated SDK with new features
- 3+ polished examples

### Phase 6: Polish & Optimization (Estimated Effort: Small)

**Objectives:**
- Performance optimization
- Memory optimization
- API stability
- Release preparation

**Deliverables:**
- Meet all performance targets
- Zero memory leaks
- Frozen API
- Migration guide

---

## 15. Success Metrics

### 15.1 Performance Targets

| Metric | Target |
|--------|--------|
| Sprites rendered | 100,000 @ 60fps |
| Draw calls for 10K sprites (same texture) | <100 |
| Entity creation/destruction | >100,000/second |
| Component queries | >1M entities/frame |
| Physics bodies simulated | 10,000 @ 60fps |
| Audio latency | <50ms |
| Asset load time (100MB) | <2 seconds |
| FFI batch vs single | 10x speedup |

### 15.2 Quality Targets

| Metric | Target |
|--------|--------|
| Rust test coverage | >80% |
| C# test coverage | >70% |
| Documentation coverage | 100% public API |
| CI build time | <10 minutes |
| Memory leaks | Zero known |
| Thread safety issues | Zero known |

### 15.3 Developer Experience Targets

| Metric | Target |
|--------|--------|
| Time to hello world | <5 minutes |
| Breaking changes per minor version | 0 |
| Examples compile and run | 100% |
| Binding generation success | 100% |

---

## 16. Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Custom ECS complexity | High | Thorough testing, Bevy as reference |
| OpenGL deprecation (macOS) | High | Backend abstraction enables alternatives |
| FFI complexity | Medium | Comprehensive testing, automation |
| Scope creep | High | Strict phase boundaries |
| Performance regression | Medium | Automated benchmarks in CI |
| Memory safety across FFI | High | Handle-based design, fuzzing |

---

## 17. Appendices

### Appendix A: Glossary

| Term | Definition |
|------|------------|
| **Archetype** | Unique combination of component types |
| **Component** | Pure data attached to an entity |
| **Entity** | Lightweight ID representing a game object |
| **Handle** | Type-safe, validated reference to a resource |
| **Query** | Request for entities matching component criteria |
| **Resource** | Singleton data outside entity-component model |
| **System** | Function that processes component data |
| **World** | Container for all entities, components, and resources |

### Appendix B: File Structure (Target)

```
goud_engine/
├── src/
│   ├── lib.rs                      # Crate root
│   ├── prelude.rs                  # Common re-exports
│   │
│   ├── core/                       # Core engine types
│   │   ├── mod.rs
│   │   ├── context.rs              # Application context
│   │   ├── error.rs                # Error types
│   │   ├── handle.rs               # Handle system
│   │   └── event.rs                # Event bus
│   │
│   ├── ecs/                        # Custom ECS implementation
│   │   ├── mod.rs
│   │   ├── world.rs                # World container
│   │   ├── entity.rs               # Entity management
│   │   ├── component.rs            # Component traits
│   │   ├── archetype.rs            # Archetype storage
│   │   ├── query.rs                # Query system
│   │   ├── system.rs               # System traits
│   │   ├── schedule.rs             # System scheduling
│   │   ├── resource.rs             # Resource storage
│   │   └── commands.rs             # Deferred operations
│   │
│   ├── components/                 # Built-in components
│   │   ├── mod.rs
│   │   ├── transform.rs            # Transform components
│   │   ├── hierarchy.rs            # Parent/Children
│   │   ├── rendering.rs            # Sprite, Mesh, Camera, Light
│   │   ├── physics.rs              # RigidBody, Collider
│   │   └── audio.rs                # AudioSource, AudioListener
│   │
│   ├── systems/                    # Built-in systems
│   │   ├── mod.rs
│   │   ├── transform.rs            # Transform propagation
│   │   ├── rendering.rs            # Render systems
│   │   ├── physics.rs              # Physics systems
│   │   └── audio.rs                # Audio systems
│   │
│   ├── graphics/                   # Rendering subsystem
│   │   ├── mod.rs
│   │   ├── backend/
│   │   │   ├── mod.rs              # Backend trait
│   │   │   └── opengl/             # OpenGL implementation
│   │   ├── renderer.rs             # High-level renderer
│   │   ├── batch.rs                # Draw batching
│   │   ├── material.rs             # Material system
│   │   ├── shader.rs               # Shader management
│   │   ├── texture.rs              # Texture handling
│   │   ├── camera.rs               # Camera utilities
│   │   └── light.rs                # Lighting utilities
│   │
│   ├── physics/                    # Physics subsystem
│   │   ├── mod.rs
│   │   ├── world.rs                # Physics world
│   │   ├── body.rs                 # Rigid bodies
│   │   ├── collider.rs             # Collision shapes
│   │   ├── broad_phase.rs          # Broad phase detection
│   │   ├── narrow_phase.rs         # Narrow phase detection
│   │   └── queries.rs              # Raycasting, etc.
│   │
│   ├── audio/                      # Audio subsystem
│   │   ├── mod.rs
│   │   ├── manager.rs              # Audio manager
│   │   ├── source.rs               # Audio sources
│   │   └── spatial.rs              # Spatial audio
│   │
│   ├── input/                      # Input subsystem
│   │   ├── mod.rs
│   │   ├── manager.rs              # Input manager
│   │   ├── action_map.rs           # Action mapping
│   │   └── devices.rs              # Input devices
│   │
│   ├── assets/                     # Asset management
│   │   ├── mod.rs
│   │   ├── manager.rs              # Asset manager
│   │   ├── loader.rs               # Async loading
│   │   └── cache.rs                # Asset caching
│   │
│   ├── platform/                   # Platform abstraction
│   │   ├── mod.rs
│   │   ├── window.rs               # Window management
│   │   └── time.rs                 # Time utilities
│   │
│   └── ffi/                        # FFI layer
│       ├── mod.rs
│       ├── context.rs
│       ├── entities.rs
│       ├── components.rs
│       ├── resources.rs
│       ├── rendering.rs
│       ├── physics.rs
│       ├── audio.rs
│       ├── input.rs
│       ├── batch.rs
│       └── utils.rs
│
├── tests/                          # Integration tests
│   ├── ecs_tests.rs
│   ├── rendering_tests.rs
│   ├── physics_tests.rs
│   └── ffi_tests.rs
│
├── benches/                        # Benchmarks
│   ├── ecs_bench.rs
│   ├── rendering_bench.rs
│   └── ffi_bench.rs
│
└── examples/                       # Rust examples
    ├── hello_world.rs
    ├── sprites.rs
    └── physics.rs
```

### Appendix C: References

- Bevy ECS Architecture: https://bevyengine.org/learn/book/ecs/
- Data-Oriented Design: https://www.dataorienteddesign.com/dodbook/
- MonoGame Documentation: https://docs.monogame.net/
- Rapier Physics: https://rapier.rs/
- Rodio Audio: https://docs.rs/rodio/

---

*Document Version: 2.0*
*Last Updated: 2026-01-04*
*Status: Approved for Implementation*
