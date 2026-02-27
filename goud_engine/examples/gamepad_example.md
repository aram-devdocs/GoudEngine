# Gamepad Support Example

This example demonstrates comprehensive gamepad support in GoudEngine, including:
- Button input detection
- Analog stick reading
- Trigger input
- Connection status tracking
- Vibration/rumble control
- Analog deadzone configuration

## Basic Gamepad Button Input

```rust
use goud_engine::ecs::{InputManager, World};
use glfw::GamepadAxis;

fn main() {
    let mut world = World::new();
    let mut input = InputManager::new();

    // In your game loop, check for gamepad button presses
    loop {
        // Read button state from GLFW (in actual game code)
        // For this example, we'll simulate button presses
        input.press_gamepad_button(0, 0); // Gamepad 0, button 0 (A on Xbox, Cross on PS)

        // Check if button is pressed
        if input.gamepad_button_pressed(0, 0) {
            println!("Button A pressed!");
        }

        // Check for just pressed (fires once per press)
        if input.gamepad_button_just_pressed(0, 1) {
            println!("Button B just pressed!");
        }

        // Update at end of frame
        input.update();
    }
}
```

## Analog Stick Input

Analog sticks return Vec2 values where:
- X-axis: -1.0 (left) to 1.0 (right)
- Y-axis: -1.0 (down) to 1.0 (up)

```rust
use goud_engine::ecs::InputManager;
use goud_engine::core::math::Vec2;
use glfw::GamepadAxis;

fn player_movement_system(input: &InputManager) {
    // Get left stick position
    let left_stick = input.gamepad_left_stick(0);

    if left_stick.length() > 0.0 {
        println!("Moving player: x={:.2}, y={:.2}", left_stick.x, left_stick.y);

        // Calculate movement direction and magnitude
        let direction = left_stick.normalize();
        let magnitude = left_stick.length().min(1.0); // Clamp to max 1.0

        // Apply movement to player
        // player.velocity = direction * magnitude * player.speed;
    }

    // Get right stick for camera control
    let right_stick = input.gamepad_right_stick(0);

    if right_stick.length() > 0.0 {
        println!("Camera look: x={:.2}, y={:.2}", right_stick.x, right_stick.y);
        // camera.rotate(right_stick.x * sensitivity, right_stick.y * sensitivity);
    }

    // Individual axis access (if needed)
    let left_x = input.gamepad_axis(0, GamepadAxis::AxisLeftX);
    let left_y = input.gamepad_axis(0, GamepadAxis::AxisLeftY);
}
```

## Trigger Input

Triggers return values from 0.0 (not pressed) to 1.0 (fully pressed).

```rust
use goud_engine::ecs::InputManager;

fn vehicle_system(input: &InputManager) {
    // Read trigger values (0.0 to 1.0)
    let gas = input.gamepad_right_trigger(0);
    let brake = input.gamepad_left_trigger(0);

    if gas > 0.0 {
        println!("Gas: {:.0}%", gas * 100.0);
        // vehicle.accelerate(gas);
    }

    if brake > 0.0 {
        println!("Brake: {:.0}%", brake * 100.0);
        // vehicle.brake(brake);
    }

    // Analog shooting/aiming intensity
    let aim_intensity = input.gamepad_left_trigger(0);
    if aim_intensity > 0.1 {
        // Zoom camera based on trigger pressure
        // camera.zoom = 1.0 + (aim_intensity * 2.0);
    }
}
```

## Deadzone Configuration

Deadzones prevent stick drift and accidental input from centered sticks.

```rust
use goud_engine::ecs::InputManager;
use std::time::Duration;

fn configure_input() -> InputManager {
    let mut input = InputManager::new();

    // Default deadzone is 0.1 (10%)
    println!("Default deadzone: {}", input.analog_deadzone());

    // Increase deadzone for worn controllers
    input.set_analog_deadzone(0.15); // 15%

    // Decrease for precision input
    input.set_analog_deadzone(0.05); // 5%

    // Values within deadzone are clamped to 0.0
    input.set_gamepad_axis(0, glfw::GamepadAxis::AxisLeftX, 0.08);
    assert_eq!(input.gamepad_axis(0, glfw::GamepadAxis::AxisLeftX), 0.0); // Within deadzone

    input.set_gamepad_axis(0, glfw::GamepadAxis::AxisLeftX, 0.2);
    assert_eq!(input.gamepad_axis(0, glfw::GamepadAxis::AxisLeftX), 0.2); // Outside deadzone

    input
}
```

