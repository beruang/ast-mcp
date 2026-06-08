//! ast-mcp entry point.
#[macro_use]
extern crate serde_json;

mod mcp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    mcp::transport::run().await
}
