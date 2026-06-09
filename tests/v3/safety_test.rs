use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools::{find_dependency_edges, find_routes, find_tests};
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

#[test]
fn safety_rejects_path_outside_workspace() {
    let r = find_routes::handle(&workspace(), json!({"file_path": "../outside/file.ts"}));
    assert!(r["error"].is_object());
    assert_eq!(r["error"]["code"].as_str(), Some("path_outside_workspace"));
}

#[test]
fn safety_rejects_missing_file() {
    let r = find_routes::handle(&workspace(), json!({"file_path": "nonexistent.ts"}));
    assert!(r["error"].is_object());
    assert_eq!(r["error"]["code"].as_str(), Some("file_not_found"));
}

#[test]
fn safety_rejects_no_file_or_glob() {
    let r = find_dependency_edges::handle(&workspace(), json!({}));
    assert!(r["error"].is_object());
    assert!(r["error"]["code"].as_str().is_some());
}

#[test]
fn safety_results_have_returned_and_scanned() {
    let r = find_tests::handle(
        &workspace(),
        json!({
            "file_path": "tests/fixtures/v3/tests/jest.ts"
        }),
    );
    assert!(r.get("error").is_none());
    assert!(r["returned"].as_u64().is_some());
    assert!(r["scannedFiles"].as_u64().is_some());
    assert!(r["truncated"].as_bool().is_some());
}

#[test]
fn safety_evidence_text_bounded() {
    let r = find_routes::handle(
        &workspace(),
        json!({"file_path": "tests/fixtures/v3/routes/express.ts"}),
    );
    assert!(r.get("error").is_none());
    for rt in r["routes"].as_array().unwrap() {
        for ev in rt["evidence"].as_array().unwrap() {
            if let Some(text) = ev["text"].as_str() {
                assert!(text.len() <= 500, "evidence text exceeds 500 chars: {}", text.len());
            }
        }
    }
}

#[test]
fn safety_no_file_writes() {
    // V3 tools must not write files. This is asserted by the architecture test.
    // Just verify no tool panics on edge cases.
    let r = find_routes::handle(
        &workspace(),
        json!({"file_path": "tests/fixtures/v3/routes/express.ts"}),
    );
    assert!(r.get("error").is_none());
}
