//! UI node manipulation FFI functions.
//!
//! Provides C-compatible functions for creating, removing, and reparenting
//! UI nodes within a [`UiManager`].
//!
//! # ID Packing
//!
//! All node IDs cross the FFI boundary as `u64` values. See the
//! [parent module](super) for the packing layout.

use crate::ui::{UiButton, UiComponent, UiImage, UiLabel, UiManager, UiSlider};

use super::{pack_node_id, unpack_node_id, INVALID_NODE_U64};

/// Maps an FFI `i32` component type to a [`UiComponent`] variant.
///
/// * `0` -> `Some(UiComponent::Panel)`
/// * `1` -> `Some(UiComponent::Button(default))`
/// * `2` -> `Some(UiComponent::Label(default))`
/// * `3` -> `Some(UiComponent::Image(default))`
/// * `4` -> `Some(UiComponent::Slider(default))`
/// * `-1` -> `None` (no component)
/// * anything else -> `None` (no component, logs a warning)
fn component_from_ffi(component_type: i32) -> Option<UiComponent> {
    match component_type {
        0 => Some(UiComponent::Panel),
        1 => Some(UiComponent::Button(UiButton::default())),
        2 => Some(UiComponent::Label(UiLabel::default())),
        3 => Some(UiComponent::Image(UiImage::default())),
        4 => Some(UiComponent::Slider(UiSlider::new(0.0, 1.0, 0.0))),
        -1 => None,
        _ => {
            log::warn!("Unknown component type: {}", component_type);
            None
        }
    }
}

// ============================================================================
// Node Lifecycle
// ============================================================================

/// Creates a new UI node in the manager.
///
/// # Arguments
///
/// * `mgr` - Mutable pointer to the [`UiManager`]. Must not be null.
/// * `component_type` - The component variant to attach:
///   - `0` = Panel
///   - `1` = Button
///   - `2` = Label
///   - `3` = Image
///   - `4` = Slider
///   - `-1` = no component
///
/// # Returns
///
/// The packed `u64` node ID on success, or [`INVALID_NODE_U64`] if `mgr`
/// is null.
///
/// # Ownership
///
/// The node is owned by the manager. The caller receives an ID handle, not
/// a pointer.
///
/// # Safety
///
/// `mgr` must be either null (returns sentinel) or a valid, exclusively-owned
/// pointer to a `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_create_node(mgr: *mut UiManager, component_type: i32) -> u64 {
    if mgr.is_null() {
        return INVALID_NODE_U64;
    }
    // SAFETY: Caller guarantees `mgr` is a valid, exclusively-owned UiManager
    // pointer. We borrow it mutably for the duration of this call.
    let manager = &mut *mgr;
    let component = component_from_ffi(component_type);
    let id = manager.create_node(component);
    pack_node_id(id)
}

/// Removes a UI node (and its entire subtree) from the manager.
///
/// # Arguments
///
/// * `mgr` - Mutable pointer to the [`UiManager`]. Must not be null.
/// * `node_id` - Packed `u64` ID of the node to remove.
///
/// # Returns
///
/// * `0` on success (node existed and was removed)
/// * `-1` if `mgr` is null
/// * `-2` if the node was not found
///
/// # Safety
///
/// `mgr` must be either null (returns -1) or a valid, exclusively-owned
/// pointer to a `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_remove_node(mgr: *mut UiManager, node_id: u64) -> i32 {
    if mgr.is_null() {
        return -1;
    }
    // SAFETY: Caller guarantees `mgr` is a valid, exclusively-owned UiManager
    // pointer. We borrow it mutably for the duration of this call.
    let manager = &mut *mgr;
    let id = unpack_node_id(node_id);
    if manager.remove_node(id) {
        0
    } else {
        -2
    }
}

// ============================================================================
// Hierarchy
// ============================================================================

/// Sets (or clears) the parent of a UI node.
///
/// # Arguments
///
/// * `mgr` - Mutable pointer to the [`UiManager`]. Must not be null.
/// * `child_id` - Packed `u64` ID of the child node.
/// * `parent_id` - Packed `u64` ID of the new parent, or [`INVALID_NODE_U64`]
///   (`u64::MAX`) to detach the child (make it a root).
///
/// # Returns
///
/// * `0` on success
/// * `-1` if `mgr` is null
/// * `-2` if the operation failed (node not found, cycle detected, etc.)
///
/// # Safety
///
/// `mgr` must be either null (returns -1) or a valid, exclusively-owned
/// pointer to a `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_set_parent(
    mgr: *mut UiManager,
    child_id: u64,
    parent_id: u64,
) -> i32 {
    if mgr.is_null() {
        return -1;
    }
    // SAFETY: Caller guarantees `mgr` is a valid, exclusively-owned UiManager
    // pointer. We borrow it mutably for the duration of this call.
    let manager = &mut *mgr;
    let child = unpack_node_id(child_id);
    let parent = if parent_id == INVALID_NODE_U64 {
        None
    } else {
        Some(unpack_node_id(parent_id))
    };

    match manager.set_parent(child, parent) {
        Ok(()) => 0,
        Err(_) => -2,
    }
}

