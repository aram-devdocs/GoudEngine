# GoudEngine Examples & Documentation

This directory contains comprehensive examples and documentation for using the GoudEngine game engine.

## Quick Start

**New to GoudEngine?** Start here:
1. [Game Migration Guide](game_migration_guide.md) - Migrating from old API to ECS
2. [Flappy Goud ECS Example](flappy_goud_ecs_example.md) - Complete Flappy Bird clone
3. [Goud Jumper ECS Example](goud_jumper_ecs_example.md) - Complete platformer game

## Complete Game Examples

### [Flappy Goud - Complete ECS Implementation](flappy_goud_ecs_example.md)
A complete Flappy Bird clone demonstrating:
- Component-based entity architecture
- Physics simulation with gravity
- Collision detection system
- Input action mapping
- Score tracking and game state management
- Automatic sprite batching (10x+ performance)

**Best for:** Learning ECS basics, simple physics, game state management

### [Goud Jumper - Complete Platformer](goud_jumper_ecs_example.md)
A complete platformer game demonstrating:
- Player movement and jumping physics
- Platform collision detection (including one-way platforms)
- Moving platforms and collectibles
- Camera system that follows the player
- Checkpoint system with respawning
- Parallax scrolling backgrounds

**Best for:** Advanced physics, camera systems, complex collision detection

---

## Migration & Architecture

### [Game Migration Guide](game_migration_guide.md)
Complete guide for migrating games from the old API to the new ECS architecture:
- Step-by-step migration process
- Side-by-side code comparisons (old vs new)
- Common patterns: animations, health systems, spawning
- Performance comparison (10x+ speedup with batching)
- Troubleshooting common issues
- Migration checklist

**Best for:** Updating existing games, understanding ECS benefits

---

## System-Specific Examples

### Rendering

#### [ECS Rendering Integration](ecs_rendering_example.md)
How to use the ECS system with the rendering pipeline:
- SpriteRenderSystem setup and usage
- Sprite and Transform2D components
- Z-layer sorting for correct draw order
- Texture batching for performance (100:1 ratio)
- Custom rendering configuration
- Integration with game loop

**Best for:** Understanding the rendering pipeline, optimization

### Input

#### [Input Manager](input_manager_example.md)
Basic input handling with InputManager:
- Keyboard, mouse, and gamepad input
- Frame-based state queries (pressed, just_pressed, just_released)
- Mouse position and delta tracking
- Basic usage patterns

**Best for:** Basic input handling

#### [Action Mapping](action_mapping_example.md)
Semantic input binding with action mapping:
- Map multiple inputs to single actions (e.g., Jump = Space OR W)
- Cross-platform input abstraction
- Multiple bindings per action
- Action strength queries
- Configuration and best practices

**Best for:** Flexible, rebindable controls

#### [Input Buffering](input_buffering_example.md)
Advanced input buffering for responsive controls:
- Buffer window for lenient input timing
- Fighting game-style input sequences
- Action prioritization
- Integration with action mapping

**Best for:** Fighting games, platformers requiring precise timing

#### [Gamepad Support](gamepad_example.md)
Comprehensive gamepad integration:
- Analog stick and trigger support
- Deadzone configuration
- Connection management
- Vibration/rumble control
- Multi-player support (up to 4 controllers)

**Best for:** Console-style games, local multiplayer

### Assets

#### [Hot Reloading](hot_reload_example.md)
Asset hot reloading for fast iteration:
- File system watching with debouncing
- Extension filtering
- Configuration options
- Platform-specific notes (inotify, FSEvents, etc.)
- Production vs development modes

**Best for:** Development workflow, rapid iteration

### FFI & Performance

#### [Batch Operations](batch_operations_example.md)
High-performance batch operations for entity and component manipulation:
- Entity spawning (10x speedup)
- Component operations
- Performance benchmarks
- Best practices for bulk operations

**Best for:** Spawning large numbers of entities, performance-critical code

#### [OpenGL Buffer Management](opengl_buffer_example.md)
Low-level OpenGL buffer operations:
- Vertex buffer creation and updates
- Index buffer management
- Buffer binding and state management
- Memory management best practices

**Best for:** Custom rendering, advanced graphics

#### [Texture Loading](texture_loading_example.md)
Asset loading and texture management:
- Supported formats (PNG, JPEG, BMP, etc.)
- Texture settings (flip, color space, wrap mode)
- AssetServer integration

**Best for:** Understanding asset pipeline

---

## Example Index by Topic

### Getting Started
- [Game Migration Guide](game_migration_guide.md) - **START HERE** for understanding ECS
- [Flappy Goud ECS Example](flappy_goud_ecs_example.md) - Simple complete game
- [ECS Rendering Example](ecs_rendering_example.md) - Rendering basics

### Game Development
- [Flappy Goud ECS Example](flappy_goud_ecs_example.md) - Physics, collision, game state
- [Goud Jumper ECS Example](goud_jumper_ecs_example.md) - Platformer, camera, checkpoints

### Input Systems
- [Input Manager](input_manager_example.md) - Basic input
- [Action Mapping](action_mapping_example.md) - Semantic input binding
- [Input Buffering](input_buffering_example.md) - Advanced input timing
- [Gamepad Support](gamepad_example.md) - Controller integration

