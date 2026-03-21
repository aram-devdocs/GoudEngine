//! UI system FFI exports.
//!
//! Provides C-compatible functions for creating and managing UI trees.
//! The UI system is independent of the ECS world -- nodes live in their
//! own [`UiManager`](crate::ui::UiManager).
//!
//! # ID Packing
//!
//! [`UiNodeId`] is packed into a `u64` for the FFI
//! boundary:
//!
//! ```text
//! +----------------------------------+----------------------------------+
//! |  generation (upper 32 bits)      |  index (lower 32 bits)           |
//! +----------------------------------+----------------------------------+
//! ```
//!
//! The sentinel value `u64::MAX` represents "no node" / invalid.

pub mod events;
pub mod manager;
pub mod node;
pub mod widget;

use std::ffi::c_void;

use crate::core::math::Color;
use crate::ui::UiNodeId;
pub(crate) use crate::ui::{component_from_widget_kind, map_ui_event};

/// Sentinel `u64` returned when a node operation fails or no node exists.
pub const INVALID_NODE_U64: u64 = u64::MAX;

/// Shared error code for FFI functions receiving a null [`UiManager`] pointer.
pub(super) const ERR_NULL_MANAGER: i32 = -1;

/// Shared error code for FFI functions receiving a null output or argument pointer.
pub(super) const ERR_NULL_PTR: i32 = -2;

/// Callback signature for UI events crossing the FFI boundary.
pub type UiEventCallback = extern "C" fn(
    node_id: u64,
    event_kind: u32,
    previous_node_id: u64,
    current_node_id: u64,
    user_data: *mut c_void,
);

/// FFI-safe style payload used by [`widget::goud_ui_set_style`].
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiUiStyle {
    /// True when `background_color` should be applied.
    pub has_background_color: bool,
    /// Background color override when `has_background_color` is true.
    pub background_color: Color,
    /// True when `foreground_color` should be applied.
    pub has_foreground_color: bool,
    /// Foreground/text color override when `has_foreground_color` is true.
    pub foreground_color: Color,
    /// True when `border_color` should be applied.
    pub has_border_color: bool,
    /// Border color override when `has_border_color` is true.
    pub border_color: Color,
    /// True when `border_width` should be applied.
    pub has_border_width: bool,
    /// Border width override when `has_border_width` is true.
    pub border_width: f32,
    /// True when font family bytes should be read from `font_family_ptr/len`.
    pub has_font_family: bool,
    /// UTF-8 font family bytes pointer; nullable when not set.
    pub font_family_ptr: *const u8,
    /// Length in bytes for `font_family_ptr`.
    pub font_family_len: usize,
    /// True when `font_size` should be applied.
    pub has_font_size: bool,
    /// Font size override when `has_font_size` is true.
    pub font_size: f32,
    /// True when texture path bytes should be read from `texture_path_ptr/len`.
    pub has_texture_path: bool,
    /// UTF-8 texture path bytes pointer; nullable when not set.
    pub texture_path_ptr: *const u8,
    /// Length in bytes for `texture_path_ptr`.
    pub texture_path_len: usize,
    /// True when `widget_spacing` should be applied.
    pub has_widget_spacing: bool,
    /// Widget spacing override when `has_widget_spacing` is true.
    pub widget_spacing: f32,
}

impl Default for FfiUiStyle {
    fn default() -> Self {
        Self {
            has_background_color: false,
            background_color: Color::TRANSPARENT,
            has_foreground_color: false,
            foreground_color: Color::TRANSPARENT,
            has_border_color: false,
            border_color: Color::TRANSPARENT,
            has_border_width: false,
            border_width: 0.0,
            has_font_family: false,
            font_family_ptr: std::ptr::null(),
            font_family_len: 0,
            has_font_size: false,
            font_size: 0.0,
            has_texture_path: false,
            texture_path_ptr: std::ptr::null(),
            texture_path_len: 0,
            has_widget_spacing: false,
            widget_spacing: 0.0,
        }
    }
}

/// FFI-safe event payload used by deterministic event polling/read APIs.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FfiUiEvent {
    /// Numeric UI event kind (matches the engine's UI event enum discriminant).
    pub event_kind: u32,
    /// Packed node id of the event source.
    pub node_id: u64,
    /// Packed node id that previously had focus/hover state, or `INVALID_NODE_U64`.
    pub previous_node_id: u64,
    /// Packed node id that currently has focus/hover state, or `INVALID_NODE_U64`.
    pub current_node_id: u64,
}

/// Packs a [`UiNodeId`] into a `u64`.
///
/// Layout: `index` in the lower 32 bits, `generation` in the upper 32 bits.
#[inline]
fn pack_node_id(id: UiNodeId) -> u64 {
    (id.index() as u64) | ((id.generation() as u64) << 32)
}

/// Unpacks a `u64` into a [`UiNodeId`].
///
/// Inverse of [`pack_node_id`].
#[inline]
fn unpack_node_id(packed: u64) -> UiNodeId {
    let index = packed as u32;
    let generation = (packed >> 32) as u32;
    UiNodeId::new(index, generation)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_roundtrip() {
        let id = UiNodeId::new(42, 7);
        let packed = pack_node_id(id);
        let unpacked = unpack_node_id(packed);
        assert_eq!(unpacked.index(), 42);
        assert_eq!(unpacked.generation(), 7);
    }

    #[test]
    fn test_pack_layout() {
        let id = UiNodeId::new(0xDEAD, 0xBEEF);
        let packed = pack_node_id(id);
        assert_eq!(packed & 0xFFFF_FFFF, 0xDEAD);
        assert_eq!(packed >> 32, 0xBEEF);
    }

    #[test]
    fn test_pack_zero() {
        let id = UiNodeId::new(0, 0);
        assert_eq!(pack_node_id(id), 0);
    }

    #[test]
    fn test_invalid_sentinel_is_distinct() {
        let max_id = UiNodeId::new(u32::MAX, u32::MAX);
        assert_eq!(pack_node_id(max_id), INVALID_NODE_U64);
    }

    #[test]
    fn test_invalid_sentinel_roundtrip() {
        let unpacked = unpack_node_id(INVALID_NODE_U64);
        assert!(unpacked.is_invalid());
    }
}
