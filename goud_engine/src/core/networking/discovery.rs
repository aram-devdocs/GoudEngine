//! Session discovery primitives.

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Mutex, OnceLock};

use super::types::SessionDescriptor;

/// Discoverable session entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredSession {
    /// Session metadata.
    pub session: SessionDescriptor,
}

impl DiscoveredSession {
    /// Creates a discovered session entry.
    pub fn new(session: SessionDescriptor) -> Self {
        Self { session }
    }
}

/// Discovery mode supported by the session client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMode {
    /// Direct join using explicit address.
    Direct {
        /// Target join address.
        address: String,
    },
    /// Native LAN discovery provider.
    Lan,
    /// Pluggable directory provider mode.
    Directory,
}

/// Discovery-layer error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryError {
    /// LAN discovery is unavailable on this target.
    LanUnsupported(String),
    /// Directory mode was requested without a configured provider.
    DirectoryProviderUnavailable,
    /// Provider-specific failure.
    ProviderFailure(String),
}

/// Pluggable directory discovery source.
pub trait DirectoryDiscoveryProvider: Send + Sync {
    /// Returns discoverable sessions.
    fn discover_sessions(&self) -> Result<Vec<DiscoveredSession>, DiscoveryError>;
}

/// Pluggable LAN discovery source.
pub trait LanDiscoveryProvider: Send + Sync {
    /// Returns discoverable LAN sessions.
    fn discover_sessions(&self) -> Result<Vec<DiscoveredSession>, DiscoveryError>;
}

/// Native LAN discovery provider.
///
/// On native targets, this uses an in-process best-effort LAN registry for host
/// advertisements. On unsupported targets, it returns an explicit error.
#[derive(Debug, Default, Clone)]
pub struct NativeLanDiscoveryProvider;

impl LanDiscoveryProvider for NativeLanDiscoveryProvider {
    fn discover_sessions(&self) -> Result<Vec<DiscoveredSession>, DiscoveryError> {
        native_lan_discover()
    }
}

/// Discovery service supporting direct, LAN, and pluggable directory modes.
pub struct DiscoveryService {
    lan_provider: Box<dyn LanDiscoveryProvider>,
    directory_provider: Option<Box<dyn DirectoryDiscoveryProvider>>,
}

