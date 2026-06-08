//! Stdio JSON-RPC transport: read requests, dispatch, write responses.
use crate::mcp::register_tools::{dispatch, tools as register_tools_tools};
use crate::mcp::responses::{error_envelope, success_envelope, text_envelope};
use serde::Deserialize;
use serde_json::Value;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};

/// JSON-RPC request envelope we parse from stdin.
#[derive(Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

/// Write a JSON-RPC response line to stdout, flushed.
fn write_response(resp: Value) -> io::Result<()> {
    let line = serde_json::to_string(&resp).unwrap();
    println!("{}", line);
    // Explicitly flush after each response.
    io::stdout().flush()
}

/// Read lines from stdin, dispatch, write responses.
/// Exits when stdin closes.
pub async fn run() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        // Ignore empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse JSON-RPC request
        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let err = error_envelope(-32700, &format!("Parse error: {}", e));
                write_response(err)?;
                continue;
            }
        };

        // Dispatch
        let resp = match req.method.as_str() {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "ast-mcp",
                        "version": "0.1.0"
                    },
                    "capabilities": {
                        "tools": {}
                    }
                });
                success_envelope(result, req.id)
            }

            "tools/list" => {
                let tool_list = register_tools_tools();
                let tools: Vec<Value> = tool_list
                    .into_iter()
                    .map(|t| {
                        json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": t.input_schema
                        })
                    })
                    .collect();
                let result = json!({ "tools": tools });
                success_envelope(result, req.id)
            }

            "tools/call" => {
                let name = req
                    .params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = req.params.get("arguments").cloned().unwrap_or(json!({}));

                match dispatch(name, arguments) {
                    Some(payload) => success_envelope(text_envelope(payload), req.id),
                    None => {
                        let err = error_envelope(-32602, &format!("Tool not found: {}", name));
                        // Attach id to error response
                        let mut err_obj = err;
                        if let Some(id) = req
                            .id
                            .as_i64()
                            .or_else(|| req.id.as_str().and_then(|s| s.parse().ok()))
                        {
                            if let Some(obj) = err_obj.as_object_mut() {
                                obj.insert("id".to_string(), serde_json::json!(id));
                            }
                        }
                        err_obj
                    }
                }
            }

            // JSON-RPC 2.0: notifications have no id and require no response.
            "notifications/initialized" => {
                // No response for notifications.
                continue;
            }

            _ => {
                let err = error_envelope(-32601, &format!("Method not found: {}", req.method));
                let mut err_obj = err;
                if let Some(id) = req
                    .id
                    .as_i64()
                    .or_else(|| req.id.as_str().and_then(|s| s.parse().ok()))
                {
                    if let Some(obj) = err_obj.as_object_mut() {
                        obj.insert("id".to_string(), serde_json::json!(id));
                    }
                }
                err_obj
            }
        };

        write_response(resp)?;
    }

    Ok(())
}
