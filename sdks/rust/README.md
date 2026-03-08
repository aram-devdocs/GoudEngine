# GoudEngine Rust SDK

[![crates.io](https://img.shields.io/crates/v/goud-engine.svg)](https://crates.io/crates/goud-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Alpha** — This SDK is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues) · [Contact](mailto:aram.devdocs@gmail.com)

Build 2D and 3D games in Rust with zero FFI overhead.

## Install

```bash
cargo add goud-engine
```

## Quick Start

```rust
use goudengine::*;

fn main() {
    // GoudGame, Transform2D, Sprite, Vec2, Color, etc.
    // are all available directly through this crate.
}
```

## Flappy Bird Example

A complete Flappy Bird game is included in [`examples/rust/flappy_bird/`](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/rust/flappy_bird). Run it with:

```bash
cargo run -p flappy-bird
```

The example demonstrates the main loop, sprite rendering, physics, and collision detection:

```rust
use goudengine::*;

fn main() {
    let engine = Engine::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, "Flappy Bird");
    let mut manager = GameManager::new(&engine, "examples/csharp/flappy_goud/assets");
    manager.start();
    engine.enable_blending();

    while !engine.should_close() {
        let dt = engine.poll_events();
        engine.begin_frame();
        engine.clear(0.4, 0.7, 0.9, 1.0);

        if !manager.update(&engine, dt) { break; }
        manager.draw(&engine);

        engine.end_frame();
        engine.swap_buffers();
    }
}

// Bird physics: gravity, jump cooldown, rotation smoothing
pub struct Movement {
    pub velocity: f32,
    gravity: f32,
    jump_strength: f32,
    jump_cooldown_timer: f32,
}

impl Movement {
    pub fn apply_gravity(&mut self, dt: f32) {
        self.velocity += self.gravity * dt * TARGET_FPS;
        self.jump_cooldown_timer = (self.jump_cooldown_timer - dt).max(0.0);
    }

    pub fn try_jump(&mut self) {
        if self.jump_cooldown_timer <= 0.0 {
            self.velocity = self.jump_strength * TARGET_FPS;
            self.jump_cooldown_timer = JUMP_COOLDOWN;
        }
    }
}

// Pipe pair: random gap position, AABB collision
pub struct Pipe { pub x: f32, pub gap_y: f32 }

impl Pipe {
    pub fn collides_with_bird(&self, bird_x: f32, bird_y: f32, w: f32, h: f32) -> bool {
        aabb_overlap(bird_x, bird_y, w, h, self.x, self.top_pipe_y(), PIPE_IMAGE_WIDTH, PIPE_IMAGE_HEIGHT)
        || aabb_overlap(bird_x, bird_y, w, h, self.x, self.bottom_pipe_y(), PIPE_IMAGE_WIDTH, PIPE_IMAGE_HEIGHT)
    }
}

// Game loop: update → collision → spawn pipes → draw layers back-to-front
pub fn update(&mut self, engine: &Engine, dt: f32) -> bool {
    self.bird.update(dt, engine.is_key_pressed(KEY_SPACE));

    for pipe in &mut self.pipes {
        pipe.update(dt);
        if pipe.collides_with_bird(self.bird.x, self.bird.y, BIRD_WIDTH, BIRD_HEIGHT) {
            self.start(); // reset on collision
            return true;
        }
    }

    self.pipe_spawn_timer += dt;
    if self.pipe_spawn_timer > PIPE_SPAWN_INTERVAL {
        self.pipe_spawn_timer = 0.0;
        self.pipes.push(Pipe::new());
    }

    self.pipes.retain(|p| !p.is_off_screen());
    true
}
```

Controls: Space / Left Click to flap, R to restart, Escape to quit.

## Features

- 2D and 3D rendering with runtime renderer selection
- Entity Component System (ECS) with Transform2D, Sprite, and more
- Physics simulation (Rapier2D/3D): rigid bodies, colliders, raycasting, collision events
- Audio playback with per-channel volume (Music, SFX, Ambience, UI, Voice) and spatial audio
- Text rendering with TrueType/bitmap fonts, alignment, and word-wrapping
- Sprite animation with state machine controller, multi-layer blending, and tweening
- Scene management with transitions (instant, fade, custom)
- UI component system with hierarchical node tree
- Tiled map support for 2D worlds
- Input handling (keyboard, mouse)
- Asset hot-reloading during development
- Zero FFI overhead — links directly to the Rust engine
- Structured error diagnostics with error codes and recovery hints

## Why a Separate Crate

This crate re-exports `goud_engine::sdk::*` from the internal engine. A standalone crate lets you depend on `goud-engine` without pulling in FFI exports, codegen build scripts, or napi dependencies. It also provides a clean versioned package for crates.io.

The internal crate `goud-engine-core` on crates.io is not intended for direct use.

## Design

Unlike the C#, Python, and TypeScript SDKs which call through FFI, this crate links directly against the Rust engine with zero overhead.

```rust
// This crate is a single re-export:
pub use goud_engine::sdk::*;
```

## Links

- [Repository](https://github.com/aram-devdocs/GoudEngine)
- [Rust Examples](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/rust)
- [License: MIT](https://github.com/aram-devdocs/GoudEngine/blob/main/LICENSE)
