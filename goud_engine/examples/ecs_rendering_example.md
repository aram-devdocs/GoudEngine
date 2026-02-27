# ECS Rendering Integration Example

This example demonstrates how to use the ECS system with the rendering pipeline for 2D sprite rendering.

## Overview

The ECS rendering integration provides:
- **Component-based sprites**: Sprite and Transform2D components
- **Batched rendering**: Automatic texture batching for performance
- **Z-layer sorting**: Correct draw order for 2D scenes
- **Asset integration**: Texture loading via AssetServer

## Basic Setup

```rust
use goud_engine::ecs::{World, SpriteRenderSystem};
use goud_engine::ecs::components::{Sprite, Transform2D};
use goud_engine::assets::{AssetServer, loaders::TextureLoader};
use goud_engine::graphics::backend::OpenGLBackend;
use goud_engine::core::math::{Vec2, Color};

// Initialize ECS world
let mut world = World::new();

// Initialize asset server
let mut asset_server = AssetServer::new();
asset_server.register_loader(TextureLoader::default());

// Initialize rendering system
let backend = OpenGLBackend::new()?;
let mut render_system = SpriteRenderSystem::new(backend)?;

// Game loop
loop {
    // Update game logic...

    // Render all sprites
    render_system.run(&world, &asset_server)?;

    // Swap buffers...
}
```

## Creating Sprite Entities

```rust
// Load a texture
let texture = asset_server.load::<TextureAsset>("sprites/player.png");

// Spawn a sprite entity
let player = world.spawn()
    .insert(Sprite::new(texture).with_color(Color::WHITE))
    .insert(Transform2D::from_position(Vec2::new(100.0, 100.0)))
    .id();

// Spawn multiple sprites
for i in 0..10 {
    world.spawn()
        .insert(Sprite::new(texture))
        .insert(Transform2D::from_position(Vec2::new(i as f32 * 50.0, 200.0)))
        .id();
}
```

## Sprite Properties

```rust
// Color tinting
let sprite = Sprite::new(texture)
    .with_color(Color::rgba(1.0, 0.5, 0.5, 1.0)); // Red tint

// Sprite sheets (source rectangle)
let sprite = Sprite::new(texture)
    .with_source_rect(Rect::new(0.0, 0.0, 32.0, 32.0));

// Flipping
let sprite = Sprite::new(texture)
    .with_flip_x(true)  // Horizontal flip
    .with_flip_y(false);

// Custom size (scaling)
let sprite = Sprite::new(texture)
    .with_custom_size(Vec2::new(128.0, 128.0));

// Custom anchor point
let sprite = Sprite::new(texture)
    .with_anchor(Vec2::new(0.0, 1.0)); // Bottom-left anchor
```

## Transform Properties

```rust
// Position
let transform = Transform2D::from_position(Vec2::new(100.0, 100.0));

// Rotation (radians)
let transform = Transform2D::from_rotation(std::f32::consts::PI / 4.0);

// Scale
let transform = Transform2D::from_scale(Vec2::new(2.0, 2.0));
let transform = Transform2D::from_scale_uniform(1.5);

// Combined
let transform = Transform2D::new(
    Vec2::new(100.0, 100.0),  // position
    0.0,                       // rotation
    Vec2::new(1.0, 1.0),      // scale
);

// Mutation
transform.translate(Vec2::new(10.0, 0.0));
transform.rotate(0.1);
transform.scale_by(1.1);
```

## Z-Layer Sorting

Sprites are sorted by their Transform2D Y position (bottom-to-top rendering):

```rust
// Background layer (drawn first)
world.spawn()
    .insert(Sprite::new(background_texture))
    .insert(Transform2D::from_position(Vec2::new(0.0, 0.0)))
    .id();

// Middle layer
world.spawn()
    .insert(Sprite::new(platform_texture))
    .insert(Transform2D::from_position(Vec2::new(100.0, 100.0)))
    .id();

// Foreground layer (drawn last)
world.spawn()
    .insert(Sprite::new(player_texture))
    .insert(Transform2D::from_position(Vec2::new(150.0, 150.0)))
    .id();
```

Sprites with higher Y values are drawn on top (typical for isometric/top-down games).

## Texture Batching

The render system automatically batches sprites by texture to minimize draw calls:

```rust
// These 100 sprites will be rendered in 1-2 draw calls
let texture = asset_server.load::<TextureAsset>("sprites/coin.png");
for i in 0..100 {
    world.spawn()
        .insert(Sprite::new(texture))
        .insert(Transform2D::from_position(Vec2::new(
            (i % 10) as f32 * 50.0,
            (i / 10) as f32 * 50.0,
        )))
        .id();
}

// Check rendering stats
let (sprite_count, batch_count, batch_ratio) = render_system.stats();
println!("Rendered {} sprites in {} batches ({}:1 ratio)",
    sprite_count, batch_count, batch_ratio);
```

## Custom Rendering Configuration

```rust
use goud_engine::graphics::sprite_batch::SpriteBatchConfig;

let config = SpriteBatchConfig {
    initial_capacity: 2048,    // Pre-allocate for 2048 sprites
    max_batch_size: 10000,     // Flush after 10K sprites
    enable_z_sorting: true,    // Sort by Z-layer
    enable_batching: true,     // Batch by texture
};

let render_system = SpriteRenderSystem::with_config(backend, config)?;
```

## Performance Tips

1. **Use texture atlases**: Pack multiple sprites into one texture for better batching
2. **Minimize texture swaps**: Group sprites using the same texture
3. **Z-sorting overhead**: Disable for UI layers that don't need depth sorting
4. **Pre-allocate capacity**: Set `initial_capacity` based on expected sprite count
5. **Batch size tuning**: Adjust `max_batch_size` to balance memory and draw calls

## Integration with Game Loop

```rust
use goud_engine::ecs::{World, SpriteRenderSystem};
use goud_engine::assets::AssetServer;
use goud_engine::graphics::backend::OpenGLBackend;

struct Game {
    world: World,
    asset_server: AssetServer,
    render_system: SpriteRenderSystem<OpenGLBackend>,
}

impl Game {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let world = World::new();
        let mut asset_server = AssetServer::new();
        asset_server.register_loader(TextureLoader::default());

        let backend = OpenGLBackend::new()?;
        let render_system = SpriteRenderSystem::new(backend)?;

        Ok(Self {
            world,
            asset_server,
            render_system,
        })
    }

    fn update(&mut self, delta_time: f32) {
        // Update game logic
        // ...
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear screen
        gl::clear_color(0.1, 0.1, 0.1, 1.0);
        gl::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        // Render all sprites
        self.render_system.run(&self.world, &self.asset_server)?;

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut game = Game::new()?;

    // Game loop
    loop {
        game.update(0.016); // 60 FPS
        game.render()?;
        // Swap buffers...
    }
}
```

## Comparison with Old API

### Old (Renderer2D):

```rust
// Manual sprite management
let mut sprites = SpriteMap::new();
let mut texture_manager = TextureManager::new();

// Add sprites manually
sprites.insert(0, 0, Sprite::new(...));

// Manual rendering
renderer.render(&sprites, &texture_manager);
```

### New (ECS):

```rust
// ECS entities with components
world.spawn()
    .insert(Sprite::new(texture))
    .insert(Transform2D::from_position(pos))
    .id();

// Automatic rendering via system
render_system.run(&world, &asset_server)?;
```

## See Also

- `sprite_batch.rs` - Low-level batching implementation
- `Transform2D` component - 2D transformation
- `Sprite` component - Sprite rendering properties
- `AssetServer` - Asset loading and caching
- `World` - ECS world container
