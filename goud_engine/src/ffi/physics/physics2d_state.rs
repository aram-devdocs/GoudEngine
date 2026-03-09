//! Per-context FFI state for 2D physics extensions.
//!
//! Tracks collider filter metadata (layer/mask/sensor), buffered collision
//! events for deterministic polling, and an optional C callback registration.

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use crate::core::providers::types::CollisionEvent;
use crate::ffi::context::GoudContextId;

/// C ABI callback invoked for each filtered collision event emitted by
/// `goud_physics_step`.
///
/// Kind mapping: 0 = Enter, 1 = Stay, 2 = Exit.
///
/// Ownership: `user_data` is borrowed, never freed by the engine.
pub type CollisionCallback =
    extern "C" fn(ctx: GoudContextId, body_a: u64, body_b: u64, kind: u32, user_data: *mut c_void);

#[derive(Clone, Copy)]
struct ColliderFilterMeta {
    body_handle: u64,
    layer: u32,
    mask: u32,
    is_sensor: bool,
}

#[derive(Default)]
struct ContextPhysics2DState {
    collider_filters: HashMap<u64, ColliderFilterMeta>,
    body_colliders: HashMap<u64, Vec<u64>>,
    collision_events: Vec<CollisionEvent>,
    callback: Option<CollisionCallback>,
    callback_user_data: usize,
}

#[derive(Default)]
struct Physics2DStateRegistry {
    contexts: HashMap<GoudContextId, ContextPhysics2DState>,
}

fn registry() -> &'static Mutex<Physics2DStateRegistry> {
    static REGISTRY: OnceLock<Mutex<Physics2DStateRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Physics2DStateRegistry::default()))
}

fn can_collide(meta_a: &ColliderFilterMeta, meta_b: &ColliderFilterMeta) -> bool {
    (meta_a.layer & meta_b.mask) != 0 && (meta_b.layer & meta_a.mask) != 0
}

fn event_passes_layer_filter(state: &ContextPhysics2DState, event: &CollisionEvent) -> bool {
    let Some(colliders_a) = state.body_colliders.get(&event.body_a.0) else {
        // No metadata means legacy/no-filter path; preserve old behavior.
        return true;
    };
    let Some(colliders_b) = state.body_colliders.get(&event.body_b.0) else {
        return true;
    };

    colliders_a.iter().any(|collider_a| {
        state
            .collider_filters
            .get(collider_a)
            .is_some_and(|meta_a| {
                colliders_b.iter().any(|collider_b| {
                    state
                        .collider_filters
                        .get(collider_b)
                        .is_some_and(|meta_b| can_collide(meta_a, meta_b))
                })
            })
    })
}

pub(super) fn clear_context(ctx: GoudContextId) {
    let Ok(mut guard) = registry().lock() else {
        return;
    };
    guard.contexts.remove(&ctx);
}

pub(super) fn remove_body(ctx: GoudContextId, body_handle: u64) {
    let Ok(mut guard) = registry().lock() else {
        return;
    };

    let Some(state) = guard.contexts.get_mut(&ctx) else {
        return;
    };

    if let Some(collider_handles) = state.body_colliders.remove(&body_handle) {
        for collider_handle in collider_handles {
            state.collider_filters.remove(&collider_handle);
        }
    }
}

pub(super) fn register_collider(
    ctx: GoudContextId,
    body_handle: u64,
    collider_handle: u64,
    layer: u32,
    mask: u32,
    is_sensor: bool,
) {
    let Ok(mut guard) = registry().lock() else {
        return;
    };

    let state = guard.contexts.entry(ctx).or_default();

    // Handle collider-handle reuse by removing any stale body mapping first.
    if let Some(previous) = state.collider_filters.get(&collider_handle).copied() {
        if let Some(body_list) = state.body_colliders.get_mut(&previous.body_handle) {
            body_list.retain(|h| *h != collider_handle);
            if body_list.is_empty() {
                state.body_colliders.remove(&previous.body_handle);
            }
        }
    }

    state.collider_filters.insert(
        collider_handle,
        ColliderFilterMeta {
            body_handle,
            layer,
            mask,
            is_sensor,
        },
    );

    let colliders = state.body_colliders.entry(body_handle).or_default();
    if !colliders.contains(&collider_handle) {
        colliders.push(collider_handle);
    }
}

pub(super) fn collider_matches_layer_mask(
    ctx: GoudContextId,
    collider_handle: u64,
    layer_mask: u32,
) -> bool {
    let Ok(guard) = registry().lock() else {
        // If state cannot be read, fail open to preserve existing behavior.
        return true;
    };

    let Some(state) = guard.contexts.get(&ctx) else {
        return true;
    };

    let Some(meta) = state.collider_filters.get(&collider_handle) else {
        return true;
    };

    let _ = meta.is_sensor;
    (meta.layer & layer_mask) != 0
}

pub(super) fn set_collision_callback(
    ctx: GoudContextId,
    callback: Option<CollisionCallback>,
    user_data: *mut c_void,
) {
    let Ok(mut guard) = registry().lock() else {
        return;
    };

    let state = guard.contexts.entry(ctx).or_default();
    state.callback = callback;
    state.callback_user_data = user_data as usize;
}

pub(super) fn capture_step_collision_events(
    ctx: GoudContextId,
    raw_events: Vec<CollisionEvent>,
) -> (Vec<CollisionEvent>, Option<(CollisionCallback, usize)>) {
    let Ok(mut guard) = registry().lock() else {
        return (Vec::new(), None);
    };

    let state = guard.contexts.entry(ctx).or_default();
    let filtered: Vec<CollisionEvent> = raw_events
        .into_iter()
        .filter(|event| event_passes_layer_filter(state, event))
        .collect();

    state.collision_events = filtered.clone();

    let callback = state.callback.map(|f| (f, state.callback_user_data));
    (filtered, callback)
}

pub(super) fn collision_event_count(ctx: GoudContextId) -> usize {
    let Ok(guard) = registry().lock() else {
        return 0;
    };

    guard
        .contexts
        .get(&ctx)
        .map(|state| state.collision_events.len())
        .unwrap_or(0)
}

pub(super) fn collision_event_at(ctx: GoudContextId, index: usize) -> Option<CollisionEvent> {
    let Ok(guard) = registry().lock() else {
        return None;
    };

    let state = guard.contexts.get(&ctx)?;
    state.collision_events.get(index).cloned()
}

#[cfg(test)]
mod tests {
    use super::{clear_context, collider_matches_layer_mask, register_collider};
    use crate::ffi::context::{goud_context_create, goud_context_destroy, GOUD_INVALID_CONTEXT_ID};

    #[test]
    fn test_collider_layer_mask_matching_is_per_collider_not_body() {
        let ctx = goud_context_create();
        assert_ne!(ctx, GOUD_INVALID_CONTEXT_ID);

        let body = 1_u64;
        let non_matching_collider = 101_u64;
        let matching_collider = 102_u64;

        register_collider(ctx, body, non_matching_collider, 0b0001, u32::MAX, false);
        register_collider(ctx, body, matching_collider, 0b0010, u32::MAX, false);

        assert!(
            !collider_matches_layer_mask(ctx, non_matching_collider, 0b0010),
            "non-matching collider should fail even if another collider on the same body matches"
        );
        assert!(
            collider_matches_layer_mask(ctx, matching_collider, 0b0010),
            "matching collider should pass"
        );

        clear_context(ctx);
        assert!(goud_context_destroy(ctx));
    }
}
