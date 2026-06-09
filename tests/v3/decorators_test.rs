use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools::find_decorators;
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

fn call(file_path: &str) -> serde_json::Value {
    find_decorators::handle(&workspace(), json!({"file_path": file_path}))
}

#[test]
fn decorators_typescript_detects_all() {
    let r = call("tests/fixtures/v3/decorators/typescript.ts");
    assert!(r.get("error").is_none(), "error: {:?}", r.get("error"));
    let decs = r["decorators"].as_array().unwrap();
    assert!(decs.len() >= 4, "got {} decorators", decs.len());

    let names: Vec<&str> = decs.iter().filter_map(|d| d["name"].as_str()).collect();
    assert!(names.contains(&"Controller"));
    assert!(names.contains(&"Get"));
    assert!(names.contains(&"Post"));
    assert!(names.contains(&"Injectable"));
}

#[test]
fn decorators_typescript_has_structure() {
    let r = call("tests/fixtures/v3/decorators/typescript.ts");
    assert!(r.get("error").is_none());
    for d in r["decorators"].as_array().unwrap() {
        assert!(d["name"].as_str().is_some());
        assert!(d["confidence"].as_str().is_some());
        assert!(!d["evidence"].as_array().unwrap().is_empty());
        // Some should have target info
        if d["targetKind"].as_str().is_some() {
            assert!(d["targetName"].as_str().is_some());
        }
    }
}

#[test]
fn decorators_python_detects_decorators() {
    let r = call("tests/fixtures/v3/decorators/python.py");
    assert!(r.get("error").is_none(), "error: {:?}", r.get("error"));
    let decs = r["decorators"].as_array().unwrap();
    assert!(decs.len() >= 3);
    let names: Vec<&str> = decs.iter().map(|d| d["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"app.get"));
}

#[test]
fn decorators_rust_detects_attributes() {
    let r = call("tests/fixtures/v3/decorators/rust.rs");
    assert!(r.get("error").is_none(), "error: {:?}", r.get("error"));
    let decs = r["decorators"].as_array().unwrap();
    assert!(decs.len() >= 2);
    let names: Vec<&str> = decs.iter().filter_map(|d| d["name"].as_str()).collect();
    assert!(names.contains(&"derive") || names.contains(&"test"));
}

#[test]
fn decorators_name_filter_works() {
    let r = find_decorators::handle(
        &workspace(),
        json!({
            "file_path": "tests/fixtures/v3/decorators/typescript.ts",
            "names": ["Controller", "Get"]
        }),
    );
    assert!(r.get("error").is_none());
    let decs = r["decorators"].as_array().unwrap();
    for d in decs {
        let name = d["name"].as_str().unwrap();
        assert!(name.contains("Controller") || name.contains("Get"));
    }
}
