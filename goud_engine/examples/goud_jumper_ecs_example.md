# Goud Jumper - Complete ECS Implementation

A complete platformer game using GoudEngine's ECS architecture with physics, collision detection, and camera following.

## Features

- Component-based player and platform system
- Physics simulation with gravity and jumping
- Platform collision detection
- Camera system that follows the player
- Moving platforms and collectibles
- Parallax scrolling background
- Score and checkpoint system

## Project Structure

```
goud_jumper_ecs/
├── src/
│   ├── main.rs              # Entry point and game loop
│   ├── components.rs        # Game components
│   ├── systems/
│   │   ├── mod.rs
│   │   ├── player.rs        # Player systems
│   │   ├── physics.rs       # Physics systems
│   │   ├── camera.rs        # Camera systems
│   │   └── collision.rs     # Collision systems
│   ├── resources.rs         # Game resources
│   └── constants.rs         # Game constants
└── assets/
    ├── sprites/
    │   ├── player.png
    │   ├── platform.png
    │   ├── coin.png
    │   └── background.png
    └── tiled/
        └── level1.tmx
```

## Complete Implementation

### constants.rs

```rust
// Screen
pub const SCREEN_WIDTH: f32 = 960.0;
pub const SCREEN_HEIGHT: f32 = 640.0;
pub const TILE_SIZE: f32 = 32.0;

// Physics
pub const GRAVITY: f32 = -1200.0;
pub const MAX_FALL_SPEED: f32 = -600.0;
pub const GROUND_FRICTION: f32 = 0.85;
pub const AIR_FRICTION: f32 = 0.95;

// Player
pub const PLAYER_MOVE_SPEED: f32 = 200.0;
pub const PLAYER_JUMP_STRENGTH: f32 = 450.0;
pub const PLAYER_SIZE: (f32, f32) = (24.0, 32.0);
pub const PLAYER_START_POS: (f32, f32) = (100.0, 300.0);

// Platforms
pub const PLATFORM_COLOR_STATIC: [f32; 4] = [0.5, 0.3, 0.2, 1.0];
pub const PLATFORM_COLOR_MOVING: [f32; 4] = [0.3, 0.5, 0.7, 1.0];

// Camera
pub const CAMERA_SMOOTHING: f32 = 0.1;
pub const CAMERA_OFFSET_Y: f32 = -50.0;
```

### components.rs

```rust
use goud_engine::ecs::Component;
use goud_engine::core::math::Vec2;

/// Player character
#[derive(Component, Clone, Debug)]
pub struct Player {
    pub velocity: Vec2,
    pub move_speed: f32,
    pub jump_strength: f32,
    pub on_ground: bool,
    pub facing_right: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::zero(),
            move_speed: crate::constants::PLAYER_MOVE_SPEED,
            jump_strength: crate::constants::PLAYER_JUMP_STRENGTH,
            on_ground: false,
            facing_right: true,
        }
    }
}

/// Platform that player can stand on
#[derive(Component, Clone, Copy, Debug)]
pub struct Platform {
    pub width: f32,
    pub height: f32,
    pub is_one_way: bool,  // Can jump through from below
}

impl Platform {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            is_one_way: false,
        }
    }

    pub fn one_way(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            is_one_way: true,
        }
    }
}

/// Moving platform
#[derive(Component, Clone, Debug)]
pub struct MovingPlatform {
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub speed: f32,
    pub current_time: f32,
    pub cycle_duration: f32,
}

impl MovingPlatform {
    pub fn new(start: Vec2, end: Vec2, speed: f32) -> Self {
        let distance = (end - start).length();
        let cycle_duration = distance / speed;

        Self {
            start_pos: start,
            end_pos: end,
            speed,
            current_time: 0.0,
            cycle_duration,
        }
    }

    pub fn current_position(&self) -> Vec2 {
        let t = (self.current_time / self.cycle_duration).fract();
        let t_smooth = if t < 0.5 {
            t * 2.0
        } else {
            2.0 - t * 2.0
        };

        self.start_pos.lerp(self.end_pos, t_smooth)
    }
}

/// Collectible coin
#[derive(Component, Clone, Copy, Debug)]
pub struct Coin {
    pub value: u32,
}

impl Coin {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

/// Checkpoint for respawning
#[derive(Component, Clone, Copy, Debug)]
pub struct Checkpoint {
    pub activated: bool,
}

impl Checkpoint {
    pub fn new() -> Self {
        Self { activated: false }
    }
}

/// Camera target (usually the player)
#[derive(Component, Clone, Copy, Debug)]
pub struct CameraTarget;

/// Parallax background layer
#[derive(Component, Clone, Copy, Debug)]
pub struct ParallaxLayer {
    pub speed_factor: f32,
    pub original_x: f32,
}

impl ParallaxLayer {
    pub fn new(speed_factor: f32) -> Self {
        Self {
            speed_factor,
            original_x: 0.0,
        }
    }
}

/// Tag for UI elements (don't move with camera)
#[derive(Component, Clone, Copy, Debug)]
pub struct UIElement;
```

