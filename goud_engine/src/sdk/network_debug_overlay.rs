//! Engine-side network overlay state for later FFI and rendering integration.

use std::collections::HashMap;

use crate::context_registry::GoudContextId;
use crate::core::providers::network_types::ConnectionId;

/// Supported overlay toggle keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkOverlayToggleKey {
    /// Planned runtime toggle for the network stats overlay.
    F6,
}

#[derive(Debug, Clone, Copy, Default)]
struct ContextOverlayState {
    visible: bool,
    active_handle: Option<ConnectionId>,
}

/// Per-context network overlay state.
///
/// Integration should associate host/connect-created handles with the current
/// `GoudContextId` and treat that handle as the default overlay target until an
/// explicit override is applied.
#[derive(Debug, Clone, Default)]
pub struct NetworkOverlayState {
    contexts: HashMap<GoudContextId, ContextOverlayState>,
}

impl NetworkOverlayState {
    /// Returns whether the network overlay is visible for the context.
    pub fn is_visible(&self, context_id: GoudContextId) -> bool {
        self.contexts
            .get(&context_id)
            .map(|state| state.visible)
            .unwrap_or(false)
    }

    /// Applies a toggle key for the context and returns `true` when handled.
    pub fn handle_toggle_key(
        &mut self,
        context_id: GoudContextId,
        key: NetworkOverlayToggleKey,
    ) -> bool {
        match key {
            NetworkOverlayToggleKey::F6 => {
                let visible = !self.is_visible(context_id);
                self.contexts.entry(context_id).or_default().visible = visible;
                true
            }
        }
    }

    /// Returns the active overlay handle for the context.
    pub fn active_handle(&self, context_id: GoudContextId) -> Option<ConnectionId> {
        self.contexts
            .get(&context_id)
            .and_then(|state| state.active_handle)
    }

    /// Sets the active overlay handle for the context.
    pub fn set_active_handle(
        &mut self,
        context_id: GoudContextId,
        handle: Option<ConnectionId>,
    ) {
        self.contexts.entry(context_id).or_default().active_handle = handle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_overlay_f6_toggle_switches_visibility_per_context() {
        let mut state = NetworkOverlayState::default();
        let ctx_a = GoudContextId::new(10, 1);
        let ctx_b = GoudContextId::new(11, 1);

        assert!(!state.is_visible(ctx_a));
        assert!(!state.is_visible(ctx_b));

        assert!(state.handle_toggle_key(ctx_a, NetworkOverlayToggleKey::F6));
        assert!(state.is_visible(ctx_a));
        assert!(!state.is_visible(ctx_b));

        assert!(state.handle_toggle_key(ctx_a, NetworkOverlayToggleKey::F6));
        assert!(!state.is_visible(ctx_a));
    }

    #[test]
    fn test_network_overlay_tracks_active_network_handle_per_context() {
        let mut state = NetworkOverlayState::default();
        let ctx_a = GoudContextId::new(20, 1);
        let ctx_b = GoudContextId::new(21, 1);
        let conn_a = ConnectionId(101);
        let conn_b = ConnectionId(202);

        assert_eq!(state.active_handle(ctx_a), None);
        assert_eq!(state.active_handle(ctx_b), None);

        state.set_active_handle(ctx_a, Some(conn_a));
        state.set_active_handle(ctx_b, Some(conn_b));

        assert_eq!(state.active_handle(ctx_a), Some(conn_a));
        assert_eq!(state.active_handle(ctx_b), Some(conn_b));

        state.set_active_handle(ctx_a, None);
        assert_eq!(state.active_handle(ctx_a), None);
        assert_eq!(state.active_handle(ctx_b), Some(conn_b));
    }
}
