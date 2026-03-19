# Lua SDK API Reference

All APIs are registered as global tables and constructor functions when the Lua VM starts. No `require()` calls are needed.

---

## Types

Types are constructed by calling the global constructor with a table of named fields. All fields are optional and default to zero (or `false` for booleans).

### Color

RGBA color with float components in 0.0--1.0 range.

```lua
local c = Color({ r = 1.0, g = 0.5, b = 0.0, a = 1.0 })
print(c.r, c.g, c.b, c.a)

-- Fields are read/write
c.a = 0.5
```

| Field | Type | Description |
|-------|------|-------------|
| `r` | number | Red channel (0.0--1.0) |
| `g` | number | Green channel (0.0--1.0) |
| `b` | number | Blue channel (0.0--1.0) |
| `a` | number | Alpha channel (0.0--1.0) |

### Vec2

2D vector.

```lua
local v = Vec2({ x = 10, y = 20 })
v.x = v.x + 1
```

| Field | Type | Description |
|-------|------|-------------|
| `x` | number | X component |
| `y` | number | Y component |

### Rect

Axis-aligned rectangle.

```lua
local r = Rect({ x = 0, y = 0, width = 64, height = 64 })
```

| Field | Type | Description |
|-------|------|-------------|
| `x` | number | Left edge |
| `y` | number | Top edge |
| `width` | number | Width |
| `height` | number | Height |

### Transform2D

2D spatial transform.

```lua
local t = Transform2D({
    position_x = 100,
    position_y = 200,
    rotation = 0,
    scale_x = 1,
    scale_y = 1,
})
```

| Field | Type | Description |
|-------|------|-------------|
| `position_x` | number | X position |
| `position_y` | number | Y position |
| `rotation` | number | Rotation in radians |
| `scale_x` | number | Horizontal scale |
| `scale_y` | number | Vertical scale |

### Sprite

Sprite component data. Attach to an entity for 2D rendering.

```lua
local s = Sprite({
    texture_handle = tex,
    color_r = 1, color_g = 1, color_b = 1, color_a = 1,
    z_layer = 0,
})
```

| Field | Type | Description |
|-------|------|-------------|
| `texture_handle` | integer | Texture asset handle |
| `color_r` | number | Tint red |
| `color_g` | number | Tint green |
| `color_b` | number | Tint blue |
| `color_a` | number | Tint alpha |
| `source_rect_x` | number | Source rectangle X |
| `source_rect_y` | number | Source rectangle Y |
| `source_rect_width` | number | Source rectangle width |
| `source_rect_height` | number | Source rectangle height |
| `has_source_rect` | boolean | Whether source rect is active |
| `flip_x` | boolean | Flip horizontally |
| `flip_y` | boolean | Flip vertically |
| `z_layer` | integer | Draw order layer |
| `anchor_x` | number | Anchor point X (0.0--1.0) |
| `anchor_y` | number | Anchor point Y (0.0--1.0) |
| `custom_size_x` | number | Override width |
| `custom_size_y` | number | Override height |
| `has_custom_size` | boolean | Whether custom size is active |

### Text

Text rendering component data.

```lua
local t = Text({
    font_handle = font,
    font_size = 24,
    color_r = 1, color_g = 1, color_b = 1, color_a = 1,
    alignment = text_alignment.left,
    line_spacing = 1.2,
})
```

| Field | Type | Description |
|-------|------|-------------|
| `font_handle` | integer | Font asset handle |
| `font_size` | number | Size in pixels |
| `color_r` | number | Text color red |
| `color_g` | number | Text color green |
| `color_b` | number | Text color blue |
| `color_a` | number | Text color alpha |
| `alignment` | integer | `text_alignment.*` constant |
| `max_width` | number | Word-wrap width |
| `has_max_width` | boolean | Whether max_width is active |
| `line_spacing` | number | Multiplier for line height |

---

## Enums

Enums are global tables mapping names to integer constants.

