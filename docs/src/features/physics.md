# Physics

GoudEngine provides 2D and 3D rigid body physics through the Rapier physics library.

## Providers

| Provider | Backend | Dimensions |
|---|---|---|
| `Rapier2DPhysicsProvider` | rapier2d | 2D |
| `Rapier3DPhysicsProvider` | rapier3d | 3D |
| `NullPhysicsProvider` | none | fallback |

## PhysicsWorld

The `PhysicsWorld` resource controls simulation parameters:

- **Timestep**: fixed at 1/60s by default
- **Iterations**: 8 velocity, 3 position (configurable)
- **Gravity**: `Vec2` for 2D, `Vec3` for 3D
- **Time scale**: 0.0–10.0 range for slow-motion or fast-forward
- **Sleeping**: idle bodies stop simulating to save CPU

The simulation uses a fixed-timestep accumulator pattern for deterministic behavior regardless of frame rate.

## Components

### RigidBody

Attached to entities that participate in physics simulation.

| Field | Type | Description |
|---|---|---|
| `body_type` | `RigidBodyType` | Dynamic, Kinematic, or Static |
| `velocity` | `Vec2` | Linear velocity |
| `mass` | `f32` | Body mass |
| `gravity_scale` | `f32` | Per-body gravity multiplier |
| `angular_velocity` | `f32` | Rotational speed |
| `linear_damping` | `f32` | Velocity decay per frame |
| `angular_damping` | `f32` | Angular velocity decay |
| `can_sleep` | `bool` | Whether the body can enter sleep state |

### Collider

Defines collision geometry attached to a body.

| Field | Type | Description |
|---|---|---|
| `shape` | `ColliderShape` | Circle, Box (AABB), Capsule, or Polygon |
| `friction` | `f32` | Surface friction coefficient |
| `restitution` | `f32` | Bounciness (0 = no bounce, 1 = full) |
| `is_sensor` | `bool` | Trigger-only (no physical response) |
| `layer` | `u32` | Collision layer bitmask |
| `mask` | `u32` | Which layers this collider interacts with |

### Collision Shapes

- `ColliderShape::Circle(radius)` — circle with given radius
- `ColliderShape::Aabb(half_extents)` — axis-aligned box
- `ColliderShape::Capsule(radius, height)` — capsule (rounded box)
- `ColliderShape::Polygon(vertices)` — convex polygon from vertex list

## Layer Filtering

Layer-based collision filtering uses bitmasks on `Collider`. A body's `layer` is compared against the other body's `mask`. If the bitwise AND is zero, the pair is skipped before narrow-phase collision detection.

## FFI

Physics FFI functions are in `goud_engine/src/ffi/physics/`. Key functions:

- `goud_physics_create()` / `goud_physics_destroy()`
- `goud_physics_set_gravity()`
- `goud_physics_add_rigid_body()` / `goud_physics_remove_body()`

Physics providers are registered globally per context ID.
