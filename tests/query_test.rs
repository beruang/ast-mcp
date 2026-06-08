use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools;
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

#[test]
fn query_ts_function_declarations() {
    let ws = workspace();
    let result = tools::query::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/query/functions.ts",
            "query": "(function_declaration name: (identifier) @func.name) @func.def"
        }),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let matches = result["matches"].as_array().unwrap();
    assert!(
        matches.len() >= 2,
        "expected at least 2 function matches, got {}",
        matches.len()
    );
    // Each match should have captures
    for m in matches {
        assert!(m["captures"].is_array());
    }
}

#[test]
fn query_python_function_definitions() {
    let ws = workspace();
    let result = tools::query::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/query/functions.py",
            "query": "(function_definition name: (identifier) @func.name) @func.def"
        }),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let matches = result["matches"].as_array().unwrap();
    assert!(
        matches.len() >= 2,
        "expected at least 2 function matches, got {}",
        matches.len()
    );
    for m in matches {
        assert!(m["captures"].is_array());
    }
}

#[test]
fn query_invalid_syntax() {
    let ws = workspace();
    let result = tools::query::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/query/functions.ts",
            "query": "((((("
        }),
    );
    assert!(
        result["error"].is_object(),
        "expected error object, got: {:?}",
        result
    );
    assert_eq!(
        result["error"]["code"],
        json!("query_invalid"),
        "expected query_invalid, got: {:?}",
        result
    );
    // Should have details
    assert!(result["error"]["details"].is_object());
    assert!(result["error"]["details"]["row"].is_number());
}

#[test]
fn query_empty_returns_error() {
    let ws = workspace();
    let result = tools::query::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/query/functions.ts",
            "query": ""
        }),
    );
    assert!(
        result["error"].is_object(),
        "expected error object, got: {:?}",
        result
    );
    assert_eq!(result["error"]["code"], json!("query_invalid"));
}

#[test]
fn query_without_node_text() {
    let ws = workspace();
    let result = tools::query::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/query/functions.ts",
            "query": "(function_declaration name: (identifier) @func.name) @func.def",
            "include_node_text": false
        }),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let matches = result["matches"].as_array().unwrap();
    assert!(matches.len() >= 2);
    // Check that captures don't have text
    for m in matches {
        for c in m["captures"].as_array().unwrap() {
            assert!(c.get("text").is_none_or(|v| v.is_null()));
        }
    }
}

#[test]
fn query_returns_metadata() {
    let ws = workspace();
    let result = tools::query::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/query/functions.ts",
            "query": "(function_declaration name: (identifier) @func.name) @func.def"
        }),
    );
    assert!(result["error"].is_null());
    assert_eq!(
        result["filePath"],
        json!("tests/fixtures/query/functions.ts")
    );
    assert_eq!(result["language"], json!("typescript"));
    assert!(!result["query"].as_str().unwrap().is_empty());
    assert!(result["returnedCount"].as_u64().is_some());
    assert!(result["truncated"].as_bool().is_some());
    assert!(result["parseTimeMs"].as_u64().is_some());
    assert!(result["hasSyntaxError"].as_bool().is_some());
}

#[test]
fn query_missing_file() {
    let ws = workspace();
    let result = tools::query::handle(
        &ws,
        json!({
            "file_path": "nonexistent.ts",
            "query": "(function_declaration) @f"
        }),
    );
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], json!("file_not_found"));
}

#[test]
fn query_unsupported_language() {
    let ws = workspace();
    let result = tools::query::handle(
        &ws,
        json!({
            "file_path": "Cargo.toml",
            "query": "(function_declaration) @f"
        }),
    );
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], json!("unsupported_language"));
}