### key

Keyboard key codes. Used with input functions.

```lua
-- Common keys:
key.space       -- 32
key.escape      -- 256
key.enter       -- 257
key.up          -- 265
key.down        -- 264
key.left        -- 263
key.right       -- 262
key.a .. key.z  -- 65..90
key.digit0 .. key.digit9  -- 48..57
key.f1 .. key.f12         -- 290..301
key.left_shift  -- 340
key.left_control -- 341
```

### mouse_button

```lua
mouse_button.left    -- 0
mouse_button.right   -- 1
mouse_button.middle  -- 2
```

### renderer_type

```lua
renderer_type.renderer2_d  -- 0
renderer_type.renderer3_d  -- 1
```

### overlay_corner

```lua
overlay_corner.top_left      -- 0
overlay_corner.top_right     -- 1
overlay_corner.bottom_left   -- 2
overlay_corner.bottom_right  -- 3
```

### body_type

Physics rigid body types.

```lua
body_type.dynamic    -- 0
body_type.static     -- 1  (use body_type["static"] in Lua)
body_type.kinematic  -- 2
```

### shape_type

Physics collider shapes.

```lua
shape_type["box"]   -- 0  (use bracket syntax since "box" is not a reserved word but "shape_type.box" works too)
shape_type.circle   -- 1
```

### playback_mode

Animation playback.

```lua
playback_mode["loop"]  -- 0
playback_mode.one_shot -- 1
```

### easing_type

Tween easing curves.

```lua
easing_type.linear          -- 0
easing_type.ease_in_quad    -- 1
easing_type.ease_out_quad   -- 2
easing_type.ease_in_out_quad -- 3
```

### text_alignment

```lua
text_alignment.left   -- 0
text_alignment.center -- 1
text_alignment.right  -- 2
```

### text_direction

```lua
text_direction.auto  -- 0
text_direction.ltr   -- 1
text_direction.rtl   -- 2
```

### blend_mode

```lua
blend_mode.override  -- 0  (use blend_mode["override"])
blend_mode.additive  -- 1
```

### transition_type

Scene transitions.

```lua
transition_type.instant -- 0
transition_type.fade    -- 1
transition_type.custom  -- 2
```

### network_protocol

```lua
network_protocol.udp        -- 0
network_protocol.web_socket -- 1
network_protocol.tcp        -- 2
```

### physics_backend2_d

```lua
physics_backend2_d.default -- 0
physics_backend2_d.rapier  -- 1
physics_backend2_d.simple  -- 2
```

### render_backend_kind

```lua
render_backend_kind.wgpu           -- 0
render_backend_kind.open_gl_legacy -- 1
```

### window_backend_kind

```lua
window_backend_kind.winit       -- 0
window_backend_kind.glfw_legacy -- 1
```

### debugger_step_kind

```lua
debugger_step_kind.frame -- 0
debugger_step_kind.tick  -- 1
```

### event_payload_type

```lua
event_payload_type.none   -- 0
event_payload_type.int    -- 1
event_payload_type.float  -- 2
event_payload_type.string -- 3
```

---

## Tool Functions

Tool functions are organized into global tables. Each table groups related engine operations.

### goud_game

The primary game interface. Contains entity management, window control, rendering, collision, audio, and debug methods.

#### Bridge Functions (hand-written)

These handle string parameters that the code generator cannot auto-wrap.

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `texture_load` | `(path: string)` | integer | Load a texture file, returns a handle |
| `font_load` | `(path: string)` | integer | Load a font file, returns a handle |
| `draw_text` | `(font, text, x, y, size, r, g, b, a)` | integer | Draw text at (x,y) with color |
| `delta_time` | `()` | number | Current frame delta time in seconds |

#### Window

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `should_close` | `()` | boolean | True if the window close was requested |
| `set_window_size` | `(width, height)` | -- | Resize the window |
| `destroy` | `()` | -- | Destroy the window |