## Gamepad Connection Management

Track which gamepads are connected and handle hot-plugging.

```rust
use goud_engine::ecs::InputManager;

fn check_gamepad_connections(input: &mut InputManager) {
    // Check connection status
    if input.is_gamepad_connected(0) {
        println!("Gamepad 0 is connected");
    } else {
        println!("Gamepad 0 not found - waiting for controller");
    }

    // Get count of connected gamepads
    let count = input.connected_gamepad_count();
    println!("Connected gamepads: {}", count);

    // Iterate over all connected gamepads
    for gamepad_id in input.connected_gamepads() {
        println!("Gamepad {} is ready", gamepad_id);

        // You can process input for each connected gamepad
        if input.gamepad_button_just_pressed(gamepad_id, 0) {
            println!("Player {} pressed A!", gamepad_id + 1);
        }
    }

    // Simulate connection/disconnection (in actual code, read from GLFW)
    // When gamepad connects:
    input.set_gamepad_connected(0, true);

    // When gamepad disconnects:
    // input.set_gamepad_connected(0, false);
}
```

## Vibration/Rumble

Control gamepad vibration intensity (note: actual vibration requires platform support).

```rust
use goud_engine::ecs::InputManager;

fn explosion_effect(input: &mut InputManager, gamepad_id: usize) {
    // Strong vibration for explosions
    input.set_gamepad_vibration(gamepad_id, 1.0); // Full intensity

    // After some time, reduce vibration
    // (In actual code, do this over multiple frames)
    // input.set_gamepad_vibration(gamepad_id, 0.5); // Half intensity
    // input.stop_gamepad_vibration(gamepad_id); // Stop vibration
}

fn damage_feedback(input: &mut InputManager) {
    // Medium vibration for damage
    input.set_gamepad_vibration(0, 0.6);

    // Auto-stop after duration (platform layer handles this)
    // Or manually stop after timer expires:
    // input.stop_gamepad_vibration(0);
}

fn stop_all_effects(input: &mut InputManager) {
    // Stop vibration on all gamepads
    input.stop_all_vibration();
}
```

## Complete Game Loop Example

```rust
use goud_engine::ecs::{InputManager, World};
use goud_engine::core::math::Vec2;
use glfw::GamepadAxis;

struct Player {
    position: Vec2,
    velocity: Vec2,
    health: f32,
}

impl Player {
    fn new() -> Self {
        Self {
            position: Vec2::zero(),
            velocity: Vec2::zero(),
            health: 100.0,
        }
    }

    fn update(&mut self, input: &InputManager, gamepad_id: usize, delta_time: f32) {
        // Movement with left stick
        let movement = input.gamepad_left_stick(gamepad_id);

        if movement.length() > 0.0 {
            let speed = 200.0; // pixels per second
            self.velocity = movement * speed;
            self.position = self.position + (self.velocity * delta_time);
        } else {
            self.velocity = Vec2::zero();
        }

        // Jump with A button (button 0)
        if input.gamepad_button_just_pressed(gamepad_id, 0) {
            println!("Jump!");
            // self.velocity.y = jump_force;
        }

        // Attack with B button (button 1)
        if input.gamepad_button_just_pressed(gamepad_id, 1) {
            println!("Attack!");
            // self.attack();
        }

        // Aim with right stick
        let aim_direction = input.gamepad_right_stick(gamepad_id);
        if aim_direction.length() > 0.1 {
            // Rotate player to face aim direction
            // let angle = aim_direction.y.atan2(aim_direction.x);
        }

        // Special ability with right trigger
        let special_intensity = input.gamepad_right_trigger(gamepad_id);
        if special_intensity > 0.5 {
            println!("Charging special ability: {:.0}%", special_intensity * 100.0);
            // self.charge_special(special_intensity);
        }
    }

    fn take_damage(&mut self, damage: f32, input: &mut InputManager, gamepad_id: usize) {
        self.health -= damage;

        // Vibration feedback for damage
        let intensity = (damage / 100.0).min(1.0);
        input.set_gamepad_vibration(gamepad_id, intensity);

        if self.health <= 0.0 {
            println!("Player defeated!");
            input.stop_gamepad_vibration(gamepad_id);
        }
    }
}

fn game_loop() {
    let mut input = InputManager::new();
    let mut player = Player::new();

    // Configure deadzone
    input.set_analog_deadzone(0.12);

    // Check for connected gamepads
    input.set_gamepad_connected(0, true); // Simulated connection

    let delta_time = 0.016; // 60 FPS

    loop {
        // In real game, read input from GLFW here
        // For example: process GLFW gamepad events
        // glfw.poll_events();
        // for (_, event) in glfw::flush_messages(&events) {
        //     match event {
        //         glfw::WindowEvent::GamepadButton(gamepad, button, action) => {
        //             if action == glfw::Action::Press {
        //                 input.press_gamepad_button(gamepad as usize, button as u32);
        //             } else {
        //                 input.release_gamepad_button(gamepad as usize, button as u32);
        //             }
        //         }
        //         glfw::WindowEvent::GamepadAxis(gamepad, axis, value) => {
        //             input.set_gamepad_axis(gamepad as usize, axis, value);
        //         }
        //         _ => {}
        //     }
        // }

        // Update player
        if input.is_gamepad_connected(0) {
            player.update(&input, 0, delta_time);
        } else {
            println!("Waiting for gamepad connection...");
        }

        // Update input manager at end of frame
        input.update();

        // Break condition for example
        if input.gamepad_button_pressed(0, 7) {
            // Start/Options button
            break;
        }
    }
}
```

