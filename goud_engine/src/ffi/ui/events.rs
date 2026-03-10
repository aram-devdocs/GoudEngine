//! UI event callback and polling FFI exports.

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use crate::ui::UiManager;

use super::{FfiUiEvent, UiEventCallback, map_ui_event, ERR_NULL_MANAGER};

const ERR_NULL_OUT: i32 = -2;

#[derive(Clone, Copy)]
struct CallbackRegistration {
    callback: Option<UiEventCallback>,
    user_data: usize,
}

fn callback_registry() -> &'static Mutex<HashMap<usize, CallbackRegistration>> {
    static REGISTRY: OnceLock<Mutex<HashMap<usize, CallbackRegistration>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn event_snapshots() -> &'static Mutex<HashMap<usize, Vec<FfiUiEvent>>> {
    static SNAPSHOTS: OnceLock<Mutex<HashMap<usize, Vec<FfiUiEvent>>>> = OnceLock::new();
    SNAPSHOTS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn capture_and_dispatch_events(manager_key: usize, manager: &mut UiManager) {
    let ffi_events: Vec<FfiUiEvent> = manager
        .take_events()
        .into_iter()
        .map(|event| {
            let mapped = map_ui_event(event);
            FfiUiEvent {
                event_kind: mapped.event_kind,
                node_id: mapped.node_id,
                previous_node_id: mapped.previous_node_id,
                current_node_id: mapped.current_node_id,
            }
        })
        .collect();

    if let Ok(mut snapshots) = event_snapshots().lock() {
        snapshots.insert(manager_key, ffi_events.clone());
    }

    let registration = match callback_registry().lock() {
        Ok(registry) => registry.get(&manager_key).copied(),
        Err(_) => None,
    };
    if let Some(CallbackRegistration {
        callback: Some(callback),
        user_data,
    }) = registration
    {
        let user_data_ptr = user_data as *mut c_void;
        for event in ffi_events {
            callback(
                event.node_id,
                event.event_kind,
                event.previous_node_id,
                event.current_node_id,
                user_data_ptr,
            );
        }
    }
}

pub(super) fn unregister_manager(manager_key: usize) {
    if let Ok(mut registry) = callback_registry().lock() {
        registry.remove(&manager_key);
    }
    if let Ok(mut snapshots) = event_snapshots().lock() {
        snapshots.remove(&manager_key);
    }
}

/// Registers or clears a UI event callback for a standalone `UiManager`.
///
/// Pass `None` as callback to clear it.
///
/// # Safety
///
/// This function is `extern "C"` and accepts raw pointers:
/// * `mgr` must be a valid pointer to a live `UiManager` for the duration of
///   the call.
/// * If `callback` is `Some`, `user_data` must be a valid, non-null pointer whose
///   pointee remains valid and properly aligned for reads and writes by that
///   callback until the callback is cleared by passing `None` (or the manager is
///   dropped). If callback is `None`, `user_data` is ignored.
/// 
/// # Returns
/// * `0` on success
/// * `-1` if manager is null
#[no_mangle]
pub extern "C" fn goud_ui_set_event_callback(
    mgr: *mut UiManager,
    callback: Option<UiEventCallback>,
    user_data: *mut c_void,
) -> i32 {
    if mgr.is_null() {
        return ERR_NULL_MANAGER;
    }
    let key = mgr as usize;
    if let Ok(mut registry) = callback_registry().lock() {
        registry.insert(
            key,
            CallbackRegistration {
                callback,
                user_data: user_data as usize,
            },
        );
    }
    0
}

/// Returns the number of events captured in the latest FFI UI update tick.
#[no_mangle]
pub extern "C" fn goud_ui_event_count(mgr: *const UiManager) -> u32 {
    if mgr.is_null() {
        return 0;
    }
    let key = mgr as usize;
    match event_snapshots().lock() {
        Ok(snapshots) => snapshots.get(&key).map_or(0, |events| events.len() as u32),
        Err(_) => 0,
    }
}

