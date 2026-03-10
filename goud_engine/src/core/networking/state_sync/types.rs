//! Shared state-sync types.

use std::collections::{HashMap, VecDeque};
use std::f32::consts::{PI, TAU};

use crate::core::math::{Vec2, Vec3};
use crate::core::serialization::DeltaPayload;
use crate::ecs::components::{Transform, Transform2D};
use crate::ecs::entity::Entity;
use crate::ecs::Component;

/// Marker component for entities included in networking state sync snapshots.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NetworkSync;

impl Component for NetworkSync {}

/// State-sync configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateSyncConfig {
    /// Snapshot send rate in hertz.
    pub snapshot_rate_hz: u32,
    /// Interpolation delay in seconds.
    pub interpolation_delay: f32,
    /// Maximum number of resolved snapshots retained for interpolation.
    pub max_buffered_snapshots: usize,
}

impl Default for StateSyncConfig {
    fn default() -> Self {
        Self {
            snapshot_rate_hz: 20,
            interpolation_delay: 0.1,
            max_buffered_snapshots: 4,
        }
    }
}

/// Serialized transform payload for 2D snapshots.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Transform2DSnapshot {
    /// Full transform state.
    Full(Transform2D),
    /// Delta from the last known transform state.
    Delta(DeltaPayload<u8>),
}

/// Serialized transform payload for 3D snapshots.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TransformSnapshot {
    /// Full transform state.
    Full(Transform),
    /// Delta from the last known transform state.
    Delta(DeltaPayload<u16>),
}

/// Wire payload for one synchronized entity.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateSyncEntitySnapshot {
    /// The synchronized entity.
    pub entity: Entity,
    /// Optional 2D transform payload.
    pub transform2d: Option<Transform2DSnapshot>,
    /// Optional 3D transform payload.
    pub transform: Option<TransformSnapshot>,
    /// Additional serializable components keyed by type name and encoded as JSON bytes.
    pub components: HashMap<String, Vec<u8>>,
}

/// Wire payload carried inside [`crate::core::serialization::NetworkMessage`].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateSnapshotPayload {
    /// Synchronized entity payloads.
    pub entities: Vec<StateSyncEntitySnapshot>,
}

/// Resolved entity state after applying deltas.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSyncEntity {
    /// The synchronized entity.
    pub entity: Entity,
    /// Full 2D transform state, if present.
    pub transform2d: Option<Transform2D>,
    /// Full 3D transform state, if present.
    pub transform: Option<Transform>,
    /// Additional serializable component values.
    pub components: HashMap<String, serde_json::Value>,
}

/// Resolved snapshot retained for interpolation.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedStateSnapshot {
    /// Message sequence from the transport envelope.
    pub sequence: u32,
    /// Resolved entity states for this sequence.
    pub entities: Vec<ResolvedSyncEntity>,
}

impl ResolvedStateSnapshot {
    /// Returns the resolved entity state for one synchronized entity.
    pub fn entity(&self, entity: Entity) -> Option<&ResolvedSyncEntity> {
        self.entities
            .iter()
            .find(|candidate| candidate.entity == entity)
    }
}

/// Aggregate bandwidth stats for one synchronized entity.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityBandwidthStat {
    /// Total bytes sent for this entity.
    pub bytes_sent: u64,
    /// Number of full snapshots containing this entity.
    pub full_snapshots: u64,
    /// Number of delta snapshots containing this entity.
    pub delta_snapshots: u64,
}

/// Aggregate bandwidth stats keyed by synchronized entity.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StateSyncBandwidthStats {
    /// Per-entity byte counters.
    pub per_entity: HashMap<Entity, EntityBandwidthStat>,
}

impl StateSyncBandwidthStats {
    /// Returns bandwidth stats for one entity, if tracked.
    pub fn entity(&self, entity: Entity) -> Option<&EntityBandwidthStat> {
        self.per_entity.get(&entity)
    }
}

