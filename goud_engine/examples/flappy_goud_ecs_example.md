# Flappy Goud - Complete ECS Implementation

A complete Flappy Bird clone using GoudEngine's ECS architecture with sprite batching, physics, and collision detection.

## Features

- Component-based entity architecture
- Automatic sprite batching for performance
- Physics simulation with gravity
- Collision detection system
- Input action mapping
- Score tracking
- Game state management
- Infinite scrolling background

## Project Structure

```
flappy_goud_ecs/
├── src/
│   ├── main.rs              # Entry point and game loop
│   ├── components.rs        # Game components
│   ├── systems.rs           # Game systems
│   ├── resources.rs         # Game resources
│   └── constants.rs         # Game constants
└── assets/
    └── sprites/
        ├── background-day.png
        ├── base.png
        ├── bird.png
        └── pipe.png
```

## Complete Implementation

### constants.rs

```rust
// Game constants
pub const SCREEN_WIDTH: f32 = 288.0;
pub const SCREEN_HEIGHT: f32 = 512.0;
pub const BASE_HEIGHT: f32 = 112.0;
pub const TARGET_FPS: f32 = 60.0;

// Physics
pub const GRAVITY: f32 = -980.0;
pub const FLAP_STRENGTH: f32 = 300.0;
pub const BIRD_START_X: f32 = 100.0;
pub const BIRD_START_Y: f32 = 300.0;

// Pipes
pub const PIPE_SPEED: f32 = 100.0;
pub const PIPE_SPAWN_INTERVAL: f32 = 2.0;
pub const PIPE_GAP: f32 = 150.0;
pub const PIPE_MIN_Y: f32 = 50.0;
pub const PIPE_MAX_Y: f32 = 350.0;
```

### components.rs

```rust
use goud_engine::ecs::Component;
use goud_engine::core::math::Vec2;

/// The player-controlled bird
#[derive(Component, Clone, Copy, Debug)]
pub struct Bird {
    pub velocity: Vec2,
    pub flap_strength: f32,
}

impl Bird {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::zero(),
            flap_strength: crate::constants::FLAP_STRENGTH,
        }
    }

    pub fn flap(&mut self) {
        self.velocity.y = self.flap_strength;
    }
}

/// Scrolling pipes (obstacles)
#[derive(Component, Clone, Copy, Debug)]
pub struct Pipe {
    pub speed: f32,
    pub is_top: bool,
    pub scored: bool,
}

impl Pipe {
    pub fn new(is_top: bool) -> Self {
        Self {
            speed: crate::constants::PIPE_SPEED,
            is_top,
            scored: false,
        }
    }
}

/// Ground/base that scrolls
#[derive(Component, Clone, Copy, Debug)]
pub struct Ground {
    pub speed: f32,
}

impl Ground {
    pub fn new() -> Self {
        Self {
            speed: crate::constants::PIPE_SPEED,
        }
    }
}

/// Static background
#[derive(Component, Clone, Copy, Debug)]
pub struct Background;

/// Tag component for entities that should despawn when off-screen
#[derive(Component, Clone, Copy, Debug)]
pub struct DespawnOffScreen;
```

### resources.rs

```rust
use goud_engine::core::math::Vec2;

/// Game state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Menu,
    Playing,
    GameOver,
}

/// Score tracking
#[derive(Debug, Clone)]
pub struct Score {
    pub current: u32,
    pub high_score: u32,
}

impl Score {
    pub fn new() -> Self {
        Self {
            current: 0,
            high_score: 0,
        }
    }

    pub fn reset(&mut self) {
        if self.current > self.high_score {
            self.high_score = self.current;
        }
        self.current = 0;
    }

    pub fn increment(&mut self) {
        self.current += 1;
    }
}

/// Pipe spawning timer
#[derive(Debug, Clone)]
pub struct PipeSpawner {
    pub timer: f32,
    pub interval: f32,
    pub next_spawn_x: f32,
}

impl PipeSpawner {
    pub fn new() -> Self {
        Self {
            timer: 0.0,
            interval: crate::constants::PIPE_SPAWN_INTERVAL,
            next_spawn_x: crate::constants::SCREEN_WIDTH + 50.0,
        }
    }

    pub fn should_spawn(&self) -> bool {
        self.timer >= self.interval
    }

    pub fn reset_timer(&mut self) {
        self.timer = 0.0;
    }
}

/// Game state manager
#[derive(Debug)]
pub struct GameStateManager {
    pub state: GameState,
}

impl GameStateManager {
    pub fn new() -> Self {
        Self {
            state: GameState::Menu,
        }
    }

    pub fn start_game(&mut self) {
        self.state = GameState::Playing;
    }

    pub fn game_over(&mut self) {
        self.state = GameState::GameOver;
    }

    pub fn reset(&mut self) {
        self.state = GameState::Menu;
    }

    pub fn is_playing(&self) -> bool {
        self.state == GameState::Playing
    }
}
```

