use ast_mcp::config::defaults::MAX_FILE_BYTES;
use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools;
use serde_json::json;
use std::fs;
use std::io::Write;

fn workspace(dir: &tempfile::TempDir) -> Workspace {
    std::env::set_var("WORKSPACE_PATH", dir.path().to_string_lossy().as_ref());
    Workspace::from_env().unwrap()
}

/// Call a tool handler that takes a file_path parameter by name.
fn call(tool: &str, workspace: &Workspace, file_path: &str) -> serde_json::Value {
    match tool {
        "ast_parse_file" => tools::parse_file::handle(workspace, json!({"file_path": file_path})),
        "ast_file_outline" => {
            tools::file_outline::handle(workspace, json!({"file_path": file_path}))
        }
        "ast_top_level_nodes" => {
            tools::top_level_nodes::handle(workspace, json!({"file_path": file_path}))
        }
        "ast_query" => tools::query::handle(
            workspace,
            json!({"file_path": file_path, "query": "(function_declaration) @f"}),
        ),
        "ast_find_imports" => {
            tools::find_imports::handle(workspace, json!({"file_path": file_path}))
        }
        "ast_find_exports" => {
            tools::find_exports::handle(workspace, json!({"file_path": file_path}))
        }
        "ast_find_functions" => {
            tools::find_functions::handle(workspace, json!({"file_path": file_path}))
        }
        "ast_find_classes" => {
            tools::find_classes::handle(workspace, json!({"file_path": file_path}))
        }
        "ast_chunk_file" => tools::chunk_file::handle(workspace, json!({"file_path": file_path})),
        "ast_enclosing_node" => tools::enclosing_node::handle(
            workspace,
            json!({"file_path": file_path, "line": 0, "character": 0}),
        ),
        _ => panic!("unknown tool: {}", tool),
    }
}

const FILE_TOOLS: &[&str] = &[
    "ast_parse_file",
    "ast_file_outline",
    "ast_top_level_nodes",
    "ast_query",
    "ast_find_imports",
    "ast_find_exports",
    "ast_find_functions",
    "ast_find_classes",
    "ast_chunk_file",
    "ast_enclosing_node",
];

// ---------------------------------------------------------------
// Path outside workspace — dot-dot traversal
// ---------------------------------------------------------------
#[test]
fn reject_traversal_dotdot() {
    let dir = tempfile::tempdir().unwrap();
    let ws = workspace(&dir);

    for tool in FILE_TOOLS {
        let result = call(tool, &ws, "../outside.ts");
        let err = result.get("error").unwrap_or_else(|| {
            panic!("{}: expected error for ../outside.ts, got: {:?}", tool, result)
        });
        assert_eq!(
            err["code"], "path_outside_workspace",
            "{}: expected path_outside_workspace, got {:?}",
            tool, err
        );
    }
}

// ---------------------------------------------------------------
// Path outside workspace — absolute path
// ---------------------------------------------------------------
#[test]
fn reject_absolute_path() {
    let dir = tempfile::tempdir().unwrap();
    let ws = workspace(&dir);

    for tool in FILE_TOOLS {
        let result = call(tool, &ws, "/etc/passwd");
        let err = result.get("error").unwrap_or_else(|| {
            panic!("{}: expected error for /etc/passwd, got: {:?}", tool, result)
        });
        assert_eq!(
            err["code"], "path_outside_workspace",
            "{}: expected path_outside_workspace, got {:?}",
            tool, err
        );
    }
}

// ---------------------------------------------------------------
// File not found — missing file
// ---------------------------------------------------------------
#[test]
fn reject_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let ws = workspace(&dir);

    for tool in FILE_TOOLS {
        let result = call(tool, &ws, "does_not_exist.ts");
        let err = result.get("error").unwrap_or_else(|| {
            panic!("{}: expected error for missing file, got: {:?}", tool, result)
        });
        assert_eq!(
            err["code"], "file_not_found",
            "{}: expected file_not_found, got {:?}",
            tool, err
        );
    }
}

// ---------------------------------------------------------------
// File not found — directory passed as file
// ---------------------------------------------------------------
#[test]
fn reject_directory_as_file() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("subdir");
    fs::create_dir(&sub).unwrap();
    let ws = workspace(&dir);

    for tool in FILE_TOOLS {
        let result = call(tool, &ws, "subdir");
        let err = result
            .get("error")
            .unwrap_or_else(|| panic!("{}: expected error for directory, got: {:?}", tool, result));
        assert_eq!(
            err["code"], "file_not_found",
            "{}: directory path should be file_not_found, got {:?}",
            tool, err
        );
    }
}

// ---------------------------------------------------------------
// Unsupported language — unknown extension
// ---------------------------------------------------------------
#[test]
fn reject_unsupported_language() {
    let dir = tempfile::tempdir().unwrap();
    let rb_path = dir.path().join("test.rb");
    fs::write(&rb_path, "puts 'hello'\n").unwrap();
    let ws = workspace(&dir);

    for tool in FILE_TOOLS {
        let result = call(tool, &ws, "test.rb");
        let err = result
            .get("error")
            .unwrap_or_else(|| panic!("{}: expected error for .rb file, got: {:?}", tool, result));
        assert_eq!(
            err["code"], "unsupported_language",
            "{}: expected unsupported_language, got {:?}",
            tool, err
        );
    }
}

// ---------------------------------------------------------------
// File too large
// ---------------------------------------------------------------
#[test]
fn reject_file_too_large() {
    let dir = tempfile::tempdir().unwrap();
    let big_path = dir.path().join("big.ts");
    let mut f = fs::File::create(&big_path).unwrap();
    // Write a valid header then pad with lines to exceed MAX_FILE_BYTES (1 MiB)
    f.write_all(b"const x = 1;\n").unwrap();
    // Each line is ~24 bytes — write 44,000 lines to exceed 1 MiB (1_048_576)
    let line = b"const p = 0; // padding\n";
    for _ in 0..44_000 {
        f.write_all(line).unwrap();
    }
    drop(f);
    // Sanity check: the file must actually exceed the limit
    let sz = fs::metadata(&big_path).unwrap().len();
    assert!(sz > MAX_FILE_BYTES, "test file is {sz} bytes, expected > {MAX_FILE_BYTES}");
    let ws = workspace(&dir);

    // Only test tools that go through the full resolve + ensure_under_size path.
    // Some tools may short-circuit on language detection before checking size,
    // but parse_file always checks size.
    let result = tools::parse_file::handle(&ws, json!({"file_path": "big.ts"}));
    let err = result
        .get("error")
        .unwrap_or_else(|| panic!("expected error for too-large file, got: {:?}", result));
    assert_eq!(err["code"], "file_too_large");

    let result = tools::file_outline::handle(&ws, json!({"file_path": "big.ts"}));
    let err = result.get("error").unwrap_or_else(|| {
        panic!("expected error for too-large file (outline), got: {:?}", result)
    });
    assert_eq!(err["code"], "file_too_large");
}