#### Entities

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `spawn_empty` | `()` | integer | Spawn an empty entity, returns handle |
| `despawn` | `(entity)` | integer | Remove an entity (0 = success) |
| `clone_entity` | `(entity)` | integer | Clone an entity, returns new handle |
| `clone_entity_recursive` | `(entity)` | integer | Clone entity and children |
| `entity_count` | `()` | integer | Total live entity count |
| `is_alive` | `(entity)` | boolean | Check if an entity handle is valid |

#### Collision

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `point_in_rect` | `(px, py, rx, ry, rw, rh)` | boolean | Test if point is inside rectangle |
| `point_in_circle` | `(px, py, cx, cy, radius)` | boolean | Test if point is inside circle |
| `aabb_overlap` | `(x1, y1, w1, h1, x2, y2, w2, h2)` | boolean | Test AABB overlap |
| `circle_overlap` | `(x1, y1, r1, x2, y2, r2)` | boolean | Test circle overlap |
| `distance` | `(x1, y1, x2, y2)` | number | Euclidean distance between two points |
| `distance_squared` | `(x1, y1, x2, y2)` | number | Squared distance (avoids sqrt) |

#### Animation

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `play` | `(entity)` | integer | Start animation playback |
| `stop` | `(entity)` | integer | Stop animation playback |

#### Audio

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `audio_stop` | `(handle)` | integer | Stop a playing sound |
| `audio_pause` | `(handle)` | integer | Pause a playing sound |
| `audio_resume` | `(handle)` | integer | Resume a paused sound |
| `audio_stop_all` | `()` | integer | Stop all sounds |
| `audio_set_global_volume` | `(volume)` | integer | Set master volume (0.0--1.0) |
| `audio_get_global_volume` | `()` | number | Get master volume |
| `audio_set_channel_volume` | `(channel, volume)` | integer | Set channel volume |
| `audio_get_channel_volume` | `(channel)` | number | Get channel volume |
| `audio_is_playing` | `(handle)` | integer | Check if sound is playing |
| `audio_active_count` | `()` | integer | Number of active sounds |
| `audio_cleanup_finished` | `()` | integer | Remove finished sound entries |
| `audio_activate` | `()` | integer | Activate the audio system |
| `audio_set_player_volume` | `(handle, volume)` | integer | Set per-player volume |
| `audio_set_player_speed` | `(handle, speed)` | integer | Set playback speed |
| `audio_crossfade` | `(from, to, duration)` | integer | Crossfade between two sounds |
| `audio_update_crossfades` | `(dt)` | integer | Tick active crossfades |
| `audio_active_crossfade_count` | `()` | integer | Number of active crossfades |

#### Audio (3D Spatial)

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `audio_update_spatial3d` | `(handle, sx, sy, sz, lx, ly, lz, max_dist, ref_dist)` | integer | Update 3D spatial volume |
| `audio_set_listener_position3d` | `(x, y, z)` | integer | Set 3D listener position |
| `audio_set_source_position3d` | `(handle, x, y, z, max_dist, ref_dist)` | integer | Set 3D source position |

#### 3D Rendering

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `create_cube` | `(material_id, w, h, d)` | integer | Create a 3D cube |
| `create_plane` | `(material_id, w, d)` | integer | Create a 3D plane |
| `create_sphere` | `(material_id, radius, segments)` | integer | Create a 3D sphere |
| `create_cylinder` | `(material_id, radius, height, segments)` | integer | Create a 3D cylinder |
| `set_object_position` | `(id, x, y, z)` | -- | Set 3D object position |
| `set_object_rotation` | `(id, x, y, z)` | -- | Set 3D object rotation (radians) |
| `set_object_scale` | `(id, x, y, z)` | -- | Set 3D object scale |
| `destroy_object` | `(id)` | -- | Remove a 3D object |
| `remove_light` | `(id)` | -- | Remove a light |
| `set_camera_position3_d` | `(x, y, z)` | -- | Set 3D camera position |
| `set_camera_rotation3_d` | `(x, y, z)` | -- | Set 3D camera rotation |
| `render3_d` | `()` | -- | Execute the 3D render pass |

