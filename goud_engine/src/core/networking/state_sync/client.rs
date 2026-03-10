//! State-sync client helpers.

use std::collections::HashMap;

use crate::core::error::{GoudError, GoudResult};
use crate::core::networking::{ClientEvent, SessionClient};
use crate::core::serialization::{binary, DeltaEncode, NetworkMessage};
use crate::ecs::entity::Entity;
use crate::ecs::World;

use super::types::{
    NetworkSync, ResolvedStateSnapshot, ResolvedSyncEntity, StateSnapshotPayload, StateSyncConfig,
    StateSyncEntityMap, StateSyncInterpolationBuffer, Transform2DSnapshot, TransformSnapshot,
};

/// Client wrapper that resolves synchronized snapshots and feeds an interpolation buffer.
pub struct StateSyncClient {
    session: SessionClient,
    config: StateSyncConfig,
    latest_sequence: Option<u32>,
    last_resolved: HashMap<Entity, ResolvedSyncEntity>,
    entity_map: StateSyncEntityMap,
    interpolation_buffer: StateSyncInterpolationBuffer,
}

impl StateSyncClient {
    /// Creates a new state-sync client wrapper.
    pub fn new(session: SessionClient, config: StateSyncConfig) -> Self {
        Self {
            session,
            config,
            latest_sequence: None,
            last_resolved: HashMap::new(),
            entity_map: StateSyncEntityMap::default(),
            interpolation_buffer: StateSyncInterpolationBuffer::new(config.max_buffered_snapshots),
        }
    }

    /// Returns the wrapped session client.
    pub fn session(&self) -> &SessionClient {
        &self.session
    }

    /// Returns the wrapped session client mutably.
    pub fn session_mut(&mut self) -> &mut SessionClient {
        &mut self.session
    }

    /// Returns state-sync configuration.
    pub fn config(&self) -> StateSyncConfig {
        self.config
    }

    /// Returns the latest accepted sequence.
    pub fn latest_sequence(&self) -> Option<u32> {
        self.latest_sequence
    }

    /// Returns the interpolation buffer.
    pub fn interpolation_buffer(&self) -> &StateSyncInterpolationBuffer {
        &self.interpolation_buffer
    }

    /// Returns the current remote-to-local entity mapping used during world apply.
    pub fn entity_map(&self) -> &StateSyncEntityMap {
        &self.entity_map
    }

    /// Returns the latest resolved snapshot, if any.
    pub fn latest_snapshot(&self) -> Option<&ResolvedStateSnapshot> {
        self.interpolation_buffer.latest_snapshot()
    }

    /// Ingests a client event emitted by the wrapped session client.
    pub fn ingest_client_event(&mut self, event: &ClientEvent) -> GoudResult<bool> {
        match event {
            ClientEvent::Joined { snapshot, .. } => self.ingest_encoded(snapshot),
            ClientEvent::StateUpdated { payload, .. } => self.ingest_encoded(payload),
            _ => Ok(false),
        }
    }

    /// Ingests a fully encoded [`NetworkMessage`].
    pub fn ingest_encoded(&mut self, bytes: &[u8]) -> GoudResult<bool> {
        let message = NetworkMessage::decode(bytes)?;
        self.ingest_message(&message)
    }

    /// Ingests a decoded network message, ignoring out-of-order messages.
    pub fn ingest_message(&mut self, message: &NetworkMessage) -> GoudResult<bool> {
        if self
            .latest_sequence
            .is_some_and(|latest| message.sequence <= latest)
        {
            return Ok(false);
        }

        let payload: StateSnapshotPayload = binary::decode(&message.payload)?;
        let resolved_snapshot = self.resolve_payload(message.sequence, &payload)?;
        self.latest_sequence = Some(message.sequence);
        self.last_resolved = resolved_snapshot
            .entities
            .iter()
            .cloned()
            .map(|entity| (entity.entity, entity))
            .collect();
        self.interpolation_buffer.push(resolved_snapshot);
        Ok(true)
    }

    /// Applies the latest resolved snapshot directly to a world.
    pub fn apply_latest_to_world(&mut self, world: &mut World) -> GoudResult<()> {
        let Some(snapshot) = self.latest_snapshot().cloned() else {
            return Ok(());
        };

        self.apply_snapshot_to_world(world, &snapshot, None)
    }