## Multi-Player Support

Handle multiple gamepads for local multiplayer.

```rust
use goud_engine::ecs::InputManager;

struct GameSession {
    input: InputManager,
    players: Vec<Player>,
}

impl GameSession {
    fn new(player_count: usize) -> Self {
        let mut input = InputManager::new();

        // Connect gamepads for each player
        for i in 0..player_count {
            input.set_gamepad_connected(i, true);
        }

        // Create players
        let mut players = Vec::new();
        for _ in 0..player_count {
            players.push(Player::new());
        }

        Self { input, players }
    }

    fn update(&mut self, delta_time: f32) {
        // Update each player with their respective gamepad
        for (i, player) in self.players.iter_mut().enumerate() {
            if self.input.is_gamepad_connected(i) {
                player.update(&self.input, i, delta_time);
            }
        }

        self.input.update();
    }
}

fn multiplayer_game() {
    let mut session = GameSession::new(2); // 2 players

    // Game loop
    loop {
        session.update(0.016);

        // Check for disconnections
        for (i, _) in session.players.iter().enumerate() {
            if !session.input.is_gamepad_connected(i) {
                println!("Player {} gamepad disconnected!", i + 1);
            }
        }

        // Break condition
        // if all_players_ready_to_quit { break; }
    }
}
```

## Action Mapping with Gamepads

Combine keyboard, mouse, and gamepad inputs for flexible controls.

```rust
use goud_engine::ecs::{InputManager, InputBinding};
use glfw::Key;

fn setup_controls() -> InputManager {
    let mut input = InputManager::new();

    // Map "Jump" to multiple inputs
    input.map_action("Jump", InputBinding::Key(Key::Space)); // Keyboard
    input.map_action("Jump", InputBinding::GamepadButton { gamepad_id: 0, button: 0 }); // A button
    input.map_action("Jump", InputBinding::GamepadButton { gamepad_id: 1, button: 0 }); // Player 2

    // Map "Attack"
    input.map_action("Attack", InputBinding::Key(Key::E));
    input.map_action("Attack", InputBinding::GamepadButton { gamepad_id: 0, button: 1 }); // B button

    // Map "Pause"
    input.map_action("Pause", InputBinding::Key(Key::Escape));
    input.map_action("Pause", InputBinding::GamepadButton { gamepad_id: 0, button: 7 }); // Start

    input
}

fn game_update(input: &InputManager) {
    // Query actions instead of specific inputs
    if input.action_just_pressed("Jump") {
        println!("Jump! (from any input)");
    }

    if input.action_pressed("Attack") {
        println!("Attacking...");
    }

    if input.action_just_pressed("Pause") {
        println!("Game paused");
    }
}
```

