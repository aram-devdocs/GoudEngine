# Action Mapping Example

This example demonstrates the action mapping system in GoudEngine's InputManager, which allows you to create semantic input bindings that work across multiple input devices.

## Table of Contents

- [Basic Action Mapping](#basic-action-mapping)
- [Multiple Bindings](#multiple-bindings)
- [Cross-Platform Support](#cross-platform-support)
- [Managing Actions](#managing-actions)
- [Action Queries](#action-queries)
- [Practical Example: Player Controller](#practical-example-player-controller)
- [Best Practices](#best-practices)

## Basic Action Mapping

Map keyboard keys to semantic actions:

```rust
use goud_engine::ecs::{InputManager, InputBinding};
use glfw::Key;

let mut input = InputManager::new();

// Map actions to keys
input.map_action("Jump", InputBinding::Key(Key::Space));
input.map_action("Attack", InputBinding::Key(Key::E));
input.map_action("Defend", InputBinding::Key(Key::Q));

// Query action state
if input.action_pressed("Jump") {
    println!("Player is jumping!");
}

if input.action_just_pressed("Attack") {
    println!("Player attacked!");
}
```

## Multiple Bindings

An action can have multiple bindings, allowing alternative inputs:

```rust
use goud_engine::ecs::{InputManager, InputBinding};
use glfw::{Key, MouseButton};

let mut input = InputManager::new();

// Map "Jump" to multiple inputs
input.map_action("Jump", InputBinding::Key(Key::Space));
input.map_action("Jump", InputBinding::Key(Key::W));
input.map_action("Jump", InputBinding::MouseButton(MouseButton::Button2));

// ANY of these inputs will trigger the action
if input.action_pressed("Jump") {
    // Returns true if Space OR W OR Right Mouse Button is pressed
    player.jump();
}
```

## Cross-Platform Support

Map actions to keyboard, mouse, and gamepad inputs:

```rust
use goud_engine::ecs::{InputManager, InputBinding};
use glfw::{Key, MouseButton};

let mut input = InputManager::new();

// Fire action: Keyboard, Mouse, or Gamepad
input.map_action("Fire", InputBinding::Key(Key::Space));
input.map_action("Fire", InputBinding::MouseButton(MouseButton::Button1));
input.map_action("Fire", InputBinding::GamepadButton {
    gamepad_id: 0,
    button: 0,  // A button on Xbox controller
});

// Works with any input device
if input.action_just_pressed("Fire") {
    player.shoot();
}
```

## Managing Actions

Add, remove, and query action mappings:

```rust
use goud_engine::ecs::{InputManager, InputBinding};
use glfw::Key;

let mut input = InputManager::new();

// Add bindings
input.map_action("Jump", InputBinding::Key(Key::Space));
input.map_action("Jump", InputBinding::Key(Key::W));

// Check if action exists
if input.has_action("Jump") {
    println!("Jump action is configured");
}

// Get all bindings for an action
let bindings = input.get_action_bindings("Jump");
println!("Jump has {} bindings", bindings.len());

// Remove a specific binding
let space_binding = InputBinding::Key(Key::Space);
if input.unmap_action("Jump", space_binding) {
    println!("Removed Space from Jump");
}

// Clear all bindings for an action
if input.clear_action("Jump") {
    println!("Cleared all Jump bindings");
}

// Clear all actions
input.clear_all_actions();

// List all actions
for action_name in input.action_names() {
    println!("Action: {}", action_name);
}

println!("Total actions: {}", input.action_count());
```

## Action Queries

Query action state in different ways:

```rust
use goud_engine::ecs::InputManager;

fn player_system(input: &InputManager) {
    // Continuous input (held down)
    if input.action_pressed("MoveForward") {
        player.move_forward(); // Called every frame while held
    }

    // Single press detection
    if input.action_just_pressed("Jump") {
        player.jump(); // Called only once per press
    }

    // Release detection
    if input.action_just_released("Charge") {
        player.release_charged_attack(); // Called when released
    }

    // Analog-style strength (currently 0.0 or 1.0 for digital inputs)
    let fire_strength = input.action_strength("Fire");
    if fire_strength > 0.0 {
        player.shoot(fire_strength); // Future: analog triggers return 0.0-1.0
    }
}
```

## Practical Example: Player Controller

Complete player controller with configurable inputs:

```rust
use goud_engine::ecs::{InputManager, InputBinding, World};
use glfw::Key;

struct PlayerController {
    move_speed: f32,
    jump_force: f32,
}

impl PlayerController {
    fn new() -> Self {
        Self {
            move_speed: 5.0,
            jump_force: 10.0,
        }
    }

    fn setup_default_bindings(input: &mut InputManager) {
        // Movement (WASD + Arrow keys)
        input.map_action("MoveForward", InputBinding::Key(Key::W));
        input.map_action("MoveForward", InputBinding::Key(Key::Up));

        input.map_action("MoveBackward", InputBinding::Key(Key::S));
        input.map_action("MoveBackward", InputBinding::Key(Key::Down));

        input.map_action("MoveLeft", InputBinding::Key(Key::A));
        input.map_action("MoveLeft", InputBinding::Key(Key::Left));

        input.map_action("MoveRight", InputBinding::Key(Key::D));
        input.map_action("MoveRight", InputBinding::Key(Key::Right));

        // Actions (multiple options)
        input.map_action("Jump", InputBinding::Key(Key::Space));
        input.map_action("Jump", InputBinding::Key(Key::W));

        input.map_action("Attack", InputBinding::Key(Key::E));
        input.map_action("Attack", InputBinding::Key(Key::LeftControl));

        input.map_action("Defend", InputBinding::Key(Key::Q));
        input.map_action("Defend", InputBinding::Key(Key::LeftShift));

        // Gamepad support
        input.map_action("Jump", InputBinding::GamepadButton {
            gamepad_id: 0,
            button: 0,  // A button
        });
        input.map_action("Attack", InputBinding::GamepadButton {
            gamepad_id: 0,
            button: 2,  // X button
        });
    }

    fn update(&self, input: &InputManager, delta_time: f32) {
        let mut velocity = (0.0, 0.0);

        // Movement input
        if input.action_pressed("MoveForward") {
            velocity.1 += 1.0;
        }
        if input.action_pressed("MoveBackward") {
            velocity.1 -= 1.0;
        }
        if input.action_pressed("MoveLeft") {
            velocity.0 -= 1.0;
        }
        if input.action_pressed("MoveRight") {
            velocity.0 += 1.0;
        }

        // Apply movement
        if velocity != (0.0, 0.0) {
            let normalized = self.normalize_velocity(velocity);
            self.move_player(
                normalized.0 * self.move_speed * delta_time,
                normalized.1 * self.move_speed * delta_time,
            );
        }

        // Action input
        if input.action_just_pressed("Jump") {
            self.jump(self.jump_force);
        }

        if input.action_just_pressed("Attack") {
            self.attack();
        }

        if input.action_pressed("Defend") {
            self.defend(); // Hold to defend
        } else if input.action_just_released("Defend") {
            self.stop_defending();
        }
    }

    fn normalize_velocity(&self, velocity: (f32, f32)) -> (f32, f32) {
        let magnitude = (velocity.0 * velocity.0 + velocity.1 * velocity.1).sqrt();
        if magnitude > 0.0 {
            (velocity.0 / magnitude, velocity.1 / magnitude)
        } else {
            velocity
        }
    }

    fn move_player(&self, dx: f32, dy: f32) {
        println!("Moving player: ({:.2}, {:.2})", dx, dy);
    }

    fn jump(&self, force: f32) {
        println!("Jumping with force: {:.2}", force);
    }

    fn attack(&self) {
        println!("Attacking!");
    }

    fn defend(&self) {
        println!("Defending...");
    }

    fn stop_defending(&self) {
        println!("Stopped defending");
    }
}

// ECS integration example
fn main() {
    let mut world = World::new();
    let mut input = InputManager::new();

    // Setup controller
    let controller = PlayerController::new();
    PlayerController::setup_default_bindings(&mut input);

    world.insert_resource(input);

    // Game loop (simplified)
    let delta_time = 0.016; // ~60 FPS

    loop {
        // Update input state (from GLFW events in real implementation)
        let mut input = world.resource_mut::<InputManager>();
        input.update();

        // Run game systems
        let input = world.resource::<InputManager>();
        controller.update(&input, delta_time);

        // ... render, etc.
        break; // Exit for example
    }
}
```

## Best Practices

### 1. Use Semantic Action Names

```rust
// Good: Describes WHAT the action does
input.map_action("Jump", InputBinding::Key(Key::Space));
input.map_action("Attack", InputBinding::Key(Key::E));

// Avoid: Describes HOW to trigger it
input.map_action("SpaceBar", InputBinding::Key(Key::Space));
input.map_action("EKey", InputBinding::Key(Key::E));
```

### 2. Support Multiple Input Methods

```rust
// Good: Supports keyboard, mouse, and gamepad
input.map_action("Fire", InputBinding::Key(Key::Space));
input.map_action("Fire", InputBinding::MouseButton(MouseButton::Button1));
input.map_action("Fire", InputBinding::GamepadButton { gamepad_id: 0, button: 0 });

// Okay: Keyboard only (but less accessible)
input.map_action("Fire", InputBinding::Key(Key::Space));
```

### 3. Provide Alternative Bindings

```rust
// Good: WASD + Arrow keys for movement
input.map_action("MoveForward", InputBinding::Key(Key::W));
input.map_action("MoveForward", InputBinding::Key(Key::Up));

// Good: Multiple jump options
input.map_action("Jump", InputBinding::Key(Key::Space));
input.map_action("Jump", InputBinding::Key(Key::W));
```

### 4. Persist Action Mappings

Action mappings persist across frame updates and input state clears:

```rust
let mut input = InputManager::new();
input.map_action("Jump", InputBinding::Key(Key::Space));

// Mappings survive update() calls
input.update();
assert!(input.has_action("Jump"));

// Mappings survive clear() calls (which only clears input state)
input.clear();
assert!(input.has_action("Jump"));

// To remove mappings, use clear_action() or clear_all_actions()
input.clear_all_actions();
assert!(!input.has_action("Jump"));
```

### 5. Make Controls Configurable

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct InputConfig {
    bindings: HashMap<String, Vec<String>>,  // action -> ["Key(Space)", "Key(W)"]
}

impl InputConfig {
    fn apply_to_input_manager(&self, input: &mut InputManager) {
        input.clear_all_actions();

        for (action, binding_strs) in &self.bindings {
            for binding_str in binding_strs {
                if let Some(binding) = self.parse_binding(binding_str) {
                    input.map_action(action, binding);
                }
            }
        }
    }

    fn parse_binding(&self, s: &str) -> Option<InputBinding> {
        // Parse "Key(Space)" -> InputBinding::Key(Key::Space)
        // Parse "MouseButton(Button1)" -> InputBinding::MouseButton(MouseButton::Button1)
        // Parse "GamepadButton(0, 5)" -> InputBinding::GamepadButton { gamepad_id: 0, button: 5 }
        // Implementation left as exercise
        None
    }
}
```

### 6. Use action_strength() for Future Analog Support

```rust
// Current: Digital inputs return 0.0 or 1.0
let fire_strength = input.action_strength("Fire");
if fire_strength > 0.0 {
    player.shoot();
}

// Future: Analog triggers return partial values
let fire_strength = input.action_strength("Fire");
if fire_strength > 0.5 {
    player.shoot_charged(fire_strength); // 0.5-1.0 range
}
```

## Summary

The action mapping system provides:

- **Semantic Input**: Name actions by what they do, not how to trigger them
- **Multi-Binding**: One action can have multiple input sources
- **Cross-Platform**: Keyboard, mouse, and gamepad support
- **Flexible**: Add, remove, and query bindings at runtime
- **Configurable**: Load bindings from config files for user customization
- **Persistent**: Mappings survive frame updates and input state clears
- **Performant**: Hash-based lookups, zero allocation queries

Use action mapping to create flexible, accessible, and user-friendly input systems!
