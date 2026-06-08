//! ast-mcp entry point.

use ast_mcp::config::workspace::Workspace;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize workspace — validates WORKSPACE_PATH or CWD.
    let _workspace = Workspace::from_env()?;

    ast_mcp::mcp::transport::run().await
}
