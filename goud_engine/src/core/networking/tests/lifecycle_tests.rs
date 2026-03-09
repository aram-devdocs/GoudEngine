use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::networking::authority::{
    AuthorityDecision, AuthorityPolicy, BuiltInAuthorityPolicy, RateLimitConfig, ValidationContext,
};
use crate::core::networking::discovery::{
    unregister_native_lan_session, DiscoveryMode, DiscoveryService,
};
use crate::core::networking::protocol::{encode_message, ProtocolMessage};
use crate::core::networking::types::{ClientEvent, ServerEvent, SessionChannels};
use crate::core::networking::{SessionClient, SessionServer};
use crate::core::providers::network_types::{ConnectionId, DisconnectReason};

use super::mock_provider::{create_server_config, pump, MockNetworkHub};

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
