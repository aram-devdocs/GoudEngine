use std::time::Duration;

use crate::core::networking::authority::{
    BuiltInAuthorityPolicy, RateLimitConfig, SchemaBoundsConfig,
};
use crate::core::networking::discovery::{
    DirectoryDiscoveryProvider, DiscoveredSession, DiscoveryError, DiscoveryMode, DiscoveryService,
};
use crate::core::networking::types::{ClientEvent, ServerEvent, SessionDescriptor};
use crate::core::networking::{SessionClient, SessionServer};

use super::mock_provider::{create_server_config, pump, MockNetworkHub};

#[derive(Clone)]
struct StubDirectoryProvider {
    sessions: Vec<DiscoveredSession>,
}

impl DirectoryDiscoveryProvider for StubDirectoryProvider {
    fn discover_sessions(&self) -> Result<Vec<DiscoveredSession>, DiscoveryError> {
        Ok(self.sessions.clone())
    }
}

#[test]
fn server_hosts_and_accepts_multiple_clients() {
    let hub = MockNetworkHub::default();

    let mut server =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    server
        .host(create_server_config(7101, "session-multi", false))
        .unwrap();

    let mut client1 = SessionClient::new(Box::new(hub.provider()));
    let mut client2 = SessionClient::new(Box::new(hub.provider()));

    client1.join_direct("127.0.0.1:7101").unwrap();
    client2.join_direct("127.0.0.1:7101").unwrap();

    let (server_events, _client_events) = pump(&mut server, &mut [&mut client1, &mut client2], 3);

    let joined_count = server_events
        .iter()
        .filter(|event| matches!(event, ServerEvent::ClientJoined { .. }))
        .count();

    assert_eq!(joined_count, 2);
    assert_eq!(server.client_count(), 2);
}

#[test]
fn client_discovery_supports_direct_lan_and_directory_modes() {
    let hub = MockNetworkHub::default();

    let mut server =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    server
        .host(create_server_config(7102, "session-discovery", true))
        .unwrap();

    let mut client = SessionClient::new(Box::new(hub.provider()));
    let discovery =
        DiscoveryService::new().with_directory_provider(Box::new(StubDirectoryProvider {
            sessions: vec![DiscoveredSession::new(SessionDescriptor::new(
                "directory-1",
                "Directory Session",
                "127.0.0.1:7102",
            ))],
        }));

    let direct = client
        .discover_sessions(
            &discovery,
            DiscoveryMode::Direct {
                address: "127.0.0.1:7102".to_string(),
            },
        )
        .unwrap();
    assert_eq!(direct.len(), 1);

    let lan = client
        .discover_sessions(&discovery, DiscoveryMode::Lan)
        .expect("lan discover should work on native targets");
    assert!(!lan.is_empty());

    let directory = client
        .discover_sessions(&discovery, DiscoveryMode::Directory)
        .unwrap();
    assert_eq!(directory.len(), 1);

    client.join_discovered(&direct[0]).unwrap();
    let (_server_events, client_events) = pump(&mut server, &mut [&mut client], 3);

    assert!(client_events[0]
        .iter()
        .any(|event| matches!(event, ClientEvent::Joined { .. })));
}

#[test]
fn server_validates_state_change_commands_and_rejects_invalid_payloads() {
    let hub = MockNetworkHub::default();

    let mut schema_config = SchemaBoundsConfig::new(2, 8);
    schema_config.allowed_command_tags = Some(vec![0xA0]);

    let mut server = SessionServer::with_policy(
        Box::new(hub.provider()),
        BuiltInAuthorityPolicy::SchemaBounds(schema_config),
    );
    server
        .host(create_server_config(7103, "session-authority", false))
        .unwrap();

    let mut client = SessionClient::new(Box::new(hub.provider()));
    client.join_direct("127.0.0.1:7103").unwrap();
    let _ = pump(&mut server, &mut [&mut client], 2);

    client.send_state_command(vec![0xB0, 1]).unwrap();
    let (server_events, client_events) = pump(&mut server, &mut [&mut client], 2);

    assert!(server_events.iter().any(|event| {
        matches!(
            event,
            ServerEvent::CommandRejected { payload, .. } if payload == &vec![0xB0, 1]
        )
    }));

    assert!(client_events[0].iter().any(|event| {
        matches!(
            event,
            ClientEvent::ValidationRejected { payload, .. } if payload == &vec![0xB0, 1]
        )
    }));
}

