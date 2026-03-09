use crate::core::networking::authority::BuiltInAuthorityPolicy;
use crate::core::networking::discovery::{
    unregister_native_lan_session, DirectoryDiscoveryProvider, DiscoveredSession, DiscoveryError,
    DiscoveryMode, DiscoveryService,
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
fn rehost_unregisters_old_lan_advertisement_entries() {
    let hub = MockNetworkHub::default();

    let mut server =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);

    let first_id = "session-rehost-lan-first";
    let second_id = "session-rehost-lan-second";
    let third_id = "session-rehost-lan-disabled";
    let _ = unregister_native_lan_session(first_id);
    let _ = unregister_native_lan_session(second_id);
    let _ = unregister_native_lan_session(third_id);

    server
        .host(create_server_config(7110, first_id, true))
        .unwrap();
    let lan_after_first_host = DiscoveryService::new()
        .discover(DiscoveryMode::Lan)
        .unwrap();
    assert!(lan_after_first_host
        .iter()
        .any(|entry| entry.session.id == first_id));

    server
        .host(create_server_config(7111, second_id, true))
        .unwrap();
    let lan_after_second_host = DiscoveryService::new()
        .discover(DiscoveryMode::Lan)
        .unwrap();
    assert!(!lan_after_second_host
        .iter()
        .any(|entry| entry.session.id == first_id));
    assert!(lan_after_second_host
        .iter()
        .any(|entry| entry.session.id == second_id));

    server
        .host(create_server_config(7112, third_id, false))
        .unwrap();
    let lan_after_advertise_disabled = DiscoveryService::new()
        .discover(DiscoveryMode::Lan)
        .unwrap();
    assert!(!lan_after_advertise_disabled
        .iter()
        .any(|entry| entry.session.id == first_id));
    assert!(!lan_after_advertise_disabled
        .iter()
        .any(|entry| entry.session.id == second_id));
    assert!(!lan_after_advertise_disabled
        .iter()
        .any(|entry| entry.session.id == third_id));

    let _ = unregister_native_lan_session(first_id);
    let _ = unregister_native_lan_session(second_id);
    let _ = unregister_native_lan_session(third_id);
}
