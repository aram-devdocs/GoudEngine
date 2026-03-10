//! State-sync server helpers.

use std::any::type_name;
use std::collections::HashMap;

use crate::core::error::{GoudError, GoudResult};
use crate::core::serialization::{MessageKind, NetworkMessage};
use crate::ecs::components::{Transform, Transform2D};
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::types::{
    NetworkSync, ResolvedSyncEntity, StateSnapshotPayload, StateSyncBandwidthStats,
    StateSyncConfig, StateSyncEntitySnapshot, Transform2DSnapshot, TransformSnapshot,
};
use crate::core::networking::SessionServer;
use crate::core::serialization::{binary, DeltaEncode};

/// Server wrapper that produces synchronized snapshots from tagged ECS entities.
pub struct StateSyncServer {
    session: SessionServer,
    config: StateSyncConfig,
    next_sequence: u32,
    time_since_last_snapshot: f32,
    last_resolved: HashMap<Entity, ResolvedSyncEntity>,
    latest_full_payload: Option<StateSnapshotPayload>,
    latest_full_sequence: Option<u32>,
    bandwidth_stats: StateSyncBandwidthStats,
}

impl StateSyncServer {
    /// Creates a new state-sync server wrapper.
    pub fn new(session: SessionServer, config: StateSyncConfig) -> Self {
        Self {
            session,
            config,
            next_sequence: 0,
            time_since_last_snapshot: 0.0,
            last_resolved: HashMap::new(),
            latest_full_payload: None,
            latest_full_sequence: None,
            bandwidth_stats: StateSyncBandwidthStats::default(),
        }
    }

    /// Returns the wrapped session server.
    pub fn session(&self) -> &SessionServer {
        &self.session
    }

    /// Returns the wrapped session server mutably.
    pub fn session_mut(&mut self) -> &mut SessionServer {
        &mut self.session
    }

    /// Returns state-sync configuration.
    pub fn config(&self) -> StateSyncConfig {
        self.config
    }

    /// Returns tracked bandwidth stats.
    pub fn bandwidth_stats(&self) -> &StateSyncBandwidthStats {
        &self.bandwidth_stats
    }

    /// Returns a full snapshot message for late joiners, if one has been prepared.
    pub fn latest_full_snapshot_message(&self) -> GoudResult<Option<NetworkMessage>> {
        let Some(payload) = &self.latest_full_payload else {
            return Ok(None);
        };
        let Some(sequence) = self.latest_full_sequence else {
            return Ok(None);
        };
        let payload_bytes = binary::encode(payload)?;
        Ok(Some(NetworkMessage::new(
            MessageKind::Full,
            sequence,
            payload_bytes,
        )))
    }

    /// Builds the next network message for the current world state.
    pub fn prepare_snapshot_message(&mut self, world: &World) -> GoudResult<NetworkMessage> {
        let (payload, resolved_entities) = self.capture_payload(world)?;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        let kind = if self.last_resolved.is_empty() {
            MessageKind::Full
        } else {
            MessageKind::Delta
        };
        let payload_bytes = binary::encode(&payload)?;
        self.record_bandwidth(kind, &payload)?;
        self.latest_full_payload = Some(build_full_payload(&resolved_entities)?);
        self.latest_full_sequence = Some(self.next_sequence);
        self.last_resolved = resolved_entities;
        Ok(NetworkMessage::new(kind, self.next_sequence, payload_bytes))
    }

    /// Builds the next network message only when the configured send interval has elapsed.
    pub fn prepare_snapshot_message_if_due(
        &mut self,
        world: &World,
        delta_seconds: f32,
    ) -> GoudResult<Option<NetworkMessage>> {
        if !self.should_emit_snapshot(delta_seconds) {
            return Ok(None);
        }

        self.prepare_snapshot_message(world).map(Some)
    }

    /// Broadcasts the next synchronized snapshot through the wrapped session server.
    pub fn sync_now(&mut self, world: &World) -> GoudResult<crate::core::networking::ServerEvent> {
        let message = self.prepare_snapshot_message(world)?;
        self.session
            .broadcast_authoritative_state(message.encode()?)
    }

    /// Broadcasts a synchronized snapshot only when the configured send interval has elapsed.
    pub fn sync_if_due(
        &mut self,
        world: &World,
        delta_seconds: f32,
    ) -> GoudResult<Option<crate::core::networking::ServerEvent>> {
        let Some(message) = self.prepare_snapshot_message_if_due(world, delta_seconds)? else {
            return Ok(None);
        };

        self.session
            .broadcast_authoritative_state(message.encode()?)
            .map(Some)
    }