impl std::fmt::Debug for DiscoveryService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscoveryService")
            .field("has_directory_provider", &self.directory_provider.is_some())
            .finish()
    }
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryService {
    /// Creates a discovery service using the built-in native LAN provider.
    pub fn new() -> Self {
        Self {
            lan_provider: Box::new(NativeLanDiscoveryProvider),
            directory_provider: None,
        }
    }

    /// Overrides the LAN provider.
    pub fn with_lan_provider(mut self, provider: Box<dyn LanDiscoveryProvider>) -> Self {
        self.lan_provider = provider;
        self
    }

    /// Sets a pluggable directory provider.
    pub fn with_directory_provider(
        mut self,
        provider: Box<dyn DirectoryDiscoveryProvider>,
    ) -> Self {
        self.directory_provider = Some(provider);
        self
    }

    /// Discovers sessions via the requested mode.
    pub fn discover(&self, mode: DiscoveryMode) -> Result<Vec<DiscoveredSession>, DiscoveryError> {
        match mode {
            DiscoveryMode::Direct { address } => {
                let session = SessionDescriptor::new(
                    format!("direct:{address}"),
                    format!("Direct {address}"),
                    address,
                );
                Ok(vec![DiscoveredSession::new(session)])
            }
            DiscoveryMode::Lan => self.lan_provider.discover_sessions(),
            DiscoveryMode::Directory => self
                .directory_provider
                .as_ref()
                .ok_or(DiscoveryError::DirectoryProviderUnavailable)?
                .discover_sessions(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct LanRegistryEntry {
    session: SessionDescriptor,
}

#[cfg(not(target_arch = "wasm32"))]
static LAN_REGISTRY: OnceLock<Mutex<HashMap<String, LanRegistryEntry>>> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn lan_registry() -> &'static Mutex<HashMap<String, LanRegistryEntry>> {
    LAN_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Registers or updates a session for native LAN discovery.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn register_native_lan_session(
    session: SessionDescriptor,
) -> Result<(), DiscoveryError> {
    let mut registry = lan_registry()
        .lock()
        .map_err(|e| DiscoveryError::ProviderFailure(format!("LAN registry lock poisoned: {e}")))?;
    registry.insert(session.id.clone(), LanRegistryEntry { session });
    Ok(())
}

/// Removes a session from native LAN discovery.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn unregister_native_lan_session(session_id: &str) -> Result<(), DiscoveryError> {
    let mut registry = lan_registry()
        .lock()
        .map_err(|e| DiscoveryError::ProviderFailure(format!("LAN registry lock poisoned: {e}")))?;
    registry.remove(session_id);
    Ok(())
}

/// Updates the known client count for a registered LAN session.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn update_native_lan_population(
    session_id: &str,
    current_clients: u32,
) -> Result<(), DiscoveryError> {
    let mut registry = lan_registry()
        .lock()
        .map_err(|e| DiscoveryError::ProviderFailure(format!("LAN registry lock poisoned: {e}")))?;
    if let Some(entry) = registry.get_mut(session_id) {
        entry.session.current_clients = current_clients;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn native_lan_discover() -> Result<Vec<DiscoveredSession>, DiscoveryError> {
    let registry = lan_registry()
        .lock()
        .map_err(|e| DiscoveryError::ProviderFailure(format!("LAN registry lock poisoned: {e}")))?;

    let mut sessions: Vec<DiscoveredSession> = registry
        .values()
        .cloned()
        .map(|entry| DiscoveredSession::new(entry.session))
        .collect();

    sessions.sort_by(|left, right| left.session.id.cmp(&right.session.id));
    Ok(sessions)
}

#[cfg(target_arch = "wasm32")]
fn native_lan_discover() -> Result<Vec<DiscoveredSession>, DiscoveryError> {
    Err(DiscoveryError::LanUnsupported(
        "LAN discovery is unsupported on wasm32 targets".to_string(),
    ))
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn register_native_lan_session(
    _session: SessionDescriptor,
) -> Result<(), DiscoveryError> {
    Err(DiscoveryError::LanUnsupported(
        "LAN discovery is unsupported on wasm32 targets".to_string(),
    ))
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn unregister_native_lan_session(_session_id: &str) -> Result<(), DiscoveryError> {
    Err(DiscoveryError::LanUnsupported(
        "LAN discovery is unsupported on wasm32 targets".to_string(),
    ))
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn update_native_lan_population(
    _session_id: &str,
    _current_clients: u32,
) -> Result<(), DiscoveryError> {
    Err(DiscoveryError::LanUnsupported(
        "LAN discovery is unsupported on wasm32 targets".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubDirectoryProvider {
        sessions: Vec<DiscoveredSession>,
    }

    impl DirectoryDiscoveryProvider for StubDirectoryProvider {
        fn discover_sessions(&self) -> Result<Vec<DiscoveredSession>, DiscoveryError> {
            Ok(self.sessions.clone())
        }
    }

    #[test]
    fn direct_mode_returns_single_address() {
        let service = DiscoveryService::new();
        let sessions = service
            .discover(DiscoveryMode::Direct {
                address: "127.0.0.1:9000".to_string(),
            })
            .unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session.address, "127.0.0.1:9000");
    }

    #[test]
    fn directory_mode_requires_provider() {
        let service = DiscoveryService::new();
        let result = service.discover(DiscoveryMode::Directory);
        assert_eq!(result, Err(DiscoveryError::DirectoryProviderUnavailable));
    }

    #[test]
    fn directory_mode_uses_pluggable_provider() {
        let session = SessionDescriptor::new("abc", "Test", "10.0.0.1:7000");
        let service =
            DiscoveryService::new().with_directory_provider(Box::new(StubDirectoryProvider {
                sessions: vec![DiscoveredSession::new(session.clone())],
            }));

        let sessions = service.discover(DiscoveryMode::Directory).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session, session);
    }
}
