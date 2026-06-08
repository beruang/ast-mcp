use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools;
use serde_json::json;
use std::fs;
use std::io::Write;

fn workspace(dir: &tempfile::TempDir) -> Workspace {
    std::env::set_var("WORKSPACE_PATH", dir.path().to_string_lossy().as_ref());
    Workspace::from_env().unwrap()
}

// ---------------------------------------------------------------
// parse_file with includeTree on a file with > MAX_NODES nodes
// ---------------------------------------------------------------
#[test]
fn parse_file_truncates_large_tree() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("many_nodes.ts");
    let mut f = fs::File::create(&file_path).unwrap();

    // Generate 600 variable declarations to produce >500 named nodes.
    // Each "let xN = N;\n" is a variable_declaration node.
    for i in 0..600u32 {
        writeln!(f, "let x{i} = {i};").unwrap();
    }
    drop(f);
    let ws = workspace(&dir);

    let result = tools::parse_file::handle(
        &ws,
        json!({"file_path": "many_nodes.ts", "include_tree": true, "max_depth": 1}),
    );

    // Should succeed
    assert!(result["error"].is_null(), "expected success, got error: {:?}", result);
    // Should be truncated
    assert!(
        result["truncated"].as_bool().unwrap_or(false),
        "expected truncated: true, got: {:?}",
        result
    );
    // Should still have the tree
    assert!(result["tree"].is_object(), "expected tree in response");
    assert!(result["nodeCount"].as_u64().unwrap_or(0) > 500, "expected nodeCount > 500");
}

// ---------------------------------------------------------------
// query with > MAX_QUERY_MATCHES (200) matches
// ---------------------------------------------------------------
#[test]
fn query_truncates_large_result_set() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("many_funcs.ts");
    let mut f = fs::File::create(&file_path).unwrap();

    // Generate 300 function declarations — more than MAX_QUERY_MATCHES (200)
    for i in 0..300u32 {
        writeln!(f, "function f{i}() {{ return {i}; }}").unwrap();
    }
    drop(f);
    let ws = workspace(&dir);

    let result = tools::query::handle(
        &ws,
        json!({
            "file_path": "many_funcs.ts",
            "query": "(function_declaration) @f"
        }),
    );

    assert!(result["error"].is_null(), "expected success, got error: {:?}", result);
    assert!(
        result["truncated"].as_bool().unwrap_or(false),
        "expected truncated: true for query with >200 matches, got: {:?}",
        result
    );
    let returned = result["returnedCount"].as_u64().unwrap_or(0);
    assert!(returned <= 200, "returnedCount {} should be <= 200", returned);
}