#### 3D Materials

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `create_material` | `(type, r, g, b, a, metallic, roughness, emissive_r, emissive_g)` | integer | Create a material |
| `update_material` | `(id, type, r, g, b, a, metallic, roughness, emissive_r, emissive_g)` | -- | Update material properties |
| `remove_material` | `(id)` | -- | Remove a material |
| `set_object_material` | `(obj_id, mat_id)` | -- | Assign material to object |
| `get_object_material` | `(obj_id)` | integer | Get object's material ID |

#### 3D Environment

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `configure_grid` | `(enabled, spacing, line_count)` | -- | Configure the debug grid |
| `set_grid_enabled` | `(enabled)` | -- | Toggle grid visibility |
| `configure_skybox` | `(enabled, r, g, b, a)` | -- | Configure skybox color |
| `configure_fog` | `(enabled, r, g, b, density)` | -- | Configure fog |
| `set_fog_enabled` | `(enabled)` | -- | Toggle fog |

#### 3D Post-Processing

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `add_bloom_pass` | `(threshold, intensity)` | integer | Add bloom effect |
| `add_blur_pass` | `(radius)` | integer | Add blur effect |
| `add_color_grade_pass` | `(brightness, contrast, saturation)` | integer | Add color grading |
| `remove_postprocess_pass` | `(id)` | -- | Remove a post-process pass |
| `postprocess_pass_count` | `()` | integer | Number of active passes |

#### Renderer Utilities

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `set_viewport` | `(x, y, width, height)` | -- | Set the rendering viewport |
| `enable_depth_test` | `()` | -- | Enable depth testing |
| `disable_depth_test` | `()` | -- | Disable depth testing |
| `clear_depth` | `()` | -- | Clear the depth buffer |
| `disable_blending` | `()` | -- | Disable alpha blending |

#### Skinned Mesh

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `remove_skinned_mesh` | `(id)` | -- | Remove a skinned mesh |
| `set_skinned_mesh_position` | `(id, x, y, z)` | -- | Set skinned mesh position |
| `set_skinned_mesh_rotation` | `(id, x, y, z)` | -- | Set skinned mesh rotation |
| `set_skinned_mesh_scale` | `(id, x, y, z)` | -- | Set skinned mesh scale |

#### Debug

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `set_fps_overlay_enabled` | `(enabled)` | integer | Toggle FPS overlay |
| `set_fps_update_interval` | `(seconds)` | integer | FPS counter update rate |
| `set_fps_overlay_corner` | `(corner)` | integer | FPS overlay position (use `overlay_corner.*`) |
| `set_debugger_paused` | `(paused)` | integer | Pause/unpause the debugger |
| `set_debugger_time_scale` | `(scale)` | integer | Set time scale |
| `set_debugger_debug_draw_enabled` | `(enabled)` | integer | Toggle debug drawing |
| `set_debugger_profiling_enabled` | `(enabled)` | integer | Toggle profiling |
| `set_debugger_selected_entity` | `(entity)` | integer | Select entity for inspection |
| `clear_debugger_selected_entity` | `()` | integer | Clear entity selection |
| `start_debugger_recording` | `()` | integer | Start recording frames |
| `stop_debugger_replay` | `()` | integer | Stop replay |

#### Network

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `network_host` | `(protocol, port)` | integer | Start hosting |
| `network_disconnect` | `(handle)` | integer | Disconnect |
| `network_poll` | `(handle)` | integer | Poll for events |
| `network_peer_count` | `(handle)` | integer | Connected peer count |
| `clear_network_simulation` | `(handle)` | integer | Clear simulation state |
| `set_network_overlay_handle` | `(handle)` | integer | Set overlay handle |
| `clear_network_overlay_handle` | `()` | integer | Clear overlay handle |

