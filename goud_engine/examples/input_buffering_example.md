# Input Buffering Example

This example demonstrates how to use the input buffering system in GoudEngine for detecting input sequences and combos.

## Overview

Input buffering allows you to:
- Detect input sequences (e.g., fighting game combos)
- Support double-tap detection
- Implement combo systems
- Create time-sensitive input patterns

The buffer remembers inputs for a configurable duration (default: 200ms), allowing you to detect sequences even if the player doesn't press buttons at exactly the same time.

## Basic Sequence Detection

```rust
use goud_engine::ecs::{InputManager, InputBinding};
use glfw::Key;

fn main() {
    let mut input = InputManager::new();

    // Define a simple sequence: A -> B -> C
    let sequence = vec![
        InputBinding::Key(Key::A),
        InputBinding::Key(Key::B),
        InputBinding::Key(Key::C),
    ];

    // In your game loop, check for the sequence
    if input.sequence_detected(&sequence) {
        println!("Sequence detected!");
        // Optionally consume the sequence to prevent repeated detection
        input.clear_buffer();
    }
}
```

## Fighting Game Combos

```rust
use goud_engine::ecs::{InputManager, InputBinding};
use glfw::Key;
use std::time::Duration;

fn setup_fighting_game_input() -> InputManager {
    // Use a longer buffer for fighting games (300ms is common)
    let mut input = InputManager::with_buffer_duration(Duration::from_millis(300));

    input
}

fn check_combos(input: &mut InputManager, player: &mut Player) {
    // Hadouken combo: Down -> Down -> Forward -> Punch
    let hadouken = vec![
        InputBinding::Key(Key::Down),
        InputBinding::Key(Key::Down),
        InputBinding::Key(Key::Right),
        InputBinding::Key(Key::Space),
    ];

    // Shoryuken combo: Forward -> Down -> Down-Forward -> Punch
    let shoryuken = vec![
        InputBinding::Key(Key::Right),
        InputBinding::Key(Key::Down),
        InputBinding::Key(Key::Down), // In practice, you'd check diagonal
        InputBinding::Key(Key::Space),
    ];

    // Check combos in priority order (most specific first)
    if input.consume_sequence(&hadouken) {
        player.perform_hadouken();
    } else if input.consume_sequence(&shoryuken) {
        player.perform_shoryuken();
    }
}
```

## Double-Tap Detection

```rust
use goud_engine::ecs::{InputManager, InputBinding};
use glfw::Key;

fn check_double_tap(input: &mut InputManager) {
    // Detect double-tap forward (common for dashing)
    let double_forward = vec![
        InputBinding::Key(Key::Right),
        InputBinding::Key(Key::Right),
    ];

    if input.consume_sequence(&double_forward) {
        player.dash_forward();
    }

    // You can also check double-tap for any direction
    let double_left = vec![
        InputBinding::Key(Key::Left),
        InputBinding::Key(Key::Left),
    ];

    if input.consume_sequence(&double_left) {
        player.dash_backward();
    }
}
```

## Custom Buffer Duration

Different game types need different buffer durations:

```rust
use goud_engine::ecs::InputManager;
use std::time::Duration;

// Strict timing (rhythm games, competitive fighters)
let strict_input = InputManager::with_buffer_duration(Duration::from_millis(50));

// Balanced (most action games)
let balanced_input = InputManager::with_buffer_duration(Duration::from_millis(200));

// Lenient (casual games, accessibility)
let lenient_input = InputManager::with_buffer_duration(Duration::from_millis(500));

// Runtime adjustment
let mut input = InputManager::new();
input.set_buffer_duration(Duration::from_millis(300));
```

## Debugging Input Buffer

```rust
use goud_engine::ecs::InputManager;

fn debug_input_buffer(input: &InputManager) {
    println!("Buffer size: {}", input.buffer_size());
    println!("Time since last input: {:?}", input.time_since_last_input());

    // Print all buffered inputs with their ages
    for (binding, age) in input.buffered_inputs() {
        println!("  {} - {:.3}s ago", binding, age);
    }
}
```

