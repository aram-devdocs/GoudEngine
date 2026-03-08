# Scene Management

Scenes provide isolated ECS worlds that can be switched at runtime with optional transitions.

## Scene Lifecycle

Each context has a `SceneManager` that owns named scenes. A default scene (ID 0) is created automatically and cannot be destroyed.

### Creating Scenes

Scenes are created per context with a name. Each scene owns an independent ECS `World`, so entities in one scene are fully isolated from entities in another.

### Switching Scenes

Transition to a scene by ID with a specified transition type:

| Transition | Behavior |
|---|---|
| `Instant` | Immediate switch, no animation |
| `Fade` | Fade out current scene, fade in next |
| `Custom` | SDK-managed transition with manual progress control |

### Transition Progress

During a transition, `goud_scene_transition_progress()` returns a value from 0.0 to 1.0. Use this to drive custom visual effects. Call `goud_scene_transition_tick()` each frame to advance the transition.

## Scene Isolation

Entities and components in one scene are not visible to queries in another. Switching scenes activates the target scene's `World` and deactivates the current one. There is no shared entity space across scenes.

## FFI

Scene FFI functions are in `goud_engine/src/ffi/scene.rs` and `scene_transition.rs`:

- `goud_scene_create()` / `goud_scene_destroy()`
- `goud_scene_lookup_by_name()`
- `goud_scene_transition_to()`
- `goud_scene_transition_progress()` / `goud_scene_transition_is_active()`
- `goud_scene_transition_tick()`
