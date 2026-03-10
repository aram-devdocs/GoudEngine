use crate::core::error::{set_last_error, GoudError, ERR_INVALID_STATE};
use crate::core::providers::network::NetworkProvider;
use crate::core::providers::ProviderLifecycle;
#[cfg(any(debug_assertions, test))]
use crate::libs::providers::impls::NetworkSimProvider;
#[cfg(feature = "net-tcp")]
use crate::libs::providers::impls::TcpNetProvider;
#[cfg(feature = "net-udp")]
use crate::libs::providers::impls::UdpNetProvider;
#[cfg(feature = "net-ws")]
use crate::libs::providers::impls::WsNetProvider;

/// Protocol selector passed from FFI callers.
pub(super) const PROTOCOL_UDP: i32 = 0;
pub(super) const PROTOCOL_WS: i32 = 1;
pub(super) const PROTOCOL_TCP: i32 = 2;

pub(super) fn create_provider(protocol: i32) -> Result<Box<dyn NetworkProvider>, i32> {
    match protocol {
        PROTOCOL_UDP => create_udp_provider(),
        PROTOCOL_WS => create_ws_provider(),
        PROTOCOL_TCP => create_tcp_provider(),
        _ => {
            set_last_error(GoudError::InvalidState(format!(
                "Unknown protocol: {}",
                protocol
            )));
            Err(ERR_INVALID_STATE)
        }
    }
}

#[cfg(any(debug_assertions, test))]
fn init_provider<P>(provider: P) -> Result<Box<dyn NetworkProvider>, i32>
where
    P: NetworkProvider + 'static,
{
    let mut provider = NetworkSimProvider::new(provider);
    ProviderLifecycle::init(&mut provider).map_err(|e| {
        let code = e.error_code();
        set_last_error(e);
        code
    })?;
    Ok(Box::new(provider))
}

#[cfg(not(any(debug_assertions, test)))]
fn init_provider<P>(provider: P) -> Result<Box<dyn NetworkProvider>, i32>
where
    P: NetworkProvider + 'static,
{
    let mut provider = provider;
    ProviderLifecycle::init(&mut provider).map_err(|e| {
        let code = e.error_code();
        set_last_error(e);
        code
    })?;
    Ok(Box::new(provider))
}

#[cfg(feature = "net-udp")]
fn create_udp_provider() -> Result<Box<dyn NetworkProvider>, i32> {
    init_provider(UdpNetProvider::new())
}

#[cfg(not(feature = "net-udp"))]
fn create_udp_provider() -> Result<Box<dyn NetworkProvider>, i32> {
    set_last_error(GoudError::InvalidState(
        "UDP networking not available (net-udp feature disabled)".to_string(),
    ));
    Err(ERR_INVALID_STATE)
}

#[cfg(feature = "net-ws")]
fn create_ws_provider() -> Result<Box<dyn NetworkProvider>, i32> {
    init_provider(WsNetProvider::new())
}

#[cfg(feature = "net-tcp")]
fn create_tcp_provider() -> Result<Box<dyn NetworkProvider>, i32> {
    init_provider(TcpNetProvider::new())
}

#[cfg(not(feature = "net-tcp"))]
fn create_tcp_provider() -> Result<Box<dyn NetworkProvider>, i32> {
    set_last_error(GoudError::InvalidState(
        "TCP networking not available (net-tcp feature disabled)".to_string(),
    ));
    Err(ERR_INVALID_STATE)
}

#[cfg(not(feature = "net-ws"))]
fn create_ws_provider() -> Result<Box<dyn NetworkProvider>, i32> {
    set_last_error(GoudError::InvalidState(
        "WebSocket networking not available (net-ws feature disabled)".to_string(),
    ));
    Err(ERR_INVALID_STATE)
}
