# Build Your First Game

This beginner guide walks from zero to a playable Flappy-style loop.

No prior GoudEngine knowledge is assumed. Use one of these tracks:

- C# track: `examples/csharp/flappy_goud/`
- Python track: `examples/python/flappy_bird.py`

## What you will build

By the end of this guide, your game can:

1. Open a window and run a frame loop.
2. Draw a player sprite.
3. Handle jump input.
4. Move one obstacle lane.
5. Detect collision and reset.

## Step 0: Run the reference project first

Run the final reference before writing code.

```bash
./dev.sh --game flappy_goud
./dev.sh --sdk python --game flappy_bird
```

If this fails, fix setup first using:

- [Getting Started — C#](../getting-started/csharp.md)
- [Getting Started — Python](../getting-started/python.md)

## Step 1: Create a minimal frame loop

Goal: open a window and render empty frames.

Verification:

- Window opens.
- Escape closes (desktop).
- Frame timing (`dt`) updates each frame.

Reference files:

- C#: `examples/csharp/flappy_goud/Program.cs`
- Python: `examples/python/flappy_bird.py`

## Step 2: Add a player sprite and gravity

Track per-frame state:

- `x`, `y`
- vertical velocity `vy`
- gravity constant
- jump impulse

Update loop:

- `vy += gravity * dt`
- `y += vy * dt`

Verification:

- Player falls without input.
- Jump input applies one upward impulse.
- Player stays in visible bounds (clamp or reset).

## Step 3: Add input

Map one-shot jump input:

- C#: key press from input API
- Python: same behavior through Python wrapper

Verification:

- Pressing jump changes `vy` once per intent.
- Holding jump does not spam if your design expects one-shot input.

## Step 4: Add one pipe lane

Represent one lane with:

- lane `x`
- `gap_y`
- `gap_height`

Per frame:

- move lane left
- when off-screen, reset to the right and randomize `gap_y`

Verification:

- Pipe lane scrolls smoothly.
- Reset logic reuses the lane without crashes.

## Step 5: Add collision + restart

Check AABB overlap between:

- player and top pipe
- player and bottom pipe
- player and floor/ceiling

On collision:

- reset player state
- reset pipe state
- reset score

Verification:

- Collision triggers a full reset.
- New run starts in a clean state.

## C# beginner variant

Use this order:

1. [Getting Started — C#](../getting-started/csharp.md)
2. Build the five steps above.
3. Compare to `examples/csharp/flappy_goud/` only when stuck.

Run command:

```bash
./dev.sh --game flappy_goud
```

## Python beginner variant

Use this order:

1. [Getting Started — Python](../getting-started/python.md)
2. Build the five steps above.
3. Compare to `examples/python/flappy_bird.py` only when stuck.

Run command:

```bash
./dev.sh --sdk python --game flappy_bird
```

## Downloadable final project handoff

The hosted docs now ship generated bundles for the final reference projects:

- [Download C# Flappy Goud](../generated/downloads/flappy-csharp.zip)
- [Download Python Flappy Bird](../generated/downloads/flappy-python.zip)
- [Download TypeScript Flappy Bird](../generated/downloads/flappy-typescript.zip)
- [Download Rust Flappy Bird](../generated/downloads/flappy-rust.zip)

Canonical source locations in this repository:

- `examples/csharp/flappy_goud/`
- `examples/python/flappy_bird.py`
- `examples/typescript/flappy_bird/`
- `examples/rust/flappy_bird/`

To refresh the downloadable bundles locally:

```bash
PATH="$HOME/.cargo/bin:$HOME/.dotnet/tools:$PATH" bash scripts/clean-room-regenerate.sh --docs
```

## Next step

After this tutorial, run the dedicated [Sandbox Guide](sandbox.md) and the generated [Example Showcase](showcase.md) to cover more of the SDK surface. Keep [Feature Lab](showcase.md#feature-lab-smoke-coverage) as the supplemental smoke path.
