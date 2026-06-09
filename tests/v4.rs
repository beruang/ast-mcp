//! V4 integration tests — rewrite preview tools across TypeScript, JavaScript, and Python.

use ast_mcp::mcp::server_context::ServerContext;
use serde_json::json;

// ── Helpers ──

fn project_workspace() -> ast_mcp::config::workspace::Workspace {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    std::env::set_var("WORKSPACE_PATH", &manifest_dir);
    ast_mcp::config::workspace::Workspace::from_env().unwrap()
}

fn project_ctx() -> ServerContext {
    ServerContext::for_testing(project_workspace())
}

fn temp_workspace() -> (tempfile::TempDir, ast_mcp::config::workspace::Workspace) {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("WORKSPACE_PATH", dir.path().to_string_lossy().as_ref());
    let w = ast_mcp::config::workspace::Workspace::from_env().unwrap();
    (dir, w)
}

#[allow(dead_code)]
fn temp_ctx() -> (tempfile::TempDir, ServerContext) {
    let (dir, ws) = temp_workspace();
    (dir, ServerContext::for_testing(ws))
}

// ── Tool presence ──

#[test]
fn all_v4_tools_registered() {
    let ctx = project_ctx();
    let tools = ast_mcp::mcp::register_tools::tools(&ctx);
    let names: Vec<_> = tools.iter().map(|t| t.name).collect();
    let v4_tools = [
        "ast_rewrite_preview",
        "ast_validate_rewrite",
        "ast_parse_after_rewrite",
        "ast_insert_import_preview",
        "ast_remove_unused_import_preview",
        "ast_rename_local_preview",
        "ast_wrap_node_preview",
        "ast_add_decorator_preview",
        "ast_modify_function_signature_preview",
    ];
    for t in &v4_tools {
        assert!(names.contains(t), "missing V4 tool: {}", t);
    }
}

// ── ast_validate_rewrite ──

#[test]
fn validate_rejects_empty_operations() {
    let w = project_workspace();
    let resp = ast_mcp::tools::validate_rewrite::handle(&w, json!({"operations": []}));
    assert_eq!(resp["safe"], json!(false));
}

#[test]
fn validate_rejects_outside_workspace() {
    let w = project_workspace();
    let resp = ast_mcp::tools::validate_rewrite::handle(
        &w,
        json!({
            "operations": [{
                "kind": "replace_range",
                "file_path": "../outside.ts",
                "range": {"start": {"line":0,"character":0}, "end": {"line":0,"character":0}},
                "new_text": "x"
            }]
        }),
    );
    assert_eq!(resp["safe"], json!(false));
}

#[test]
fn validate_rejects_too_many_edits() {
    let w = project_workspace();
    let ops: Vec<serde_json::Value> = (0..250)
        .map(|i| {
            json!({
                "kind": "replace_range",
                "file_path": "src/lib.rs",
                "range": {"start": {"line":i,"character":0}, "end": {"line":i,"character":0}},
                "new_text": "x"
            })
        })
        .collect();
    let resp = ast_mcp::tools::validate_rewrite::handle(
        &w,
        json!({
            "operations": ops,
            "max_edits": 200
        }),
    );
    assert_eq!(resp["safe"], json!(false));
}

// ── ast_parse_after_rewrite ──

#[test]
fn parse_after_edits_rejects_invalid_range() {
    let w = project_workspace();
    let resp = ast_mcp::tools::parse_after_rewrite::handle(
        &w,
        json!({
            "edits": [{
                "file_path": "src/lib.rs",
                "range": {"start": {"line": 99999,"character":0}, "end": {"line":99999,"character":0}},
                "new_text": "x"
            }]
        }),
    );
    assert_eq!(resp["ok"], json!(false));
}

// ── ast_insert_import_preview ──

#[test]
fn insert_import_ts() {
    let (dir, w) = temp_workspace();
    std::fs::write(dir.path().join("test.ts"), "const x = 1;\n").unwrap();
    let resp = ast_mcp::rewrite_tools::insert_import::handle(
        &w,
        json!({
            "file_path": "test.ts",
            "import": {"source": "react", "named_imports": ["useState"]},
            "include_diff": true, "parse_check": true
        }),
    );
    assert_eq!(resp["safe"], json!(true), "got: {}", resp);
}