#### Miscellaneous

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `check_hot_swap_shortcut` | `()` | integer | Check if the hot-swap shortcut was pressed |

---

### goud_context

Lower-level context management. Mirrors many `goud_game` functions but operates on the engine context directly. Includes scene management.

#### Context Lifecycle

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `destroy` | `()` | -- | Destroy the context |
| `is_valid` | `()` | boolean | Check if context is valid |

#### Entities (same as goud_game)

| Function | Signature | Returns |
|----------|-----------|---------|
| `spawn_empty` | `()` | integer |
| `despawn` | `(entity)` | integer |
| `clone_entity` | `(entity)` | integer |
| `clone_entity_recursive` | `(entity)` | integer |
| `is_alive` | `(entity)` | boolean |
| `entity_count` | `()` | integer |

#### Scene Management

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `scene_destroy` | `(scene_id)` | integer | Destroy a scene |
| `set_active_scene` | `(scene_id, active)` | integer | Activate/deactivate a scene |
| `scene_set_active` | `(scene_id, active)` | integer | Alias for set_active_scene |
| `scene_is_active` | `(scene_id)` | boolean | Check if a scene is active |
| `scene_count` | `()` | integer | Number of scenes |
| `scene_set_current` | `(scene_id)` | integer | Switch to a scene |
| `scene_get_current` | `()` | integer | Get current scene ID |
| `scene_transition_to` | `(from, to, type, duration)` | integer | Start a scene transition |
| `scene_transition_progress` | `()` | number | Get transition progress (0.0--1.0) |
| `scene_transition_is_active` | `()` | boolean | Check if a transition is running |
| `scene_transition_tick` | `(dt)` | integer | Advance the transition timer |

---

### audio

Standalone audio control table. Contains the same audio functions as `goud_game.audio_*` but with shorter names.

| Function | Signature | Returns |
|----------|-----------|---------|
| `stop` | `(handle)` | integer |
| `pause` | `(handle)` | integer |
| `resume` | `(handle)` | integer |
| `stop_all` | `()` | integer |
| `set_global_volume` | `(volume)` | integer |
| `get_global_volume` | `()` | number |
| `set_channel_volume` | `(channel, volume)` | integer |
| `get_channel_volume` | `(channel)` | number |
| `is_playing` | `(handle)` | integer |
| `active_count` | `()` | integer |
| `cleanup_finished` | `()` | integer |

---

### animation_controller

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `create` | `(entity)` | integer | Create an animation controller for entity |
| `update` | `(entity, dt)` | integer | Tick the controller |

### animation_events

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `count` | `()` | integer | Number of pending animation events |

### animation_layer_stack

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `create` | `(entity)` | integer | Create a layer stack |
| `set_layer_weight` | `(entity, layer, weight)` | integer | Set layer blend weight |
| `play` | `(entity, layer)` | integer | Play a layer |
| `set_clip` | `(entity, layer, clip, speed, mode)` | integer | Assign a clip to a layer |
| `add_frame` | `(entity, clip, x, y, w, h)` | integer | Add a frame to a clip |
| `reset_layer` | `(entity, layer)` | integer | Reset a layer |

---

### tween

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `create` | `(from, to, duration, easing)` | integer | Create a tween (use `easing_type.*`) |
| `update` | `(handle, dt)` | integer | Tick the tween |
| `is_complete` | `(handle)` | integer | Check if the tween finished |
| `reset` | `(handle)` | integer | Reset to start |
| `destroy` | `(handle)` | integer | Remove the tween |

---

### skeleton

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `create` | `(entity)` | integer | Create a skeleton for an entity |
| `set_bone_transform` | `(entity, bone_index, x, y, rotation)` | integer | Set bone transform |

---

