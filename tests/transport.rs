//! Integration test: spawn the binary, send a `tools/list` request, assert valid JSON.
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[tokio::test]
async fn tools_list_returns_health_check() {
    let mut child = Command::new("./target/debug/ast-mcp")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn ast-mcp");

    let mut stdin = child.stdin.take().expect("stdin not captured");

    // Send initialize first (required by JSON-RPC handshake)
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    stdin.write_all(serde_json::to_string(&init_req).unwrap().as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();

    // Send tools/list
    let list_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    stdin.write_all(serde_json::to_string(&list_req).unwrap().as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    drop(stdin);

    // Read response
    let output = child.wait_with_output().await.unwrap();
    let response_text = String::from_utf8(output.stdout).unwrap();
    let first_line = response_text.lines().last().unwrap();
    let resp: serde_json::Value = serde_json::from_str(first_line).expect("valid JSON");

    // Assert tools list contains ast_health_check
    let tools =
        resp.pointer("/result/tools").and_then(|t| t.as_array()).expect("tools array in result");
    let has_health_check =
        tools.iter().any(|t| t.get("name").and_then(|n| n.as_str()) == Some("ast_health_check"));
    assert!(has_health_check, "ast_health_check must be in tools list");
}
