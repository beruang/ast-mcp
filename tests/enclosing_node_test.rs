use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools;
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

// ---------------------------------------------------------------------------
// ast_enclosing_node — basic
// ---------------------------------------------------------------------------

#[test]
fn enclosing_node_inside_if_returns_if_statement() {
    let ws = workspace();
    // The if statement in nested.ts starts at line 5.  Position inside the
    // `if` body (line 6, char 12) should report an ancestor of kind
    // `if_statement`.
    let result = tools::enclosing_node::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/enclosing/nested.ts",
            "line": 5,
            "character": 8
        }),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    let ancestors = result["ancestors"].as_array().expect("ancestors should be an array");
    let kinds: Vec<&str> = ancestors.iter().map(|a| a["kind"].as_str().unwrap_or("")).collect();

    // Should find if_statement somewhere in the ancestor chain.
    assert!(
        kinds.contains(&"if_statement"),
        "expected if_statement in ancestor chain, got: {:?}",
        kinds
    );
}

#[test]
fn enclosing_node_kinds_filter_returns_only_class() {
    let ws = workspace();
    // Position inside class body (line 5, char 8).
    // With kind filter `["class_declaration"]`, only the class should appear.
    let result = tools::enclosing_node::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/enclosing/nested.ts",
            "line": 5,
            "character": 8,
            "kinds": ["class_declaration"]
        }),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    let ancestors = result["ancestors"].as_array().expect("ancestors should be an array");
    let kinds: Vec<&str> = ancestors.iter().map(|a| a["kind"].as_str().unwrap_or("")).collect();

    // With the kind filter, all results should be class_declaration.
    for k in &kinds {
        assert_eq!(*k, "class_declaration", "all filtered ancestors should be class_declaration");
    }
}

#[test]
fn enclosing_node_out_of_bounds_returns_error() {
    let ws = workspace();
    let result = tools::enclosing_node::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/enclosing/nested.ts",
            "line": 999,
            "character": 0
        }),
    );
    assert!(result["error"].is_object(), "expected error for OOB position");
    assert_eq!(result["error"]["code"], json!("invalid_position"));
}

#[test]
fn enclosing_node_returns_ancestors_outermost_first() {
    let ws = workspace();
    // Position deep inside class method (line 5, char 12 — inside `this.value += n;`).
    let result = tools::enclosing_node::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/enclosing/nested.ts",
            "line": 5,
            "character": 12
        }),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);

    let ancestors = result["ancestors"].as_array().expect("ancestors should be an array");
    assert!(!ancestors.is_empty(), "should have ancestors at a valid position");

    // The root (program) should be first (outermost).
    let first_kind = ancestors.first().unwrap()["kind"].as_str().unwrap();
    assert_eq!(first_kind, "program", "first ancestor should be the root program");

    // Last ancestor should be the deepest node covering the position.
    let last_kind = ancestors.last().unwrap()["kind"].as_str().unwrap();
    assert!(!last_kind.is_empty(), "last (innermost) ancestor should have a kind");
}

#[test]
fn enclosing_node_missing_file() {
    let ws = workspace();
    let result = tools::enclosing_node::handle(
        &ws,
        json!({
            "file_path": "nonexistent.ts",
            "line": 0,
            "character": 0
        }),
    );
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], json!("file_not_found"));
}

#[test]
fn enclosing_node_unsupported_language() {
    let ws = workspace();
    let result = tools::enclosing_node::handle(
        &ws,
        json!({
            "file_path": "Cargo.toml",
            "line": 0,
            "character": 0
        }),
    );
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], json!("unsupported_language"));
}