#[test]
fn server_broadcasts_authoritative_updates_to_all_connected_clients() {
    let hub = MockNetworkHub::default();

    let mut server =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    server
        .host(create_server_config(7104, "session-broadcast", false))
        .unwrap();

    let mut client1 = SessionClient::new(Box::new(hub.provider()));
    let mut client2 = SessionClient::new(Box::new(hub.provider()));

    client1.join_direct("127.0.0.1:7104").unwrap();
    client2.join_direct("127.0.0.1:7104").unwrap();
    let _ = pump(&mut server, &mut [&mut client1, &mut client2], 3);

    client1.send_state_command(vec![0xAA, 0x55]).unwrap();
    let (server_events, client_events) = pump(&mut server, &mut [&mut client1, &mut client2], 3);

    assert!(server_events.iter().any(|event| {
        matches!(
            event,
            ServerEvent::StateBroadcast {
                recipients,
                payload,
                ..
            } if *recipients == 2 && payload == &vec![0xAA, 0x55]
        )
    }));

    let updates_client1 = client_events[0]
        .iter()
        .filter(
            |event| matches!(event, ClientEvent::StateUpdated { payload, .. } if payload == &vec![0xAA, 0x55]),
        )
        .count();
    let updates_client2 = client_events[1]
        .iter()
        .filter(
            |event| matches!(event, ClientEvent::StateUpdated { payload, .. } if payload == &vec![0xAA, 0x55]),
        )
        .count();

    assert!(updates_client1 >= 1);
    assert!(updates_client2 >= 1);
}

#[test]
fn mid_session_join_and_leave_are_handled_gracefully() {
    let hub = MockNetworkHub::default();

    let mut server = SessionServer::with_policy(
        Box::new(hub.provider()),
        BuiltInAuthorityPolicy::RateLimited(RateLimitConfig::new(16, Duration::from_secs(1))),
    );
    server
        .host(create_server_config(7105, "session-join-leave", false))
        .unwrap();

    let mut client1 = SessionClient::new(Box::new(hub.provider()));
    client1.join_direct("127.0.0.1:7105").unwrap();
    let _ = pump(&mut server, &mut [&mut client1], 2);

    client1.send_state_command(vec![0x01]).unwrap();
    let _ = pump(&mut server, &mut [&mut client1], 2);

    let mut client2 = SessionClient::new(Box::new(hub.provider()));
    client2.join_direct("127.0.0.1:7105").unwrap();
    let (_server_events, client_events) = pump(&mut server, &mut [&mut client1, &mut client2], 3);

    assert!(client_events[1].iter().any(|event| {
        matches!(event, ClientEvent::Joined { snapshot, .. } if snapshot == &vec![0x01])
    }));

    client1.leave("left on purpose").unwrap();
    let (server_events, _client_events) = pump(&mut server, &mut [&mut client1, &mut client2], 3);

    assert!(server_events
        .iter()
        .any(|event| matches!(event, ServerEvent::ClientLeft { .. })));
    assert_eq!(server.client_count(), 1);

    client2.send_state_command(vec![0x02]).unwrap();
    let (server_events_after, _client_events_after) = pump(&mut server, &mut [&mut client2], 2);

    assert!(server_events_after.iter().any(|event| {
        matches!(
            event,
            ServerEvent::StateBroadcast {
                recipients,
                payload,
                ..
            } if *recipients == 1 && payload == &vec![0x02]
        )
    }));
}
