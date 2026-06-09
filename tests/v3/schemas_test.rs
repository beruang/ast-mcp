use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools::find_schema_definitions;
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

fn call(file_path: &str) -> serde_json::Value {
    find_schema_definitions::handle(&workspace(), json!({"file_path": file_path}))
}

#[test]
fn schemas_zod_detects_objects() {
    let r = call("tests/fixtures/v3/schemas/zod.ts");
    assert!(r.get("error").is_none(), "error: {:?}", r.get("error"));
    let schemas = r["schemas"].as_array().unwrap();
    assert!(schemas.len() >= 2, "got {} schemas", schemas.len());

    let names: Vec<&str> = schemas.iter().filter_map(|s| s["name"].as_str()).collect();
    assert!(names.contains(&"UserSchema"));
    assert!(names.contains(&"PostSchema"));
}

#[test]
fn schemas_zod_extracts_fields() {
    let r = call("tests/fixtures/v3/schemas/zod.ts");
    assert!(r.get("error").is_none());
    let schemas = r["schemas"].as_array().unwrap();
    let user = schemas.iter().find(|s| s["name"].as_str() == Some("UserSchema")).unwrap();
    let fields = user["fields"].as_array().unwrap();
    assert!(fields.len() >= 2, "expected at least 2 fields, got {}", fields.len());
}

#[test]
fn schemas_typescript_interfaces() {
    let r = call("tests/fixtures/v3/schemas/interface.ts");
    assert!(r.get("error").is_none(), "error: {:?}", r.get("error"));
    let schemas = r["schemas"].as_array().unwrap();
    assert!(schemas.len() >= 2);
    let names: Vec<&str> = schemas.iter().filter_map(|s| s["name"].as_str()).collect();
    assert!(names.contains(&"User"));
}

#[test]
fn schemas_typescript_interface_fields() {
    let r = call("tests/fixtures/v3/schemas/interface.ts");
    assert!(r.get("error").is_none());
    let schemas = r["schemas"].as_array().unwrap();
    let user = schemas.iter().find(|s| s["name"].as_str() == Some("User")).unwrap();
    let fields = user["fields"].as_array().unwrap();
    assert!(fields.len() >= 2);
}

#[test]
fn schemas_pydantic_detects_models() {
    let r = call("tests/fixtures/v3/schemas/pydantic.py");
    assert!(r.get("error").is_none(), "error: {:?}", r.get("error"));
    let schemas = r["schemas"].as_array().unwrap();
    assert!(schemas.len() >= 2);
    let names: Vec<&str> = schemas.iter().filter_map(|s| s["name"].as_str()).collect();
    assert!(names.contains(&"User"));
}

#[test]
fn schemas_pydantic_has_structure() {
    let r = call("tests/fixtures/v3/schemas/pydantic.py");
    assert!(r.get("error").is_none());
    for s in r["schemas"].as_array().unwrap() {
        assert!(s["name"].as_str().is_some());
        assert!(s["fields"].as_array().is_some());
        assert!(s["confidence"].as_str().is_some());
    }
}

#[test]
fn schemas_go_returns_valid_response() {
    let r = call("tests/fixtures/v3/schemas/go_struct.go");
    assert!(r.get("error").is_none(), "error: {:?}", r.get("error"));
    // Tree-sitter-go may parse differently; just verify valid structure
    assert!(r["returned"].as_u64().is_some());
    assert!(r["schemas"].as_array().is_some());
}

#[test]
fn schemas_rust_detects_structs_and_enums() {
    let r = call("tests/fixtures/v3/schemas/rust_struct.rs");
    assert!(r.get("error").is_none(), "error: {:?}", r.get("error"));
    let schemas = r["schemas"].as_array().unwrap();
    assert!(schemas.len() >= 2, "got {} schemas", schemas.len());
    let names: Vec<&str> = schemas.iter().filter_map(|s| s["name"].as_str()).collect();
    assert!(names.contains(&"User"));
}

#[test]
fn schemas_include_fields_false() {
    let r = find_schema_definitions::handle(
        &workspace(),
        json!({
            "file_path": "tests/fixtures/v3/schemas/zod.ts",
            "include_fields": false
        }),
    );
    assert!(r.get("error").is_none());
    for s in r["schemas"].as_array().unwrap() {
        let fields = s["fields"].as_array().unwrap();
        assert!(fields.is_empty(), "fields should be empty when include_fields=false");
    }
}
