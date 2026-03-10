use crate::core::networking::authority::BuiltInAuthorityPolicy;
use crate::core::networking::discovery::{DiscoveryMode, DiscoveryService};
use crate::core::networking::{
    LobbyClient, LobbyCommand, LobbyConfig, LobbyEvent, LobbyServer, LobbyState, LobbyVisibility,
    SessionClient, SessionServer,
};

use super::mock_provider::{create_server_config, MockNetworkHub};

fn pump_lobby(
    server: &mut LobbyServer,
    clients: &mut [&mut LobbyClient],
    ticks: usize,
) -> (Vec<LobbyEvent>, Vec<Vec<LobbyEvent>>) {
    let mut server_events = Vec::new();
    let mut client_events = (0..clients.len()).map(|_| Vec::new()).collect::<Vec<_>>();

    for _ in 0..ticks {
        server_events.extend(server.tick().expect("server tick should succeed"));
        for (index, client) in clients.iter_mut().enumerate() {
            client_events[index].extend(client.tick().expect("client tick should succeed"));
        }
    }

    (server_events, client_events)
}

fn host_lobby(
    hub: &MockNetworkHub,
    port: u16,
    config: LobbyConfig,
    advertise_on_lan: bool,
) -> LobbyServer {
    let session =
        SessionServer::with_policy(Box::new(hub.provider()), BuiltInAuthorityPolicy::AllowAll);
    let mut server = LobbyServer::new(session, config);
    server
        .host(create_server_config(port, "unused", advertise_on_lan))
        .unwrap();
    server
}

#[test]
fn public_lobbies_are_listed_and_private_lobbies_are_hidden() {
    let hub = MockNetworkHub::default();
    let _public_server = host_lobby(
        &hub,
        7201,
        LobbyConfig::new("public-room", "Public Room", 4, LobbyVisibility::Public),
        true,
    );
    let _private_server = host_lobby(
        &hub,
        7202,
        LobbyConfig::new("private-room", "Private Room", 2, LobbyVisibility::Private),
        true,
    );

    let client = LobbyClient::new(SessionClient::new(Box::new(hub.provider())));
    let listed = client
        .discover_public_lobbies(&DiscoveryService::new(), DiscoveryMode::Lan)
        .unwrap();

    assert!(listed.iter().any(|entry| entry
        .session
        .metadata
        .get("lobby.id")
        .is_some_and(|id| id == "public-room")));
    assert!(!listed.iter().any(|entry| {
        entry
            .session
            .metadata
            .get("lobby.id")
            .is_some_and(|id| id == "private-room")
    }));

    let public_entry = listed
        .iter()
        .find(|entry| {
            entry
                .session
                .metadata
                .get("lobby.id")
                .is_some_and(|id| id == "public-room")
        })
        .unwrap();
    assert_eq!(public_entry.session.metadata["lobby.name"], "Public Room");
    assert_eq!(public_entry.session.metadata["lobby.visibility"], "public");
    assert_eq!(public_entry.session.metadata["lobby.state"], "waiting");
    assert_eq!(public_entry.session.metadata["lobby.current_players"], "0");
    assert_eq!(public_entry.session.metadata["lobby.max_players"], "4");
}

#[test]
fn lobby_client_can_join_from_list_and_by_id() {
    let hub = MockNetworkHub::default();
    let mut server = host_lobby(
        &hub,
        7203,
        LobbyConfig::new("join-room", "Join Room", 4, LobbyVisibility::Public),
        true,
    );
    let discovery = DiscoveryService::new();

    let mut listed_client = LobbyClient::new(SessionClient::new(Box::new(hub.provider())));
    let listed = listed_client
        .discover_public_lobbies(&discovery, DiscoveryMode::Lan)
        .unwrap();
    let join_room = listed
        .iter()
        .find(|entry| {
            entry
                .session
                .metadata
                .get("lobby.id")
                .is_some_and(|id| id == "join-room")
        })
        .unwrap()
        .clone();
    listed_client.join_listed_lobby(&join_room).unwrap();

    let mut by_id_client = LobbyClient::new(SessionClient::new(Box::new(hub.provider())));
    by_id_client
        .join_lobby_by_id(&discovery, DiscoveryMode::Lan, "join-room")
        .unwrap();

    let (_server_events, client_events) =
        pump_lobby(&mut server, &mut [&mut listed_client, &mut by_id_client], 3);

    assert!(client_events[0].iter().any(|event| {
        matches!(
            event,
            LobbyEvent::SnapshotUpdated { state, members }
                if *state == LobbyState::Waiting && members.len() == 2
        )
    }));
    assert!(client_events[1].iter().any(|event| {
        matches!(
            event,
            LobbyEvent::SnapshotUpdated { state, members }
                if *state == LobbyState::Waiting && members.len() == 2
        )
    }));
}

