//! Contact rebuilding and overlap resolution helpers for `SimplePhysicsProvider`.

use std::collections::HashSet;

use crate::core::providers::types::{BodyHandle, CollisionEvent, CollisionEventKind, ContactPair};

use super::geometry::{layers_interact, ordered_pair, overlap};
use super::SimplePhysicsProvider;

pub(super) fn rebuild_contacts(provider: &mut SimplePhysicsProvider) {
    provider.collision_events.clear();
    provider.contacts.clear();
    let ids: Vec<u64> = provider.colliders.keys().copied().collect();
    let mut current_overlaps = HashSet::new();

    for (index, left_id) in ids.iter().enumerate() {
        for right_id in ids.iter().skip(index + 1) {
            let Some(left) = provider.colliders.get(left_id).cloned() else {
                continue;
            };
            let Some(right) = provider.colliders.get(right_id).cloned() else {
                continue;
            };
            if !layers_interact(&left.desc, &right.desc) {
                continue;
            }
            let Ok(left_aabb) = provider.body_aabb(&left) else {
                continue;
            };
            let Ok(right_aabb) = provider.body_aabb(&right) else {
                continue;
            };
            let Some((normal, depth)) = overlap(left_aabb, right_aabb) else {
                continue;
            };

            let bodies = ordered_pair(left.body.0, right.body.0);
            current_overlaps.insert(bodies);
            let kind = if provider.previous_overlaps.contains(&bodies) {
                CollisionEventKind::Stay
            } else {
                CollisionEventKind::Enter
            };
            provider.collision_events.push(CollisionEvent {
                body_a: BodyHandle(bodies.0),
                body_b: BodyHandle(bodies.1),
                kind,
            });

            if !(left.desc.is_sensor || right.desc.is_sensor) {
                provider.contacts.push(ContactPair {
                    body_a: left.body,
                    body_b: right.body,
                    normal,
                    depth,
                });
                resolve_overlap(provider, left.body, right.body, normal, depth);
            }
        }
    }

    for bodies in provider.previous_overlaps.drain() {
        if !current_overlaps.contains(&bodies) {
            provider.collision_events.push(CollisionEvent {
                body_a: BodyHandle(bodies.0),
                body_b: BodyHandle(bodies.1),
                kind: CollisionEventKind::Exit,
            });
        }
    }

    provider.previous_overlaps = current_overlaps;
}

pub(super) fn resolve_overlap(
    provider: &mut SimplePhysicsProvider,
    left_handle: BodyHandle,
    right_handle: BodyHandle,
    normal: [f32; 2],
    depth: f32,
) {
    let (left_type, right_type) = match (
        provider.bodies.get(&left_handle.0),
        provider.bodies.get(&right_handle.0),
    ) {
        (Some(left), Some(right)) => (left.body_type, right.body_type),
        _ => return,
    };
    if left_type == 0 && right_type == 0 {
        return;
    }

    let left_share = if left_type == 1 && right_type == 1 {
        0.5
    } else if left_type == 1 {
        1.0
    } else {
        0.0
    };
    let right_share = if left_type == 1 && right_type == 1 {
        0.5
    } else if right_type == 1 {
        1.0
    } else {
        0.0
    };

    if let Some(left) = provider.bodies.get_mut(&left_handle.0) {
        left.position[0] -= normal[0] * depth * left_share;
        left.position[1] -= normal[1] * depth * left_share;
        if normal[0] != 0.0 {
            left.velocity[0] = 0.0;
        }
        if normal[1] != 0.0 {
            left.velocity[1] = 0.0;
        }
    }
    if let Some(right) = provider.bodies.get_mut(&right_handle.0) {
        right.position[0] += normal[0] * depth * right_share;
        right.position[1] += normal[1] * depth * right_share;
        if normal[0] != 0.0 {
            right.velocity[0] = 0.0;
        }
        if normal[1] != 0.0 {
            right.velocity[1] = 0.0;
        }
    }
}
