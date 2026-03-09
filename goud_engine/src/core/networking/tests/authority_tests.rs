use crate::core::networking::authority::{BuiltInAuthorityPolicy, SchemaBoundsConfig};
use crate::core::networking::protocol::{encode_message, ProtocolMessage};
use crate::core::networking::types::{ClientEvent, ServerEvent, SessionChannels};
use crate::core::networking::{SessionClient, SessionServer};
use crate::core::providers::network::NetworkProvider;

use super::mock_provider::{create_server_config, pump, MockNetworkHub};

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
