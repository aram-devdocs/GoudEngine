use crate::core::math::Vec2;
use crate::core::networking::authority::BuiltInAuthorityPolicy;
use crate::core::networking::{
    NetworkSync, SessionClient, SessionServer, StateSnapshotPayload, StateSyncClient,
    StateSyncConfig, StateSyncEntitySnapshot, StateSyncServer, Transform2DSnapshot,
};
use crate::core::serialization::{DeltaEncode, MessageKind, NetworkMessage};

type Transform2D = crate::ecs::components::Transform2D;
type World = crate::ecs::World;

use super::mock_provider::MockNetworkHub;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Health(u32);

impl crate::ecs::Component for Health {}

fn sync_world() -> World {
    let mut world = World::new();
    world.register_builtin_serializables();
    world
}

#[test]
fn state_sync_only_includes_network_sync_tagged_entities() {
    let hub = MockNetworkHub::default();
    let session =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    let mut sync_server = StateSyncServer::new(session, StateSyncConfig::default());

    let mut world = sync_world();
    let tagged = world.spawn_empty();
    world.insert(tagged, NetworkSync);
    world.insert(
        tagged,
        Transform2D::new(Vec2::new(10.0, 20.0), 0.0, Vec2::one()),
    );

    let untagged = world.spawn_empty();
    world.insert(
        untagged,
        Transform2D::new(Vec2::new(-5.0, -10.0), 0.0, Vec2::one()),
    );

    let message = sync_server.prepare_snapshot_message(&world).unwrap();
    assert_eq!(message.kind, MessageKind::Full);

    let payload: StateSnapshotPayload =
        crate::core::serialization::binary::decode(&message.payload).unwrap();
    assert_eq!(payload.entities.len(), 1);
    assert_eq!(payload.entities[0].entity, tagged);
    assert!(sync_server.bandwidth_stats().entity(tagged).is_some());
    assert!(sync_server.bandwidth_stats().entity(untagged).is_none());
}

#[test]
fn state_sync_client_ignores_out_of_order_snapshots() {
    let hub = MockNetworkHub::default();
    let client = SessionClient::new(Box::new(hub.provider()));
    let mut sync_client = StateSyncClient::new(client, StateSyncConfig::default());
    let entity = crate::ecs::entity::Entity::new(7, 1);

    let first = NetworkMessage::new(
        MessageKind::Full,
        2,
        crate::core::serialization::binary::encode(&StateSnapshotPayload {
            entities: vec![StateSyncEntitySnapshot {
                entity,
                transform2d: Some(Transform2DSnapshot::Full(Transform2D::from_position(
                    Vec2::new(4.0, 0.0),
                ))),
                transform: None,
                components: Default::default(),
            }],
        })
        .unwrap(),
    );

    let stale = NetworkMessage::new(
        MessageKind::Full,
        1,
        crate::core::serialization::binary::encode(&StateSnapshotPayload {
            entities: vec![StateSyncEntitySnapshot {
                entity,
                transform2d: Some(Transform2DSnapshot::Full(Transform2D::from_position(
                    Vec2::new(1.0, 0.0),
                ))),
                transform: None,
                components: Default::default(),
            }],
        })
        .unwrap(),
    );

    assert!(sync_client.ingest_message(&first).unwrap());
    assert!(!sync_client.ingest_message(&stale).unwrap());
    assert_eq!(sync_client.latest_sequence(), Some(2));
    assert_eq!(sync_client.interpolation_buffer().len(), 1);
    let transform = sync_client
        .interpolation_buffer()
        .interpolate_transform2d(entity, 0.5)
        .unwrap();
    assert!((transform.position.x - 4.0).abs() < f32::EPSILON);
}

#[test]
fn state_sync_interpolation_buffer_interpolates_between_latest_snapshots() {
    let hub = MockNetworkHub::default();
    let client = SessionClient::new(Box::new(hub.provider()));
    let mut sync_client = StateSyncClient::new(
        client,
        StateSyncConfig {
            max_buffered_snapshots: 2,
            ..StateSyncConfig::default()
        },
    );
    let entity = crate::ecs::entity::Entity::new(9, 1);
    let start = Transform2D::from_position(Vec2::new(0.0, 0.0));
    let end = Transform2D::new(Vec2::new(10.0, 0.0), 1.0, Vec2::one());
    let delta = end.delta_from(&start).unwrap();

    let first = NetworkMessage::new(
        MessageKind::Full,
        1,
        crate::core::serialization::binary::encode(&StateSnapshotPayload {
            entities: vec![StateSyncEntitySnapshot {
                entity,
                transform2d: Some(Transform2DSnapshot::Full(start)),
                transform: None,
                components: Default::default(),
            }],
        })
        .unwrap(),
    );
    let second = NetworkMessage::new(
        MessageKind::Delta,
        2,
        crate::core::serialization::binary::encode(&StateSnapshotPayload {
            entities: vec![StateSyncEntitySnapshot {
                entity,
                transform2d: Some(Transform2DSnapshot::Delta(delta)),
                transform: None,
                components: Default::default(),
            }],
        })
        .unwrap(),
    );

    assert!(sync_client.ingest_message(&first).unwrap());
    assert!(sync_client.ingest_message(&second).unwrap());
    assert_eq!(sync_client.interpolation_buffer().len(), 2);

    let interpolated = sync_client
        .interpolation_buffer()
        .interpolate_transform2d(entity, 0.25)
        .unwrap();
    assert!((interpolated.position.x - 2.5).abs() < 0.001);
    assert!((interpolated.rotation - 0.25).abs() < 0.001);
}

