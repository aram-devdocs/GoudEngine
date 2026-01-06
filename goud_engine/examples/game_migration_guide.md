# Game Migration Guide: Old API → ECS Architecture

This guide shows how to migrate existing GoudEngine games from the old rendering API to the new ECS architecture.

## Table of Contents

1. [Migration Overview](#migration-overview)
2. [Step-by-Step Migration](#step-by-step-migration)
3. [Flappy Goud Example](#flappy-goud-example)
4. [Goud Jumper Example](#goud-jumper-example)
5. [Common Patterns](#common-patterns)
6. [Performance Comparison](#performance-comparison)

---

## Migration Overview

### Old Architecture (Pre-ECS)

```csharp
// Manual sprite management
GoudGame game = new GoudGame(800, 600, "My Game");
uint textureId = game.CreateTexture("sprite.png");
SpriteCreateDto sprite = new SpriteCreateDto {
    x = 100, y = 100, texture_id = textureId
};
game.AddSprite(sprite);

// Manual update loop
game.Update(() => {
    // Update sprite positions manually
    // Check collisions manually
});
```

**Limitations:**
- Manual sprite lifecycle management
- No component reusability
- Hard to organize game logic
- Tight coupling between rendering and logic
- No batching optimization

### New Architecture (ECS)

```rust
// Component-based entities
let world = World::new();
let texture = asset_server.load("sprite.png");

world.spawn()
    .insert(Sprite::new(texture))
    .insert(Transform2D::from_position(Vec2::new(100.0, 100.0)))
    .id();

// Automatic batched rendering
render_system.run(&world, &asset_server)?;
```

**Benefits:**
- Automatic sprite batching (10x+ faster)
- Component composition for game logic
- Clear separation of data and systems
- Reusable components across entities
- Industry-standard ECS pattern

---

## Step-by-Step Migration

### Step 1: Initialize ECS World

**Old:**
```csharp
GoudGame game = new GoudGame(800, 600, "Game Title");
GameManager gameManager = new GameManager(game);
```

**New:**
```rust
use goud_engine::ecs::World;
use goud_engine::assets::AssetServer;
use goud_engine::ecs::SpriteRenderSystem;

let mut world = World::new();
let mut asset_server = AssetServer::new();
let mut render_system = SpriteRenderSystem::new(backend)?;
```

### Step 2: Replace Sprite Creation

**Old:**
```csharp
uint textureId = game.CreateTexture("player.png");
SpriteCreateDto playerSprite = new SpriteCreateDto {
    x = 100,
    y = 200,
    z_layer = 1,
    texture_id = textureId
};
uint spriteId = game.AddSprite(playerSprite);
```

**New:**
```rust
let texture = asset_server.load::<TextureAsset>("player.png");
let player = world.spawn()
    .insert(Sprite::new(texture))
    .insert(Transform2D::from_position(Vec2::new(100.0, 200.0)))
    .id();
```

### Step 3: Replace Sprite Updates

**Old:**
```csharp
// Manual position updates
bird.Y += velocity * deltaTime;
game.UpdateSpritePosition(spriteId, bird.X, bird.Y);
```

**New:**
```rust
// Query and update components
for (entity, mut transform) in world.query::<(Entity, &mut Transform2D)>().iter() {
    transform.translate(Vec2::new(velocity.x * delta_time, velocity.y * delta_time));
}
```

### Step 4: Replace Collision Detection

**Old:**
```csharp
if (game.CheckCollision(birdSpriteId, pipeSpriteId)) {
    ResetGame();
}
```

**New:**
```rust
// Use ECS collision system (see collision.rs)
for (entity_a, transform_a, collider_a) in query_a.iter() {
    for (entity_b, transform_b, collider_b) in query_b.iter() {
        if let Some(contact) = detect_collision(collider_a, collider_b, transform_a, transform_b) {
            // Handle collision
        }
    }
}
```

### Step 5: Replace Rendering

**Old:**
```csharp
game.Update(() => {
    gameManager.Update(game.UpdateResponseData.delta_time);
    // Rendering happens automatically inside GoudGame
});
```

**New:**
```rust
loop {
    // Update game logic
    update_systems(&mut world, delta_time);

    // Render all sprites
    render_system.run(&world, &asset_server)?;

    // Swap buffers
}
```

---

## Flappy Goud Example

### Original Implementation

```csharp
// GameManager.cs
public class GameManager {
    private Bird bird;
    private List<Pipe> pipes;
    private uint BaseTextureId;

    public void Initialize() {
        uint BackgroundTextureId = game.CreateTexture("background.png");
        BaseTextureId = game.CreateTexture("base.png");

        SpriteCreateDto backgroundData = new SpriteCreateDto {
            x = 0, y = 0, z_layer = 0, texture_id = BackgroundTextureId
        };
        game.AddSprite(backgroundData);

        bird.Initialize();
    }

    public void Update(float deltaTime) {
        bird.Update(deltaTime);

        // Check collision with base
        if (game.CheckCollision(bird.GetSpriteId(), BaseTextureId)) {
            ResetGame();
        }

        // Update pipes
        foreach (var pipe in pipes) {
            pipe.Update(deltaTime);
            if (game.CheckCollision(bird.GetSpriteId(), pipe.topSpriteId)) {
                ResetGame();
            }
        }
    }
}
```

### ECS Migration

```rust
// flappy_goud_ecs.rs

use goud_engine::ecs::{World, Query};
use goud_engine::ecs::components::{Sprite, Transform2D};
use goud_engine::core::math::{Vec2, Rect};

// Components
#[derive(Component)]
struct Bird {
    velocity: Vec2,
    flap_strength: f32,
}

#[derive(Component)]
struct Pipe {
    speed: f32,
}

#[derive(Component)]
struct Ground;

// Systems
fn bird_input_system(world: &mut World, input: &InputManager) {
    for (entity, bird, mut transform) in world.query::<(Entity, &Bird, &mut Transform2D)>() {
        if input.action_just_pressed("Jump") {
            // Apply flap (handled by physics system)
        }
    }
}

fn bird_physics_system(world: &mut World, delta_time: f32) {
    const GRAVITY: f32 = -980.0;

    for (entity, mut bird, mut transform) in world.query::<(Entity, &mut Bird, &mut Transform2D)>() {
        // Apply gravity
        bird.velocity.y += GRAVITY * delta_time;

        // Apply velocity
        transform.translate(bird.velocity * delta_time);
    }
}

fn pipe_movement_system(world: &mut World, delta_time: f32) {
    for (entity, pipe, mut transform) in world.query::<(Entity, &Pipe, &mut Transform2D)>() {
        transform.translate(Vec2::new(-pipe.speed * delta_time, 0.0));
    }
}

fn collision_system(world: &mut World) {
    // Query bird
    let mut bird_entity = None;
    let mut bird_bounds = Rect::default();

    for (entity, transform, sprite) in world.query::<(Entity, &Transform2D, &Sprite)>() {
        if world.has::<Bird>(entity) {
            bird_entity = Some(entity);
            // Compute AABB from transform and sprite
            bird_bounds = compute_sprite_bounds(transform, sprite);
            break;
        }
    }

    if let Some(bird_entity) = bird_entity {
        // Check collision with pipes and ground
        for (entity, transform, sprite) in world.query::<(Entity, &Transform2D, &Sprite)>() {
            if world.has::<Pipe>(entity) || world.has::<Ground>(entity) {
                let bounds = compute_sprite_bounds(transform, sprite);
                if bird_bounds.intersects(&bounds) {
                    // Emit collision event
                    world.send_event(GameOver);
                }
            }
        }
    }
}

// Game setup
fn setup_game(world: &mut World, asset_server: &mut AssetServer) {
    // Load textures
    let bg_texture = asset_server.load("background.png");
    let bird_texture = asset_server.load("bird.png");
    let pipe_texture = asset_server.load("pipe.png");
    let base_texture = asset_server.load("base.png");

    // Spawn background
    world.spawn()
        .insert(Sprite::new(bg_texture))
        .insert(Transform2D::from_position(Vec2::new(0.0, 0.0)))
        .id();

    // Spawn bird
    world.spawn()
        .insert(Bird {
            velocity: Vec2::zero(),
            flap_strength: 300.0,
        })
        .insert(Sprite::new(bird_texture))
        .insert(Transform2D::from_position(Vec2::new(100.0, 300.0)))
        .id();

    // Spawn ground
    world.spawn()
        .insert(Ground)
        .insert(Sprite::new(base_texture))
        .insert(Transform2D::from_position(Vec2::new(0.0, 512.0)))
        .id();
}

// Game loop
fn game_loop(
    world: &mut World,
    asset_server: &mut AssetServer,
    render_system: &mut SpriteRenderSystem<OpenGLBackend>,
    input: &InputManager,
    delta_time: f32,
) {
    // Update systems
    bird_input_system(world, input);
    bird_physics_system(world, delta_time);
    pipe_movement_system(world, delta_time);
    collision_system(world);

    // Render
    render_system.run(world, asset_server)?;
}
```

---

## Goud Jumper Example

### Original Implementation

```csharp
public class GameManager {
    private Player player;
    private List<Platform> platforms;
    private TiledMap map;

    public void Update(float deltaTime) {
        player.Update(deltaTime);

        // Check platform collisions
        foreach (var platform in platforms) {
            if (CheckCollision(player.bounds, platform.bounds)) {
                player.OnGround = true;
            }
        }
    }
}
```

### ECS Migration

```rust
// goud_jumper_ecs.rs

// Components
#[derive(Component)]
struct Player {
    velocity: Vec2,
    on_ground: bool,
    jump_strength: f32,
}

#[derive(Component)]
struct Platform;

#[derive(Component)]
struct Velocity(Vec2);

// Systems
fn player_input_system(world: &mut World, input: &InputManager) {
    for (entity, mut player, transform) in world.query::<(Entity, &mut Player, &Transform2D)>() {
        // Horizontal movement
        let mut move_x = 0.0;
        if input.action_pressed("MoveLeft") {
            move_x = -200.0;
        }
        if input.action_pressed("MoveRight") {
            move_x = 200.0;
        }
        player.velocity.x = move_x;

        // Jump
        if input.action_just_pressed("Jump") && player.on_ground {
            player.velocity.y = player.jump_strength;
            player.on_ground = false;
        }
    }
}

fn physics_system(world: &mut World, delta_time: f32) {
    const GRAVITY: f32 = -980.0;

    for (entity, mut player, mut transform) in world.query::<(Entity, &mut Player, &mut Transform2D)>() {
        // Apply gravity
        if !player.on_ground {
            player.velocity.y += GRAVITY * delta_time;
        }

        // Apply velocity
        transform.translate(player.velocity * delta_time);
    }
}

fn platform_collision_system(world: &mut World) {
    // Find player
    let mut player_data = None;
    for (entity, mut player, transform, sprite) in world.query::<(Entity, &mut Player, &Transform2D, &Sprite)>() {
        let bounds = compute_sprite_bounds(transform, sprite);
        player_data = Some((entity, bounds, player.velocity.y));
    }

    if let Some((player_entity, player_bounds, velocity_y)) = player_data {
        // Only check collision if falling
        if velocity_y <= 0.0 {
            for (platform_entity, transform, sprite) in world.query::<(Entity, &Transform2D, &Sprite)>() {
                if world.has::<Platform>(platform_entity) {
                    let platform_bounds = compute_sprite_bounds(transform, sprite);

                    // Simple AABB collision
                    if player_bounds.intersects(&platform_bounds) {
                        // Get mutable player to set on_ground
                        if let Some(mut player) = world.get_mut::<Player>(player_entity) {
                            player.on_ground = true;
                            player.velocity.y = 0.0;
                        }
                    }
                }
            }
        }
    }
}

// Setup
fn setup_game(world: &mut World, asset_server: &mut AssetServer) {
    let player_texture = asset_server.load("player.png");
    let platform_texture = asset_server.load("platform.png");

    // Spawn player
    world.spawn()
        .insert(Player {
            velocity: Vec2::zero(),
            on_ground: false,
            jump_strength: 400.0,
        })
        .insert(Sprite::new(player_texture))
        .insert(Transform2D::from_position(Vec2::new(320.0, 240.0)))
        .id();

    // Spawn platforms
    for i in 0..10 {
        world.spawn()
            .insert(Platform)
            .insert(Sprite::new(platform_texture))
            .insert(Transform2D::from_position(Vec2::new(
                (i * 100) as f32,
                100.0,
            )))
            .id();
    }
}
```

---

## Common Patterns

### Pattern 1: Animated Sprites

**Old:**
```csharp
public class Bird {
    private int currentFrame = 0;
    private float animTimer = 0;

    public void Update(float deltaTime) {
        animTimer += deltaTime;
        if (animTimer > 0.1f) {
            currentFrame = (currentFrame + 1) % 3;
            // Update sprite frame manually
        }
    }
}
```

**New:**
```rust
#[derive(Component)]
struct SpriteAnimation {
    frames: Vec<Rect>,  // Source rectangles
    current_frame: usize,
    frame_duration: f32,
    timer: f32,
}

fn sprite_animation_system(world: &mut World, delta_time: f32) {
    for (entity, mut anim, mut sprite) in world.query::<(Entity, &mut SpriteAnimation, &mut Sprite)>() {
        anim.timer += delta_time;
        if anim.timer >= anim.frame_duration {
            anim.timer = 0.0;
            anim.current_frame = (anim.current_frame + 1) % anim.frames.len();
            sprite.source_rect = Some(anim.frames[anim.current_frame]);
        }
    }
}
```

### Pattern 2: Health System

**Old:**
```csharp
public class Player {
    public int health = 100;

    public void TakeDamage(int damage) {
        health -= damage;
        if (health <= 0) {
            Die();
        }
    }
}
```

**New:**
```rust
#[derive(Component)]
struct Health {
    current: i32,
    max: i32,
}

fn damage_system(world: &mut World) {
    // Use events for damage
    for event in world.read_events::<DamageEvent>() {
        if let Some(mut health) = world.get_mut::<Health>(event.entity) {
            health.current -= event.amount;
            if health.current <= 0 {
                world.send_event(DeathEvent { entity: event.entity });
            }
        }
    }
}
```

### Pattern 3: Spawning Entities

**Old:**
```csharp
private void SpawnPipe() {
    var pipe = new Pipe(game);
    pipes.Add(pipe);
}
```

**New:**
```rust
fn spawn_pipe(world: &mut World, asset_server: &AssetServer, x: f32) -> Entity {
    let pipe_texture = asset_server.load("pipe.png");

    world.spawn()
        .insert(Pipe { speed: 100.0 })
        .insert(Sprite::new(pipe_texture))
        .insert(Transform2D::from_position(Vec2::new(x, 300.0)))
        .id()
}

// In spawner system
fn pipe_spawner_system(world: &mut World, asset_server: &AssetServer, delta_time: f32) {
    // Access spawner resource
    if let Some(mut spawner) = world.get_resource_mut::<PipeSpawner>() {
        spawner.timer += delta_time;
        if spawner.timer >= spawner.interval {
            spawner.timer = 0.0;
            spawn_pipe(world, asset_server, 800.0);
        }
    }
}
```

---

## Performance Comparison

### Rendering Performance

**Old API (Renderer2D):**
- 1000 sprites = 1000 draw calls = ~30 FPS
- 10000 sprites = 10000 draw calls = ~3 FPS
- No batching, no sorting optimization

**New ECS API (SpriteBatch):**
- 1000 sprites = 10-20 draw calls = 60 FPS
- 10000 sprites = 100-200 draw calls = 60 FPS
- Automatic texture batching (100:1 ratio)
- Z-layer sorting with minimal overhead

### Memory Usage

**Old API:**
- Manual sprite management = fragmented memory
- No component reuse = duplicate data

**New ECS API:**
- Archetype storage = cache-friendly iteration
- Component composition = data reuse
- HandleMap = generational indices (prevents leaks)

### Code Organization

**Old API:**
- Tight coupling between game logic and rendering
- Hard to extend with new features
- Duplicate code across game objects

**New ECS API:**
- Systems operate on components (single responsibility)
- Easy to add new components and systems
- Component composition eliminates duplicate code

---

## Migration Checklist

- [ ] Initialize ECS World and AssetServer
- [ ] Replace sprite creation with entity spawning
- [ ] Convert game objects to components
- [ ] Implement systems for game logic
- [ ] Replace manual rendering with SpriteRenderSystem
- [ ] Migrate collision detection to ECS queries
- [ ] Set up input action mappings
- [ ] Test rendering performance (batching)
- [ ] Verify game logic correctness
- [ ] Profile and optimize (if needed)

---

## Troubleshooting

### Issue: Sprites not rendering

**Solution:** Ensure entities have both Sprite AND Transform2D components
```rust
// Missing Transform2D = sprite won't render
world.spawn()
    .insert(Sprite::new(texture))
    .insert(Transform2D::from_position(Vec2::new(100.0, 100.0))) // Required!
    .id();
```

### Issue: Z-layer sorting incorrect

**Solution:** Remember Y position determines Z-layer (bottom-to-top)
```rust
// Higher Y = drawn on top (foreground)
let foreground = Transform2D::from_position(Vec2::new(0.0, 500.0));
let background = Transform2D::from_position(Vec2::new(0.0, 0.0));
```

### Issue: Poor batching performance

**Solution:** Use texture atlases and minimize texture count
```rust
// Good: Same texture = 1 draw call
for i in 0..100 {
    world.spawn()
        .insert(Sprite::new(same_texture)) // Batched!
        .insert(Transform2D::from_position(Vec2::new(i as f32 * 10.0, 0.0)))
        .id();
}

// Bad: Different textures = 100 draw calls
for i in 0..100 {
    let texture = asset_server.load(&format!("sprite_{}.png", i)); // Not batched
    world.spawn()
        .insert(Sprite::new(texture))
        .insert(Transform2D::from_position(Vec2::new(i as f32 * 10.0, 0.0)))
        .id();
}
```

---

## Further Reading

- [ECS Rendering Example](ecs_rendering_example.md) - Detailed rendering guide
- [Input Manager Example](input_manager_example.md) - Input system usage
- [Collision System](../src/ecs/collision.rs) - Collision detection API
- [Component Reference](../src/ecs/components/) - All built-in components

---

**Migration Support:** For help migrating your game, see the examples in `goud_engine/examples/` or open an issue on GitHub.