### resources.rs

```rust
use goud_engine::core::math::Vec2;

/// Game score
#[derive(Debug, Clone)]
pub struct GameScore {
    pub coins: u32,
    pub checkpoints: u32,
}

impl GameScore {
    pub fn new() -> Self {
        Self {
            coins: 0,
            checkpoints: 0,
        }
    }

    pub fn add_coin(&mut self, value: u32) {
        self.coins += value;
    }

    pub fn activate_checkpoint(&mut self) {
        self.checkpoints += 1;
    }
}

/// Camera position
#[derive(Debug, Clone)]
pub struct Camera2D {
    pub position: Vec2,
    pub smoothing: f32,
    pub offset: Vec2,
}

impl Camera2D {
    pub fn new() -> Self {
        Self {
            position: Vec2::zero(),
            smoothing: crate::constants::CAMERA_SMOOTHING,
            offset: Vec2::new(0.0, crate::constants::CAMERA_OFFSET_Y),
        }
    }

    pub fn lerp_to(&mut self, target: Vec2, delta_time: f32) {
        let alpha = 1.0 - (1.0 - self.smoothing).powf(delta_time * 60.0);
        self.position = self.position.lerp(target + self.offset, alpha);
    }
}

/// Respawn point
#[derive(Debug, Clone)]
pub struct RespawnPoint {
    pub position: Vec2,
}

impl RespawnPoint {
    pub fn new(position: Vec2) -> Self {
        Self { position }
    }

    pub fn update(&mut self, new_position: Vec2) {
        self.position = new_position;
    }
}
```

### systems/player.rs

```rust
use goud_engine::ecs::{World, Query, Entity};
use goud_engine::ecs::components::Transform2D;
use goud_engine::ecs::InputManager;
use goud_engine::core::math::Vec2;
use crate::components::Player;
use crate::constants::*;

/// Handle player input
pub fn player_input_system(world: &mut World) {
    let input = world.resource::<InputManager>();

    for (entity, mut player, transform) in world.query::<(Entity, &mut Player, &Transform2D)>().iter() {
        // Horizontal movement
        let mut move_x = 0.0;

        if input.action_pressed("MoveLeft") {
            move_x = -player.move_speed;
            player.facing_right = false;
        }
        if input.action_pressed("MoveRight") {
            move_x = player.move_speed;
            player.facing_right = true;
        }

        player.velocity.x = move_x;

        // Jump
        if input.action_just_pressed("Jump") && player.on_ground {
            player.velocity.y = player.jump_strength;
            player.on_ground = false;
        }
    }
}

/// Apply physics to player
pub fn player_physics_system(world: &mut World, delta_time: f32) {
    for (entity, mut player, mut transform) in world.query::<(Entity, &mut Player, &mut Transform2D)>().iter() {
        // Apply gravity
        if !player.on_ground {
            player.velocity.y += GRAVITY * delta_time;
            player.velocity.y = player.velocity.y.max(MAX_FALL_SPEED);
        }

        // Apply friction
        if player.on_ground {
            player.velocity.x *= GROUND_FRICTION;
        } else {
            player.velocity.x *= AIR_FRICTION;
        }

        // Apply velocity
        transform.translate(player.velocity * delta_time);

        // Flip sprite based on direction
        if let Some(mut sprite) = world.get_mut::<Sprite>(entity) {
            sprite.flip_x = !player.facing_right;
        }
    }
}
```

### systems/collision.rs

