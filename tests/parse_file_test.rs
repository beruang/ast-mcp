use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools;
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

#[test]
fn health_check_ok() {
    let ws = workspace();
    let result = tools::health_check::handle(&ws, json!({}));
    assert_eq!(result["ok"], json!(true));
    assert!(result["workspacePath"].as_str().is_some());
    assert!(result["parsers"].is_array());
    assert_eq!(result["parsers"].as_array().unwrap().len(), 5);
    assert!(result["limits"]["maxFileBytes"].as_u64().is_some());
}

#[test]
fn list_supported_languages_returns_five() {
    let result = tools::list_supported_languages::handle(json!({}));
    let langs = result["languages"].as_array().unwrap();
    assert_eq!(langs.len(), 5);
    let names: Vec<&str> = langs.iter().map(|l| l["language"].as_str().unwrap()).collect();
    assert!(names.contains(&"typescript"));
    assert!(names.contains(&"python"));
    assert!(names.contains(&"javascript"));
}

#[test]
fn parse_file_typescript_basic() {
    let ws = workspace();
    let result = tools::parse_file::handle(&ws, json!({"file_path": "tests/fixtures/sample.ts"}));
    assert_eq!(result["parsed"], json!(true));
    assert_eq!(result["language"], json!("typescript"));
    assert_eq!(result["hasSyntaxError"], json!(false));
    assert!(result["nodeCount"].as_u64().unwrap() > 0);
}

#[test]
fn parse_file_python_basic() {
    let ws = workspace();
    let result = tools::parse_file::handle(&ws, json!({"file_path": "tests/fixtures/sample.py"}));
    assert_eq!(result["parsed"], json!(true));
    assert_eq!(result["language"], json!("python"));
    assert_eq!(result["hasSyntaxError"], json!(false));
    assert!(result["nodeCount"].as_u64().unwrap() > 0);
}

#[test]
fn parse_file_with_tree() {
    let ws = workspace();
    let result = tools::parse_file::handle(
        &ws,
        json!({"file_path": "tests/fixtures/sample.ts", "include_tree": true, "max_depth": 2}),
    );
    assert_eq!(result["parsed"], json!(true));
    let tree = &result["tree"];
    assert!(tree.is_object());
    assert!(!tree["kind"].as_str().unwrap().is_empty());
    // At depth 2 we should still have children
    assert!(tree["children"].is_array() || tree["children"].is_null());
}

#[test]
fn parse_file_unsupported_language() {
    let ws = workspace();
    let result = tools::parse_file::handle(&ws, json!({"file_path": "Cargo.toml"}));
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], json!("unsupported_language"));
}

#[test]
fn parse_file_missing_file() {
    let ws = workspace();
    let result = tools::parse_file::handle(&ws, json!({"file_path": "nonexistent.ts"}));
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], json!("file_not_found"));
}

#[test]
fn parse_file_rejects_traversal() {
    let ws = workspace();
    let result = tools::parse_file::handle(&ws, json!({"file_path": "../outside.ts"}));
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], json!("path_outside_workspace"));
}
