mod integration;

use super::*;
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::network_types::{
    Channel, ConnectionState, HostConfig, NetworkEvent, TurnServer, WebRtcConfig,
};
use crate::core::providers::ProviderLifecycle;

fn host_config() -> HostConfig {
    HostConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 8,
        tls_cert_path: None,
        tls_key_path: None,
    }
}

// =========================================================================
// Construction and defaults
// =========================================================================

#[test]
fn test_webrtc_construction_defaults() {
    let provider = WebRtcNetProvider::new();
    assert_eq!(provider.name(), "webrtc");
    assert_eq!(provider.version(), "0.1.0");
    assert!(provider.connections().is_empty());
    assert!(provider.local_addr().is_none());
    assert!(provider.public_addr().is_none());

    let caps = provider.network_capabilities();
    assert!(caps.supports_hosting);
    assert_eq!(caps.max_connections, 32);

    let stats = provider.stats();
    assert_eq!(stats.bytes_sent, 0);
    assert_eq!(stats.bytes_received, 0);
}

#[test]
fn test_webrtc_with_config_stores_stun_and_turn() {
    let config = WebRtcConfig {
        stun_servers: vec!["stun:stun.l.google.com:19302".to_string()],
        turn_servers: vec![TurnServer {
            url: "turn:relay.example.com:3478".to_string(),
            username: "user".to_string(),
            credential: "pass".to_string(),
        }],
    };
    let provider = WebRtcNetProvider::with_config(config);
    let cfg = provider.webrtc_config();
    assert_eq!(cfg.stun_servers.len(), 1);
    assert_eq!(cfg.turn_servers.len(), 1);
    assert_eq!(cfg.turn_servers[0].username, "user");
}

// =========================================================================
// Lifecycle
// =========================================================================

#[test]
fn test_webrtc_lifecycle_init_update_shutdown() {
    let mut provider = WebRtcNetProvider::new();
    assert!(provider.init().is_ok());
    assert!(provider.update(0.0).is_ok());
    provider.shutdown();
}

// =========================================================================
// Host and connect round-trip
// =========================================================================