## Complete Example: Combo System

```rust
use goud_engine::ecs::{InputManager, InputBinding};
use glfw::Key;
use std::time::Duration;

struct ComboSystem {
    input: InputManager,
    combos: Vec<Combo>,
}

struct Combo {
    name: String,
    sequence: Vec<InputBinding>,
    damage: u32,
}

impl ComboSystem {
    fn new() -> Self {
        let mut input = InputManager::with_buffer_duration(Duration::from_millis(300));

        let combos = vec![
            Combo {
                name: "Lightning Strike".to_string(),
                sequence: vec![
                    InputBinding::Key(Key::Down),
                    InputBinding::Key(Key::Down),
                    InputBinding::Key(Key::Right),
                    InputBinding::Key(Key::Space),
                ],
                damage: 50,
            },
            Combo {
                name: "Rising Dragon".to_string(),
                sequence: vec![
                    InputBinding::Key(Key::Right),
                    InputBinding::Key(Key::Down),
                    InputBinding::Key(Key::Space),
                ],
                damage: 40,
            },
            Combo {
                name: "Quick Jab".to_string(),
                sequence: vec![
                    InputBinding::Key(Key::Space),
                    InputBinding::Key(Key::Space),
                ],
                damage: 20,
            },
        ];

        Self { input, combos }
    }

    fn update(&mut self, player: &mut Player) {
        // Check combos in priority order (longest sequences first)
        for combo in &self.combos {
            if self.input.consume_sequence(&combo.sequence) {
                println!("{} executed! Damage: {}", combo.name, combo.damage);
                player.deal_damage(combo.damage);
                return; // Only execute one combo per frame
            }
        }
    }

    fn handle_input(&mut self, key: Key, pressed: bool) {
        if pressed {
            self.input.press_key(key);
        } else {
            self.input.release_key(key);
        }
    }

    fn reset(&mut self) {
        self.input.clear_buffer();
    }
}
```

## Key Features

### Buffer Management
- **Auto-expiration**: Old inputs are automatically removed based on buffer duration
- **Size limiting**: Buffer caps at 32 inputs to prevent memory growth
- **Manual clearing**: Call `clear_buffer()` to reset after combo execution

### Sequence Detection
- **Order-sensitive**: Inputs must match the exact sequence order
- **Extra inputs allowed**: Sequence detection ignores inputs not in the sequence
- **Partial sequences**: Incomplete sequences don't trigger false positives

### Performance
- **Efficient scanning**: O(n*m) where n=buffer size, m=sequence length
- **Minimal allocation**: Uses VecDeque for fast push/pop operations
- **Frame-independent**: Buffer persists across frames until expiration

## Best Practices

1. **Consume sequences**: Use `consume_sequence()` instead of `sequence_detected()` to prevent repeated triggers
2. **Check longest first**: Test longer combos before shorter ones to prevent premature matches
3. **Tune buffer duration**: Adjust based on player feedback and game feel
4. **Clear on context switch**: Reset buffer when changing game states (pause menu, level transitions)
5. **Visual feedback**: Show buffer contents during development to tune timing

## Integration with ECS

```rust
use goud_engine::ecs::{World, Res, ResMut};

fn input_system(mut input: ResMut<InputManager>) {
    // Update is called automatically by the engine
    // You can check sequences in your game systems
}

fn combo_detection_system(
    input: Res<InputManager>,
    mut query: Query<&mut Player>,
) {
    for mut player in query.iter_mut() {
        // Check for combos
        if input.sequence_detected(&player.current_combo) {
            player.execute_combo();
        }
    }
}
```

## Troubleshooting

**Combos not detecting:**
- Check buffer duration (too short?)
- Verify input order matches sequence
- Ensure inputs are being buffered (use `buffered_inputs()` to debug)
- Check if buffer is being cleared too early

**Combos triggering multiple times:**
- Use `consume_sequence()` instead of `sequence_detected()`
- Clear buffer after combo execution

**Inputs not buffering:**
- Only new presses are buffered (not held inputs)
- Release key before pressing again for double-tap detection