### Rendering & Graphics
- [ECS Rendering Example](ecs_rendering_example.md) - Sprite batching, Z-sorting
- [OpenGL Buffer Management](opengl_buffer_example.md) - Low-level buffers
- [Texture Loading](texture_loading_example.md) - Asset loading

### Performance
- [Batch Operations](batch_operations_example.md) - Bulk entity operations
- [ECS Rendering Example](ecs_rendering_example.md) - Texture batching
- [Game Migration Guide](game_migration_guide.md) - Performance comparison

### Development Workflow
- [Hot Reloading](hot_reload_example.md) - Fast iteration
- [Game Migration Guide](game_migration_guide.md) - Migrating existing code

---

## Code Snippets

### Spawn a Sprite Entity

```rust
use goud_engine::ecs::{World, SpriteRenderSystem};
use goud_engine::ecs::components::{Sprite, Transform2D};
use goud_engine::assets::AssetServer;
use goud_engine::core::math::{Vec2, Color};

let mut world = World::new();
let mut asset_server = AssetServer::new();

let texture = asset_server.load("sprite.png");

world.spawn()
    .insert(Sprite::new(texture).with_color(Color::WHITE))
    .insert(Transform2D::from_position(Vec2::new(100.0, 100.0)))
    .id();
```

### Create a System

```rust
fn my_system(world: &mut World, delta_time: f32) {
    for (entity, mut transform) in world.query::<(Entity, &mut Transform2D)>().iter() {
        transform.translate(Vec2::new(100.0 * delta_time, 0.0));
    }
}
```

### Handle Input

```rust
use goud_engine::ecs::InputManager;

let mut input = InputManager::new();
input.map_action("Jump", InputBinding::Key(Keys::Space));

// In game loop
if input.action_just_pressed("Jump") {
    player.velocity.y = jump_strength;
}
```

### Render Sprites

```rust
use goud_engine::ecs::SpriteRenderSystem;

let mut render_system = SpriteRenderSystem::new(backend)?;

// In game loop
render_system.run(&world, &asset_server)?;

// Check performance
let (sprite_count, batch_count, ratio) = render_system.stats();
```

---

## Performance Tips

1. **Texture Batching**: Use texture atlases to maximize batching (100:1+ ratio achievable)
2. **Z-Sorting**: Disable for UI layers that don't need depth sorting
3. **Batch Operations**: Use `spawn_batch()` for spawning many entities (10x faster)
4. **Component Composition**: Prefer small, focused components over large monolithic ones
5. **System Ordering**: Group systems that access same components for cache efficiency

## Common Patterns

### Component Composition

```rust
// Good: Small, focused components
#[derive(Component)]
struct Health { current: i32, max: i32 }

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Player;

// Spawn with composition
world.spawn()
    .insert(Player)
    .insert(Health { current: 100, max: 100 })
    .insert(Velocity(Vec2::zero()))
    .insert(Sprite::new(texture))
    .insert(Transform2D::from_position(pos))
    .id();
```

### Resource Management

```rust
// Define resource
#[derive(Debug, Clone)]
struct GameScore {
    points: u32,
}

// Insert resource
world.insert_resource(GameScore { points: 0 });

// Access in system
fn scoring_system(world: &mut World) {
    let mut score = world.resource_mut::<GameScore>();
    score.points += 10;
}
```

### Event Handling

```rust
// Define event
#[derive(Clone, Debug)]
struct CollisionEvent {
    entity_a: Entity,
    entity_b: Entity,
}

// Send event
world.send_event(CollisionEvent { entity_a, entity_b });

// Read events
for event in world.read_events::<CollisionEvent>() {
    println!("Collision: {:?} and {:?}", event.entity_a, event.entity_b);
}
```

---

## Troubleshooting

### Sprites not rendering?
- Ensure entities have BOTH Sprite AND Transform2D components
- Check texture is loaded via AssetServer
- Verify render_system.run() is called in game loop

### Poor batching performance?
- Use texture atlases (pack multiple sprites into one texture)
- Check stats with `render_system.stats()`
- Enable batching in SpriteBatchConfig

### Input not working?
- Ensure InputManager is updated each frame: `input.update()`
- Check action mappings are registered before use
- Verify key/button names match InputBinding enum

### Collision detection missing?
- Use broad phase (SpatialHash) for large numbers of entities
- Implement collision response after detection
- Check AABB computation includes transform and sprite size

---

## See Also

- [Full API Documentation](../src/) - Complete Rust API reference
- [C# SDK Examples](../../sdks/GoudEngine/Examples/) - C# integration examples
- [Architecture Spec](../.zenflow/tasks/full-refactor-audit-5545/spec.md) - Engine architecture
- [GitHub Issues](https://github.com/yourusername/goudengine/issues) - Bug reports and feature requests

---

## Contributing Examples

Have a cool example? We'd love to include it!

1. Follow the existing format (markdown with code blocks)
2. Include complete, runnable code
3. Add inline comments explaining key concepts
4. Include verification steps
5. Cross-reference related examples
6. Submit a PR with your example

## License

All examples are licensed under the same license as GoudEngine (MIT).