### network

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `host` | `(protocol, port)` | integer | Start hosting |
| `disconnect` | `(handle)` | integer | Disconnect |
| `poll` | `(handle)` | integer | Poll network events |
| `peer_count` | `(handle)` | integer | Connected peer count |

---

### physics_world2_d

Available when the engine is built with `rapier2d` feature.

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `create` | `(gravity_x, gravity_y)` | integer | Create a 2D physics world |
| `create_with_backend` | `(gx, gy, backend)` | integer | Create with specific backend |
| `destroy` | `()` | integer | Destroy the physics world |
| `set_gravity` | `(x, y)` | integer | Update gravity |
| `add_rigid_body` | `(body_type, x, y, mass)` | integer | Add a rigid body |
| `add_rigid_body_ex` | `(body_type, x, y, mass, fixed_rotation)` | integer | Add body with options |
| `add_collider` | `(body, shape, w, h, friction, restitution, density)` | integer | Add a collider |
| `add_collider_ex` | `(body, shape, w, h, friction, restitution, density, sensor, group, mask)` | integer | Add collider with options |
| `remove_body` | `(handle)` | integer | Remove a body |
| `remove_joint` | `(handle)` | integer | Remove a joint |
| `step` | `(dt)` | integer | Step the simulation |
| `set_velocity` | `(body, vx, vy)` | integer | Set body velocity |
| `apply_force` | `(body, fx, fy)` | integer | Apply force |
| `apply_impulse` | `(body, ix, iy)` | integer | Apply impulse |
| `collision_events_count` | `()` | integer | Collision event count |
| `collision_event_count` | `()` | integer | Alias |
| `set_body_gravity_scale` | `(body, scale)` | integer | Per-body gravity scale |
| `set_collider_friction` | `(collider, friction)` | integer | Set friction |
| `set_collider_restitution` | `(collider, restitution)` | integer | Set restitution |
| `set_timestep` | `(dt)` | integer | Set fixed timestep |

---

### physics_world3_d

Available when the engine is built with `rapier3d` feature.

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `create` | `(gx, gy, gz)` | integer | Create a 3D physics world |
| `destroy` | `()` | integer | Destroy the physics world |
| `set_gravity` | `(x, y, z)` | integer | Update gravity |
| `add_rigid_body` | `(body_type, x, y, z, mass)` | integer | Add a rigid body |
| `add_rigid_body_ex` | `(body_type, x, y, z, mass, fixed_rotation)` | integer | Add body with options |
| `add_collider` | `(body, shape, w, h, d, friction, restitution, density)` | integer | Add a collider |
| `remove_body` | `(handle)` | integer | Remove a body |
| `remove_joint` | `(handle)` | integer | Remove a joint |
| `step` | `(dt)` | integer | Step the simulation |
| `set_velocity` | `(body, vx, vy, vz)` | integer | Set body velocity |
| `apply_force` | `(body, fx, fy, fz)` | integer | Apply force |
| `apply_impulse` | `(body, ix, iy, iz)` | integer | Apply impulse |
| `set_body_gravity_scale` | `(body, scale)` | integer | Per-body gravity scale |
| `set_collider_friction` | `(collider, friction)` | integer | Set friction |
| `set_collider_restitution` | `(collider, restitution)` | integer | Set restitution |
| `set_timestep` | `(dt)` | integer | Set fixed timestep |

---

## Known Limitations

- **Input not yet exposed.** Keyboard and mouse query functions (`is_key_pressed`, `is_key_down`, `get_mouse_position`, etc.) are not yet available in the Lua bindings. The `key` and `mouse_button` enum tables are registered for forward compatibility.
- **Audio loading.** There is no `audio_load` bridge function yet. Audio handles must be obtained through other means or future bridge additions.
- **No event callbacks.** Collision events and custom events are exposed as counts but individual event data cannot yet be read from Lua.
- **Component access.** Sprite and Transform2D types can be constructed but there are no Lua-side functions to attach/detach/query components on entities yet.
