//! gilrs gamepad integration bridge.
//!
//! Extracts gamepad polling and mapping from `winit_platform.rs` into a
//! dedicated module to keep file sizes manageable. All functions are
//! `pub(super)` so only sibling platform modules can call them.

#![cfg(feature = "gilrs")]

use crate::core::input_manager::InputManager;

/// Maximum number of gamepad slots supported by the engine.
///
/// Re-exported from [`InputManager`]'s canonical constant for use in
/// slot-bound comparisons.
use crate::core::input_manager::MAX_GAMEPAD_SLOTS;

/// Polls all pending gilrs events and forwards them to the [`InputManager`].
///
/// Each physical gamepad is mapped to a slot index (0..MAX_GAMEPAD_SLOTS-1)
/// based on insertion order. Gamepads beyond the last slot are silently ignored.
pub(super) fn poll_gilrs_events(gilrs_instance: &mut gilrs::Gilrs, input: &mut InputManager) {
    while let Some(gilrs::Event { id, event, .. }) = gilrs_instance.next_event() {
        let Some(slot) = map_gilrs_gamepad_id(gilrs_instance, id) else {
            continue;
        };

        match event {
            gilrs::EventType::ButtonPressed(button, _) => {
                if let Some(btn) = map_gilrs_button(button) {
                    input.press_gamepad_button(slot, btn);
                }
            }
            gilrs::EventType::ButtonReleased(button, _) => {
                if let Some(btn) = map_gilrs_button(button) {
                    input.release_gamepad_button(slot, btn);
                }
            }
            gilrs::EventType::AxisChanged(axis, value, _) => {
                if let Some(engine_axis) = map_gilrs_axis(axis) {
                    input.set_gamepad_axis(slot, engine_axis, value);
                }
            }
            gilrs::EventType::Connected => {
                input.set_gamepad_connected(slot, true);
                log::info!("Gamepad connected: slot {slot}");
            }
            gilrs::EventType::Disconnected => {
                input.set_gamepad_connected(slot, false);
                log::info!("Gamepad disconnected: slot {slot}");
            }
            _ => {}
        }
    }
}

/// Maps a gilrs `GamepadId` to a local slot index (0..MAX_GAMEPAD_SLOTS-1).
///
/// Uses the order of connected gamepads as reported by gilrs. Returns `None`
/// if the gamepad would exceed the maximum slot count.
fn map_gilrs_gamepad_id(gilrs_instance: &gilrs::Gilrs, id: gilrs::GamepadId) -> Option<usize> {
    let slot = gilrs_instance
        .gamepads()
        .enumerate()
        .find_map(|(idx, (gid, _))| if gid == id { Some(idx) } else { None })?;
    if slot < MAX_GAMEPAD_SLOTS {
        Some(slot)
    } else {
        None
    }
}

/// Maps a gilrs button to the engine's FFI button constant (u32).
fn map_gilrs_button(button: gilrs::Button) -> Option<u32> {
    Some(match button {
        gilrs::Button::South => 0,
        gilrs::Button::East => 1,
        gilrs::Button::West => 2,
        gilrs::Button::North => 3,
        gilrs::Button::LeftTrigger => 4,
        gilrs::Button::RightTrigger => 5,
        gilrs::Button::Select => 6,
        gilrs::Button::Start => 7,
        gilrs::Button::Mode => 8,
        gilrs::Button::LeftThumb => 9,
        gilrs::Button::RightThumb => 10,
        gilrs::Button::DPadUp => 11,
        gilrs::Button::DPadRight => 12,
        gilrs::Button::DPadDown => 13,
        gilrs::Button::DPadLeft => 14,
        _ => return None,
    })
}

/// Maps a gilrs axis to the engine's [`GamepadAxis`].
fn map_gilrs_axis(axis: gilrs::Axis) -> Option<crate::core::providers::input_types::GamepadAxis> {
    use crate::core::providers::input_types::GamepadAxis as GA;
    Some(match axis {
        gilrs::Axis::LeftStickX => GA::LeftStickX,
        gilrs::Axis::LeftStickY => GA::LeftStickY,
        gilrs::Axis::RightStickX => GA::RightStickX,
        gilrs::Axis::RightStickY => GA::RightStickY,
        gilrs::Axis::LeftZ => GA::LeftTrigger,
        gilrs::Axis::RightZ => GA::RightTrigger,
        _ => return None,
    })
}
