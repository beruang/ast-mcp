use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools;
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

// ---------------------------------------------------------------------------
// find_functions — TypeScript
// ---------------------------------------------------------------------------

#[test]
fn find_functions_typescript_all_forms() {
    let ws = workspace();
    let result = tools::find_functions::handle(
        &ws,
        json!({"file_path": "tests/fixtures/functions/all_forms.ts"}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let functions = result["functions"].as_array().unwrap();
    assert!(
        functions.len() >= 5,
        "expected at least 5 functions, got {}",
        functions.len()
    );

    let kinds: Vec<&str> = functions
        .iter()
        .map(|f| f["kind"].as_str().unwrap())
        .collect();
    assert!(kinds.contains(&"function"), "expected 'function' kind");
    assert!(kinds.contains(&"generator"), "expected 'generator' kind");
    assert!(kinds.contains(&"method"), "expected 'method' kind");
}

#[test]
fn find_functions_typescript_exported() {
    let ws = workspace();
    let result = tools::find_functions::handle(
        &ws,
        json!({"file_path": "tests/fixtures/functions/all_forms.ts"}),
    );
    let functions = result["functions"].as_array().unwrap();
    let exported = functions.iter().any(|f| f["exported"] == true);
    assert!(exported, "expected at least one exported function");
}

#[test]
fn find_functions_typescript_parameters() {
    let ws = workspace();
    let result = tools::find_functions::handle(
        &ws,
        json!({"file_path": "tests/fixtures/functions/all_forms.ts"}),
    );
    let functions = result["functions"].as_array().unwrap();

    // Find greet function with parameter "name"
    let greet = functions
        .iter()
        .find(|f| f["name"].as_str() == Some("greet"));
    assert!(greet.is_some(), "expected 'greet' function");

    let params = greet.unwrap()["parameters"].as_array();
    assert!(params.is_some(), "greet should have parameters");
    let params = params.unwrap();
    assert!(
        !params.is_empty(),
        "greet should have at least one parameter"
    );
}

#[test]
fn find_functions_typescript_async() {
    let ws = workspace();
    let result = tools::find_functions::handle(
        &ws,
        json!({"file_path": "tests/fixtures/functions/all_forms.ts"}),
    );
    let functions = result["functions"].as_array().unwrap();
    let fetch_data = functions
        .iter()
        .find(|f| f["name"].as_str() == Some("fetchData"));
    assert!(fetch_data.is_some(), "expected async 'fetchData' function");
    assert_eq!(fetch_data.unwrap()["async"], true);
}

#[test]
fn find_functions_typescript_method_parent() {
    let ws = workspace();
    let result = tools::find_functions::handle(
        &ws,
        json!({"file_path": "tests/fixtures/functions/all_forms.ts"}),
    );
    let functions = result["functions"].as_array().unwrap();
    let method = functions
        .iter()
        .find(|f| f["parentName"].as_str() == Some("MathUtil"));
    assert!(
        method.is_some(),
        "expected method with parentName 'MathUtil'"
    );
}

#[test]
fn find_functions_without_anonymous() {
    let ws = workspace();
    let result = tools::find_functions::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/functions/all_forms.ts",
            "include_anonymous": false
        }),
    );
    let functions = result["functions"].as_array().unwrap();
    // All should have names
    for f in functions {
        assert!(
            f["name"].as_str().is_some(),
            "all functions should have names when include_anonymous=false"
        );
    }
}

// ---------------------------------------------------------------------------
// find_functions — Python
// ---------------------------------------------------------------------------

