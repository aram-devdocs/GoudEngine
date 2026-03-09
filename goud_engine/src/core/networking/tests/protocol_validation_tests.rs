use crate::core::networking::authority::BuiltInAuthorityPolicy;
use crate::core::networking::protocol::{encode_message, ProtocolMessage};
use crate::core::networking::types::{ClientEvent, SessionChannels};
use crate::core::networking::{SessionClient, SessionServer};
use crate::core::providers::network_types::ConnectionId;

use super::mock_provider::{create_server_config, pump, MockNetworkHub};

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
fn client_rejects_leave_notice_from_non_server_connections() {
    let hub = MockNetworkHub::default();

    let mut server =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    server
        .host(create_server_config(
            7114,
            "session-client-leave-source-validation",
            false,
        ))
        .unwrap();

    let client_provider = hub.provider();
    let client_endpoint = client_provider.endpoint_id();
    let mut client = SessionClient::new(Box::new(client_provider));
    let server_connection = client.join_direct("127.0.0.1:7114").unwrap();
    let _ = pump(&mut server, &mut [&mut client], 3);
    assert!(client.is_joined());

    let rogue_connection = ConnectionId(server_connection.0.wrapping_add(9_998));
    let rogue_leave_notice = encode_message(&ProtocolMessage::LeaveNotice {
        reason: "rogue".to_string(),
    })
    .unwrap();

    hub.enqueue_received(
        client_endpoint,
        rogue_connection,
        SessionChannels::default().control,
        rogue_leave_notice,
    )
    .unwrap();

    let events = client.tick().unwrap();

    assert!(events.iter().any(|event| {
        matches!(
            event,
            ClientEvent::ProtocolError { reason }
                if reason == "Received LeaveNotice from non-server connection"
        )
    }));
    assert!(!events.iter().any(|event| {
        matches!(
            event,
            ClientEvent::Left {
                connection,
                ..
            } if *connection == rogue_connection
        )
    }));
    assert!(client.is_joined());
    assert_eq!(client.server_connection(), Some(server_connection));
}