#[test]
fn lobby_requires_all_players_ready_before_normal_start() {
    let hub = MockNetworkHub::default();
    let mut server = host_lobby(
        &hub,
        7204,
        LobbyConfig::new("ready-room", "Ready Room", 4, LobbyVisibility::Public),
        false,
    );

    let mut host = LobbyClient::new(SessionClient::new(Box::new(hub.provider())));
    let mut guest = LobbyClient::new(SessionClient::new(Box::new(hub.provider())));
    host.join_private_lobby("127.0.0.1:7204").unwrap();
    guest.join_private_lobby("127.0.0.1:7204").unwrap();
    let _ = pump_lobby(&mut server, &mut [&mut host, &mut guest], 3);

    host.start_game().unwrap();
    let (_server_events, client_events) = pump_lobby(&mut server, &mut [&mut host, &mut guest], 2);
    assert!(client_events[0].iter().any(|event| {
        matches!(
            event,
            LobbyEvent::CommandRejected { command: Some(LobbyCommand::StartGame), reason }
                if reason == "all players must be ready before starting"
        )
    }));

    host.set_ready(true).unwrap();
    guest.set_ready(true).unwrap();
    let _ = pump_lobby(&mut server, &mut [&mut host, &mut guest], 2);

    host.start_game().unwrap();
    let (_server_events, client_events) = pump_lobby(&mut server, &mut [&mut host, &mut guest], 2);
    assert!(client_events[0].iter().any(|event| {
        matches!(
            event,
            LobbyEvent::SnapshotUpdated { state, .. } if *state == LobbyState::InGame
        )
    }));
}

#[test]
fn lobby_rejects_non_host_actions_and_allows_host_kick_and_early_start() {
    let hub = MockNetworkHub::default();
    let mut server = host_lobby(
        &hub,
        7205,
        LobbyConfig::new("host-room", "Host Room", 4, LobbyVisibility::Public),
        false,
    );

    let mut host = LobbyClient::new(SessionClient::new(Box::new(hub.provider())));
    let mut guest = LobbyClient::new(SessionClient::new(Box::new(hub.provider())));
    host.join_private_lobby("127.0.0.1:7205").unwrap();
    guest.join_private_lobby("127.0.0.1:7205").unwrap();
    let _ = pump_lobby(&mut server, &mut [&mut host, &mut guest], 3);

    let guest_connection = guest
        .members()
        .unwrap()
        .into_iter()
        .find(|member| !member.is_host)
        .map(|member| member.connection)
        .unwrap();

    guest.start_early().unwrap();
    let (_server_events, client_events) = pump_lobby(&mut server, &mut [&mut host, &mut guest], 2);
    assert!(client_events[1].iter().any(|event| {
        matches!(
            event,
            LobbyEvent::CommandRejected { command: Some(LobbyCommand::StartEarly), reason }
                if reason == "only the host can start early"
        )
    }));

    guest.kick(guest_connection).unwrap();
    let (_server_events, client_events) = pump_lobby(&mut server, &mut [&mut host, &mut guest], 2);
    assert!(client_events[1].iter().any(|event| {
        matches!(
            event,
            LobbyEvent::CommandRejected { command: Some(LobbyCommand::Kick { .. }), reason }
                if reason == "only the host can kick members"
        )
    }));

    let host_connection = host
        .members()
        .unwrap()
        .into_iter()
        .find(|member| member.is_host)
        .map(|member| member.connection)
        .unwrap();
    host.kick(host_connection).unwrap();
    let (_server_events, client_events) = pump_lobby(&mut server, &mut [&mut host, &mut guest], 2);
    assert!(client_events[0].iter().any(|event| {
        matches!(
            event,
            LobbyEvent::CommandRejected { command: Some(LobbyCommand::Kick { connection }), reason }
                if *connection == host_connection && reason == "host cannot kick themselves"
        )
    }));
    assert_eq!(server.members().len(), 2);

    host.kick(guest_connection).unwrap();
    let (_server_events, client_events) = pump_lobby(&mut server, &mut [&mut host, &mut guest], 2);
    assert!(client_events[0].iter().any(|event| {
        matches!(
            event,
            LobbyEvent::SnapshotUpdated { members, .. } if members.len() == 1
        )
    }));

    host.start_early().unwrap();
    let (_server_events, client_events) = pump_lobby(&mut server, &mut [&mut host], 2);
    assert!(client_events[0].iter().any(|event| {
        matches!(
            event,
            LobbyEvent::SnapshotUpdated { state, .. } if *state == LobbyState::InGame
        )
    }));
}

#[test]
fn lobby_rejects_join_when_room_is_full() {
    let hub = MockNetworkHub::default();
    let mut server = host_lobby(
        &hub,
        7206,
        LobbyConfig::new("full-room", "Full Room", 1, LobbyVisibility::Public),
        false,
    );

    let mut first = LobbyClient::new(SessionClient::new(Box::new(hub.provider())));
    let mut second = LobbyClient::new(SessionClient::new(Box::new(hub.provider())));
    first.join_private_lobby("127.0.0.1:7206").unwrap();
    let _ = pump_lobby(&mut server, &mut [&mut first], 3);

    second.join_private_lobby("127.0.0.1:7206").unwrap();
    let (server_events, client_events) = pump_lobby(&mut server, &mut [&mut first, &mut second], 3);

    assert!(server_events.iter().any(|event| {
        matches!(
            event,
            LobbyEvent::JoinDenied { reason, .. } if reason == "lobby is full"
        )
    }));
    assert!(!server_events
        .iter()
        .any(|event| matches!(event, LobbyEvent::JoinRejected { .. })));
    assert!(client_events[1].iter().any(|event| {
        matches!(event, LobbyEvent::JoinRejected { reason } if reason == "lobby is full")
    }));
}
