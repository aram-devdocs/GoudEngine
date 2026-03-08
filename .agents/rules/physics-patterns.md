---
globs:
  - "**/physics*/**"
  - "**/physics_world/**"
  - "**/ffi/physics/**"
---

# Physics Subsystem Patterns

## Architecture

- Physics uses the provider pattern: `PhysicsProvider` (2D) and `PhysicsProvider3D` (3D)
- Native implementations wrap Rapier: `Rapier2DPhysicsProvider` and `Rapier3DPhysicsProvider`
- `NullPhysicsProvider` enables headless testing without a physics backend
- `PhysicsWorld` resource lives in `ecs/physics_world/` and controls simulation parameters

## Components

- `RigidBody` — body type (dynamic/kinematic/static), velocity, mass, damping, gravity scale
- `Collider` — shape (circle, box, capsule, polygon), friction, restitution, sensor flag, layer/mask

## Simulation

- Fixed timestep accumulator pattern (1/60s default) for deterministic behavior
- Velocity iterations (8) and position iterations (3) are configurable
- Sleeping optimization: idle bodies stop simulating when below threshold
- Time scale (0.0–10.0) for slow-motion or fast-forward

## FFI

- Physics FFI in `ffi/physics/` with separate 2D and 3D modules
- Physics providers registered globally per context ID
- Material properties have dedicated FFI modules (`physics2d_material.rs`, `physics3d_material.rs`)

## Testing

- Physics math tests do not require GL context
- Simulation tests use `NullPhysicsProvider` or construct `PhysicsWorld` directly
- Test files in `ecs/physics_world/tests.rs`