#[test]
fn state_sync_server_provides_latest_full_snapshot_for_late_join() {
    let hub = MockNetworkHub::default();
    let session =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    let mut sync_server = StateSyncServer::new(session, StateSyncConfig::default());

    let mut world = sync_world();
    let entity = world.spawn_empty();
    world.insert(entity, NetworkSync);
    world.insert(
        entity,
        Transform2D::new(Vec2::new(1.0, 2.0), 0.0, Vec2::one()),
    );

    let first = sync_server.prepare_snapshot_message(&world).unwrap();
    assert_eq!(first.kind, MessageKind::Full);

    world.insert(
        entity,
        Transform2D::new(Vec2::new(8.0, 3.0), 0.5, Vec2::new(2.0, 1.0)),
    );
    let second = sync_server.prepare_snapshot_message(&world).unwrap();
    assert_eq!(second.kind, MessageKind::Delta);

    let join_snapshot = sync_server
        .latest_full_snapshot_message()
        .unwrap()
        .expect("latest full snapshot should exist");
    assert_eq!(join_snapshot.kind, MessageKind::Full);
    assert_eq!(join_snapshot.sequence, second.sequence);

    let payload: StateSnapshotPayload =
        crate::core::serialization::binary::decode(&join_snapshot.payload).unwrap();
    assert_eq!(payload.entities.len(), 1);
    let transform = match &payload.entities[0].transform2d {
        Some(Transform2DSnapshot::Full(transform)) => *transform,
        _ => panic!("late join snapshot must carry full transform"),
    };
    assert!((transform.position.x - 8.0).abs() < f32::EPSILON);
    assert!((transform.position.y - 3.0).abs() < f32::EPSILON);
    assert!((transform.rotation - 0.5).abs() < f32::EPSILON);
}

#[test]
fn state_sync_bandwidth_stats_track_per_entity_full_and_delta_counts() {
    let hub = MockNetworkHub::default();
    let session =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    let mut sync_server = StateSyncServer::new(session, StateSyncConfig::default());

    let mut world = sync_world();
    let entity_a = world.spawn_empty();
    world.insert(entity_a, NetworkSync);
    world.insert(entity_a, Transform2D::from_position(Vec2::new(0.0, 0.0)));

    let entity_b = world.spawn_empty();
    world.insert(entity_b, NetworkSync);
    world.insert(entity_b, Transform2D::from_position(Vec2::new(10.0, 0.0)));

    let first = sync_server.prepare_snapshot_message(&world).unwrap();
    assert_eq!(first.kind, MessageKind::Full);

    world.insert(entity_a, Transform2D::from_position(Vec2::new(2.0, 0.0)));
    let second = sync_server.prepare_snapshot_message(&world).unwrap();
    assert_eq!(second.kind, MessageKind::Delta);

    let stats_a = sync_server.bandwidth_stats().entity(entity_a).unwrap();
    assert_eq!(stats_a.full_snapshots, 1);
    assert_eq!(stats_a.delta_snapshots, 1);
    assert!(stats_a.bytes_sent > 0);

    let stats_b = sync_server.bandwidth_stats().entity(entity_b).unwrap();
    assert_eq!(stats_b.full_snapshots, 1);
    assert_eq!(stats_b.delta_snapshots, 1);
    assert!(stats_b.bytes_sent > 0);
}

#[test]
fn state_sync_snapshot_rate_throttles_emission() {
    let hub = MockNetworkHub::default();
    let session =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    let mut sync_server = StateSyncServer::new(
        session,
        StateSyncConfig {
            snapshot_rate_hz: 2,
            ..StateSyncConfig::default()
        },
    );

    let mut world = sync_world();
    let entity = world.spawn_empty();
    world.insert(entity, NetworkSync);
    world.insert(entity, Transform2D::from_position(Vec2::new(1.0, 1.0)));

    let first = sync_server
        .prepare_snapshot_message_if_due(&world, 0.0)
        .unwrap();
    assert!(first.is_some());

    let second = sync_server
        .prepare_snapshot_message_if_due(&world, 0.2)
        .unwrap();
    assert!(second.is_none());

    let third = sync_server
        .prepare_snapshot_message_if_due(&world, 0.3)
        .unwrap();
    assert!(third.is_some());
}

#[test]
fn state_sync_apply_latest_to_world_applies_non_interpolated_components() {
    let hub = MockNetworkHub::default();
    let session =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    let mut sync_server = StateSyncServer::new(session, StateSyncConfig::default());

    let mut server_world = World::new();
    server_world.register_builtin_serializables();
    server_world.register_serializable::<Health>();

    let remote = server_world.spawn_empty();
    server_world.insert(remote, NetworkSync);
    server_world.insert(remote, Transform2D::from_position(Vec2::new(6.0, 3.0)));
    server_world.insert(remote, Health(42));

    let message = sync_server.prepare_snapshot_message(&server_world).unwrap();

    let client = SessionClient::new(Box::new(hub.provider()));
    let mut sync_client = StateSyncClient::new(client, StateSyncConfig::default());
    assert!(sync_client.ingest_message(&message).unwrap());

    let mut client_world = World::new();
    client_world.register_builtin_serializables();
    client_world.register_serializable::<Health>();
    sync_client
        .apply_latest_to_world(&mut client_world)
        .unwrap();

    let local = sync_client
        .entity_map()
        .local(remote)
        .expect("remote entity should be mapped locally");
    let transform = client_world
        .get::<Transform2D>(local)
        .expect("transform should be applied");
    let health = client_world
        .get::<Health>(local)
        .expect("custom component should be applied");

    assert_eq!(transform.position, Vec2::new(6.0, 3.0));
    assert_eq!(*health, Health(42));
}
