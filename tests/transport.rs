//! Integration test: spawn the binary, send `tools/list` request via MCP line-delimited JSON.
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};

fn send_message(stdin: &mut dyn Write, msg: &serde_json::Value) {
    let body = serde_json::to_string(msg).unwrap();
    stdin.write_all(body.as_bytes()).unwrap();
    stdin.write_all(b"\n").unwrap();
    stdin.flush().unwrap();
}

fn read_message(reader: &mut BufReader<impl Read>) -> Option<serde_json::Value> {
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    if line.trim().is_empty() {
        return None;
    }
    serde_json::from_str(line.trim()).ok()
}

#[test]
fn tools_list_returns_health_check() {
    let mut child = Command::new("./target/debug/ast-mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn ast-mcp");

    let mut stdin = child.stdin.take().expect("stdin not captured");
    let stdout = child.stdout.take().expect("stdout not captured");
    let mut reader = BufReader::new(stdout);

    // 1. Send initialize
    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "capabilities": {} }
        }),
    );

    // Read initialize response
    let init_resp = read_message(&mut reader).expect("expected initialize response");
    eprintln!("init response: {}", serde_json::to_string_pretty(&init_resp).unwrap());
    assert!(init_resp.get("result").is_some(), "initialize should succeed, got: {}", init_resp);

    // 2. Send initialized notification
    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }),
    );

    // 3. Send tools/list
    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
    );

    // Read tools/list response
    let list_resp = read_message(&mut reader).expect("expected tools/list response");
    eprintln!("tools/list response: {}", serde_json::to_string_pretty(&list_resp).unwrap());
    let tools = list_resp
        .pointer("/result/tools")
        .and_then(|t| t.as_array())
        .expect("tools array in result");
    let has_health_check =
        tools.iter().any(|t| t.get("name").and_then(|n| n.as_str()) == Some("ast_health_check"));
    assert!(has_health_check, "ast_health_check must be in tools list");

    drop(stdin);
    child.wait().unwrap();
}
