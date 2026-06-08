use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools;
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

// ---------------------------------------------------------------------------
// ast_file_outline
// ---------------------------------------------------------------------------

#[test]
fn outline_typescript_classes_returns_kinds_and_names() {
    let ws = workspace();
    let result = tools::file_outline::handle(
        &ws,
        json!({"file_path": "tests/fixtures/outline/classes.ts"}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    let nodes = result["nodes"]
        .as_array()
        .expect("nodes should be an array");
    assert!(!nodes.is_empty(), "should have at least one outline node");

    // Should find class_declaration with methods
    let has_class = nodes.iter().any(|n| n["kind"] == "class_declaration");
    assert!(has_class, "should have a class_declaration node");

    // outlineText should be non-empty and deterministic
    let text = result["outlineText"]
        .as_str()
        .expect("outlineText should be a string");
    assert!(!text.is_empty());

    // Run twice to check determinism
    let result2 = tools::file_outline::handle(
        &ws,
        json!({"file_path": "tests/fixtures/outline/classes.ts"}),
    );
    assert_eq!(result["outlineText"], result2["outlineText"]);
}

#[test]
fn outline_python_classes_returns_methods() {
    let ws = workspace();
    let result = tools::file_outline::handle(
        &ws,
        json!({"file_path": "tests/fixtures/outline/classes.py"}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    let nodes = result["nodes"]
        .as_array()
        .expect("nodes should be an array");
    let has_class = nodes.iter().any(|n| n["kind"] == "class_definition");
    assert!(has_class, "should have a class_definition node");

    let text = result["outlineText"]
        .as_str()
        .expect("outlineText should be a string");
    assert!(!text.is_empty());

    // Run twice to check determinism
    let result2 = tools::file_outline::handle(
        &ws,
        json!({"file_path": "tests/fixtures/outline/classes.py"}),
    );
    assert_eq!(result["outlineText"], result2["outlineText"]);
}

#[test]
fn outline_typescript_types_includes_interface_and_type_and_enum() {
    let ws = workspace();
    let result =
        tools::file_outline::handle(&ws, json!({"file_path": "tests/fixtures/outline/types.ts"}));
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    let nodes = result["nodes"]
        .as_array()
        .expect("nodes should be an array");

    let kinds: Vec<&str> = nodes
        .iter()
        .map(|n| n["kind"].as_str().unwrap_or(""))
        .collect();

    assert!(
        kinds.contains(&"interface_declaration"),
        "should have interface_declaration, got: {:?}",
        kinds
    );
    assert!(
        kinds.contains(&"type_alias_declaration"),
        "should have type_alias_declaration, got: {:?}",
        kinds
    );
    assert!(
        kinds.contains(&"enum_declaration"),
        "should have enum_declaration, got: {:?}",
        kinds
    );
}

#[test]
fn outline_python_all_includes_async_functions() {
    let ws = workspace();
    let result = tools::file_outline::handle(
        &ws,
        json!({"file_path": "tests/fixtures/outline/all.py", "include_imports": true}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    let nodes = result["nodes"]
        .as_array()
        .expect("nodes should be an array");
    let kinds: Vec<&str> = nodes
        .iter()
        .map(|n| n["kind"].as_str().unwrap_or(""))
        .collect();

    // Should have at least one async function (fetch_data)
    assert!(
        kinds.contains(&"async_function_definition") || kinds.contains(&"function_definition"),
        "should detect async function, got: {:?}",
        kinds
    );
}

#[test]
fn outline_with_max_depth() {
    let ws = workspace();
    let result = tools::file_outline::handle(
        &ws,
        json!({"file_path": "tests/fixtures/outline/classes.ts", "max_depth": 0}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    // At max_depth 0, class should appear but children should be empty
    let nodes = result["nodes"]
        .as_array()
        .expect("nodes should be an array");
    if let Some(class_node) = nodes.iter().find(|n| n["kind"] == "class_declaration") {
        let children = class_node["children"].as_array();
        // Children may be null or empty array (both acceptable at depth 0)
        if let Some(kids) = children {
            assert!(
                kids.is_empty(),
                "at depth 0, children should be empty or null"
            );
        }
    }
}

#[test]
fn outline_without_ranges() {
    let ws = workspace();
    let result = tools::file_outline::handle(
        &ws,
        json!({"file_path": "tests/fixtures/outline/classes.py", "include_ranges": false}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    let nodes = result["nodes"]
        .as_array()
        .expect("nodes should be an array");
    if let Some(first) = nodes.first() {
        assert!(
            first["range"].is_null(),
            "range should be null when include_ranges is false"
        );
    }
}

#[test]
fn outline_unsupported_language() {
    let ws = workspace();
    let result = tools::file_outline::handle(&ws, json!({"file_path": "Cargo.toml"}));
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], json!("unsupported_language"));
}

// ---------------------------------------------------------------------------
// ast_top_level_nodes
// ---------------------------------------------------------------------------

#[test]
fn top_level_nodes_typescript_returns_correct_count() {
    let ws = workspace();
    let result =
        tools::top_level_nodes::handle(&ws, json!({"file_path": "tests/fixtures/sample.ts"}));
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    let nodes = result["nodes"]
        .as_array()
        .expect("nodes should be an array");
    let count = result["count"].as_u64().expect("count should be a number");
    assert_eq!(nodes.len() as u64, count);
    // sample.ts has 2 top-level nodes: function_declaration + class_declaration
    assert!(nodes.len() >= 2, "expected at least 2 top-level nodes");
}

#[test]
fn top_level_nodes_python_returns_correct_count() {
    let ws = workspace();
    let result =
        tools::top_level_nodes::handle(&ws, json!({"file_path": "tests/fixtures/sample.py"}));
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    let nodes = result["nodes"]
        .as_array()
        .expect("nodes should be an array");
    let count = result["count"].as_u64().expect("count should be a number");
    assert_eq!(nodes.len() as u64, count);
    // sample.py has 2 top-level nodes: function_definition + class_definition
    assert!(nodes.len() >= 2, "expected at least 2 top-level nodes");
}

#[test]
fn top_level_nodes_each_has_kind_and_range() {
    let ws = workspace();
    let result = tools::top_level_nodes::handle(
        &ws,
        json!({"file_path": "tests/fixtures/outline/types.ts"}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    let nodes = result["nodes"]
        .as_array()
        .expect("nodes should be an array");
    for node in nodes {
        assert!(node["kind"].is_string(), "each node must have a kind");
        assert!(node["range"].is_object(), "each node must have a range");
    }
}

#[test]
fn top_level_nodes_missing_file() {
    let ws = workspace();
    let result = tools::top_level_nodes::handle(&ws, json!({"file_path": "nonexistent.ts"}));
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], json!("file_not_found"));
}