    fn capture_payload(
        &self,
        world: &World,
    ) -> GoudResult<(StateSnapshotPayload, HashMap<Entity, ResolvedSyncEntity>)> {
        let mut entities = Vec::new();
        let mut resolved_entities = HashMap::new();

        for entity in world
            .archetypes()
            .iter()
            .flat_map(|archetype| archetype.entities().iter().copied())
        {
            if world.get::<NetworkSync>(entity).is_none() {
                continue;
            }

            let serialized = world.serialize_entity(entity).ok_or_else(|| {
                GoudError::InternalError(format!(
                    "Failed to serialize synchronized entity {:?}",
                    entity
                ))
            })?;
            let component_values = serialized
                .get("components")
                .and_then(|value| value.as_object())
                .ok_or_else(|| {
                    GoudError::InternalError("Serialized entity missing components map".to_string())
                })?;

            let mut snapshot = StateSyncEntitySnapshot {
                entity,
                transform2d: None,
                transform: None,
                components: HashMap::new(),
            };
            let mut resolved = ResolvedSyncEntity {
                entity,
                transform2d: None,
                transform: None,
                components: HashMap::new(),
            };

            for (component_name, value) in component_values {
                if component_name == type_name::<Transform2D>() {
                    let transform: Transform2D =
                        serde_json::from_value(value.clone()).map_err(to_internal_error)?;
                    snapshot.transform2d = match self
                        .last_resolved
                        .get(&entity)
                        .and_then(|previous| previous.transform2d)
                    {
                        Some(previous) => transform
                            .delta_from(&previous)
                            .map(Transform2DSnapshot::Delta),
                        None => Some(Transform2DSnapshot::Full(transform)),
                    };
                    resolved.transform2d = Some(transform);
                    continue;
                }

                if component_name == type_name::<Transform>() {
                    let transform: Transform =
                        serde_json::from_value(value.clone()).map_err(to_internal_error)?;
                    snapshot.transform = match self
                        .last_resolved
                        .get(&entity)
                        .and_then(|previous| previous.transform)
                    {
                        Some(previous) => transform
                            .delta_from(&previous)
                            .map(TransformSnapshot::Delta),
                        None => Some(TransformSnapshot::Full(transform)),
                    };
                    resolved.transform = Some(transform);
                    continue;
                }

                snapshot.components.insert(
                    component_name.clone(),
                    serde_json::to_vec(value).map_err(to_internal_error)?,
                );
                resolved
                    .components
                    .insert(component_name.clone(), value.clone());
            }

            entities.push(snapshot);
            resolved_entities.insert(entity, resolved);
        }

        Ok((StateSnapshotPayload { entities }, resolved_entities))
    }

    fn record_bandwidth(
        &mut self,
        kind: MessageKind,
        payload: &StateSnapshotPayload,
    ) -> GoudResult<()> {
        for entity_snapshot in &payload.entities {
            let encoded = binary::encode(entity_snapshot)?;
            let entry = self
                .bandwidth_stats
                .per_entity
                .entry(entity_snapshot.entity)
                .or_default();
            entry.bytes_sent += encoded.len() as u64;
            match kind {
                MessageKind::Full => entry.full_snapshots += 1,
                MessageKind::Delta => entry.delta_snapshots += 1,
            }
        }
        Ok(())
    }

    fn should_emit_snapshot(&mut self, delta_seconds: f32) -> bool {
        if self.last_resolved.is_empty() || self.config.snapshot_rate_hz == 0 {
            return true;
        }

        self.time_since_last_snapshot += delta_seconds.max(0.0);
        let interval_seconds = 1.0 / self.config.snapshot_rate_hz as f32;
        if self.time_since_last_snapshot + f32::EPSILON < interval_seconds {
            return false;
        }

        while self.time_since_last_snapshot >= interval_seconds {
            self.time_since_last_snapshot -= interval_seconds;
        }

        true
    }
}

fn to_internal_error(error: impl std::fmt::Display) -> GoudError {
    GoudError::InternalError(error.to_string())
}

fn build_full_payload(
    resolved_entities: &HashMap<Entity, ResolvedSyncEntity>,
) -> GoudResult<StateSnapshotPayload> {
    let mut entities: Vec<StateSyncEntitySnapshot> = resolved_entities
        .values()
        .map(|entity| {
            Ok(StateSyncEntitySnapshot {
                entity: entity.entity,
                transform2d: entity.transform2d.map(Transform2DSnapshot::Full),
                transform: entity.transform.map(TransformSnapshot::Full),
                components: encode_component_map(&entity.components)?,
            })
        })
        .collect::<GoudResult<Vec<_>>>()?;
    entities.sort_by_key(|entity| (entity.entity.index(), entity.entity.generation()));
    Ok(StateSnapshotPayload { entities })
}

fn encode_component_map(
    components: &HashMap<String, serde_json::Value>,
) -> GoudResult<HashMap<String, Vec<u8>>> {
    components
        .iter()
        .map(|(name, value)| {
            serde_json::to_vec(value)
                .map(|bytes| (name.clone(), bytes))
                .map_err(to_internal_error)
        })
        .collect()
}