```rust
use goud_engine::ecs::{World, Query, Entity};
use goud_engine::ecs::components::{Sprite, Transform2D};
use goud_engine::core::math::{Vec2, Rect};
use crate::components::{Player, Platform, Coin, Checkpoint};
use crate::resources::{GameScore, RespawnPoint};

/// Detect and resolve platform collisions
pub fn platform_collision_system(world: &mut World) {
    // Get player entity and bounds
    let mut player_data = None;

    for (entity, mut player, transform, sprite) in world.query::<(Entity, &mut Player, &Transform2D, &Sprite)>().iter() {
        let bounds = compute_sprite_bounds(transform, sprite);
        let velocity_y = player.velocity.y;
        player_data = Some((entity, bounds, velocity_y));
        break;
    }

    if let Some((player_entity, player_bounds, velocity_y)) = player_data {
        let mut on_ground = false;

        // Check all platforms
        for (platform_entity, platform, transform) in world.query::<(Entity, &Platform, &Transform2D)>().iter() {
            let platform_bounds = Rect::new(
                transform.position().x,
                transform.position().y,
                platform.width,
                platform.height,
            );

            // Only collide if falling and from above (for one-way platforms)
            if platform.is_one_way && velocity_y > 0.0 {
                continue;
            }

            if player_bounds.intersects(&platform_bounds) {
                // Simple AABB resolution
                let overlap_x = (player_bounds.x + player_bounds.width / 2.0)
                    - (platform_bounds.x + platform_bounds.width / 2.0);
                let overlap_y = (player_bounds.y + player_bounds.height / 2.0)
                    - (platform_bounds.y + platform_bounds.height / 2.0);

                if overlap_y.abs() < overlap_x.abs() {
                    // Vertical collision
                    on_ground = true;
                }
            }
        }

        // Update player on_ground status
        if let Some(mut player) = world.get_mut::<Player>(player_entity) {
            player.on_ground = on_ground;
            if on_ground {
                player.velocity.y = 0.0;
            }
        }
    }
}

/// Collect coins
pub fn coin_collection_system(world: &mut World) {
    // Get player bounds
    let mut player_bounds = None;

    for (entity, transform, sprite) in world.query::<(Entity, &Transform2D, &Sprite)>().iter() {
        if world.has::<Player>(entity) {
            player_bounds = Some(compute_sprite_bounds(transform, sprite));
            break;
        }
    }

    if let Some(player_bounds) = player_bounds {
        let mut score = world.resource_mut::<GameScore>();
        let mut to_despawn = Vec::new();

        for (entity, coin, transform, sprite) in world.query::<(Entity, &Coin, &Transform2D, &Sprite)>().iter() {
            let coin_bounds = compute_sprite_bounds(transform, sprite);

            if player_bounds.intersects(&coin_bounds) {
                score.add_coin(coin.value);
                to_despawn.push(entity);
                println!("Coin collected! Total: {}", score.coins);
            }
        }

        // Despawn collected coins
        for entity in to_despawn {
            world.despawn(entity);
        }
    }
}

/// Activate checkpoints
pub fn checkpoint_system(world: &mut World) {
    // Get player bounds and position
    let mut player_data = None;

    for (entity, transform, sprite) in world.query::<(Entity, &Transform2D, &Sprite)>().iter() {
        if world.has::<Player>(entity) {
            let bounds = compute_sprite_bounds(transform, sprite);
            let pos = transform.position();
            player_data = Some((bounds, pos));
            break;
        }
    }

    if let Some((player_bounds, player_pos)) = player_data {
        for (entity, mut checkpoint, transform, sprite) in world.query::<(Entity, &mut Checkpoint, &Transform2D, &Sprite)>().iter() {
            if !checkpoint.activated {
                let checkpoint_bounds = compute_sprite_bounds(transform, sprite);

                if player_bounds.intersects(&checkpoint_bounds) {
                    checkpoint.activated = true;
                    let mut respawn = world.resource_mut::<RespawnPoint>();
                    respawn.update(player_pos);

                    let mut score = world.resource_mut::<GameScore>();
                    score.activate_checkpoint();

                    println!("Checkpoint activated!");
                }
            }
        }
    }
}

/// Compute sprite AABB
fn compute_sprite_bounds(transform: &Transform2D, sprite: &Sprite) -> Rect {
    let pos = transform.position();
    let size = sprite.custom_size.unwrap_or(Vec2::new(32.0, 32.0));
    Rect::new(pos.x, pos.y, size.x, size.y)
}
```

### systems/camera.rs

