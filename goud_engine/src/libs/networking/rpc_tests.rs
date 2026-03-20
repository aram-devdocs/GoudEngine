//! Tests for the RPC framework.

use super::rpc::{
    rpc_decode_message, rpc_encode_message, RpcConfig, RpcDirection, RpcFramework, RpcResult,
    RPC_HEADER_SIZE,
};

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

#[test]
fn test_register_and_query() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    assert!(!rpc.is_registered(1));

    rpc.register(
        1,
        "ping".to_string(),
        RpcDirection::Bidirectional,
        Box::new(|_| vec![]),
    )
    .unwrap();

    assert!(rpc.is_registered(1));
    assert!(!rpc.is_registered(2));
}

#[test]
fn test_duplicate_registration_fails() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    rpc.register(
        1,
        "ping".to_string(),
        RpcDirection::Bidirectional,
        Box::new(|_| vec![]),
    )
    .unwrap();

    let result = rpc.register(
        1,
        "ping2".to_string(),
        RpcDirection::Bidirectional,
        Box::new(|_| vec![]),
    );
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Wire format
// ---------------------------------------------------------------------------

#[test]
fn test_encode_decode_roundtrip() {
    let payload = b"hello world";
    let encoded = rpc_encode_message(42, 12345, 0, payload);
    assert_eq!(encoded.len(), RPC_HEADER_SIZE + payload.len());

    let (rpc_id, call_id, msg_type, decoded_payload) = rpc_decode_message(&encoded).unwrap();
    assert_eq!(rpc_id, 42);
    assert_eq!(call_id, 12345);
    assert_eq!(msg_type, 0);
    assert_eq!(decoded_payload, payload);
}

#[test]
fn test_decode_too_short() {
    let result = rpc_decode_message(&[0u8; 5]);
    assert!(result.is_err());
}

#[test]
fn test_encode_empty_payload() {
    let encoded = rpc_encode_message(1, 1, 1, &[]);
    assert_eq!(encoded.len(), RPC_HEADER_SIZE);

    let (_, _, _, payload) = rpc_decode_message(&encoded).unwrap();
    assert!(payload.is_empty());
}

// ---------------------------------------------------------------------------
// Call / Response lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_call_produces_outbox_message() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    let call_id = rpc.call(99, 1, b"request").unwrap();
    assert!(call_id > 0);

    let msgs = rpc.drain_outbox();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].peer_id, 99);

    // Decode the outbox message
    let (rpc_id, cid, msg_type, payload) = rpc_decode_message(&msgs[0].data).unwrap();
    assert_eq!(rpc_id, 1);
    assert_eq!(cid, call_id);
    assert_eq!(msg_type, 0); // MSG_TYPE_CALL
    assert_eq!(payload, b"request");
}

#[test]
fn test_call_ids_increment() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    let id1 = rpc.call(1, 1, b"").unwrap();
    let id2 = rpc.call(1, 1, b"").unwrap();
    assert_eq!(id2, id1 + 1);
}

#[test]
fn test_payload_too_large_rejected() {
    let config = RpcConfig {
        max_payload_size: 10,
        ..RpcConfig::default()
    };
    let mut rpc = RpcFramework::new(config);
    let result = rpc.call(1, 1, &[0u8; 11]);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// process_incoming: call handling
// ---------------------------------------------------------------------------

#[test]
fn test_incoming_call_dispatches_handler() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    rpc.register(
        10,
        "echo".to_string(),
        RpcDirection::Bidirectional,
        Box::new(|payload| {
            let mut resp = b"echo:".to_vec();
            resp.extend_from_slice(payload);
            resp
        }),
    )
    .unwrap();

    // Simulate an incoming call
    let call_data = rpc_encode_message(10, 42, 0, b"hello");
    rpc.process_incoming(5, &call_data).unwrap();

    // Should have produced a response in the outbox
    let msgs = rpc.drain_outbox();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].peer_id, 5);

    let (rpc_id, call_id, msg_type, payload) = rpc_decode_message(&msgs[0].data).unwrap();
    assert_eq!(rpc_id, 10);
    assert_eq!(call_id, 42);
    assert_eq!(msg_type, 1); // MSG_TYPE_RESPONSE
    assert_eq!(payload, b"echo:hello");
}

#[test]
fn test_incoming_call_unknown_rpc_sends_error() {
    let mut rpc = RpcFramework::new(RpcConfig::default());

    let call_data = rpc_encode_message(99, 1, 0, b"");
    rpc.process_incoming(5, &call_data).unwrap();

    let msgs = rpc.drain_outbox();
    assert_eq!(msgs.len(), 1);

    let (_, _, msg_type, payload) = rpc_decode_message(&msgs[0].data).unwrap();
    assert_eq!(msg_type, 2); // MSG_TYPE_ERROR
    let err_msg = String::from_utf8_lossy(payload);
    assert!(err_msg.contains("Unknown RPC id 99"));
}