/// Resolved snapshot buffer used for interpolation.
#[derive(Debug, Clone)]
pub struct StateSyncInterpolationBuffer {
    snapshots: VecDeque<ResolvedStateSnapshot>,
    max_snapshots: usize,
}

impl StateSyncInterpolationBuffer {
    /// Creates a new interpolation buffer.
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: VecDeque::new(),
            max_snapshots: max_snapshots.max(1),
        }
    }

    /// Pushes a resolved snapshot, dropping the oldest when full.
    pub fn push(&mut self, snapshot: ResolvedStateSnapshot) {
        self.snapshots.push_back(snapshot);
        while self.snapshots.len() > self.max_snapshots {
            self.snapshots.pop_front();
        }
    }

    /// Returns the number of buffered snapshots.
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Returns whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Returns the latest buffered sequence, if any.
    pub fn latest_sequence(&self) -> Option<u32> {
        self.snapshots.back().map(|snapshot| snapshot.sequence)
    }

    /// Returns the newest resolved snapshot, if any.
    pub fn latest_snapshot(&self) -> Option<&ResolvedStateSnapshot> {
        self.snapshots.back()
    }

    /// Interpolates a 2D transform from the newest two buffered snapshots.
    pub fn interpolate_transform2d(&self, entity: Entity, alpha: f32) -> Option<Transform2D> {
        let end = self.snapshots.back()?;
        let start = self.snapshots.iter().rev().nth(1).unwrap_or(end);

        let end_transform = find_entity(end, entity)?.transform2d?;
        let Some(start_transform) = find_entity(start, entity).and_then(|state| state.transform2d)
        else {
            return Some(end_transform);
        };

        let alpha = alpha.clamp(0.0, 1.0);
        Some(Transform2D::new(
            lerp_vec2(start_transform.position, end_transform.position, alpha),
            lerp_angle(start_transform.rotation, end_transform.rotation, alpha),
            lerp_vec2(start_transform.scale, end_transform.scale, alpha),
        ))
    }

    /// Interpolates a 3D transform from the newest two buffered snapshots.
    pub fn interpolate_transform(&self, entity: Entity, alpha: f32) -> Option<Transform> {
        let end = self.snapshots.back()?;
        let start = self.snapshots.iter().rev().nth(1).unwrap_or(end);

        let end_transform = find_entity(end, entity)?.transform?;
        let Some(start_transform) = find_entity(start, entity).and_then(|state| state.transform)
        else {
            return Some(end_transform);
        };

        let alpha = alpha.clamp(0.0, 1.0);
        Some(Transform::new(
            lerp_vec3(start_transform.position, end_transform.position, alpha),
            start_transform
                .rotation
                .slerp(end_transform.rotation, alpha),
            lerp_vec3(start_transform.scale, end_transform.scale, alpha),
        ))
    }
}

/// Remote-to-local entity mapping used when applying synchronized snapshots.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StateSyncEntityMap {
    remote_to_local: HashMap<Entity, Entity>,
}

impl StateSyncEntityMap {
    /// Returns the local entity mapped to one remote entity, if present.
    pub fn local(&self, remote: Entity) -> Option<Entity> {
        self.remote_to_local.get(&remote).copied()
    }

    /// Records or replaces the mapping for one remote entity.
    pub fn insert(&mut self, remote: Entity, local: Entity) {
        self.remote_to_local.insert(remote, local);
    }
}

fn lerp_vec2(start: Vec2, end: Vec2, alpha: f32) -> Vec2 {
    start + (end - start) * alpha
}

fn lerp_vec3(start: Vec3, end: Vec3, alpha: f32) -> Vec3 {
    start + (end - start) * alpha
}

fn lerp_angle(start: f32, end: f32, alpha: f32) -> f32 {
    let mut delta = end - start;
    while delta > PI {
        delta -= TAU;
    }
    while delta < -PI {
        delta += TAU;
    }
    start + delta * alpha
}

fn find_entity(snapshot: &ResolvedStateSnapshot, entity: Entity) -> Option<&ResolvedSyncEntity> {
    snapshot.entity(entity)
}
