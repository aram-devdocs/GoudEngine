//! Client-server networking session orchestration built on top of transport providers.
//!
//! This module sits above [`crate::core::providers::network::NetworkProvider`] and
//! provides session-level behavior:
//!
//! - Hosting and joining client-server sessions
//! - Discovery modes (direct, LAN, directory-provider)
//! - Pluggable server authority policies for command validation
//! - Authoritative state broadcast flow with explicit join/leave/update events

pub mod authority;
pub mod client;
pub mod discovery;
pub mod protocol;
pub mod server;
pub mod types;

#[cfg(test)]
mod tests;

pub use authority::{
    AllowAllAuthority, AuthorityDecision, AuthorityPolicy, BuiltInAuthorityPolicy, RateLimitConfig,
    RateLimitedAuthority, SchemaBoundsAuthority, SchemaBoundsConfig, ValidationContext,
};
pub use client::SessionClient;
pub use discovery::{
    DirectoryDiscoveryProvider, DiscoveredSession, DiscoveryError, DiscoveryMode, DiscoveryService,
    LanDiscoveryProvider, NativeLanDiscoveryProvider,
};
pub use server::SessionServer;
pub use types::{ClientEvent, ServerConfig, ServerEvent, SessionChannels, SessionDescriptor};
