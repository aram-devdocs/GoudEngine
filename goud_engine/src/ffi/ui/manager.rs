//! UiManager lifecycle FFI functions.
//!
//! Provides C-compatible functions for creating, destroying, updating, and
//! querying a [`UiManager`]. The caller owns the returned pointer and MUST
//! call [`goud_ui_manager_destroy`] to free it.

use crate::ui::UiManager;

// ============================================================================
// Lifecycle
// ============================================================================

/// Creates a new [`UiManager`] and returns an owning pointer.
///
/// # Returns
///
/// A non-null pointer to a heap-allocated `UiManager`.
///
/// # Ownership
///
/// Ownership is transferred to the caller. The caller MUST eventually call
/// [`goud_ui_manager_destroy`] to free the memory.
///
/// # Note
///
/// `GoudGame` already contains an embedded `UiManager` that is updated
/// each frame. This function creates an *additional*, independent manager. Most
/// SDK users should access the game's built-in UI manager instead.
#[no_mangle]
pub extern "C" fn goud_ui_manager_create() -> *mut UiManager {
    let mgr = Box::new(UiManager::new());
    Box::into_raw(mgr)
}

/// Destroys a [`UiManager`] previously created by [`goud_ui_manager_create`].
///
/// # Arguments
///
/// * `ptr` - Owning pointer to the manager. If null, this is a no-op.
///
/// # Ownership
///
/// The caller transfers ownership back. The pointer MUST NOT be used after
/// this call.
///
/// # Safety
///
/// `ptr` must be either null or a pointer previously returned by
/// [`goud_ui_manager_create`] that has not yet been destroyed.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_manager_destroy(ptr: *mut UiManager) {
    if ptr.is_null() {
        return;
    }
    super::events::unregister_manager(ptr as usize);
    // SAFETY: Caller guarantees `ptr` was allocated by `goud_ui_manager_create`
    // via `Box::into_raw` and has not been freed yet. We reclaim ownership and
    // drop the value.
    let _ = Box::from_raw(ptr);
}

// ============================================================================
// Operations
// ============================================================================

/// Runs the update tick on the UI manager (layout computation placeholder).
///
/// # Arguments
///
/// * `ptr` - Mutable pointer to the manager. If null, this is a no-op.
///
/// # Safety
///
/// `ptr` must be either null or a valid pointer to a `UiManager` that the
/// caller has exclusive (mutable) access to.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_manager_update(ptr: *mut UiManager) {
    if ptr.is_null() {
        return;
    }
    let manager_key = ptr as usize;
    // SAFETY: Caller guarantees `ptr` is a valid, exclusively-owned UiManager
    // pointer. We borrow it mutably for the duration of this call.
    let mgr = &mut *ptr;
    mgr.update();
    super::events::capture_and_dispatch_events(manager_key, mgr);
}

/// Calls render on the UI manager.
///
/// # Arguments
///
/// * `ptr` - Const pointer to the manager. If null, this is a no-op.
///
/// # Safety
///
/// `ptr` must be either null (no-op) or a valid, exclusively-owned pointer to a `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_manager_render(ptr: *const UiManager) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: Caller guarantees `ptr` is valid and not aliased.
    let mgr = &*ptr;
    // Legacy compatibility path for existing FFI callers; keep deprecated Rust API as the shim.
    #[allow(deprecated)]
    {
        mgr.render();
    }
}

/// Returns the number of live nodes in the UI manager.
///
/// # Arguments
///
/// * `ptr` - Const pointer to the manager. If null, returns 0.
///
/// # Safety
///
/// `ptr` must be either null or a valid pointer to a `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_manager_node_count(ptr: *const UiManager) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    // SAFETY: Caller guarantees `ptr` is a valid UiManager pointer. We borrow
    // it immutably for the duration of this call.
    let mgr = &*ptr;
    mgr.node_count() as u32
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_destroy() {
        let ptr = goud_ui_manager_create();
        assert!(!ptr.is_null());
        // SAFETY: ptr was just created by goud_ui_manager_create.
        unsafe { goud_ui_manager_destroy(ptr) };
    }

    #[test]
    fn test_destroy_null_is_noop() {
        // SAFETY: Passing null is explicitly documented as a no-op.
        unsafe { goud_ui_manager_destroy(std::ptr::null_mut()) };
    }

    #[test]
    fn test_node_count_null() {
        // SAFETY: Passing null is explicitly documented to return 0.
        let count = unsafe { goud_ui_manager_node_count(std::ptr::null()) };
        assert_eq!(count, 0);
    }

    #[test]
    fn test_node_count_empty() {
        let ptr = goud_ui_manager_create();
        // SAFETY: ptr was just created by goud_ui_manager_create.
        let count = unsafe { goud_ui_manager_node_count(ptr) };
        assert_eq!(count, 0);
        unsafe { goud_ui_manager_destroy(ptr) };
    }

    #[test]
    fn test_update_null_is_noop() {
        // SAFETY: Passing null is explicitly documented as a no-op.
        unsafe { goud_ui_manager_update(std::ptr::null_mut()) };
    }

    #[test]
    fn test_update_does_not_panic() {
        let ptr = goud_ui_manager_create();
        // SAFETY: ptr was just created by goud_ui_manager_create.
        unsafe { goud_ui_manager_update(ptr) };
        unsafe { goud_ui_manager_destroy(ptr) };
    }
}
