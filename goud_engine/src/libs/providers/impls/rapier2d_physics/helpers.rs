use std::collections::HashSet;

use rapier2d::prelude::{ColliderHandle as RapierColliderHandle, RigidBodyHandle};

use super::Rapier2DPhysicsProvider;
use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::types::ColliderHandle as EngineColliderHandle;
use crate::core::providers::types::{
    BodyHandle, CollisionEvent as EngineCollisionEvent, CollisionEventKind,
};

impl Rapier2DPhysicsProvider {
    /// Allocate the next engine handle ID.
    pub(super) fn next_handle_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Drain rapier collision events from the channel and update pair state.
    ///
    /// Returns the set of pairs that entered this drain cycle.
    pub(super) fn drain_rapier_collision_events(&mut self) -> HashSet<(u64, u64)> {
        let mut entered_this_drain = HashSet::new();

        while let Ok(event) = self.collision_recv.try_recv() {
            let (h1, h2, started) = match event {
                rapier2d::prelude::CollisionEvent::Started(h1, h2, _) => (h1, h2, true),
                rapier2d::prelude::CollisionEvent::Stopped(h1, h2, _) => (h1, h2, false),
            };

            let body_a = self.collider_body_handle(h1);
            let body_b = self.collider_body_handle(h2);
            let (Some(a), Some(b)) = (body_a, body_b) else {
                continue;
            };

            if a == b {
                continue;
            }

            let pair = Self::ordered_pair(a, b);
            let collider_pair = Self::ordered_collider_pair(a, h1, b, h2);

            if started {
                let collider_pairs = self
                    .active_collision_collider_pairs
                    .entry(pair)
                    .or_default();
                if collider_pairs.insert(collider_pair) && self.active_collision_pairs.insert(pair)
                {
                    self.collision_events.push(EngineCollisionEvent {
                        body_a: BodyHandle(pair.0),
                        body_b: BodyHandle(pair.1),
                        kind: CollisionEventKind::Enter,
                    });
                    entered_this_drain.insert(pair);
                }
            } else {
                let pair_became_inactive = if let Some(collider_pairs) =
                    self.active_collision_collider_pairs.get_mut(&pair)
                {
                    let removed = collider_pairs.remove(&collider_pair);
                    removed && collider_pairs.is_empty()
                } else {
                    false
                };

                if pair_became_inactive {
                    self.active_collision_collider_pairs.remove(&pair);
                    if self.active_collision_pairs.remove(&pair) {
                        self.collision_events.push(EngineCollisionEvent {
                            body_a: BodyHandle(pair.0),
                            body_b: BodyHandle(pair.1),
                            kind: CollisionEventKind::Exit,
                        });
                    }
                }
            }
        }

        entered_this_drain
    }

    pub(super) fn ordered_pair(a: u64, b: u64) -> (u64, u64) {
        if a <= b {
            (a, b)
        } else {
            (b, a)
        }
    }

    pub(super) fn ordered_collider_pair(
        body_a: u64,
        collider_a: RapierColliderHandle,
        body_b: u64,
        collider_b: RapierColliderHandle,
    ) -> (RapierColliderHandle, RapierColliderHandle) {
        if body_a <= body_b {
            (collider_a, collider_b)
        } else {
            (collider_b, collider_a)
        }
    }

    /// Look up the engine body handle for a rapier collider handle.
    pub(super) fn collider_body_handle(&self, collider: RapierColliderHandle) -> Option<u64> {
        let parent = self.collider_set.get(collider)?.parent()?;
        self.body_handles_rev.get(&parent).copied()
    }

    /// Look up a rapier body handle from an engine handle, returning an error if not found.
    pub(super) fn get_rapier_body(&self, handle: BodyHandle) -> GoudResult<RigidBodyHandle> {
        self.body_handles
            .get(&handle.0)
            .copied()
            .ok_or(GoudError::InvalidHandle)
    }

    /// Look up a rapier collider handle from an engine handle, returning an error if not found.
    pub(super) fn get_rapier_collider(
        &self,
        handle: EngineColliderHandle,
    ) -> GoudResult<RapierColliderHandle> {
        self.collider_handles
            .get(&handle.0)
            .copied()
            .ok_or(GoudError::InvalidHandle)
    }
}
