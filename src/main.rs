//! ast-mcp entry point — tower-lsp JSON-RPC server.

use std::sync::Arc;

use std::time::Instant;

use ast_mcp::cache::CacheManager;
use ast_mcp::config::runtime_config::RuntimeConfigStore;
use ast_mcp::config::workspace::Workspace;
use ast_mcp::mcp::register_tools::{dispatch, tools as register_tools_tools};
use ast_mcp::mcp::server_context::ServerContext;
use ast_mcp::observability::request_tracker::RequestTracker;
use ast_mcp::scan::ScanRegistry;
use serde_json::Value;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    #[allow(dead_code)]
    client: Client,
    ctx: Arc<ServerContext>,
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
        tracing::info!("AST MCP v5 server initialized");
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }
}

/// Custom method: tools/list — returns all registered tool specs.
async fn tools_list(backend: &Backend, _params: Value) -> LspResult<Value> {
    let tool_list = register_tools_tools(&backend.ctx);
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

    match dispatch(name, arguments, &backend.ctx) {
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
    let runtime_config =
        Arc::new(RuntimeConfigStore::new(workspace.root().to_string_lossy().to_string()));
    let cfg = runtime_config.current();
    let request_tracker = Arc::new(RequestTracker::new(cfg.caches.request_log_max_entries));
    let cache_manager = Arc::new(CacheManager::new(
        cfg.caches.parse_tree_ttl_ms,
        cfg.caches.query_result_ttl_ms,
        cfg.caches.framework_result_ttl_ms,
        cfg.caches.max_cached_files,
    ));

    let scan_registry = Arc::new(ScanRegistry::new());
    let started_at = Instant::now();

    let ctx = Arc::new(ServerContext {
        workspace,
        runtime_config,
        request_tracker,
        cache_manager,
        scan_registry,
        started_at,
    });

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend { client, ctx: Arc::clone(&ctx) })
        .custom_method("tools/list", tools_list)
        .custom_method("tools/call", tools_call)
        .finish();

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
