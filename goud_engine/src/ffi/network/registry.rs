use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use crate::core::error::{set_last_error, GoudError, ERR_INTERNAL_ERROR, ERR_INVALID_STATE};
use crate::core::providers::network::NetworkProvider;
use crate::ffi::context::GoudContextId;

/// Error sentinel returned for handle-producing functions.
pub(super) const ERR_HANDLE: i64 = -1;

/// Internal state for a network instance.
pub(super) struct NetInstance {
    pub(super) provider: Box<dyn NetworkProvider>,
    /// Buffered received-data events from the last `poll`.
    pub(super) recv_queue: VecDeque<(u64, Vec<u8>)>, // (peer_id / conn_id, data)
}

// SAFETY: NetInstance is only accessed through the global Mutex, so all
// access is serialized. The trait object inside is not Sync on its own
// (WsNetProvider uses mpsc channels), but the Mutex serializes all access.
unsafe impl Send for NetInstance {}

pub(super) static NET_REGISTRY: Mutex<Option<NetRegistryInner>> = Mutex::new(None);

pub(super) struct NetRegistryInner {
    pub(super) instances: HashMap<i64, NetInstance>,
    pub(super) default_handles_by_context: HashMap<(u32, u32), i64>,
    pub(super) overlay_override_handles_by_context: HashMap<(u32, u32), i64>,
    next_handle: i64,
}

impl NetRegistryInner {
    pub(super) fn new() -> Self {
        Self {
            instances: HashMap::new(),
            default_handles_by_context: HashMap::new(),
            overlay_override_handles_by_context: HashMap::new(),
            next_handle: 1,
        }
    }

    pub(super) fn insert(&mut self, instance: NetInstance) -> i64 {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.instances.insert(handle, instance);
        handle
    }

    pub(super) fn context_key(context_id: GoudContextId) -> (u32, u32) {
        (context_id.index(), context_id.generation())
    }

    pub(super) fn set_default_handle_for_context(
        &mut self,
        context_id: GoudContextId,
        handle: i64,
    ) {
        self.default_handles_by_context
            .insert(Self::context_key(context_id), handle);
    }

    pub(super) fn set_overlay_override_handle_for_context(
        &mut self,
        context_id: GoudContextId,
        handle: Option<i64>,
    ) {
        let key = Self::context_key(context_id);
        if let Some(handle) = handle {
            self.overlay_override_handles_by_context.insert(key, handle);
        } else {
            self.overlay_override_handles_by_context.remove(&key);
        }
    }

    pub(super) fn active_handle_for_context(&self, context_id: GoudContextId) -> Option<i64> {
        let key = Self::context_key(context_id);
        if let Some(handle) = self.overlay_override_handles_by_context.get(&key) {
            if self.instances.contains_key(handle) {
                return Some(*handle);
            }
        }

        self.default_handles_by_context
            .get(&key)
            .copied()
            .filter(|handle| self.instances.contains_key(handle))
    }

    pub(super) fn clear_associations_for_handle(&mut self, handle: i64) {
        self.default_handles_by_context
            .retain(|_, mapped_handle| *mapped_handle != handle);
        self.overlay_override_handles_by_context
            .retain(|_, mapped_handle| *mapped_handle != handle);
    }
}

pub(super) fn with_registry<F, R>(f: F) -> Result<R, i32>
where
    F: FnOnce(&mut NetRegistryInner) -> Result<R, i32>,
{
    let mut guard = NET_REGISTRY.lock().map_err(|_| {
        set_last_error(GoudError::InternalError(
            "Failed to lock network registry".to_string(),
        ));
        ERR_INTERNAL_ERROR
    })?;
    let reg = guard.get_or_insert_with(NetRegistryInner::new);
    f(reg)
}

pub(super) fn with_instance<F, R>(handle: i64, f: F) -> Result<R, i32>
where
    F: FnOnce(&mut NetInstance) -> Result<R, i32>,
{
    with_registry(|reg| {
        let inst = reg.instances.get_mut(&handle).ok_or_else(|| {
            set_last_error(GoudError::InvalidState(format!(
                "Unknown network handle {}",
                handle
            )));
            ERR_INVALID_STATE
        })?;
        f(inst)
    })
}

#[cfg(test)]
pub(super) fn reset_registry_for_tests() {
    let mut guard = NET_REGISTRY
        .lock()
        .expect("network registry mutex poisoned");
    *guard = None;
}