### systems.rs

```rust
use goud_engine::ecs::{World, Query, Entity};
use goud_engine::ecs::components::{Sprite, Transform2D};
use goud_engine::ecs::InputManager;
use goud_engine::assets::{AssetServer, AssetHandle};
use goud_engine::assets::loaders::TextureAsset;
use goud_engine::core::math::{Vec2, Rect};
use crate::components::*;
use crate::resources::*;
use crate::constants::*;

/// Handle player input (flapping)
pub fn bird_input_system(world: &mut World) {
    // Only process input when playing
    let state = world.resource::<GameStateManager>();
    if !state.is_playing() {
        return;
    }

    let input = world.resource::<InputManager>();

    // Flap on space/click
    if input.action_just_pressed("Jump") {
        for (entity, mut bird) in world.query::<(Entity, &mut Bird)>().iter() {
            bird.flap();
        }
    }
}

/// Apply gravity and update bird position
pub fn bird_physics_system(world: &mut World, delta_time: f32) {
    // Only update physics when playing
    let state = world.resource::<GameStateManager>();
    if !state.is_playing() {
        return;
    }

    for (entity, mut bird, mut transform) in world.query::<(Entity, &mut Bird, &mut Transform2D)>().iter() {
        // Apply gravity
        bird.velocity.y += GRAVITY * delta_time;

        // Apply velocity
        transform.translate(bird.velocity * delta_time);

        // Rotate bird based on velocity (visual feedback)
        let rotation = (bird.velocity.y / 300.0).clamp(-1.5, 0.5);
        transform.set_rotation(rotation);
    }
}

/// Move pipes and ground horizontally
pub fn scrolling_system(world: &mut World, delta_time: f32) {
    let state = world.resource::<GameStateManager>();
    if !state.is_playing() {
        return;
    }

    // Move pipes
    for (entity, pipe, mut transform) in world.query::<(Entity, &Pipe, &mut Transform2D)>().iter() {
        transform.translate(Vec2::new(-pipe.speed * delta_time, 0.0));
    }

    // Move ground (infinite scroll)
    for (entity, ground, mut transform) in world.query::<(Entity, &Ground, &mut Transform2D)>().iter() {
        transform.translate(Vec2::new(-ground.speed * delta_time, 0.0));

        // Wrap around when off-screen
        let pos = transform.position();
        if pos.x < -SCREEN_WIDTH {
            transform.set_position(Vec2::new(0.0, pos.y));
        }
    }
}

/// Spawn new pipes
pub fn pipe_spawner_system(world: &mut World, asset_server: &AssetServer, delta_time: f32) {
    let state = world.resource::<GameStateManager>();
    if !state.is_playing() {
        return;
    }

    let mut spawner = world.resource_mut::<PipeSpawner>();
    spawner.timer += delta_time;

    if spawner.should_spawn() {
        spawner.reset_timer();

        // Random gap position
        let gap_y = rand::random::<f32>() * (PIPE_MAX_Y - PIPE_MIN_Y) + PIPE_MIN_Y;
        let pipe_texture = asset_server.get_handle_by_path::<TextureAsset>("pipe.png").unwrap();

        // Spawn top pipe
        world.spawn()
            .insert(Pipe::new(true))
            .insert(DespawnOffScreen)
            .insert(Sprite::new(pipe_texture).with_flip_y(true))
            .insert(Transform2D::from_position(Vec2::new(
                spawner.next_spawn_x,
                gap_y - PIPE_GAP / 2.0 - 320.0, // Pipe height ~320px
            )))
            .id();

        // Spawn bottom pipe
        world.spawn()
            .insert(Pipe::new(false))
            .insert(DespawnOffScreen)
            .insert(Sprite::new(pipe_texture))
            .insert(Transform2D::from_position(Vec2::new(
                spawner.next_spawn_x,
                gap_y + PIPE_GAP / 2.0,
            )))
            .id();
    }
}

/// Despawn entities that are off-screen
pub fn despawn_offscreen_system(world: &mut World) {
    let mut to_despawn = Vec::new();

    for (entity, transform) in world.query::<(Entity, &Transform2D)>().iter() {
        if world.has::<DespawnOffScreen>(entity) {
            let pos = transform.position();
            if pos.x < -100.0 {
                to_despawn.push(entity);
            }
        }
    }

    for entity in to_despawn {
        world.despawn(entity);
    }
}

/// Detect collisions between bird and pipes/ground/ceiling
pub fn collision_system(world: &mut World) {
    let state = world.resource::<GameStateManager>();
    if !state.is_playing() {
        return;
    }

    // Get bird bounds
    let mut bird_entity = None;
    let mut bird_bounds = Rect::default();

    for (entity, transform, sprite) in world.query::<(Entity, &Transform2D, &Sprite)>().iter() {
        if world.has::<Bird>(entity) {
            bird_entity = Some(entity);
            bird_bounds = compute_sprite_bounds(transform, sprite);
            break;
        }
    }

    if let Some(bird_entity) = bird_entity {
        let bird_y = bird_bounds.y;

        // Check ceiling collision
        if bird_y < 0.0 {
            game_over(world);
            return;
        }

        // Check ground collision
        if bird_y + bird_bounds.height > SCREEN_HEIGHT {
            game_over(world);
            return;
        }

        // Check pipe collisions
        for (entity, transform, sprite) in world.query::<(Entity, &Transform2D, &Sprite)>().iter() {
            if world.has::<Pipe>(entity) {
                let pipe_bounds = compute_sprite_bounds(transform, sprite);
                if bird_bounds.intersects(&pipe_bounds) {
                    game_over(world);
                    return;
                }
            }
        }
    }
}

/// Track score when bird passes pipes
pub fn scoring_system(world: &mut World) {
    let state = world.resource::<GameStateManager>();
    if !state.is_playing() {
        return;
    }

    // Get bird X position
    let mut bird_x = BIRD_START_X;
    for (entity, transform) in world.query::<(Entity, &Transform2D)>().iter() {
        if world.has::<Bird>(entity) {
            bird_x = transform.position().x;
            break;
        }
    }

    // Check if bird passed any pipes
    let mut score = world.resource_mut::<Score>();
    for (entity, mut pipe, transform) in world.query::<(Entity, &mut Pipe, &Transform2D)>().iter() {
        if !pipe.scored && !pipe.is_top {
            let pipe_x = transform.position().x;
            if bird_x > pipe_x + 52.0 { // Pipe width ~52px
                pipe.scored = true;
                score.increment();
                println!("Score: {}", score.current);
            }
        }
    }
}

/// Game over handler
fn game_over(world: &mut World) {
    let mut state = world.resource_mut::<GameStateManager>();
    state.game_over();

    let mut score = world.resource_mut::<Score>();
    score.reset();

    println!("Game Over! Score: {}", score.current);
}

/// Compute sprite AABB from transform and sprite
fn compute_sprite_bounds(transform: &Transform2D, sprite: &Sprite) -> Rect {
    let pos = transform.position();
    let size = sprite.custom_size.unwrap_or_else(|| {
        // Use source rect size if available, otherwise default to 32x32
        sprite.source_rect.map(|r| Vec2::new(r.width, r.height))
            .unwrap_or(Vec2::new(32.0, 32.0))
    });

    Rect::new(pos.x, pos.y, size.x, size.y)
}
```

