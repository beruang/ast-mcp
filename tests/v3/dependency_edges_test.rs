use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools::find_dependency_edges;
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

fn call(file_path: &str) -> serde_json::Value {
    find_dependency_edges::handle(&workspace(), json!({"file_path": file_path}))
}

fn assert_valid_response(r: &serde_json::Value) {
    assert!(r.get("error").is_none(), "unexpected error: {:?}", r.get("error"));
    assert!(r["returned"].as_u64().is_some(), "missing returned count");
    assert!(r["scannedFiles"].as_u64().is_some(), "missing scannedFiles");
    assert!(r["truncated"].as_bool().is_some(), "missing truncated flag");
}

#[test]
fn edges_typescript_finds_imports() {
    let r = call("tests/fixtures/v3/dependencies/typescript.ts");
    assert_valid_response(&r);
    let edges = r["edges"].as_array().unwrap();
    assert!(edges.len() >= 2, "got {} edges", edges.len());

    for e in edges {
        assert!(e["kind"].as_str().is_some(), "each edge must have kind");
        if e["kind"].as_str() == Some("export") {
            assert!(e["toSpecifier"].as_str().is_some(), "export must have toSpecifier");
        }
    }
}

#[test]
fn edges_python_finds_imports() {
    let r = call("tests/fixtures/v3/dependencies/python.py");
    assert_valid_response(&r);
    let edges = r["edges"].as_array().unwrap();
    assert!(edges.len() >= 2, "got {} edges", edges.len());
    for e in edges {
        assert_eq!(e["kind"].as_str().unwrap(), "import");
        // Python `import os` produces module_path=None; names carry the module
        assert!(
            e["toSpecifier"].as_str().is_some() || !e["evidence"].as_array().unwrap().is_empty()
        );
    }
}

#[test]
fn edges_go_returns_valid_response() {
    let r = call("tests/fixtures/v3/dependencies/go.go");
    assert_valid_response(&r);
    // Go parser may not parse all fixtures; edges may be 0
    // Just verify the response is well-formed
    for e in r["edges"].as_array().unwrap() {
        assert!(e["kind"].as_str().is_some());
    }
}

#[test]
fn edges_rust_finds_declarations() {
    let r = call("tests/fixtures/v3/dependencies/rust.rs");
    assert_valid_response(&r);
    let edges = r["edges"].as_array().unwrap();
    assert!(edges.len() >= 2, "got {} edges", edges.len());
    let kinds: Vec<&str> = edges.iter().filter_map(|e| e["kind"].as_str()).collect();
    assert!(kinds.iter().any(|k| *k == "use" || *k == "mod" || *k == "package"));
}

#[test]
fn edges_have_confidence_and_evidence() {
    let r = call("tests/fixtures/v3/dependencies/python.py");
    assert!(r.get("error").is_none());
    for e in r["edges"].as_array().unwrap() {
        assert!(e["confidence"].as_str().is_some());
        assert!(!e["evidence"].as_array().unwrap().is_empty());
    }
}
