//! Lobby and matchmaking helpers layered on top of session networking.

mod client;
mod server;
mod types;

pub use client::LobbyClient;
pub use server::LobbyServer;
pub use types::{LobbyCommand, LobbyConfig, LobbyEvent, LobbyMember, LobbyState, LobbyVisibility};
