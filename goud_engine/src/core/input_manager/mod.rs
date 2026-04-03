//! Input management system for ECS integration.
//!
//! The `InputManager` resource provides a centralized interface for querying input state
//! within the ECS. It tracks keyboard keys, mouse buttons, mouse position, and gamepad state
//! across frames, enabling queries for:
//! - Current state (is pressed)
//! - Just pressed (pressed this frame, not last frame)
//! - Just released (released this frame, was pressed last frame)
//!
//! # Architecture
//!
//! The InputManager sits between the native platform event pump and game systems:
//!
//! ```text
//! Platform Events → InputHandler → InputManager → Game Systems
//!                   (platform)     (ECS resource)   (queries)
//! ```
//!
//! # Usage
//!
//! ## Raw Input Queries
//!
//! ```ignore
//! use goud_engine::core::providers::input_types::KeyCode;
//! use goud_engine::ecs::{InputManager, Resource};
//!
//! // In your setup system:
//! world.insert_resource(InputManager::new());
//!
//! // In a system:
//! fn player_movement_system(input: Res<InputManager>) {
//!     if input.key_pressed(KeyCode::W) {
//!         // Move forward continuously while held
//!     }
//!     if input.key_just_pressed(KeyCode::Space) {
//!         // Jump only once per press
//!     }
//! }
//! ```
//!
//! ## Action Mapping
//!
//! Action mapping allows semantic names for input, supporting multiple bindings:
//!
//! ```ignore
//! use goud_engine::core::providers::input_types::KeyCode;
//! use goud_engine::ecs::{InputManager, InputBinding};
//!
//! let mut input = InputManager::new();
//!
//! // Map "Jump" to Space, W key, or gamepad button 0
//! input.map_action("Jump", InputBinding::Key(KeyCode::Space));
//! input.map_action("Jump", InputBinding::Key(KeyCode::W));
//! input.map_action("Jump", InputBinding::GamepadButton { gamepad_id: 0, button: 0 });
//!
//! // Query action state (returns true if ANY binding is pressed)
//! if input.action_pressed("Jump") {
//!     player.jump();
//! }
//!
//! if input.action_just_pressed("Attack") {
//!     player.attack();
//! }
//! ```
//!
//! # Frame Management
//!
//! Call `update()` at the start of each frame to advance the input state:
//!
//! ```ignore
//! fn input_update_system(mut input: ResMut<InputManager>) {
//!     input.update();
//! }
//! ```

mod actions;
mod buffer;
mod gamepad;
mod manager;
mod touch;
mod types;

#[cfg(test)]
mod tests;

pub use manager::InputManager;
pub use types::InputBinding;
