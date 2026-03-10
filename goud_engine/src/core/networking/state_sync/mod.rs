//! Minimal state synchronization helpers layered on top of session networking.

mod client;
mod server;
mod types;

pub use client::StateSyncClient;
pub use server::StateSyncServer;
pub use types::{
    EntityBandwidthStat, NetworkSync, ResolvedStateSnapshot, ResolvedSyncEntity,
    StateSnapshotPayload, StateSyncBandwidthStats, StateSyncConfig, StateSyncEntityMap,
    StateSyncEntitySnapshot, StateSyncInterpolationBuffer, Transform2DSnapshot, TransformSnapshot,
};
