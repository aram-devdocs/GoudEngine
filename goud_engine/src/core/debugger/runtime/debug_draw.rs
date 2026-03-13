use super::state::{with_route_state_mut, with_route_state_mut_by_context};
use super::{route_for_context, RuntimeRouteId};
use crate::core::context_id::GoudContextId;
use crate::core::providers::types::{DebugShape, DebugShape3D};

/// One route-scoped 2D debug draw entry.
#[derive(Debug, Clone)]
pub struct DebugDrawShape2DV1 {
    /// Shape payload emitted by a provider or runtime system.
    pub shape: DebugShape,
    /// Optional frame lifetime. `None` means until replaced/cleared.
    pub lifetime_frames: Option<u32>,
    /// Optional renderer routing layer.
    pub render_layer: Option<i32>,
}

impl PartialEq for DebugDrawShape2DV1 {
    fn eq(&self, other: &Self) -> bool {
        self.shape.shape_type == other.shape.shape_type
            && self.shape.position == other.shape.position
            && self.shape.size == other.shape.size
            && self.shape.rotation == other.shape.rotation
            && self.shape.color == other.shape.color
            && self.lifetime_frames == other.lifetime_frames
            && self.render_layer == other.render_layer
    }
}

/// One route-scoped 3D debug draw entry.
#[derive(Debug, Clone)]
pub struct DebugDrawShape3DV1 {
    /// Shape payload emitted by a provider or runtime system.
    pub shape: DebugShape3D,
    /// Optional frame lifetime. `None` means until replaced/cleared.
    pub lifetime_frames: Option<u32>,
    /// Optional renderer routing layer.
    pub render_layer: Option<i32>,
}

impl PartialEq for DebugDrawShape3DV1 {
    fn eq(&self, other: &Self) -> bool {
        self.shape.shape_type == other.shape.shape_type
            && self.shape.position == other.shape.position
            && self.shape.size == other.shape.size
            && self.shape.rotation == other.shape.rotation
            && self.shape.color == other.shape.color
            && self.lifetime_frames == other.lifetime_frames
            && self.render_layer == other.render_layer
    }
}

/// Route-scoped debug draw payload owned by debugger runtime state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DebugDrawPayloadV1 {
    /// 2D provider-owned payload. Replaced as a full snapshot.
    pub provider_2d: Vec<DebugDrawShape2DV1>,
    /// 3D provider-owned payload. Replaced as a full snapshot.
    pub provider_3d: Vec<DebugDrawShape3DV1>,
    /// Runtime-transient 2D entries for one frame.
    pub transient_2d: Vec<DebugDrawShape2DV1>,
    /// Runtime-transient 3D entries for one frame.
    pub transient_3d: Vec<DebugDrawShape3DV1>,
}

impl DebugDrawPayloadV1 {
    fn collect_2d_shapes(&self) -> Vec<DebugShape> {
        self.provider_2d
            .iter()
            .chain(self.transient_2d.iter())
            .map(|entry| entry.shape.clone())
            .collect()
    }
}

fn wrap_provider_2d(shapes: &[DebugShape]) -> Vec<DebugDrawShape2DV1> {
    shapes
        .iter()
        .cloned()
        .map(|shape| DebugDrawShape2DV1 {
            shape,
            lifetime_frames: None,
            render_layer: None,
        })
        .collect()
}

fn wrap_provider_3d(shapes: &[DebugShape3D]) -> Vec<DebugDrawShape3DV1> {
    shapes
        .iter()
        .cloned()
        .map(|shape| DebugDrawShape3DV1 {
            shape,
            lifetime_frames: None,
            render_layer: None,
        })
        .collect()
}

/// Replaces route-local provider-owned 2D payload with the latest provider snapshot.
pub fn replace_provider_debug_draw_2d_for_route(
    route_id: &RuntimeRouteId,
    shapes: &[DebugShape],
) -> bool {
    with_route_state_mut(route_id, |route| {
        route.debug_draw.provider_2d = wrap_provider_2d(shapes);
        true
    })
    .unwrap_or(false)
}

/// Replaces context-local provider-owned 2D payload with the latest provider snapshot.
pub fn replace_provider_debug_draw_2d_for_context(
    context_id: GoudContextId,
    shapes: &[DebugShape],
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        route.debug_draw.provider_2d = wrap_provider_2d(shapes);
        true
    })
    .unwrap_or(false)
}

/// Replaces route-local provider-owned 3D payload with the latest provider snapshot.
pub fn replace_provider_debug_draw_3d_for_route(
    route_id: &RuntimeRouteId,
    shapes: &[DebugShape3D],
) -> bool {
    with_route_state_mut(route_id, |route| {
        route.debug_draw.provider_3d = wrap_provider_3d(shapes);
        true
    })
    .unwrap_or(false)
}

/// Replaces context-local provider-owned 3D payload with the latest provider snapshot.
pub fn replace_provider_debug_draw_3d_for_context(
    context_id: GoudContextId,
    shapes: &[DebugShape3D],
) -> bool {
    with_route_state_mut_by_context(context_id, |route| {
        route.debug_draw.provider_3d = wrap_provider_3d(shapes);
        true
    })
    .unwrap_or(false)
}

/// Clears route-local transient entries while preserving provider-owned payloads.
pub fn clear_debug_draw_transient_for_route(route_id: &RuntimeRouteId) -> bool {
    with_route_state_mut(route_id, |route| {
        route.debug_draw.transient_2d.clear();
        route.debug_draw.transient_3d.clear();
        true
    })
    .unwrap_or(false)
}

/// Returns the full debug draw payload for a route.
pub fn debug_draw_payload_for_route(route_id: &RuntimeRouteId) -> Option<DebugDrawPayloadV1> {
    with_route_state_mut(route_id, |route| route.debug_draw.clone())
}

/// Returns the current 2D shape stream for a route (provider + transient).
pub fn debug_draw_shapes_2d_for_route(route_id: &RuntimeRouteId) -> Option<Vec<DebugShape>> {
    with_route_state_mut(route_id, |route| route.debug_draw.collect_2d_shapes())
}

/// Returns the current 2D shape stream for a context route (provider + transient).
pub fn debug_draw_shapes_2d_for_context(context_id: GoudContextId) -> Option<Vec<DebugShape>> {
    let route_id = route_for_context(context_id)?;
    debug_draw_shapes_2d_for_route(&route_id)
}
