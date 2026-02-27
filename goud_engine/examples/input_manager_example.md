# InputManager Example

This example demonstrates how to use the `InputManager` resource for input handling in the GoudEngine ECS.

## Basic Usage

```rust
use goud_engine::ecs::{World, InputManager};
use goud_engine::ecs::resource::{Res, ResMut};
use glfw::Key;

// Create a world and add the InputManager resource
let mut world = World::new();
world.insert_resource(InputManager::new());

// Update input state at the start of each frame
fn input_update_system(mut input: ResMut<InputManager>) {
    input.update();
}

// Query input in your game systems
fn player_movement_system(input: Res<InputManager>) {
    // Check if key is currently pressed
    if input.key_pressed(Key::W) {
        println!("Moving forward");
    }

    // Check if key was just pressed this frame
    if input.key_just_pressed(Key::Space) {
        println!("Jump!");
    }

    // Check if key was just released this frame
    if input.key_just_released(Key::LeftShift) {
        println!("Stop running");
    }
}
```

## Mouse Input

```rust
use goud_engine::ecs::InputManager;
use goud_engine::core::math::Vec2;
use glfw::MouseButton;

fn mouse_system(input: Res<InputManager>) {
    // Mouse position
    let pos = input.mouse_position();
    println!("Mouse at: ({}, {})", pos.x, pos.y);

    // Mouse delta (movement since last frame)
    let delta = input.mouse_delta();
    if delta.length() > 0.0 {
        println!("Mouse moved: ({}, {})", delta.x, delta.y);
    }

    // Mouse buttons
    if input.mouse_button_just_pressed(MouseButton::Button1) {
        println!("Left click at ({}, {})", pos.x, pos.y);
    }

    // Mouse scroll
    let scroll = input.scroll_delta();
    if scroll.y != 0.0 {
        println!("Scrolled: {}", scroll.y);
    }
}
```

## Gamepad Input

```rust
fn gamepad_system(input: Res<InputManager>) {
    // Check gamepad button state
    if input.gamepad_button_pressed(0, 0) {  // Player 0, Button 0
        println!("Button A pressed");
    }

    if input.gamepad_button_just_pressed(1, 5) {  // Player 1, Button 5
        println!("Player 2 pressed start");
    }
}
```

## Integration with GLFW Window Events

```rust
use goud_engine::ecs::InputManager;
use glfw::{WindowEvent, Action, Key, MouseButton};

fn process_window_events(
    events: &[WindowEvent],
    input_manager: &mut InputManager
) {
    for event in events {
        match event {
            WindowEvent::Key(key, _, action, _) => {
                match action {
                    Action::Press => input_manager.press_key(*key),
                    Action::Release => input_manager.release_key(*key),
                    _ => {}
                }
            }
            WindowEvent::MouseButton(button, action, _) => {
                match action {
                    Action::Press => input_manager.press_mouse_button(*button),
                    Action::Release => input_manager.release_mouse_button(*button),
                    _ => {}
                }
            }
            WindowEvent::CursorPos(x, y) => {
                input_manager.set_mouse_position(Vec2::new(*x as f32, *y as f32));
            }
            WindowEvent::Scroll(x, y) => {
                input_manager.add_scroll_delta(Vec2::new(*x as f32, *y as f32));
            }
            _ => {}
        }
    }
}
```

## Complete Game Loop Example

```rust
use goud_engine::ecs::{World, InputManager};
use goud_engine::ecs::resource::ResMut;
use glfw::Key;

fn main() {
    let mut world = World::new();
    world.insert_resource(InputManager::new());

    loop {
        // 1. Update input state for the new frame
        {
            let mut input = world.resource_mut::<InputManager>();
            input.update();
        }

        // 2. Process GLFW events and update InputManager
        // (See above example)

        // 3. Run game systems
        player_movement_system(&world);
        camera_control_system(&world);

        // 4. Render frame
        // ...
    }
}

fn player_movement_system(world: &World) {
    let input = world.resource::<InputManager>();
    let speed = 5.0;

    let mut movement = Vec2::zero();

    if input.key_pressed(Key::W) { movement.y += 1.0; }
    if input.key_pressed(Key::S) { movement.y -= 1.0; }
    if input.key_pressed(Key::A) { movement.x -= 1.0; }
    if input.key_pressed(Key::D) { movement.x += 1.0; }

    if movement.length() > 0.0 {
        movement = movement.normalize();
        // Apply movement to player entity...
    }
}

fn camera_control_system(world: &World) {
    let input = world.resource::<InputManager>();

    // First-person camera control with mouse
    let delta = input.mouse_delta();
    let sensitivity = 0.1;

    if delta.length() > 0.0 {
        let yaw = delta.x * sensitivity;
        let pitch = delta.y * sensitivity;
        // Apply camera rotation...
    }
}
```

## Features

### Frame-Based State Tracking

The InputManager tracks both current and previous frame state, enabling three types of queries:

- **`pressed()`**: Returns true every frame while the input is held
- **`just_pressed()`**: Returns true only on the first frame the input is pressed
- **`just_released()`**: Returns true only on the first frame the input is released

This is essential for:
- Continuous movement (held keys)
- Single-shot actions (jump, shoot)
- Toggle actions (pause menu)

### Input Buffering

The `update()` method must be called at the start of each frame to:
1. Copy current state to previous state
2. Reset frame-based deltas (mouse movement, scroll)

This ensures consistent frame-to-frame input queries.

### Clear on Focus Loss

```rust
fn window_focus_system(input: ResMut<InputManager>, focused: bool) {
    if !focused {
        input.clear();  // Clear all input when window loses focus
    }
}
```

## Best Practices

1. **Call `update()` once per frame** - At the very start of your game loop
2. **Use `just_pressed()` for actions** - Jump, shoot, menu navigation
3. **Use `pressed()` for continuous input** - Movement, camera control
4. **Clear on focus loss** - Prevent stuck keys when alt-tabbing
5. **Use resource access patterns** - `Res<InputManager>` for reading, `ResMut<InputManager>` for updating

## Thread Safety

InputManager is `Send + Sync` and can be safely used as an ECS resource. However, input processing should typically happen on the main thread where GLFW events are received.