#[test]
fn insert_import_python() {
    let (dir, w) = temp_workspace();
    std::fs::write(dir.path().join("test.py"), "x = 1\n").unwrap();
    let resp = ast_mcp::rewrite_tools::insert_import::handle(
        &w,
        json!({
            "file_path": "test.py",
            "import": {"source": "os.path", "named_imports": ["join"]},
            "include_diff": true, "parse_check": true
        }),
    );
    assert_eq!(resp["safe"], json!(true), "got: {}", resp);
}

// ── ast_rename_local_preview ──

#[test]
fn rename_local_in_function_scope() {
    let (dir, w) = temp_workspace();
    std::fs::write(dir.path().join("test.ts"), "function foo() { let x = 1; return x; }\n")
        .unwrap();
    // Rename 'x' inside function scope
    let resp = ast_mcp::rewrite_tools::rename_local::handle(
        &w,
        json!({
            "file_path": "test.ts",
            "position": {"line": 0, "character": 21},
            "new_name": "count",
            "include_diff": true, "parse_check": true
        }),
    );
    // Local variable rename inside function scope should succeed
    assert_eq!(resp["safe"], json!(true), "local rename should succeed, got: {}", resp);
}

// ── ast_wrap_node_preview ──

#[test]
fn wrap_with_call_expression() {
    let (dir, w) = temp_workspace();
    std::fs::write(dir.path().join("test.ts"), "getUser(id);\n").unwrap();
    let resp = ast_mcp::rewrite_tools::wrap_node::handle(
        &w,
        json!({
            "file_path": "test.ts",
            "range": {"start": {"line":0,"character":0}, "end": {"line":0,"character":12}},
            "wrapper": {"kind": "call_expression", "callee": "trace"},
            "parse_check": false
        }),
    );
    assert_eq!(resp["safe"], json!(true), "got: {}", resp);
}

// ── ast_add_decorator_preview ──

#[test]
fn add_decorator_python() {
    let (dir, w) = temp_workspace();
    std::fs::write(dir.path().join("test.py"), "def foo():\n    pass\n").unwrap();
    let resp = ast_mcp::rewrite_tools::add_decorator::handle(
        &w,
        json!({
            "file_path": "test.py",
            "target_range": {"start": {"line":0,"character":0}, "end": {"line":0,"character":11}},
            "decorator_text": "@staticmethod",
            "parse_check": true
        }),
    );
    assert_eq!(resp["safe"], json!(true), "got: {}", resp);
}

// ── ast_modify_function_signature_preview ──

#[test]
fn modify_signature_add_param() {
    let (dir, w) = temp_workspace();
    std::fs::write(
        dir.path().join("test.ts"),
        "function greet(name) { return 'hello ' + name; }\n",
    )
    .unwrap();
    let resp = ast_mcp::rewrite_tools::modify_signature::handle(
        &w,
        json!({
            "file_path": "test.ts",
            "function_range": {"start": {"line":0,"character":0}, "end": {"line":0,"character":46}},
            "operation": {"kind": "add_parameter", "parameter_text": "greeting?: string", "position": 1},
            "parse_check": false
        }),
    );
    assert_eq!(resp["safe"], json!(true), "got: {}", resp);
}

// ── Safety: no file writes ──

#[test]
fn no_v4_tool_writes_files() {
    let (dir, w) = temp_workspace();
    let src = "const x = 1;\n";
    std::fs::write(dir.path().join("test.ts"), src).unwrap();

    let _resp = ast_mcp::tools::rewrite_preview::handle(
        &w,
        json!({
            "operations": [{
                "kind": "replace_range",
                "file_path": "test.ts",
                "range": {"start": {"line":0,"character":0}, "end": {"line":0,"character":0}},
                "new_text": "// comment\n"
            }],
            "parse_check": false
        }),
    );

    let after = std::fs::read_to_string(dir.path().join("test.ts")).unwrap();
    assert_eq!(src, after, "V4 tool wrote to file — this is forbidden");
}