## Platform Integration Notes

### GLFW Gamepad Events

In actual game code, integrate with GLFW's gamepad events:

```rust
use glfw::{Glfw, Window, WindowEvent, GamepadAxis, GamepadButton, Action};
use goud_engine::ecs::InputManager;

fn process_glfw_events(
    glfw: &mut Glfw,
    events: &glfw::GlfwReceiver<(f64, WindowEvent)>,
    input: &mut InputManager,
) {
    glfw.poll_events();

    for (_, event) in glfw::flush_messages(events) {
        match event {
            // Gamepad button events
            WindowEvent::GamepadButton(gamepad, button, action) => {
                let gamepad_id = gamepad as usize;
                let button_id = button as u32;

                match action {
                    Action::Press => input.press_gamepad_button(gamepad_id, button_id),
                    Action::Release => input.release_gamepad_button(gamepad_id, button_id),
                    _ => {}
                }
            }

            // Gamepad axis events
            WindowEvent::GamepadAxis(gamepad, axis, value) => {
                input.set_gamepad_axis(gamepad as usize, axis, value as f32);
            }

            // Gamepad connection events
            WindowEvent::GamepadConnected(gamepad) => {
                println!("Gamepad {} connected", gamepad);
                input.set_gamepad_connected(gamepad as usize, true);
            }

            WindowEvent::GamepadDisconnected(gamepad) => {
                println!("Gamepad {} disconnected", gamepad);
                input.set_gamepad_connected(gamepad as usize, false);
            }

            _ => {}
        }
    }
}
```

## Best Practices

1. **Always check connection before reading gamepad input**
   ```rust
   if input.is_gamepad_connected(0) {
       let stick = input.gamepad_left_stick(0);
       // Use stick...
   }
   ```

2. **Configure deadzone based on controller quality**
   - New controllers: 0.05-0.1
   - Average controllers: 0.1-0.15
   - Worn controllers: 0.15-0.25

3. **Use action mapping for flexibility**
   - Allows players to reconfigure controls
   - Supports keyboard + gamepad simultaneously
   - Easier to maintain

4. **Provide vibration options**
   - Some players prefer no vibration
   - Allow intensity adjustment
   - Auto-stop vibration after 1-2 seconds

5. **Handle hot-plugging gracefully**
   - Detect disconnections mid-game
   - Pause game or show reconnect prompt
   - Preserve player state

6. **Normalize stick input for movement**
   ```rust
   let stick = input.gamepad_left_stick(0);
   if stick.length() > 0.0 {
       let direction = stick.normalize();
       let magnitude = stick.length().min(1.0);
       // Apply movement...
   }
   ```

## Common Gamepad Button Mappings

| Button ID | Xbox | PlayStation | Switch |
|-----------|------|-------------|--------|
| 0 | A | Cross | B |
| 1 | B | Circle | A |
| 2 | X | Square | Y |
| 3 | Y | Triangle | X |
| 4 | LB | L1 | L |
| 5 | RB | R1 | R |
| 6 | Back | Share | Minus |
| 7 | Start | Options | Plus |
| 8 | L3 | L3 | L3 |
| 9 | R3 | R3 | R3 |
| 10 | Guide | PS | Home |

## Analog Axes

| Axis | Description |
|------|-------------|
| AxisLeftX | Left stick X-axis |
| AxisLeftY | Left stick Y-axis |
| AxisRightX | Right stick X-axis |
| AxisRightY | Right stick Y-axis |
| AxisLeftTrigger | Left trigger (L2/LT) |
| AxisRightTrigger | Right trigger (R2/RT) |

## Troubleshooting

**Stick drift:**
- Increase deadzone: `input.set_analog_deadzone(0.2);`
- Check controller health

**Input not detected:**
- Verify gamepad is connected: `input.is_gamepad_connected(0)`
- Check GLFW event processing
- Ensure correct button/axis IDs

**Triggers not working:**
- Some controllers map triggers to axes 2 and 3
- Use `gamepad_left_trigger()` and `gamepad_right_trigger()` helpers
- Check GLFW gamepad mapping

**Multiple gamepads interfering:**
- Use separate gamepad IDs (0, 1, 2, 3)
- Check `connected_gamepads()` iterator
- Verify player-gamepad assignment
