use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::networking::authority::{
    AuthorityDecision, AuthorityPolicy, BuiltInAuthorityPolicy, RateLimitConfig,
    SchemaBoundsConfig, ValidationContext,
};
use crate::core::networking::discovery::{
    unregister_native_lan_session, DirectoryDiscoveryProvider, DiscoveredSession, DiscoveryError,
    DiscoveryMode, DiscoveryService,
};
use crate::core::networking::protocol::{encode_message, ProtocolMessage};
use crate::core::networking::types::{
    ClientEvent, ServerEvent, SessionChannels, SessionDescriptor,
};
use crate::core::networking::{SessionClient, SessionServer};
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{ConnectionId, DisconnectReason};

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

#[derive(Clone, Default)]
struct TrackingAuthority {
    disconnected: Arc<Mutex<Vec<ConnectionId>>>,
}

impl TrackingAuthority {
    fn disconnected_connections(&self) -> Vec<ConnectionId> {
        self.disconnected
            .lock()
            .expect("tracking authority lock should work")
            .clone()
    }
}

impl AuthorityPolicy for TrackingAuthority {
    fn validate(&mut self, _context: &ValidationContext<'_>) -> AuthorityDecision {
        AuthorityDecision::Accept
    }

    fn on_client_disconnected(&mut self, connection: ConnectionId) {
        self.disconnected
            .lock()
            .expect("tracking authority lock should work")
            .push(connection);
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

#[test]
fn non_joined_client_command_is_not_accepted_or_broadcast() {
    let hub = MockNetworkHub::default();

    let mut server =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    server
        .host(create_server_config(
            7106,
            "session-non-joined-command",
            false,
        ))
        .unwrap();

    let mut rogue_client = hub.provider();
    let connection = rogue_client.connect("127.0.0.1:7106").unwrap();
    let payload = vec![0xCC, 0xDD];
    let encoded = encode_message(&ProtocolMessage::StateCommand {
        payload: payload.clone(),
    })
    .unwrap();
    rogue_client
        .send(connection, SessionChannels::default().command, &encoded)
        .unwrap();

    let server_events = server.tick().unwrap();

    assert!(!server_events
        .iter()
        .any(|event| matches!(event, ServerEvent::CommandAccepted { .. })));
    assert!(!server_events
        .iter()
        .any(|event| matches!(event, ServerEvent::StateBroadcast { .. })));
    assert!(server_events.iter().any(|event| {
        matches!(
            event,
            ServerEvent::CommandRejected {
                payload: rejected_payload,
                ..
            } if rejected_payload == &payload
        )
    }));
    assert_eq!(server.state_sequence(), 0);
    assert!(server.authoritative_state().is_empty());
}

#[test]
fn duplicate_join_accepted_does_not_emit_duplicate_joined_event() {
    let hub = MockNetworkHub::default();

    let mut server =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    server
        .host(create_server_config(
            7107,
            "session-duplicate-join-accepted",
            false,
        ))
        .unwrap();

    let client_provider = hub.provider();
    let client_endpoint = client_provider.endpoint_id();
    let mut client = SessionClient::new(Box::new(client_provider));
    let connection = client.join_direct("127.0.0.1:7107").unwrap();

    let (_server_events, client_events) = pump(&mut server, &mut [&mut client], 3);
    let initial_joined_count = client_events[0]
        .iter()
        .filter(|event| matches!(event, ClientEvent::Joined { .. }))
        .count();
    assert_eq!(initial_joined_count, 1);
    assert!(client.is_joined());

    let duplicate_join_accepted = encode_message(&ProtocolMessage::JoinAccepted {
        snapshot: Vec::new(),
    })
    .unwrap();
    hub.enqueue_received(
        client_endpoint,
        connection,
        SessionChannels::default().control,
        duplicate_join_accepted,
    )
    .unwrap();

    let duplicate_events = client.tick().unwrap();
    let duplicate_joined_count = duplicate_events
        .iter()
        .filter(|event| matches!(event, ClientEvent::Joined { .. }))
        .count();
    assert_eq!(duplicate_joined_count, 0);
    assert!(client.is_joined());
}

#[test]
fn server_error_event_cleans_membership_authority_and_lan_population() {
    let hub = MockNetworkHub::default();
    let server_provider = hub.provider();
    let server_endpoint = server_provider.endpoint_id();

    let authority = TrackingAuthority::default();
    let authority_observer = authority.clone();
    let mut server = SessionServer::new(Box::new(server_provider), Box::new(authority));

    let session_id = "session-error-cleanup-lan";
    let _ = unregister_native_lan_session(session_id);
    server
        .host(create_server_config(7108, session_id, true))
        .unwrap();

    let mut client = SessionClient::new(Box::new(hub.provider()));
    client.join_direct("127.0.0.1:7108").unwrap();
    let (server_events, _client_events) = pump(&mut server, &mut [&mut client], 3);

    let joined_connection = server_events
        .iter()
        .find_map(|event| match event {
            ServerEvent::ClientJoined { connection } => Some(*connection),
            _ => None,
        })
        .expect("joined connection should be emitted");
    assert_eq!(server.client_count(), 1);

    let lan_before_error = DiscoveryService::new()
        .discover(DiscoveryMode::Lan)
        .unwrap();
    let before_entry = lan_before_error
        .iter()
        .find(|entry| entry.session.id == session_id)
        .expect("session should be advertised before error");
    assert_eq!(before_entry.session.current_clients, 1);

    hub.enqueue_error(
        server_endpoint,
        joined_connection,
        "simulated transport error",
    )
    .unwrap();
    let server_events_after_error = server.tick().unwrap();

    assert!(server_events_after_error.iter().any(|event| {
        matches!(
            event,
            ServerEvent::ProtocolError { connection, reason }
                if *connection == joined_connection && reason == "simulated transport error"
        )
    }));
    assert_eq!(server.client_count(), 0);
    assert_eq!(
        authority_observer.disconnected_connections(),
        vec![joined_connection]
    );

    let lan_after_error = DiscoveryService::new()
        .discover(DiscoveryMode::Lan)
        .unwrap();
    let after_entry = lan_after_error
        .iter()
        .find(|entry| entry.session.id == session_id)
        .expect("session should remain advertised after transport error");
    assert_eq!(after_entry.session.current_clients, 0);

    let _ = unregister_native_lan_session(session_id);
}

#[test]
fn client_rejects_state_messages_from_non_server_connections() {
    let hub = MockNetworkHub::default();

    let mut server =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    server
        .host(create_server_config(
            7109,
            "session-client-source-validation",
            false,
        ))
        .unwrap();

    let client_provider = hub.provider();
    let client_endpoint = client_provider.endpoint_id();
    let mut client = SessionClient::new(Box::new(client_provider));
    let server_connection = client.join_direct("127.0.0.1:7109").unwrap();
    let _ = pump(&mut server, &mut [&mut client], 3);

    let rogue_connection = ConnectionId(server_connection.0.wrapping_add(9_999));
    let rogue_update_payload = vec![0xD0, 0x01];
    let rogue_rejected_payload = vec![0xD0, 0x02];
    let rogue_state_update = encode_message(&ProtocolMessage::StateUpdate {
        sequence: 44,
        payload: rogue_update_payload.clone(),
    })
    .unwrap();
    let rogue_validation_rejected = encode_message(&ProtocolMessage::ValidationRejected {
        reason: "rogue".to_string(),
        payload: rogue_rejected_payload.clone(),
    })
    .unwrap();

    hub.enqueue_received(
        client_endpoint,
        rogue_connection,
        SessionChannels::default().state,
        rogue_state_update,
    )
    .unwrap();
    hub.enqueue_received(
        client_endpoint,
        rogue_connection,
        SessionChannels::default().control,
        rogue_validation_rejected,
    )
    .unwrap();

    let events = client.tick().unwrap();

    let protocol_errors = events
        .iter()
        .filter(|event| matches!(event, ClientEvent::ProtocolError { .. }))
        .count();
    assert_eq!(protocol_errors, 2);
    assert!(events.iter().any(|event| {
        matches!(
            event,
            ClientEvent::ProtocolError { reason }
                if reason == "Received StateUpdate from non-server connection"
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            ClientEvent::ProtocolError { reason }
                if reason == "Received ValidationRejected from non-server connection"
        )
    }));
    assert!(!events.iter().any(|event| {
        matches!(
            event,
            ClientEvent::StateUpdated { payload, .. } if payload == &rogue_update_payload
        )
    }));
    assert!(!events.iter().any(|event| {
        matches!(
            event,
            ClientEvent::ValidationRejected { payload, .. } if payload == &rogue_rejected_payload
        )
    }));
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

#[test]
fn leave_notice_maps_to_remote_close_instead_of_error() {
    let hub = MockNetworkHub::default();

    let mut server =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    server
        .host(create_server_config(
            7113,
            "session-graceful-leave-mapping",
            false,
        ))
        .unwrap();

    let client_provider = hub.provider();
    let client_endpoint = client_provider.endpoint_id();
    let mut client = SessionClient::new(Box::new(client_provider));
    let connection = client.join_direct("127.0.0.1:7113").unwrap();
    let _ = pump(&mut server, &mut [&mut client], 3);
    assert!(client.is_joined());

    let leave_notice = encode_message(&ProtocolMessage::LeaveNotice {
        reason: "server shutdown".to_string(),
    })
    .unwrap();
    hub.enqueue_received(
        client_endpoint,
        connection,
        SessionChannels::default().control,
        leave_notice,
    )
    .unwrap();

    let events = client.tick().unwrap();
    assert!(events.iter().any(|event| {
        matches!(
            event,
            ClientEvent::Left {
                connection: left_connection,
                reason: DisconnectReason::RemoteClose,
            } if *left_connection == connection
        )
    }));
    assert!(!events.iter().any(|event| {
        matches!(
            event,
            ClientEvent::Left {
                reason: DisconnectReason::Error(_),
                ..
            }
        )
    }));
    assert!(!client.is_joined());
    assert_eq!(client.server_connection(), None);
}
