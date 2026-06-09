//! ast-mcp entry point — tower-lsp JSON-RPC server.

use std::sync::Arc;

use ast_mcp::config::workspace::Workspace;
use ast_mcp::mcp::register_tools::{dispatch, tools as register_tools_tools};
use serde_json::Value;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    #[allow(dead_code)]
    client: Client,
    workspace: Arc<Workspace>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities::default(),
            server_info: Some(ServerInfo {
                name: "ast-mcp".to_string(),
                version: Some("0.2.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("AST MCP v4 server initialized");
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }
}

/// Custom method: tools/list — returns all registered tool specs.
async fn tools_list(backend: &Backend, _params: Value) -> LspResult<Value> {
    let tool_list = register_tools_tools(&backend.workspace);
    let tools: Vec<Value> = tool_list
        .into_iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema
            })
        })
        .collect();
    Ok(serde_json::json!({ "tools": tools }))
}

/// Custom method: tools/call — dispatch a tool call by name.
async fn tools_call(backend: &Backend, params: Value) -> LspResult<Value> {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or_else(|| serde_json::json!({}));

    match dispatch(name, arguments, &backend.workspace) {
        Some(payload) => Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": payload.to_string()
            }]
        })),
        None => Err(tower_lsp::jsonrpc::Error::method_not_found()),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let workspace = Arc::new(Workspace::from_env()?);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) =
        LspService::build(|client| Backend { client, workspace: Arc::clone(&workspace) })
            .custom_method("tools/list", tools_list)
            .custom_method("tools/call", tools_call)
            .finish();

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
