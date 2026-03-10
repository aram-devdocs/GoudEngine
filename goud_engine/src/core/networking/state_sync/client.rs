//! State-sync client helpers.

use std::collections::HashMap;

use crate::core::error::{GoudError, GoudResult};
use crate::core::networking::{ClientEvent, SessionClient};
use crate::core::serialization::{binary, DeltaEncode, NetworkMessage};
use crate::ecs::entity::Entity;

use super::types::{
    ResolvedStateSnapshot, ResolvedSyncEntity, StateSnapshotPayload, StateSyncConfig,
    StateSyncInterpolationBuffer, Transform2DSnapshot, TransformSnapshot,
};

/// Client wrapper that resolves synchronized snapshots and feeds an interpolation buffer.
pub struct StateSyncClient {
    session: SessionClient,
    config: StateSyncConfig,
    latest_sequence: Option<u32>,
    last_resolved: HashMap<Entity, ResolvedSyncEntity>,
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
                resolved.components = entity_snapshot.components.clone();
            }

            entities.push(resolved);
        }

        Ok(ResolvedStateSnapshot { sequence, entities })
    }
}