/// Returns the parent of a UI node.
///
/// # Arguments
///
/// * `mgr` - Const pointer to the [`UiManager`].
/// * `node_id` - Packed `u64` ID of the node to query.
///
/// # Returns
///
/// The packed `u64` parent ID, or [`INVALID_NODE_U64`] if:
/// - `mgr` is null
/// - the node does not exist
/// - the node has no parent (is a root)
///
/// # Safety
///
/// `mgr` must be either null (returns sentinel) or a valid pointer to a
/// `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_get_parent(mgr: *const UiManager, node_id: u64) -> u64 {
    if mgr.is_null() {
        return INVALID_NODE_U64;
    }
    // SAFETY: Caller guarantees `mgr` is a valid UiManager pointer. We borrow
    // it immutably for the duration of this call.
    let manager = &*mgr;
    let id = unpack_node_id(node_id);
    match manager.get_node(id).and_then(|n| n.parent()) {
        Some(parent) => pack_node_id(parent),
        None => INVALID_NODE_U64,
    }
}

/// Returns the number of children of a UI node.
///
/// # Arguments
///
/// * `mgr` - Const pointer to the [`UiManager`].
/// * `node_id` - Packed `u64` ID of the node to query.
///
/// # Returns
///
/// The child count, or `0` if `mgr` is null or the node does not exist.
///
/// # Safety
///
/// `mgr` must be either null (returns 0) or a valid pointer to a `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_get_child_count(mgr: *const UiManager, node_id: u64) -> u32 {
    if mgr.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees `mgr` is a valid UiManager pointer. We borrow
    // it immutably for the duration of this call.
    let manager = &*mgr;
    let id = unpack_node_id(node_id);
    manager
        .get_node(id)
        .map(|n| n.children().len() as u32)
        .unwrap_or(0)
}