/// Backward-compatible alias using pluralized naming.
#[no_mangle]
pub extern "C" fn goud_ui_events_count(mgr: *const UiManager) -> u32 {
    goud_ui_event_count(mgr)
}

/// Reads one captured UI event by index.
///
/// # Returns
/// * `1` if event exists and was written
/// * `0` if index is out of bounds
/// * `-1` if manager is null
/// * `-2` if output pointer is null
///
/// # Safety
/// `out_event` must be non-null and writable.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_event_read(
    mgr: *const UiManager,
    index: u32,
    out_event: *mut FfiUiEvent,
) -> i32 {
    if mgr.is_null() {
        return ERR_NULL_MANAGER;
    }
    if out_event.is_null() {
        return ERR_NULL_OUT;
    }

    let key = mgr as usize;
    let event = match event_snapshots().lock() {
        Ok(snapshots) => snapshots
            .get(&key)
            .and_then(|events| events.get(index as usize))
            .copied(),
        Err(_) => None,
    };
    let Some(event) = event else {
        return 0;
    };

    // SAFETY: `out_event` is checked non-null and caller owns writable memory.
    unsafe { *out_event = event };
    1
}

/// Backward-compatible alias using pluralized naming.
///
/// # Safety
/// Same requirements as [`goud_ui_event_read`].
#[no_mangle]
pub unsafe extern "C" fn goud_ui_events_read(
    mgr: *const UiManager,
    index: u32,
    out_event: *mut FfiUiEvent,
) -> i32 {
    // SAFETY: This is a thin alias that forwards the exact same arguments and
    // safety contract to `goud_ui_event_read`.
    unsafe { goud_ui_event_read(mgr, index, out_event) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Vec2;
    use crate::ecs::InputManager;
    use crate::ffi::ui::manager::{
        goud_ui_manager_create, goud_ui_manager_destroy, goud_ui_manager_update,
    };
    use crate::ffi::ui::node::goud_ui_create_node;
    use crate::ui::{UiAnchor, UiButton, UiComponent};
    use glfw::MouseButton;

    extern "C" fn collect_event(
        node_id: u64,
        event_kind: u32,
        previous_node_id: u64,
        current_node_id: u64,
        user_data: *mut c_void,
    ) {
        // SAFETY: Tests pass a valid Vec<FfiUiEvent> pointer as `user_data`.
        let events = unsafe { &mut *(user_data as *mut Vec<FfiUiEvent>) };
        events.push(FfiUiEvent {
            event_kind,
            node_id,
            previous_node_id,
            current_node_id,
        });
    }

    #[test]
    fn test_ui_event_callback_and_read_snapshot() {
        let mgr = goud_ui_manager_create();
        let mut callback_events = Vec::<FfiUiEvent>::new();
        let user_data = (&mut callback_events as *mut Vec<FfiUiEvent>).cast::<c_void>();

        // SAFETY: `mgr` is a valid manager pointer created above.
        unsafe {
            assert_eq!(
                goud_ui_set_event_callback(mgr, Some(collect_event), user_data),
                0
            );
            let button = goud_ui_create_node(mgr, 1);
            let id = super::super::unpack_node_id(button);
            let manager = &mut *mgr;
            let node = manager.get_node_mut(id).unwrap();
            node.set_anchor(UiAnchor::TopLeft);
            node.set_size(Vec2::new(100.0, 40.0));
            node.set_component(Some(UiComponent::Button(UiButton::default())));

            let mut input = InputManager::new();
            input.set_mouse_position(Vec2::new(10.0, 10.0));
            input.press_mouse_button(MouseButton::Button1);
            manager.process_input_frame(&mut input);

            goud_ui_manager_update(mgr);

            assert!(goud_ui_event_count(mgr) > 0);
            assert!(!callback_events.is_empty());

            let mut event = FfiUiEvent::default();
            assert_eq!(goud_ui_event_read(mgr, 0, &mut event), 1);
            assert_eq!(event, callback_events[0]);

            assert_eq!(
                goud_ui_set_event_callback(mgr, None, std::ptr::null_mut()),
                0
            );
            goud_ui_manager_destroy(mgr);
        }
    }
}
