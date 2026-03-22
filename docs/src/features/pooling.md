# Object Pooling and Frame Arenas

GoudEngine provides two allocation primitives for performance-sensitive paths: **EntityPool** for reusable entity slots, and **FrameArena** for per-frame temporary allocations.

## EntityPool

A pre-allocated, free-list pool of entity slot indices with O(1) acquire and release.

### How It Works

1. Create a pool with a fixed capacity. All slots are pre-allocated upfront.
2. Acquire a slot -- the pool returns a `(slot_index, entity_id)` pair from its LIFO free-list.
3. Release a slot -- the index returns to the free-list for reuse.

No heap allocation occurs on the hot path. The pool does not interact with the ECS `World` directly -- slot-to-entity mapping is set by the integration layer via `set_slot_entity`.

### FFI Surface

| Function | Description |
|----------|-------------|
| `goud_entity_pool_create(capacity)` | Create a pool, returns a handle |
| `goud_entity_pool_destroy(handle)` | Destroy a pool |
| `goud_entity_pool_acquire(handle)` | Acquire one entity, returns entity ID |
| `goud_entity_pool_release(handle, entity_id)` | Release an entity back to the pool |
| `goud_entity_pool_stats(handle, out)` | Query diagnostic counters |

### Stats

`PoolStats` tracks:

- `capacity` -- total slots
- `active` -- slots currently in use
- `available` -- slots ready for acquisition
- `high_water_mark` -- peak simultaneous active slots
- `total_acquires` / `total_releases` -- cumulative operation counts

### When to Use

- Bullet pools, particle systems, or any pattern where entities are created and destroyed frequently
- Scenarios where allocation jitter matters (e.g., frame-budget-sensitive gameplay)

## FrameArena

A bump allocator for per-frame temporary data. All allocations are freed in bulk with a single `reset()` call.

### How It Works

1. Allocate values with `alloc(val)` -- returns a mutable reference. Each allocation is an O(1) pointer bump.
2. Call `reset()` once per frame (typically at the start of the update loop). This frees every allocation at once with no per-object teardown.

A single global arena is shared per process (thread-safe via mutex).

### FFI Surface

| Function | Description |
|----------|-------------|
| `goud_frame_arena_reset()` | Free all allocations in one call |
| `goud_frame_arena_stats(out)` | Query diagnostic counters |

### Stats

`ArenaStats` tracks:

- `bytes_allocated` -- currently in use
- `bytes_capacity` -- total backing storage
- `reset_count` -- number of resets since creation

### When to Use

- Per-frame scratch data (collision pairs, render commands, temporary buffers)
- Any allocation pattern where all data has the same short lifetime

## Performance Characteristics

| Operation | EntityPool | FrameArena |
|-----------|-----------|------------|
| Allocate | O(1) free-list pop | O(1) pointer bump |
| Free | O(1) free-list push | O(1) bulk reset (all at once) |
| Memory | Pre-allocated at creation | Grows as needed, reuses on reset |
| Thread safety | Per-pool (no global lock) | Global mutex |

## Example: Bullet Pool (C#)

```csharp
// Create a pool of 200 bullet slots at startup
var pool = game.CreateEntityPool(200);

// In the game loop -- acquire when firing
ulong bullet = game.EntityPoolAcquire(pool);

// Release when the bullet goes off-screen
game.EntityPoolRelease(pool, bullet);

// Query stats for debugging
var stats = game.GetEntityPoolStats(pool);
Console.WriteLine($"Active: {stats.Active}/{stats.Capacity}");
```

## Example: Frame Arena Reset

The frame arena resets automatically each frame when using `GoudGame`. For manual control via `GoudContext`:

```csharp
// At the start of each frame
game.ResetFrameArena();

// Query arena usage
var stats = game.GetFrameArenaStats();
Console.WriteLine($"Arena: {stats.BytesAllocated}/{stats.BytesCapacity}");
```