/// Returns the child at a given index for a UI node.
///
/// # Arguments
///
/// * `mgr` - Const pointer to the [`UiManager`].
/// * `node_id` - Packed `u64` ID of the parent node.
/// * `index` - Zero-based child index.
///
/// # Returns
///
/// The packed `u64` child ID, or [`INVALID_NODE_U64`] if:
/// - `mgr` is null
/// - the node does not exist
/// - `index` is out of bounds
///
/// # Safety
///
/// `mgr` must be either null (returns sentinel) or a valid pointer to a
/// `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_get_child_at(
    mgr: *const UiManager,
    node_id: u64,
    index: u32,
) -> u64 {
    if mgr.is_null() {
        return INVALID_NODE_U64;
    }
    // SAFETY: Caller guarantees `mgr` is a valid UiManager pointer. We borrow
    // it immutably for the duration of this call.
    let manager = &*mgr;
    let id = unpack_node_id(node_id);
    manager
        .get_node(id)
        .and_then(|n| n.children().get(index as usize))
        .map(|&child| pack_node_id(child))
        .unwrap_or(INVALID_NODE_U64)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::super::manager::{goud_ui_manager_create, goud_ui_manager_destroy};
    use super::*;

    #[test]
    fn test_create_node_null_mgr() {
        // SAFETY: Null is explicitly handled.
        let id = unsafe { goud_ui_create_node(std::ptr::null_mut(), 0) };
        assert_eq!(id, INVALID_NODE_U64);
    }

    #[test]
    fn test_create_and_count() {
        let mgr = goud_ui_manager_create();
        // SAFETY: mgr was just created.
        unsafe {
            let id = goud_ui_create_node(mgr, 0);
            assert_ne!(id, INVALID_NODE_U64);

            let count = super::super::manager::goud_ui_manager_node_count(mgr);
            assert_eq!(count, 1);

            goud_ui_manager_destroy(mgr);
        }
    }

    #[test]
    fn test_create_node_supports_all_widget_kind_codes() {
        let mgr = goud_ui_manager_create();
        // SAFETY: mgr was just created.
        unsafe {
            let panel = goud_ui_create_node(mgr, 0);
            let button = goud_ui_create_node(mgr, 1);
            let label = goud_ui_create_node(mgr, 2);
            let image = goud_ui_create_node(mgr, 3);
            let slider = goud_ui_create_node(mgr, 4);
            let none = goud_ui_create_node(mgr, -1);

            assert_ne!(panel, INVALID_NODE_U64);
            assert_ne!(button, INVALID_NODE_U64);
            assert_ne!(label, INVALID_NODE_U64);
            assert_ne!(image, INVALID_NODE_U64);
            assert_ne!(slider, INVALID_NODE_U64);
            assert_ne!(none, INVALID_NODE_U64);
            assert_eq!(super::super::manager::goud_ui_manager_node_count(mgr), 6);

            goud_ui_manager_destroy(mgr);
        }
    }

    #[test]
    fn test_remove_node() {
        let mgr = goud_ui_manager_create();
        // SAFETY: mgr was just created.
        unsafe {
            let id = goud_ui_create_node(mgr, 0);
            let result = goud_ui_remove_node(mgr, id);
            assert_eq!(result, 0);

            // Removing again should fail.
            let result2 = goud_ui_remove_node(mgr, id);
            assert_eq!(result2, -2);

            goud_ui_manager_destroy(mgr);
        }
    }

    #[test]
    fn test_remove_node_null_mgr() {
        // SAFETY: Null is explicitly handled.
        let result = unsafe { goud_ui_remove_node(std::ptr::null_mut(), 0) };
        assert_eq!(result, -1);
    }

    #[test]
    fn test_set_and_get_parent() {
        let mgr = goud_ui_manager_create();
        // SAFETY: mgr was just created.
        unsafe {
            let parent = goud_ui_create_node(mgr, 0);
            let child = goud_ui_create_node(mgr, 0);

            let result = goud_ui_set_parent(mgr, child, parent);
            assert_eq!(result, 0);

            let got_parent = goud_ui_get_parent(mgr, child);
            assert_eq!(got_parent, parent);

            goud_ui_manager_destroy(mgr);
        }
    }

    #[test]
    fn test_set_parent_detach() {
        let mgr = goud_ui_manager_create();
        // SAFETY: mgr was just created.
        unsafe {
            let parent = goud_ui_create_node(mgr, 0);
            let child = goud_ui_create_node(mgr, 0);

            goud_ui_set_parent(mgr, child, parent);
            let result = goud_ui_set_parent(mgr, child, INVALID_NODE_U64);
            assert_eq!(result, 0);

            let got_parent = goud_ui_get_parent(mgr, child);
            assert_eq!(got_parent, INVALID_NODE_U64);

            goud_ui_manager_destroy(mgr);
        }
    }

    #[test]
    fn test_set_parent_null_mgr() {
        // SAFETY: Null is explicitly handled.
        let result = unsafe { goud_ui_set_parent(std::ptr::null_mut(), 0, 0) };
        assert_eq!(result, -1);
    }

    #[test]
    fn test_get_parent_null_mgr() {
        // SAFETY: Null is explicitly handled.
        let result = unsafe { goud_ui_get_parent(std::ptr::null(), 0) };
        assert_eq!(result, INVALID_NODE_U64);
    }

    #[test]
    fn test_child_count_and_get_child() {
        let mgr = goud_ui_manager_create();
        // SAFETY: mgr was just created.
        unsafe {
            let parent = goud_ui_create_node(mgr, 0);
            let child_a = goud_ui_create_node(mgr, 0);
            let child_b = goud_ui_create_node(mgr, -1);

            goud_ui_set_parent(mgr, child_a, parent);
            goud_ui_set_parent(mgr, child_b, parent);

            assert_eq!(goud_ui_get_child_count(mgr, parent), 2);
            assert_eq!(goud_ui_get_child_at(mgr, parent, 0), child_a);
            assert_eq!(goud_ui_get_child_at(mgr, parent, 1), child_b);
            assert_eq!(goud_ui_get_child_at(mgr, parent, 2), INVALID_NODE_U64);

            goud_ui_manager_destroy(mgr);
        }
    }

    #[test]
    fn test_child_count_null_mgr() {
        // SAFETY: Null is explicitly handled.
        let count = unsafe { goud_ui_get_child_count(std::ptr::null(), 0) };
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_child_at_null_mgr() {
        // SAFETY: Null is explicitly handled.
        let child = unsafe { goud_ui_get_child_at(std::ptr::null(), 0, 0) };
        assert_eq!(child, INVALID_NODE_U64);
    }
}
