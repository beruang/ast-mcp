use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools;
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

// ---------------------------------------------------------------------------
// find_imports — TypeScript
// ---------------------------------------------------------------------------

#[test]
fn find_imports_typescript_all_forms() {
    let ws = workspace();
    let result = tools::find_imports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/imports/all_forms.ts"}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let imports = result["imports"].as_array().unwrap();
    assert!(
        imports.len() >= 6,
        "expected at least 6 imports, got {}",
        imports.len()
    );

    // Check for at least one of each kind
    let kinds: Vec<&str> = imports
        .iter()
        .map(|i| i["kind"].as_str().unwrap())
        .collect();
    assert!(
        kinds.contains(&"import"),
        "expected at least one 'import' kind, got: {:?}",
        kinds
    );
    assert!(
        kinds.contains(&"require"),
        "expected at least one 'require' kind"
    );
    assert!(
        kinds.contains(&"dynamic_import"),
        "expected at least one 'dynamic_import' kind"
    );
}

#[test]
fn find_imports_typescript_default() {
    let ws = workspace();
    let result = tools::find_imports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/imports/all_forms.ts"}),
    );
    // Find the default import: import React from "react"
    let imports = result["imports"].as_array().unwrap();
    let react_import = imports
        .iter()
        .find(|i| i["modulePath"].as_str() == Some("react") && i["kind"] == "import");
    assert!(
        react_import.is_some(),
        "expected 'import React from \"react\"'"
    );
    let ri = react_import.unwrap();
    let names = ri["names"].as_array().unwrap();
    assert!(!names.is_empty(), "expected names in React import");
    // Should have at least the default import name
    let has_react = names
        .iter()
        .any(|n| n["imported"].as_str() == Some("React"));
    assert!(has_react, "expected 'React' in names");
}

#[test]
fn find_imports_typescript_side_effect() {
    let ws = workspace();
    let result = tools::find_imports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/imports/all_forms.ts"}),
    );
    let imports = result["imports"].as_array().unwrap();
    let side_effect = imports
        .iter()
        .find(|i| i["modulePath"].as_str() == Some("reflect-metadata"));
    assert!(
        side_effect.is_some(),
        "expected side-effect import 'reflect-metadata'"
    );
}

#[test]
fn find_imports_typescript_require() {
    let ws = workspace();
    let result = tools::find_imports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/imports/all_forms.ts"}),
    );
    let imports = result["imports"].as_array().unwrap();
    // DEBUG: print all imports
    eprintln!(
        "ALL IMPORTS: {}",
        serde_json::to_string_pretty(&imports).unwrap_or_default()
    );
    let require_import = imports.iter().find(|i| i["kind"] == "require");
    assert!(require_import.is_some(), "expected a require() call import");
    assert_eq!(require_import.unwrap()["modulePath"].as_str(), Some("fs"));
}

#[test]
fn find_imports_has_ranges() {
    let ws = workspace();
    let result = tools::find_imports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/imports/all_forms.ts"}),
    );
    let imports = result["imports"].as_array().unwrap();
    for imp in imports {
        assert!(imp["range"]["start"]["line"].as_u64().is_some());
        assert!(imp["range"]["end"]["line"].as_u64().is_some());
    }
}

// ---------------------------------------------------------------------------
// find_imports — Python
// ---------------------------------------------------------------------------

#[test]
fn find_imports_python_all_forms() {
    let ws = workspace();
    let result = tools::find_imports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/imports/all_forms.py"}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let imports = result["imports"].as_array().unwrap();
    assert!(
        imports.len() >= 6,
        "expected at least 6 imports, got {}",
        imports.len()
    );

    let kinds: Vec<&str> = imports
        .iter()
        .map(|i| i["kind"].as_str().unwrap())
        .collect();
    assert!(
        kinds.contains(&"import"),
        "expected 'import' kind: {:?}",
        kinds
    );
    assert!(
        kinds.contains(&"from_import"),
        "expected 'from_import' kind: {:?}",
        kinds
    );

    // Check for wildcard import
    let wildcard = imports.iter().find(|i| {
        i["names"]
            .as_array()
            .is_some_and(|n| n.iter().any(|a| a["imported"].as_str() == Some("*")))
    });
    assert!(wildcard.is_some(), "expected a wildcard import");
}

#[test]
fn find_imports_python_aliased() {
    let ws = workspace();
    let result = tools::find_imports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/imports/all_forms.py"}),
    );
    let imports = result["imports"].as_array().unwrap();
    let np_import = imports.iter().find(|i| {
        i["names"].as_array().is_some_and(|n| {
            n.iter().any(|a| {
                a["imported"].as_str() == Some("numpy") && a["local"].as_str() == Some("np")
            })
        })
    });
    assert!(np_import.is_some(), "expected 'import numpy as np'");
}

