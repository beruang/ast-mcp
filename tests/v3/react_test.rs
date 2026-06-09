use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools::{find_hooks, find_react_components};
use serde_json::json;

fn workspace() -> Workspace {
    Workspace::from_env().unwrap()
}

#[test]
fn react_components_detects_function_components() {
    let r = find_react_components::handle(
        &workspace(),
        json!({
            "file_path": "tests/fixtures/v3/react/components.tsx",
            "include_hooks": true
        }),
    );
    assert!(r.get("error").is_none(), "unexpected error: {:?}", r.get("error"));
    let comps = r["components"].as_array().unwrap();
    assert!(comps.len() >= 3, "expected at least 3 components, got {}", comps.len());

    let names: Vec<&str> = comps.iter().filter_map(|c| c["name"].as_str()).collect();
    assert!(names.contains(&"UserCard"));
    assert!(names.contains(&"UserPage"));
}

#[test]
fn react_components_all_have_kind_and_export_info() {
    let r = find_react_components::handle(
        &workspace(),
        json!({
            "file_path": "tests/fixtures/v3/react/components.tsx"
        }),
    );
    assert!(r.get("error").is_none());
    for c in r["components"].as_array().unwrap() {
        let kind = c["kind"].as_str().expect("missing kind");
        assert!(
            [
                "function_component",
                "arrow_function_component",
                "class_component",
                "memo_component",
                "forward_ref_component",
                "unknown"
            ]
            .contains(&kind),
            "unexpected kind: {}",
            kind
        );
        assert!(c["exported"].as_bool().is_some(), "missing exported flag");
        assert!(c["confidence"].as_str().is_some(), "missing confidence");
    }
}

#[test]
fn react_components_includes_hooks_when_requested() {
    let r = find_react_components::handle(
        &workspace(),
        json!({
            "file_path": "tests/fixtures/v3/react/components.tsx",
            "include_hooks": true
        }),
    );
    assert!(r.get("error").is_none());
    // At least one component should have hooks
    let comps = r["components"].as_array().unwrap();
    let has_hooks = comps.iter().any(|c| !c["hooks"].as_array().unwrap().is_empty());
    assert!(has_hooks, "at least one component should have hooks");
}

#[test]
fn hooks_detects_builtin_usages() {
    let r = find_hooks::handle(
        &workspace(),
        json!({
            "file_path": "tests/fixtures/v3/react/hooks.tsx"
        }),
    );
    assert!(r.get("error").is_none());
    let hooks = r["hooks"].as_array().unwrap();
    assert!(hooks.len() >= 5, "expected at least 5 hooks, got {}", hooks.len());

    let builtins: Vec<&str> = hooks
        .iter()
        .filter(|h| h["kind"].as_str() == Some("builtin_usage"))
        .filter_map(|h| h["name"].as_str())
        .collect();
    assert!(builtins.contains(&"useState"), "should find useState usage, got: {:?}", builtins);
    assert!(builtins.contains(&"useEffect"), "should find useEffect usage");
}

#[test]
fn hooks_detects_custom_definitions() {
    let r = find_hooks::handle(
        &workspace(),
        json!({
            "file_path": "tests/fixtures/v3/react/hooks.tsx"
        }),
    );
    assert!(r.get("error").is_none());
    let hooks = r["hooks"].as_array().unwrap();
    let defs: Vec<&str> = hooks
        .iter()
        .filter(|h| h["kind"].as_str() == Some("custom_definition"))
        .filter_map(|h| h["name"].as_str())
        .collect();
    assert!(defs.contains(&"useCustomHook"), "custom definitions: {:?}", defs);
    assert!(defs.contains(&"useFeatureFlag"), "custom definitions: {:?}", defs);
}

#[test]
fn hooks_detects_custom_usages() {
    let r = find_hooks::handle(
        &workspace(),
        json!({
            "file_path": "tests/fixtures/v3/react/hooks.tsx"
        }),
    );
    assert!(r.get("error").is_none());
    let hooks = r["hooks"].as_array().unwrap();
    let usages: Vec<&str> = hooks
        .iter()
        .filter(|h| h["kind"].as_str() == Some("custom_usage"))
        .filter_map(|h| h["name"].as_str())
        .collect();
    assert!(
        usages.contains(&"useCustomHook") || usages.contains(&"useFeatureFlag"),
        "custom usages: {:?}",
        usages
    );
}