### main.rs

```rust
use goud_engine::ecs::{World, SpriteRenderSystem};
use goud_engine::ecs::components::{Sprite, Transform2D};
use goud_engine::ecs::InputManager;
use goud_engine::assets::{AssetServer, loaders::TextureLoader};
use goud_engine::libs::graphics::backend::OpenGLBackend;
use goud_engine::core::math::{Vec2, Color};

mod components;
mod systems;
mod resources;
mod constants;

use components::*;
use systems::*;
use resources::*;
use constants::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Flappy Goud...");

    // Initialize ECS world
    let mut world = World::new();

    // Initialize asset server
    let mut asset_server = AssetServer::new();
    asset_server.register_loader(TextureLoader::default());

    // Initialize rendering backend
    let backend = OpenGLBackend::new()?;
    let mut render_system = SpriteRenderSystem::new(backend)?;

    // Initialize input manager
    let mut input = InputManager::new();
    input.map_action("Jump", InputBinding::Key(Keys::Space));
    input.map_action("Jump", InputBinding::MouseButton(MouseButton::Left));
    input.map_action("Restart", InputBinding::Key(Keys::R));
    input.map_action("Quit", InputBinding::Key(Keys::Escape));

    // Insert resources
    world.insert_resource(GameStateManager::new());
    world.insert_resource(Score::new());
    world.insert_resource(PipeSpawner::new());
    world.insert_resource(input);

    // Load assets
    let bg_texture = asset_server.load::<TextureAsset>("assets/sprites/background-day.png");
    let base_texture = asset_server.load::<TextureAsset>("assets/sprites/base.png");
    let bird_texture = asset_server.load::<TextureAsset>("assets/sprites/bird.png");
    let pipe_texture = asset_server.load::<TextureAsset>("assets/sprites/pipe.png");

    // Setup initial scene
    setup_scene(&mut world, &asset_server)?;

    // Game loop
    let mut last_time = std::time::Instant::now();
    loop {
        let now = std::time::Instant::now();
        let delta_time = (now - last_time).as_secs_f32();
        last_time = now;

        // Handle input
        let input = world.resource::<InputManager>();
        if input.action_just_pressed("Quit") {
            break;
        }
        if input.action_just_pressed("Restart") {
            restart_game(&mut world, &asset_server)?;
        }

        // Update systems
        bird_input_system(&mut world);
        bird_physics_system(&mut world, delta_time);
        scrolling_system(&mut world, delta_time);
        pipe_spawner_system(&mut world, &asset_server, delta_time);
        collision_system(&mut world);
        scoring_system(&mut world);
        despawn_offscreen_system(&mut world);

        // Render
        render_system.run(&world, &asset_server)?;

        // Update input for next frame
        world.resource_mut::<InputManager>().update();

        // Target 60 FPS
        std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / TARGET_FPS));
    }

    println!("Game terminated.");
    Ok(())
}

/// Setup initial scene entities
fn setup_scene(world: &mut World, asset_server: &AssetServer) -> Result<(), Box<dyn std::error::Error>> {
    let bg_texture = asset_server.get_handle_by_path::<TextureAsset>("background-day.png").unwrap();
    let base_texture = asset_server.get_handle_by_path::<TextureAsset>("base.png").unwrap();
    let bird_texture = asset_server.get_handle_by_path::<TextureAsset>("bird.png").unwrap();

    // Spawn background
    world.spawn()
        .insert(Background)
        .insert(Sprite::new(bg_texture))
        .insert(Transform2D::from_position(Vec2::new(0.0, 0.0)))
        .id();

    // Spawn ground
    world.spawn()
        .insert(Ground::new())
        .insert(Sprite::new(base_texture))
        .insert(Transform2D::from_position(Vec2::new(0.0, SCREEN_HEIGHT)))
        .id();

    // Spawn bird
    world.spawn()
        .insert(Bird::new())
        .insert(Sprite::new(bird_texture))
        .insert(Transform2D::from_position(Vec2::new(BIRD_START_X, BIRD_START_Y)))
        .id();

    Ok(())
}

/// Restart game by despawning pipes and resetting bird
fn restart_game(world: &mut World, asset_server: &AssetServer) -> Result<(), Box<dyn std::error::Error>> {
    // Despawn all pipes
    let mut to_despawn = Vec::new();
    for (entity, _) in world.query::<(Entity, &Pipe)>().iter() {
        to_despawn.push(entity);
    }
    for entity in to_despawn {
        world.despawn(entity);
    }

    // Reset bird position and velocity
    for (entity, mut bird, mut transform) in world.query::<(Entity, &mut Bird, &mut Transform2D)>().iter() {
        bird.velocity = Vec2::zero();
        transform.set_position(Vec2::new(BIRD_START_X, BIRD_START_Y));
        transform.set_rotation(0.0);
    }

    // Reset resources
    world.resource_mut::<GameStateManager>().start_game();
    world.resource_mut::<Score>().reset();
    world.resource_mut::<PipeSpawner>().timer = 0.0;

    Ok(())
}
```

