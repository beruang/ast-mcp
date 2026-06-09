//! ast-mcp — MCP JSON-RPC 2.0 server over stdin/stdout.
//!
//! Uses line-delimited JSON (one JSON message per line, terminated by `\n`).

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
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Read one line-delimited JSON message from `reader`.
async fn read_message(
    reader: &mut (impl AsyncBufReadExt + std::marker::Unpin),
) -> std::io::Result<Option<Value>> {
    let mut line = String::new();
    let n = reader.read_line(&mut line).await?;
    if n == 0 {
        return Ok(None); // EOF
    }
    let msg: Value = serde_json::from_str(line.trim())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(Some(msg))
}

/// Write a line-delimited JSON message to `writer`.
async fn write_message(
    writer: &mut (impl AsyncWriteExt + std::marker::Unpin),
    msg: &Value,
) -> std::io::Result<()> {
    let body = serde_json::to_string(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    writer.write_all(body.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

/// Build a JSON-RPC 2.0 success response.
fn success(id: &Value, result: Value) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

/// Build a JSON-RPC 2.0 error response.
fn error(id: &Value, code: i64, message: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
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

    let mut reader = BufReader::new(stdin());
    let mut writer = stdout();

    while let Some(request) = read_message(&mut reader).await? {
        let id = match request.get("id") {
            Some(id) => id.clone(),
            None => continue, // notification — no response
        };

        let method = request.get("method").and_then(|v| v.as_str()).unwrap_or("");

        match method {
            "initialize" => {
                let result = serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": {
                        "name": "ast-mcp",
                        "version": "0.2.0"
                    }
                });
                write_message(&mut writer, &success(&id, result)).await?;
            }

            "tools/list" => {
                let tool_list = register_tools_tools(&ctx);
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
                let result = serde_json::json!({ "tools": tools });
                write_message(&mut writer, &success(&id, result)).await?;
            }

            "tools/call" => {
                let name = request
                    .get("params")
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = request
                    .get("params")
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));

                match dispatch(name, arguments, &ctx) {
                    Some(payload) => {
                        let result = serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": payload.to_string()
                            }]
                        });
                        write_message(&mut writer, &success(&id, result)).await?;
                    }
                    None => {
                        let msg = format!("Tool not found: {}", name);
                        write_message(&mut writer, &error(&id, -32602, &msg)).await?;
                    }
                }
            }

            "ping" => {
                write_message(&mut writer, &success(&id, serde_json::json!({}))).await?;
            }

            _ => {
                let msg = format!("Method not found: {}", method);
                write_message(&mut writer, &error(&id, -32601, &msg)).await?;
            }
        }
    }

    Ok(())
}
