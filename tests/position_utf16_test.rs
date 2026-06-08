use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools;
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

#[test]
fn parse_emoji_ts_returns_utf16_range() {
    let ws = workspace();
    let result = tools::parse_file::handle(
        &ws,
        json!({"file_path": "tests/fixtures/utf16/emoji.ts", "include_tree": true, "include_node_text": true}),
    );
    assert_eq!(result["parsed"], json!(true));
    let tree = &result["tree"];
    // Find a string node containing the emoji
    let string_node = find_string_node(tree);
    assert!(string_node.is_some(), "Should find a string node");
    let node = string_node.unwrap();
    let range = &node["range"];
    let start = &range["start"];
    // "const greeting = \"" = 17 BMP chars before the opening quote
    assert_eq!(start["line"], json!(0));
    assert_eq!(start["character"], json!(17));
}

#[test]
fn parse_emoji_py_returns_utf16_range() {
    let ws = workspace();
    let result = tools::parse_file::handle(
        &ws,
        json!({"file_path": "tests/fixtures/utf16/emoji.py", "include_tree": true, "include_node_text": true}),
    );
    assert_eq!(result["parsed"], json!(true));
    let tree = &result["tree"];
    let string_node = find_string_node(tree);
    assert!(string_node.is_some(), "Should find a string node");
    let node = string_node.unwrap();
    let range = &node["range"];
    let start = &range["start"];
    // "GREETING = \"" = 11 BMP chars before the opening quote
    assert_eq!(start["line"], json!(0));
    assert_eq!(start["character"], json!(11));
}

/// Recursively search for a "string" node with kind containing "string".
fn find_string_node(tree: &serde_json::Value) -> Option<&serde_json::Value> {
    if tree["kind"].as_str().is_some_and(|k| k.contains("string")) {
        return Some(tree);
    }
    if let Some(children) = tree["children"].as_array() {
        for child in children {
            if let Some(found) = find_string_node(child) {
                return Some(found);
            }
        }
    }
    None
}
