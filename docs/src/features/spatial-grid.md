# Spatial Grid

Cell-based spatial partitioning for fast neighbor queries. The `SpatialGrid` divides 2D space into uniform cells so that proximity queries run in O(1) relative to total entity count.

## When to Use

Use a spatial grid when you need to find nearby entities efficiently:

- Collision broad-phase (reduce pair checks from O(n^2) to O(n))
- Area-of-effect abilities (find targets within a radius)
- AI perception (detect entities in a sensory range)
- Proximity triggers (activate when a player approaches)

## Creating a Grid

Create a grid by specifying a cell size in world units. Entities that fall within the same cell are grouped together, so choose a cell size close to your typical query radius.

| FFI Function | Description |
|---|---|
| `goud_spatial_grid_create(cell_size)` | Create a grid with the given cell size |
| `goud_spatial_grid_create_with_capacity(cell_size, capacity)` | Create a grid with pre-allocated storage |
| `goud_spatial_grid_destroy(handle)` | Destroy a grid and free resources |

The `cell_size` must be positive and finite. A good starting value is the diameter of your most common query radius.

`create_with_capacity` pre-allocates internal storage for the expected number of entities, reducing allocations during gameplay.

## Inserting and Removing Entities

| FFI Function | Description |
|---|---|
| `goud_spatial_grid_insert(handle, entity_id, x, y)` | Insert or move an entity to a position |
| `goud_spatial_grid_remove(handle, entity_id)` | Remove an entity from the grid |
| `goud_spatial_grid_update(handle, entity_id, x, y)` | Update an entity's position |
| `goud_spatial_grid_clear(handle)` | Remove all entities without destroying the grid |

**Insert** is idempotent: if the entity already exists in the grid, it is moved to the new position. **Remove** is also idempotent: removing a nonexistent entity is a no-op.

**Update** returns an error if the entity was not previously inserted. Use `insert` for entities that may or may not already be tracked.

## Querying by Radius

```
goud_spatial_grid_query_radius(handle, center_x, center_y, radius, out_entities, capacity) -> i32
```

Writes matching entity IDs into the caller-provided buffer. The return value is:

- **Non-negative**: total number of entities found (may exceed `capacity`)
- **Negative**: error code

If the buffer is too small, only `capacity` entities are written, but the full count is still returned. This allows a two-pass pattern: call once with `capacity = 0` to get the count, allocate, then call again.

## Counting Entities

```
goud_spatial_grid_entity_count(handle) -> i32
```

Returns the number of entities currently in the grid, or a negative error code.

## Choosing Cell Size

| Scenario | Suggested Cell Size |
|---|---|
| Small query radius (< 50 units) | 1x to 2x the query radius |
| Large open world | Larger cells (100-500 units) to reduce memory |
| Dense clusters | Smaller cells for finer granularity |

A cell size that is too small wastes memory on empty cells. A cell size that is too large puts too many entities in each cell, reducing the benefit of spatial partitioning.

## Error Handling

All functions return error codes on failure. Common error conditions:

- Invalid grid handle (destroyed or never created)
- Cell size not positive or not finite
- Handle space exhausted (too many concurrent grids)
- Entity not found (for `update` only)

Call `goud_last_error_message()` after any negative return value for a human-readable error string.

## FFI

Spatial grid FFI functions are in `goud_engine/src/ffi/spatial_grid/`. The module is split into:

- `lifecycle.rs` -- create, destroy, clear
- `operations.rs` -- insert, remove, update
- `queries.rs` -- query_radius, entity_count
