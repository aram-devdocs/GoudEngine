use anyhow::Result;
use goudengine_mcp::GoudEngineMcpServer;
use rmcp::{transport::io, ServiceExt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let server = GoudEngineMcpServer::new();
    let service = server.serve(io::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
