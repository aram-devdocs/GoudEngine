use std::io::{self, Read, Write};
use std::path::Path;

use goud_engine::core::debugger::{
    AttachAcceptedV1, AttachHelloV1, LocalEndpointV1, RouteSummaryV1, RuntimeManifestV1,
};
#[cfg(windows)]
use interprocess::local_socket::GenericNamespaced;
use interprocess::local_socket::{prelude::*, GenericFilePath, Stream};
use serde_json::Value;
use thiserror::Error;

const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub struct AttachedRoute {
    pub manifest: RuntimeManifestV1,
    pub route: RouteSummaryV1,
    pub accepted: AttachAcceptedV1,
    pub client: AttachClient,
}

#[derive(Debug)]
pub struct AttachClient {
    stream: Stream,
}

#[derive(Debug, Error)]
pub enum AttachError {
    #[error("attach I/O failed: {0}")]
    Io(#[from] io::Error),
    #[error("attach JSON failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("attach handshake failed: {0}")]
    Handshake(String),
    #[error("attach protocol failed: {0}")]
    Protocol(String),
    #[error("debugger request failed ({code}): {message}")]
    Debugger {
        code: String,
        message: String,
        data: Option<Value>,
    },
}

impl AttachClient {
    pub fn connect(
        manifest: RuntimeManifestV1,
        route: RouteSummaryV1,
    ) -> Result<AttachedRoute, AttachError> {
        let mut client = Self {
            stream: connect_stream(&manifest.endpoint)?,
        };
        let hello = AttachHelloV1 {
            protocol_version: 1,
            client_name: "goudengine-mcp".to_string(),
            client_pid: std::process::id(),
            route_id: route.route_id.clone(),
        };
        client.write_json(&serde_json::to_value(&hello)?)?;

        let response = client.read_json()?;
        if response.get("ok").and_then(Value::as_bool) == Some(false) {
            let message = response
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("attach hello rejected");
            return Err(AttachError::Handshake(message.to_string()));
        }

        let accepted: AttachAcceptedV1 = serde_json::from_value(response)?;
        Ok(AttachedRoute {
            manifest,
            route,
            accepted,
            client,
        })
    }

    pub fn request(&mut self, request: &Value) -> Result<Value, AttachError> {
        self.write_json(request)?;
        let response = self.read_json()?;
        if response.get("ok").and_then(Value::as_bool) == Some(true) {
            return Ok(response.get("result").cloned().unwrap_or(Value::Null));
        }

        let error = response
            .get("error")
            .ok_or_else(|| AttachError::Protocol("missing debugger error payload".to_string()))?;
        let code = error
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("unknown_error")
            .to_string();
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("debugger request failed")
            .to_string();
        Err(AttachError::Debugger {
            code,
            message,
            data: response.get("error").cloned(),
        })
    }

    fn write_json(&mut self, value: &Value) -> Result<(), AttachError> {
        let payload = serde_json::to_vec(value)?;
        if payload.len() > MAX_FRAME_BYTES {
            return Err(AttachError::Protocol(
                "attach frame exceeds maximum size".to_string(),
            ));
        }

        let len = u32::try_from(payload.len())
            .map_err(|_| AttachError::Protocol("attach frame length overflow".to_string()))?;
        self.stream.write_all(&len.to_le_bytes())?;
        self.stream.write_all(&payload)?;
        Ok(())
    }

    fn read_json(&mut self) -> Result<Value, AttachError> {
        let mut len_buf = [0_u8; 4];
        self.stream.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;
        if len > MAX_FRAME_BYTES {
            return Err(AttachError::Protocol(
                "attach frame exceeds maximum size".to_string(),
            ));
        }
        let mut payload = vec![0_u8; len];
        self.stream.read_exact(&mut payload)?;
        Ok(serde_json::from_slice(&payload)?)
    }
}

#[cfg(not(windows))]
fn connect_stream(endpoint: &LocalEndpointV1) -> io::Result<Stream> {
    if endpoint.transport != "unix" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported local transport: {}", endpoint.transport),
        ));
    }
    let name = Path::new(&endpoint.location).to_fs_name::<GenericFilePath>()?;
    Stream::connect(name)
}

#[cfg(windows)]
fn connect_stream(endpoint: &LocalEndpointV1) -> io::Result<Stream> {
    if endpoint.transport != "named_pipe" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported local transport: {}", endpoint.transport),
        ));
    }
    let pipe_name = endpoint
        .location
        .strip_prefix(r"\\.\pipe\")
        .unwrap_or(&endpoint.location);
    let name = pipe_name.to_ns_name::<GenericNamespaced>()?;
    Stream::connect(name)
}