// ---------------------------------------------------------------------------
// find_exports — TypeScript
// ---------------------------------------------------------------------------

#[test]
fn find_exports_typescript_all_forms() {
    let ws = workspace();
    let result = tools::find_exports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/exports/all_forms.ts"}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let exports = result["exports"].as_array().unwrap();
    assert!(
        exports.len() >= 8,
        "expected at least 8 exports, got {}",
        exports.len()
    );

    let kinds: Vec<&str> = exports
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert!(kinds.contains(&"function"), "expected 'function' kind");
    assert!(kinds.contains(&"class"), "expected 'class' kind");
    assert!(kinds.contains(&"const"), "expected 'const' kind");
    assert!(kinds.contains(&"re_export"), "expected 're_export' kind");
}

#[test]
fn find_exports_typescript_class_name() {
    let ws = workspace();
    let result = tools::find_exports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/exports/all_forms.ts"}),
    );
    let exports = result["exports"].as_array().unwrap();
    let calc = exports
        .iter()
        .find(|e| e["name"].as_str() == Some("Calculator"));
    assert!(calc.is_some(), "expected export class Calculator");
    assert_eq!(calc.unwrap()["kind"], "class");
}

#[test]
fn find_exports_typescript_default() {
    let ws = workspace();
    let result = tools::find_exports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/exports/all_forms.ts"}),
    );
    let exports = result["exports"].as_array().unwrap();
    let default_exports: Vec<_> = exports.iter().filter(|e| e["is_default"] == true).collect();
    assert!(
        !default_exports.is_empty(),
        "expected at least one default export"
    );
}

#[test]
fn find_exports_typescript_re_export() {
    let ws = workspace();
    let result = tools::find_exports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/exports/all_forms.ts"}),
    );
    let exports = result["exports"].as_array().unwrap();
    let from_greetings = exports
        .iter()
        .find(|e| e["reExportSource"].as_str() == Some("./greetings"));
    assert!(
        from_greetings.is_some(),
        "expected re-export from './greetings'"
    );
}

// ---------------------------------------------------------------------------
// find_exports — Python
// ---------------------------------------------------------------------------

#[test]
fn find_exports_python_public_defs() {
    let ws = workspace();
    let result = tools::find_exports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/exports/public.py"}),
    );
    assert!(result["error"].is_null(), "unexpected error: {:?}", result);
    let exports = result["exports"].as_array().unwrap();
    assert!(
        exports.len() >= 2,
        "expected at least 2 Python public exports, got {}",
        exports.len()
    );

    // Should have function 'add'
    let add = exports
        .iter()
        .find(|e| e["name"].as_str() == Some("add") && e["kind"] == "function");
    assert!(add.is_some(), "expected public function 'add'");

    // Should have class 'Calculator'
    let calc = exports
        .iter()
        .find(|e| e["name"].as_str() == Some("Calculator") && e["kind"] == "class");
    assert!(calc.is_some(), "expected public class 'Calculator'");
}

#[test]
fn find_exports_python_all() {
    let ws = workspace();
    let result = tools::find_exports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/exports/public.py"}),
    );
    let exports = result["exports"].as_array().unwrap();
    let all_export = exports.iter().find(|e| e["kind"] == "python_all");
    assert!(all_export.is_some(), "expected __all__ export");
    assert_eq!(all_export.unwrap()["name"], "__all__");
}

#[test]
fn find_exports_python_excludes_private() {
    let ws = workspace();
    let result = tools::find_exports::handle(
        &ws,
        json!({"file_path": "tests/fixtures/exports/public.py"}),
    );
    let exports = result["exports"].as_array().unwrap();
    let private_func = exports
        .iter()
        .find(|e| e["name"].as_str() == Some("_private_helper"));
    assert!(
        private_func.is_none(),
        "private function should be excluded"
    );

    let private_class = exports
        .iter()
        .find(|e| e["name"].as_str() == Some("_PrivateClass"));
    assert!(private_class.is_none(), "private class should be excluded");
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn find_imports_missing_file() {
    let ws = workspace();
    let result = tools::find_imports::handle(&ws, json!({"file_path": "nonexistent.ts"}));
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], "file_not_found");
}

#[test]
fn find_exports_unsupported_language() {
    let ws = workspace();
    let result = tools::find_exports::handle(&ws, json!({"file_path": "Cargo.toml"}));
    assert!(result["error"].is_object());
    assert_eq!(result["error"]["code"], "unsupported_language");
}
