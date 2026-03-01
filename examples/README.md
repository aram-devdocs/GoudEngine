# GoudEngine Examples

Flappy Bird is the "hello world" for GoudEngine — the same game implemented in every supported language. Comparing the versions shows what the SDK parity guarantee means in practice.

## Core game loop

This pseudocode captures the structure shared by all implementations:

```
create window (SCREEN_WIDTH x SCREEN_HEIGHT)
load textures (background, base, bird frames, pipe, digits)

loop until window closes:
    dt = poll_events()
    begin_frame()
    clear(sky_blue)

    if space or left_click pressed and cooldown elapsed:
        bird.velocity = JUMP_STRENGTH * TARGET_FPS
        reset jump cooldown

    bird.velocity += GRAVITY * dt * TARGET_FPS
    bird.y        += bird.velocity * dt

    for each pipe:
        pipe.x -= PIPE_SPEED * TARGET_FPS * dt
        if pipe collides with bird: reset game

    if pipe_spawn_timer > PIPE_SPAWN_INTERVAL:
        spawn new pipe at random gap_y
        reset pipe_spawn_timer

    remove pipes that scrolled off screen

    draw layers back-to-front: background, pipes, base, bird, score

    end_frame()
    swap_buffers()
```

## All examples

| Language | Example | Run Command |
|----------|---------|-------------|
| C# | Flappy Goud | `./dev.sh --game flappy_goud` |
| C# | 3D Cube | `./dev.sh --game 3d_cube` |
| C# | Goud Jumper | `./dev.sh --game goud_jumper` |
| C# | Isometric RPG | `./dev.sh --game isometric_rpg` |
| C# | Hello ECS | `./dev.sh --game hello_ecs` |
| Python | SDK Demo | `./dev.sh --sdk python --game python_demo` |
| Python | Flappy Bird | `./dev.sh --sdk python --game flappy_bird` |
| TypeScript | Flappy Bird (Desktop) | `./dev.sh --sdk typescript --game flappy_bird` |
| TypeScript | Flappy Bird (Web) | `./dev.sh --sdk typescript --game flappy_bird_web` |
| Rust | Flappy Bird | `cargo run -p flappy-bird` |

## SDK install instructions

- **C#** — [`sdks/csharp/README.md`](../sdks/csharp/README.md): `dotnet add package GoudEngine`
- **Python** — [`examples/python/README.md`](python/README.md): build native lib, then `import goud_engine`
- **TypeScript** — [`sdks/typescript/README.md`](../sdks/typescript/README.md): `npm install goudengine`
- **Rust** — [`sdks/rust/README.md`](../sdks/rust/README.md): `cargo add goud-engine`