#[test]
fn find_functions_python_all_forms() {
    let ws = workspace();
    let result = tools::find_functions::handle(
        &ws,
        json!({"file_path": "tests/fixtures/functions/all_forms.py"}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let functions = result["functions"].as_array().unwrap();
    assert!(
        functions.len() >= 4,
        "expected at least 4 functions, got {}",
        functions.len()
    );

    // Should have class methods
    let methods: Vec<_> = functions
        .iter()
        .filter(|f| f["kind"] == "function" && f["parentName"].as_str() == Some("Calculator"))
        .collect();
    assert!(!methods.is_empty(), "expected methods in Calculator class");
}

#[test]
fn find_functions_python_async() {
    let ws = workspace();
    let result = tools::find_functions::handle(
        &ws,
        json!({"file_path": "tests/fixtures/functions/all_forms.py"}),
    );
    let functions = result["functions"].as_array().unwrap();
    let fetch = functions
        .iter()
        .find(|f| f["name"].as_str() == Some("fetch_data"));
    assert!(fetch.is_some(), "expected 'fetch_data' function");
    assert_eq!(fetch.unwrap()["async"], true);
}

// ---------------------------------------------------------------------------
// find_classes — TypeScript
// ---------------------------------------------------------------------------

#[test]
fn find_classes_typescript_all_forms() {
    let ws = workspace();
    let result = tools::find_classes::handle(
        &ws,
        json!({"file_path": "tests/fixtures/classes/all_forms.ts"}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let classes = result["classes"].as_array().unwrap();
    assert!(
        classes.len() >= 5,
        "expected at least 5 classes, got {}",
        classes.len()
    );
}

#[test]
fn find_classes_typescript_extends() {
    let ws = workspace();
    let result = tools::find_classes::handle(
        &ws,
        json!({"file_path": "tests/fixtures/classes/all_forms.ts"}),
    );
    let classes = result["classes"].as_array().unwrap();
    let dog = classes.iter().find(|c| c["name"].as_str() == Some("Dog"));
    assert!(dog.is_some(), "expected class 'Dog'");
    assert!(
        dog.unwrap()["extendsText"].as_str().is_some(),
        "Dog should have extendsText"
    );
}

#[test]
fn find_classes_typescript_abstract() {
    let result = tools::find_classes::handle(
        &workspace(),
        json!({"file_path": "tests/fixtures/classes/all_forms.ts"}),
    );
    let classes = result["classes"].as_array().unwrap();
    let shape = classes.iter().find(|c| c["name"].as_str() == Some("Shape"));
    assert!(shape.is_some(), "expected abstract class 'Shape'");
    assert!(
        shape.unwrap()["isAbstract"] == true,
        "Shape should be abstract"
    );
}

#[test]
fn find_classes_typescript_exported() {
    let ws = workspace();
    let result = tools::find_classes::handle(
        &ws,
        json!({"file_path": "tests/fixtures/classes/all_forms.ts"}),
    );
    let classes = result["classes"].as_array().unwrap();
    let exported = classes
        .iter()
        .find(|c| c["name"].as_str() == Some("ExportedUtil"));
    assert!(exported.is_some(), "expected exported class 'ExportedUtil'");
    assert!(
        exported.unwrap()["isDefaultExport"] == true,
        "ExportedUtil should be exported"
    );
}

#[test]
fn find_classes_typescript_methods() {
    let ws = workspace();
    let result = tools::find_classes::handle(
        &ws,
        json!({"file_path": "tests/fixtures/classes/all_forms.ts"}),
    );
    let classes = result["classes"].as_array().unwrap();
    let animal = classes
        .iter()
        .find(|c| c["name"].as_str() == Some("Animal"));
    assert!(animal.is_some(), "expected class 'Animal'");

    let methods = animal.unwrap()["methods"].as_array();
    assert!(methods.is_some(), "Animal should have methods");
    let methods = methods.unwrap();
    let has_constructor = methods.iter().any(|m| m["kind"] == "constructor");
    assert!(has_constructor, "Animal should have a constructor method");
}

// ---------------------------------------------------------------------------
// find_classes — Python
// ---------------------------------------------------------------------------

#[test]
fn find_classes_python_all_forms() {
    let ws = workspace();
    let result = tools::find_classes::handle(
        &ws,
        json!({"file_path": "tests/fixtures/classes/all_forms.py"}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let classes = result["classes"].as_array().unwrap();
    assert!(
        classes.len() >= 3,
        "expected at least 3 classes, got {}",
        classes.len()
    );
}

#[test]
fn find_classes_python_extends() {
    let ws = workspace();
    let result = tools::find_classes::handle(
        &ws,
        json!({"file_path": "tests/fixtures/classes/all_forms.py"}),
    );
    let classes = result["classes"].as_array().unwrap();
    let dog = classes.iter().find(|c| c["name"].as_str() == Some("Dog"));
    assert!(dog.is_some(), "expected class 'Dog'");
    assert!(
        dog.unwrap()["extendsText"].as_str().is_some(),
        "Dog should have extendsText"
    );
}

#[test]
fn find_classes_python_methods() {
    let ws = workspace();
    let result = tools::find_classes::handle(
        &ws,
        json!({"file_path": "tests/fixtures/classes/all_forms.py"}),
    );
    let classes = result["classes"].as_array().unwrap();
    let animal = classes
        .iter()
        .find(|c| c["name"].as_str() == Some("Animal"));
    assert!(animal.is_some(), "expected class 'Animal'");

    let methods = animal.unwrap()["methods"].as_array();
    assert!(methods.is_some(), "Animal should have methods");
    let methods = methods.unwrap();
    assert!(methods.len() >= 2, "Animal should have at least 2 methods");
}

// ---------------------------------------------------------------------------
// chunk_file
// ---------------------------------------------------------------------------

#[test]
fn chunk_file_top_level() {
    let ws = workspace();
    let result = tools::chunk_file::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/chunks/mixed.ts",
            "strategy": "top_level"
        }),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let chunks = result["chunks"].as_array().unwrap();
    assert!(!chunks.is_empty(), "expected at least one chunk");
    assert_eq!(result["strategy"], "top_level");
    assert!(result["totalChunks"].as_u64().unwrap() > 0);

    // Each chunk should have required fields
    for chunk in chunks {
        assert!(chunk["startLine"].as_u64().is_some());
        assert!(chunk["endLine"].as_u64().is_some());
        assert!(chunk["startByte"].as_u64().is_some());
        assert!(chunk["endByte"].as_u64().is_some());
        assert!(chunk["text"].as_str().is_some());
        assert!(chunk["kind"].as_str().is_some());
    }
}

#[test]
fn chunk_file_function_class_strategy() {
    let ws = workspace();
    let result = tools::chunk_file::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/chunks/mixed.ts",
            "strategy": "function_class"
        }),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    assert_eq!(result["strategy"], "function_class");
    let chunks = result["chunks"].as_array().unwrap();
    assert!(!chunks.is_empty(), "expected at least one chunk");
}

#[test]
fn chunk_file_semantic_blocks_strategy() {
    let ws = workspace();
    let result = tools::chunk_file::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/chunks/mixed.ts",
            "strategy": "semantic_blocks"
        }),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    assert_eq!(result["strategy"], "semantic_blocks");
    let chunks = result["chunks"].as_array().unwrap();
    assert!(!chunks.is_empty(), "expected at least one chunk");
}

