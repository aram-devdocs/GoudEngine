use anyhow::Result;
use goudengine_mcp::ws_relay;
use goudengine_mcp::GoudEngineMcpServer;
use rmcp::{transport::io, ServiceExt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let server = GoudEngineMcpServer::new();

    // Spawn the WebSocket relay for browser-based debugger connections.
    let ws_port = std::env::var("GOUDENGINE_WS_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(ws_relay::DEFAULT_WS_PORT);
    if let Err(e) = ws_relay::start_ws_relay(server.ws_relay().clone(), ws_port).await {
        eprintln!("[mcp] ws relay failed to start: {e}");
    }

    let service = server.serve(io::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