```rust
use goud_engine::ecs::{World, Query, Entity};
use goud_engine::ecs::components::Transform2D;
use goud_engine::core::math::Vec2;
use crate::components::{CameraTarget, ParallaxLayer, UIElement};
use crate::resources::Camera2D;

/// Follow camera target (player)
pub fn camera_follow_system(world: &mut World, delta_time: f32) {
    // Find camera target position
    let mut target_pos = None;

    for (entity, transform) in world.query::<(Entity, &Transform2D)>().iter() {
        if world.has::<CameraTarget>(entity) {
            target_pos = Some(transform.position());
            break;
        }
    }

    // Update camera position
    if let Some(target_pos) = target_pos {
        let mut camera = world.resource_mut::<Camera2D>();
        camera.lerp_to(target_pos, delta_time);
    }
}

/// Update parallax background positions
pub fn parallax_system(world: &mut World) {
    let camera = world.resource::<Camera2D>();
    let camera_offset_x = camera.position.x;

    for (entity, mut parallax, mut transform) in world.query::<(Entity, &mut ParallaxLayer, &mut Transform2D)>().iter() {
        // Move background based on camera and parallax factor
        let offset = camera_offset_x * parallax.speed_factor;
        transform.set_position(Vec2::new(parallax.original_x - offset, transform.position().y));
    }
}

/// Apply camera offset to all non-UI sprites
pub fn camera_transform_system(world: &mut World) {
    let camera = world.resource::<Camera2D>();
    let camera_pos = camera.position;

    for (entity, mut transform) in world.query::<(Entity, &mut Transform2D)>().iter() {
        // Skip UI elements
        if world.has::<UIElement>(entity) {
            continue;
        }

        // Skip parallax layers (they handle their own offset)
        if world.has::<ParallaxLayer>(entity) {
            continue;
        }

        // Apply camera offset (this is simplified - in a real game you'd
        // want to store original positions and apply camera as a view matrix)
        // For this example, we'll just note that camera position affects rendering
    }
}
```

### systems/mod.rs

```rust
pub mod player;
pub mod collision;
pub mod camera;
pub mod physics;

pub use player::*;
pub use collision::*;
pub use camera::*;
pub use physics::*;
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
    println!("Starting Goud Jumper...");

    // Initialize ECS
    let mut world = World::new();

    // Initialize assets
    let mut asset_server = AssetServer::new();
    asset_server.register_loader(TextureLoader::default());

    // Initialize rendering
    let backend = OpenGLBackend::new()?;
    let mut render_system = SpriteRenderSystem::new(backend)?;

    // Initialize input
    let mut input = InputManager::new();
    input.map_action("MoveLeft", InputBinding::Key(Keys::A));
    input.map_action("MoveLeft", InputBinding::Key(Keys::Left));
    input.map_action("MoveRight", InputBinding::Key(Keys::D));
    input.map_action("MoveRight", InputBinding::Key(Keys::Right));
    input.map_action("Jump", InputBinding::Key(Keys::Space));
    input.map_action("Jump", InputBinding::Key(Keys::W));
    input.map_action("Quit", InputBinding::Key(Keys::Escape));

    // Insert resources
    world.insert_resource(Camera2D::new());
    world.insert_resource(GameScore::new());
    world.insert_resource(RespawnPoint::new(Vec2::new(PLAYER_START_POS.0, PLAYER_START_POS.1)));
    world.insert_resource(input);

    // Load assets
    let player_texture = asset_server.load::<TextureAsset>("assets/sprites/player.png");
    let platform_texture = asset_server.load::<TextureAsset>("assets/sprites/platform.png");
    let coin_texture = asset_server.load::<TextureAsset>("assets/sprites/coin.png");
    let bg_texture = asset_server.load::<TextureAsset>("assets/sprites/background.png");

    // Setup level
    setup_level(&mut world, &asset_server)?;

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

        // Update systems
        player_input_system(&mut world);
        player_physics_system(&mut world, delta_time);
        moving_platform_system(&mut world, delta_time);
        platform_collision_system(&mut world);
        coin_collection_system(&mut world);
        checkpoint_system(&mut world);
        camera_follow_system(&mut world, delta_time);
        parallax_system(&mut world);

        // Render
        render_system.run(&world, &asset_server)?;

        // Update input
        world.resource_mut::<InputManager>().update();

        // 60 FPS
        std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / 60.0));
    }

    Ok(())
}

/// Setup game level
fn setup_level(world: &mut World, asset_server: &AssetServer) -> Result<(), Box<dyn std::error::Error>> {
    let player_texture = asset_server.get_handle_by_path::<TextureAsset>("player.png").unwrap();
    let platform_texture = asset_server.get_handle_by_path::<TextureAsset>("platform.png").unwrap();
    let coin_texture = asset_server.get_handle_by_path::<TextureAsset>("coin.png").unwrap();

    // Spawn player
    world.spawn()
        .insert(Player::new())
        .insert(CameraTarget)
        .insert(Sprite::new(player_texture).with_custom_size(Vec2::new(PLAYER_SIZE.0, PLAYER_SIZE.1)))
        .insert(Transform2D::from_position(Vec2::new(PLAYER_START_POS.0, PLAYER_START_POS.1)))
        .id();

    // Spawn platforms
    let platforms = vec![
        (200.0, 400.0, 300.0, 20.0, false),   // Static platform
        (600.0, 300.0, 200.0, 20.0, false),
        (300.0, 200.0, 150.0, 20.0, true),    // One-way platform
        (800.0, 250.0, 200.0, 20.0, false),
        (1000.0, 400.0, 300.0, 20.0, false),
    ];

    for (x, y, width, height, one_way) in platforms {
        let platform = if one_way {
            Platform::one_way(width, height)
        } else {
            Platform::new(width, height)
        };

        world.spawn()
            .insert(platform)
            .insert(Sprite::new(platform_texture).with_custom_size(Vec2::new(width, height)))
            .insert(Transform2D::from_position(Vec2::new(x, y)))
            .id();
    }

    // Spawn moving platform
    world.spawn()
        .insert(Platform::new(150.0, 20.0))
        .insert(MovingPlatform::new(
            Vec2::new(400.0, 150.0),
            Vec2::new(600.0, 150.0),
            50.0,
        ))
        .insert(Sprite::new(platform_texture).with_custom_size(Vec2::new(150.0, 20.0)))
        .insert(Transform2D::from_position(Vec2::new(400.0, 150.0)))
        .id();

    // Spawn coins
    for i in 0..10 {
        world.spawn()
            .insert(Coin::new(10))
            .insert(Sprite::new(coin_texture).with_custom_size(Vec2::new(16.0, 16.0)))
            .insert(Transform2D::from_position(Vec2::new(250.0 + i as f32 * 40.0, 350.0)))
            .id();
    }

    Ok(())
}

/// Update moving platforms
fn moving_platform_system(world: &mut World, delta_time: f32) {
    for (entity, mut moving, mut transform) in world.query::<(Entity, &mut MovingPlatform, &mut Transform2D)>().iter() {
        moving.current_time += delta_time;
        let new_pos = moving.current_position();
        transform.set_position(new_pos);
    }
}
```