#[test]
fn chunk_file_max_lines_strategy() {
    let ws = workspace();
    let result = tools::chunk_file::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/chunks/mixed.ts",
            "strategy": "max_lines",
            "max_lines_per_chunk": 5
        }),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    assert_eq!(result["strategy"], "max_lines_with_ast_boundaries");
    let chunks = result["chunks"].as_array().unwrap();
    assert!(!chunks.is_empty(), "expected at least one chunk");
}

#[test]
fn chunk_file_default_strategy() {
    let ws = workspace();
    let result = tools::chunk_file::handle(
        &ws,
        json!({
            "file_path": "tests/fixtures/chunks/mixed.ts"
        }),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    assert_eq!(result["strategy"], "top_level");
}

#[test]
fn chunk_file_missing_file() {
    let ws = workspace();
    let result = tools::chunk_file::handle(&ws, json!({"file_path": "nonexistent.ts"}));
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], "file_not_found");
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn find_functions_missing_file() {
    let ws = workspace();
    let result = tools::find_functions::handle(&ws, json!({"file_path": "nonexistent.ts"}));
    assert!(result["error"].is_object());
}

#[test]
fn find_classes_unsupported_language() {
    let ws = workspace();
    let result = tools::find_classes::handle(&ws, json!({"file_path": "Cargo.toml"}));
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], "unsupported_language");
}