    /// Applies the latest resolved snapshot to a world, interpolating transforms only.
    pub fn apply_interpolated_to_world(&mut self, world: &mut World, alpha: f32) -> GoudResult<()> {
        let Some(snapshot) = self.latest_snapshot().cloned() else {
            return Ok(());
        };

        self.apply_snapshot_to_world(world, &snapshot, Some(alpha.clamp(0.0, 1.0)))
    }

    fn resolve_payload(
        &self,
        sequence: u32,
        payload: &StateSnapshotPayload,
    ) -> GoudResult<ResolvedStateSnapshot> {
        let mut entities = Vec::with_capacity(payload.entities.len());

        for entity_snapshot in &payload.entities {
            let baseline = self.last_resolved.get(&entity_snapshot.entity);
            let mut resolved = baseline.cloned().unwrap_or(ResolvedSyncEntity {
                entity: entity_snapshot.entity,
                transform2d: None,
                transform: None,
                components: HashMap::new(),
            });

            match &entity_snapshot.transform2d {
                Some(Transform2DSnapshot::Full(transform)) => {
                    resolved.transform2d = Some(*transform);
                }
                Some(Transform2DSnapshot::Delta(delta)) => {
                    let baseline_transform = baseline
                        .and_then(|entity| entity.transform2d)
                        .ok_or_else(|| {
                            GoudError::InternalError(format!(
                                "Missing Transform2D baseline for entity {:?}",
                                entity_snapshot.entity
                            ))
                        })?;
                    resolved.transform2d = Some(baseline_transform.apply_delta(delta)?);
                }
                None => {}
            }

            match &entity_snapshot.transform {
                Some(TransformSnapshot::Full(transform)) => {
                    resolved.transform = Some(*transform);
                }
                Some(TransformSnapshot::Delta(delta)) => {
                    let baseline_transform = baseline
                        .and_then(|entity| entity.transform)
                        .ok_or_else(|| {
                            GoudError::InternalError(format!(
                                "Missing Transform baseline for entity {:?}",
                                entity_snapshot.entity
                            ))
                        })?;
                    resolved.transform = Some(baseline_transform.apply_delta(delta)?);
                }
                None => {}
            }

            if !entity_snapshot.components.is_empty() {
                resolved.components = decode_component_map(&entity_snapshot.components)?;
            }

            entities.push(resolved);
        }

        Ok(ResolvedStateSnapshot { sequence, entities })
    }

    fn apply_snapshot_to_world(
        &mut self,
        world: &mut World,
        snapshot: &ResolvedStateSnapshot,
        alpha: Option<f32>,
    ) -> GoudResult<()> {
        for remote_entity in &snapshot.entities {
            let local_entity = self.resolve_local_entity(world, remote_entity.entity);
            world.insert(local_entity, NetworkSync);

            let transform2d = alpha
                .and_then(|t| {
                    self.interpolation_buffer
                        .interpolate_transform2d(remote_entity.entity, t)
                })
                .or(remote_entity.transform2d);
            if let Some(transform2d) = transform2d {
                world.insert(local_entity, transform2d);
            }

            let transform = alpha
                .and_then(|t| {
                    self.interpolation_buffer
                        .interpolate_transform(remote_entity.entity, t)
                })
                .or(remote_entity.transform);
            if let Some(transform) = transform {
                world.insert(local_entity, transform);
            }

            if !remote_entity.components.is_empty() {
                let mut json = serde_json::Map::new();
                json.insert(
                    "components".to_string(),
                    serde_json::Value::Object(
                        remote_entity
                            .components
                            .iter()
                            .map(|(name, value)| (name.clone(), value.clone()))
                            .collect(),
                    ),
                );
                world.deserialize_entity_components(local_entity, &serde_json::Value::Object(json));
            }
        }

        Ok(())
    }

    fn resolve_local_entity(&mut self, world: &mut World, remote: Entity) -> Entity {
        if let Some(local) = self.entity_map.local(remote) {
            if world.is_alive(local) {
                return local;
            }
        }

        let local = world.spawn_empty();
        self.entity_map.insert(remote, local);
        local
    }
}

fn decode_component_map(
    components: &HashMap<String, Vec<u8>>,
) -> GoudResult<HashMap<String, serde_json::Value>> {
    components
        .iter()
        .map(|(name, bytes)| {
            serde_json::from_slice(bytes)
                .map(|value| (name.clone(), value))
                .map_err(|error| {
                    GoudError::InternalError(format!(
                        "Failed to decode component '{name}' from JSON bytes: {error}"
                    ))
                })
        })
        .collect()
}