// ---------------------------------------------------------------------------
// process_incoming: response handling
// ---------------------------------------------------------------------------

#[test]
fn test_incoming_response_resolves_pending_call() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    let call_id = rpc.call(5, 1, b"req").unwrap();
    let _ = rpc.drain_outbox(); // clear

    // Simulate a response
    let resp_data = rpc_encode_message(1, call_id, 1, b"resp");
    rpc.process_incoming(5, &resp_data).unwrap();

    assert!(rpc.has_result(call_id));
    let result = rpc.take_result(call_id).unwrap();
    assert_eq!(result, RpcResult::Success(b"resp".to_vec()));
}

#[test]
fn test_incoming_error_resolves_pending_call() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    let call_id = rpc.call(5, 1, b"req").unwrap();
    let _ = rpc.drain_outbox();

    let err_data = rpc_encode_message(1, call_id, 2, b"something went wrong");
    rpc.process_incoming(5, &err_data).unwrap();

    let result = rpc.take_result(call_id).unwrap();
    assert_eq!(result, RpcResult::Error("something went wrong".to_string()));
}

// ---------------------------------------------------------------------------
// Timeout
// ---------------------------------------------------------------------------

#[test]
fn test_update_expires_timed_out_calls() {
    let config = RpcConfig {
        timeout_ms: 0, // instant timeout
        ..RpcConfig::default()
    };
    let mut rpc = RpcFramework::new(config);
    let call_id = rpc.call(1, 1, b"").unwrap();
    let _ = rpc.drain_outbox();

    // One update tick should be enough with timeout_ms=0
    std::thread::sleep(std::time::Duration::from_millis(1));
    rpc.update(0.0);

    let result = rpc.take_result(call_id).unwrap();
    assert_eq!(result, RpcResult::Timeout);
}

#[test]
fn test_pending_count() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    assert_eq!(rpc.pending_count(), 0);

    let _ = rpc.call(1, 1, b"").unwrap();
    let _ = rpc.call(1, 1, b"").unwrap();
    assert_eq!(rpc.pending_count(), 2);
}

// ---------------------------------------------------------------------------
// Full round-trip: two frameworks talking to each other
// ---------------------------------------------------------------------------

#[test]
fn test_two_frameworks_round_trip() {
    // Server side
    let mut server = RpcFramework::new(RpcConfig::default());
    server
        .register(
            1,
            "add".to_string(),
            RpcDirection::ClientToServer,
            Box::new(|payload| {
                // Expect two u32 values, return their sum
                if payload.len() == 8 {
                    let a = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    let b = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
                    let sum = a + b;
                    sum.to_le_bytes().to_vec()
                } else {
                    vec![]
                }
            }),
        )
        .unwrap();

    // Client side
    let mut client = RpcFramework::new(RpcConfig::default());
    let mut payload = Vec::new();
    payload.extend_from_slice(&10u32.to_le_bytes());
    payload.extend_from_slice(&32u32.to_le_bytes());

    let call_id = client.call(1, 1, &payload).unwrap();

    // Transfer client outbox -> server incoming
    let client_msgs = client.drain_outbox();
    assert_eq!(client_msgs.len(), 1);
    server.process_incoming(2, &client_msgs[0].data).unwrap();

    // Transfer server outbox -> client incoming
    let server_msgs = server.drain_outbox();
    assert_eq!(server_msgs.len(), 1);
    client.process_incoming(1, &server_msgs[0].data).unwrap();

    // Client should now have the result
    let result = client.take_result(call_id).unwrap();
    assert_eq!(result, RpcResult::Success(42u32.to_le_bytes().to_vec()));
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_process_incoming_too_short() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    let result = rpc.process_incoming(1, &[0u8; 3]);
    assert!(result.is_err());
}

#[test]
fn test_take_result_nonexistent_call() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    assert!(rpc.take_result(999).is_none());
}

#[test]
fn test_has_result_before_response() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    let call_id = rpc.call(1, 1, b"").unwrap();
    assert!(!rpc.has_result(call_id));
}

#[test]
fn test_drain_outbox_is_empty_after_drain() {
    let mut rpc = RpcFramework::new(RpcConfig::default());
    let _ = rpc.call(1, 1, b"").unwrap();
    assert_eq!(rpc.drain_outbox().len(), 1);
    assert!(rpc.drain_outbox().is_empty());
}

#[test]
fn test_config_accessor() {
    let config = RpcConfig {
        timeout_ms: 1234,
        max_payload_size: 999,
    };
    let rpc = RpcFramework::new(config);
    assert_eq!(rpc.config().timeout_ms, 1234);
    assert_eq!(rpc.config().max_payload_size, 999);
}