## Running the Game

```bash
cargo run --release
```

## Controls

- **Space / Left Click**: Flap
- **R**: Restart game
- **Escape**: Quit

## Performance

- **Rendering**: All sprites batched into 2-4 draw calls
- **Physics**: Simple gravity and velocity
- **Collision**: AABB-based detection
- **Target FPS**: 60

## Customization

### Adjust Difficulty

```rust
// In constants.rs
pub const GRAVITY: f32 = -1200.0;      // Harder (faster fall)
pub const FLAP_STRENGTH: f32 = 250.0;  // Harder (weaker flap)
pub const PIPE_SPEED: f32 = 150.0;     // Harder (faster pipes)
pub const PIPE_GAP: f32 = 120.0;       // Harder (smaller gap)
```

### Add Power-Ups

```rust
#[derive(Component)]
struct PowerUp {
    effect: PowerUpEffect,
}

enum PowerUpEffect {
    SlowMotion,
    Invincibility,
    DoubleScore,
}

fn powerup_system(world: &mut World) {
    // Spawn random power-ups
    // Handle bird collision with power-ups
    // Apply effects
}
```

### Add Sound Effects

```rust
use goud_engine::assets::AudioManager;

fn bird_input_system(world: &mut World, audio: &mut AudioManager) {
    if input.action_just_pressed("Jump") {
        audio.play("sounds/flap.wav");
        // ...
    }
}

fn collision_system(world: &mut World, audio: &mut AudioManager) {
    if collision_detected {
        audio.play("sounds/hit.wav");
        game_over(world);
    }
}
```

## See Also

- [Game Migration Guide](game_migration_guide.md) - Migrating from old API
- [ECS Rendering Example](ecs_rendering_example.md) - Rendering details
- [Input Manager Example](input_manager_example.md) - Input system usage