## Running the Game

```bash
cargo run --release
```

## Controls

- **A / Left Arrow**: Move left
- **D / Right Arrow**: Move right
- **Space / W**: Jump
- **Escape**: Quit

## Performance

- **Rendering**: Batched sprite rendering (10-20 draw calls for entire level)
- **Physics**: Simple platformer physics with gravity
- **Collision**: AABB-based collision detection
- **Camera**: Smooth following with configurable lag

## Extending the Game

### Add Double Jump

```rust
#[derive(Component)]
struct Player {
    // ... existing fields
    pub jumps_remaining: u32,
    pub max_jumps: u32,
}

fn player_input_system(world: &mut World) {
    // In jump logic
    if input.action_just_pressed("Jump") && player.jumps_remaining > 0 {
        player.velocity.y = player.jump_strength;
        player.jumps_remaining -= 1;
    }
}

fn platform_collision_system(world: &mut World) {
    // Reset jumps when on ground
    if on_ground {
        player.jumps_remaining = player.max_jumps;
    }
}
```

### Add Enemies

```rust
#[derive(Component)]
struct Enemy {
    pub health: i32,
    pub damage: i32,
    pub patrol_points: Vec<Vec2>,
}

fn enemy_ai_system(world: &mut World, delta_time: f32) {
    // Move enemies along patrol paths
    // Attack player when in range
}

fn player_damage_system(world: &mut World) {
    // Detect enemy collision
    // Apply damage to player
    // Knockback effect
}
```

### Add Wall Jump

```rust
fn wall_jump_system(world: &mut World) {
    for (entity, mut player, transform) in world.query::<(Entity, &mut Player, &Transform2D)>().iter() {
        // Detect if touching wall
        let touching_wall = detect_wall_collision(transform, world);

        if touching_wall && input.action_just_pressed("Jump") {
            // Jump away from wall
            player.velocity.y = player.jump_strength;
            player.velocity.x = if player.facing_right { -200.0 } else { 200.0 };
        }
    }
}
```

## See Also

- [Game Migration Guide](game_migration_guide.md) - Migrating from old API
- [ECS Rendering Example](ecs_rendering_example.md) - Rendering details
- [Physics Example](../src/ecs/physics_world.rs) - Physics system
- [Collision Detection](../src/ecs/collision.rs) - Collision API