#[test]
fn test_webrtc_host_connect_round_trip() {
    let mut host = WebRtcNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().expect("host should have a local addr");

    let mut client = WebRtcNetProvider::new();
    let client_conn = client.connect(&host_addr.to_string()).unwrap();

    // Poll until connected.
    let mut host_connected = false;
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();

        if host
            .drain_events()
            .iter()
            .any(|e| matches!(e, NetworkEvent::Connected { .. }))
        {
            host_connected = true;
        }

        if host_connected && client.connection_state(client_conn) == ConnectionState::Connected {
            client.drain_events();
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert!(host_connected, "host must see a connected event");
    assert_eq!(
        client.connection_state(client_conn),
        ConnectionState::Connected
    );

    host.shutdown();
    client.shutdown();
}

// =========================================================================
// Disconnect
// =========================================================================

#[test]
fn test_webrtc_disconnect_emits_local_close() {
    let mut host = WebRtcNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().unwrap();

    let mut client = WebRtcNetProvider::new();
    let client_conn = client.connect(&host_addr.to_string()).unwrap();

    // Wait for connection.
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        host.drain_events();
        client.drain_events();
        if client.connection_state(client_conn) == ConnectionState::Connected {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    client.disconnect(client_conn).unwrap();
    let events = client.drain_events();
    assert!(events.iter().any(|e| matches!(
        e,
        NetworkEvent::Disconnected {
            reason: DisconnectReason::LocalClose,
            ..
        }
    )));

    host.shutdown();
    client.shutdown();
}

#[test]
fn test_webrtc_disconnect_all_clears_connections() {
    let mut host = WebRtcNetProvider::new();
    host.host(&host_config()).unwrap();
    let host_addr = host.local_addr().unwrap();

    let mut client = WebRtcNetProvider::new();
    let _c1 = client.connect(&host_addr.to_string()).unwrap();

    // Wait for connection.
    for _ in 0..100 {
        host.update(0.0).unwrap();
        client.update(0.0).unwrap();
        host.drain_events();
        client.drain_events();
        if !client.connections().is_empty()
            && client.connection_state(client.connections()[0]) == ConnectionState::Connected
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    client.disconnect_all().unwrap();
    assert!(client.connections().is_empty());

    host.shutdown();
    client.shutdown();
}

// =========================================================================
// Network diagnostics
// =========================================================================

#[test]
fn test_webrtc_diagnostics_returns_valid_snapshot() {
    let provider = WebRtcNetProvider::new();
    let diag = provider.network_diagnostics();
    assert_eq!(diag.bytes_sent, 0);
    assert_eq!(diag.bytes_received, 0);
    assert_eq!(diag.active_connections, 0);
}

// =========================================================================
// STUN helpers
// =========================================================================

#[test]
fn test_stun_binding_request_format() {
    let (request, tid) = build_stun_binding_request();
    assert_eq!(request.len(), 20);
    // Type: Binding Request (0x0001)
    assert_eq!(request[0], 0x00);
    assert_eq!(request[1], 0x01);
    // Length: 0
    assert_eq!(request[2], 0x00);
    assert_eq!(request[3], 0x00);
    // Magic Cookie
    assert_eq!(&request[4..8], &stun::STUN_MAGIC_COOKIE.to_be_bytes());
    // Transaction ID is embedded.
    assert_eq!(&request[8..20], &tid);
}

#[test]
fn test_stun_response_parsing_ipv4() {
    let tid: [u8; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    // Build a synthetic STUN binding success response with XOR-MAPPED-ADDRESS.
    let mut response = Vec::new();
    // Type: Binding Success Response (0x0101)
    response.extend_from_slice(&[0x01, 0x01]);
    // Length: 12 (one attribute with 8 bytes of value)
    response.extend_from_slice(&[0x00, 0x0C]);
    // Magic Cookie
    response.extend_from_slice(&stun::STUN_MAGIC_COOKIE.to_be_bytes());
    // Transaction ID
    response.extend_from_slice(&tid);

    // XOR-MAPPED-ADDRESS attribute
    // Type: 0x0020
    response.extend_from_slice(&[0x00, 0x20]);
    // Length: 8
    response.extend_from_slice(&[0x00, 0x08]);
    // Reserved + Family (0x01 = IPv4)
    response.extend_from_slice(&[0x00, 0x01]);
    // XOR Port: 8080 ^ (STUN_MAGIC_COOKIE >> 16) = 8080 ^ 0x2112 = 0x1062 ^ 0x2112
    let port: u16 = 8080;
    let xor_port = port ^ (stun::STUN_MAGIC_COOKIE >> 16) as u16;
    response.extend_from_slice(&xor_port.to_be_bytes());
    // XOR IP: 192.168.1.100 ^ STUN_MAGIC_COOKIE
    let ip = u32::from_be_bytes([192, 168, 1, 100]);
    let xor_ip = ip ^ stun::STUN_MAGIC_COOKIE;
    response.extend_from_slice(&xor_ip.to_be_bytes());

    let result = parse_stun_response(&response, &tid);
    assert!(result.is_some(), "should parse a valid STUN response");
    let addr = result.unwrap();
    assert_eq!(addr.port(), 8080);
    match addr {
        SocketAddr::V4(v4) => {
            assert_eq!(v4.ip().octets(), [192, 168, 1, 100]);
        }
        _ => panic!("expected IPv4 address"),
    }
}

#[test]
fn test_stun_response_parsing_rejects_wrong_tid() {
    let tid: [u8; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let wrong_tid: [u8; 12] = [99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99];

    let mut response = Vec::new();
    response.extend_from_slice(&[0x01, 0x01]);
    response.extend_from_slice(&[0x00, 0x00]);
    response.extend_from_slice(&stun::STUN_MAGIC_COOKIE.to_be_bytes());
    response.extend_from_slice(&wrong_tid);

    let result = parse_stun_response(&response, &tid);
    assert!(result.is_none(), "should reject mismatched transaction ID");
}

// =========================================================================
// Frame encoding/decoding
// =========================================================================

#[test]
fn test_frame_encode_decode_round_trip() {
    let channel = Channel(1);
    let payload = b"test-data";
    let frame = encode_frame(channel, payload);

    let (decoded_channel, decoded_payload) =
        decode_frame(&frame).expect("should decode a valid frame");
    assert_eq!(decoded_channel, channel);
    assert_eq!(decoded_payload, payload);
}

#[test]
fn test_frame_decode_rejects_short_data() {
    assert!(decode_frame(&[0, 0, 0]).is_none());
    assert!(decode_frame(&[0, 0, 0, 0]).is_none()); // frame_len = 0
}
