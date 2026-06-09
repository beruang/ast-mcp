use ast_mcp::config::workspace::Workspace;
use ast_mcp::tools;
use serde_json::{json, Value};
use std::fs;
use std::sync::Mutex;

static WS_LOCK: Mutex<()> = Mutex::new(());

fn workspace(dir: &tempfile::TempDir) -> Workspace {
    let _guard = WS_LOCK.lock().unwrap();
    std::env::set_var("WORKSPACE_PATH", dir.path().to_string_lossy().as_ref());
    Workspace::from_env().unwrap()
}

fn assert_no_error(result: &Value, tool: &str) {
    if result.get("error").and_then(|e| e.as_object()).is_some() {
        panic!("{} returned error: {:?}", tool, result["error"]);
    }
}

fn assert_field_present(result: &Value, field: &str, tool: &str) {
    assert!(result.get(field).is_some(), "{}: missing field '{}' in {:?}", tool, field, result);
}

fn assert_truncated_is_bool_if_present(result: &Value, tool: &str) {
    if let Some(truncated) = result.get("truncated") {
        assert!(
            truncated.as_bool().is_some(),
            "{}: 'truncated' should be a bool, got {:?}",
            tool,
            truncated
        );
    }
}

/// Verify filePath in the response is workspace-relative (no leading /, no ..)
fn assert_workspace_relative_path(result: &Value, tool: &str) {
    if let Some(p) = result.get("filePath").and_then(|v| v.as_str()) {
        assert!(!p.starts_with('/'), "{}: filePath '{}' should be workspace-relative", tool, p);
        assert!(!p.contains(".."), "{}: filePath '{}' should not contain '..'", tool, p);
    }
}

// ---------------------------------------------------------------
// Fixture setup
// ---------------------------------------------------------------

fn setup_fixtures(dir: &tempfile::TempDir) {
    let ts = dir.path().join("sample.ts");
    fs::write(
        &ts,
        r#"
import { helper } from "./helper";
import type { Config } from "./types";

export const MAX = 100;

export interface Config {
    name: string;
    count: number;
}

export type ID = string;

export enum Kind { A = 1, B = 2 }

export function compute(input: string): number {
    return input.length * MAX;
}

export class Processor {
    private factor: number;
    constructor(f: number) { this.factor = f; }
    process(data: string): number {
        return compute(data) * this.factor;
    }
}

const arrow = (x: number): number => x * 2;

function localHelper(): void { console.log("local"); }
"#,
    )
    .unwrap();

    let tsx = dir.path().join("sample.tsx");
    fs::write(
        &tsx,
        r#"
import React from "react";

interface Props { title: string; }

const Card: React.FC<Props> = ({ title }) => {
    return <div className="card"><h2>{title}</h2></div>;
};

export default Card;
"#,
    )
    .unwrap();

    let js = dir.path().join("sample.js");
    fs::write(
        &js,
        r#"
const path = require("path");

function greet(name) {
    return "Hello, " + name;
}

module.exports = { greet };
"#,
    )
    .unwrap();

    let jsx = dir.path().join("sample.jsx");
    fs::write(
        &jsx,
        r#"
import React from "react";

const Button = ({ label, onClick }) => {
    return <button onClick={onClick}>{label}</button>;
};

export default Button;
"#,
    )
    .unwrap();

    let py = dir.path().join("sample.py");
    fs::write(
        &py,
        r#"
import os
from typing import List, Optional

__all__ = ["process", "Processor"]

MAX = 100

def process(items: List[str]) -> int:
    return len(items) * MAX

class Processor:
    factor: int

    def __init__(self, factor: int) -> None:
        self.factor = factor

    async def run(self, data: str) -> int:
        return len(data) * self.factor
"#,
    )
    .unwrap();
}

// ---------------------------------------------------------------
// Tool 1: ast_health_check
// ---------------------------------------------------------------
#[test]
fn sweep_health_check() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    let result = tools::health_check::handle(&ws, json!({}));
    assert_no_error(&result, "ast_health_check");
    assert_field_present(&result, "workspacePath", "ast_health_check");
    assert_field_present(&result, "parsers", "ast_health_check");
    assert!(result["parsers"].as_array().unwrap().len() >= 5);
}

// ---------------------------------------------------------------
// Tool 2: ast_list_supported_languages
// ---------------------------------------------------------------
#[test]
fn sweep_list_supported_languages() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let _ws = workspace(&dir);

    let result = tools::list_supported_languages::handle(json!({}));
    assert_no_error(&result, "ast_list_supported_languages");
    let langs = result["languages"].as_array().unwrap();
    assert_eq!(langs.len(), 7);
    // Each entry should have language, extensions, parser, available
    for lang in langs {
        assert!(lang["language"].is_string());
        assert!(lang["extensions"].is_array());
        assert!(lang["parser"].is_string());
        assert!(lang["available"].as_bool().unwrap_or(false));
    }
}

// ---------------------------------------------------------------
// Tool 3: ast_parse_file
// ---------------------------------------------------------------
#[test]
fn sweep_parse_file() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    for file in &["sample.ts", "sample.tsx", "sample.js", "sample.jsx", "sample.py"] {
        let result = tools::parse_file::handle(&ws, json!({"file_path": file}));
        assert_no_error(&result, "ast_parse_file");
        assert_workspace_relative_path(&result, "ast_parse_file");
        assert_field_present(&result, "language", "ast_parse_file");
        assert_field_present(&result, "rootKind", "ast_parse_file");
        assert_field_present(&result, "nodeCount", "ast_parse_file");
        assert_field_present(&result, "parseTimeMs", "ast_parse_file");
        assert_field_present(&result, "hasSyntaxError", "ast_parse_file");
        assert!(result["parsed"].as_bool().unwrap_or(false));
    }
}

// ---------------------------------------------------------------
// Tool 4: ast_file_outline
// ---------------------------------------------------------------
#[test]
fn sweep_file_outline() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    for file in &["sample.ts", "sample.tsx", "sample.js", "sample.jsx", "sample.py"] {
        let result = tools::file_outline::handle(&ws, json!({"file_path": file}));
        assert_no_error(&result, "ast_file_outline");
        assert_workspace_relative_path(&result, "ast_file_outline");
        assert_truncated_is_bool_if_present(&result, "ast_file_outline");
        assert_field_present(&result, "outlineText", "ast_file_outline");
        assert!(!result["outlineText"].as_str().unwrap_or("").is_empty());
        assert_field_present(&result, "nodes", "ast_file_outline");
    }
}

// ---------------------------------------------------------------
// Tool 5: ast_top_level_nodes
// ---------------------------------------------------------------
#[test]
fn sweep_top_level_nodes() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    for file in &["sample.ts", "sample.tsx", "sample.js", "sample.jsx", "sample.py"] {
        let result = tools::top_level_nodes::handle(&ws, json!({"file_path": file}));
        assert_no_error(&result, "ast_top_level_nodes");
        assert_workspace_relative_path(&result, "ast_top_level_nodes");
        assert_truncated_is_bool_if_present(&result, "ast_top_level_nodes");
        let nodes = result["nodes"].as_array().unwrap();
        assert!(!nodes.is_empty(), "{}: expected at least 1 top-level node", file);
        for node in nodes {
            assert!(node["kind"].is_string(), "node missing kind");
            assert!(node["range"].is_object(), "node missing range");
        }
    }
}

// ---------------------------------------------------------------
// Tool 6: ast_query
// ---------------------------------------------------------------
#[test]
fn sweep_query() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    // TypeScript query
    let result = tools::query::handle(
        &ws,
        json!({
            "file_path": "sample.ts",
            "query": "(function_declaration name: (identifier) @name) @f"
        }),
    );
    assert_no_error(&result, "ast_query");
    assert_workspace_relative_path(&result, "ast_query");
    assert_truncated_is_bool_if_present(&result, "ast_query");
    assert!(result["returnedCount"].as_u64().is_some());
    assert!(result["parseTimeMs"].as_u64().is_some());

    // Python query
    let result = tools::query::handle(
        &ws,
        json!({
            "file_path": "sample.py",
            "query": "(class_definition name: (identifier) @name) @c"
        }),
    );
    assert_no_error(&result, "ast_query");
}

// ---------------------------------------------------------------
// Tool 7: ast_find_imports
// ---------------------------------------------------------------
#[test]
fn sweep_find_imports() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    // TypeScript imports
    let result = tools::find_imports::handle(&ws, json!({"file_path": "sample.ts"}));
    assert_no_error(&result, "ast_find_imports");
    assert_workspace_relative_path(&result, "ast_find_imports");
    assert_truncated_is_bool_if_present(&result, "ast_find_imports");
    let imports = result["imports"].as_array().unwrap();
    assert!(!imports.is_empty(), "expected at least 1 import in sample.ts");

    // Python imports
    let result = tools::find_imports::handle(&ws, json!({"file_path": "sample.py"}));
    assert_no_error(&result, "ast_find_imports");
    let py_imports = result["imports"].as_array().unwrap();
    assert!(!py_imports.is_empty(), "expected at least 1 import in sample.py");
}

// ---------------------------------------------------------------
// Tool 8: ast_find_exports
// ---------------------------------------------------------------
#[test]
fn sweep_find_exports() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    let result = tools::find_exports::handle(&ws, json!({"file_path": "sample.ts"}));
    assert_no_error(&result, "ast_find_exports");
    assert_workspace_relative_path(&result, "ast_find_exports");
    assert_truncated_is_bool_if_present(&result, "ast_find_exports");
    let exports = result["exports"].as_array().unwrap();
    assert!(exports.len() >= 2, "expected at least 2 exports in sample.ts");

    let result = tools::find_exports::handle(&ws, json!({"file_path": "sample.py"}));
    assert_no_error(&result, "ast_find_exports");
    let py_exports = result["exports"].as_array().unwrap();
    assert!(!py_exports.is_empty(), "expected at least 1 export in sample.py");
}

// ---------------------------------------------------------------
// Tool 9: ast_find_functions
// ---------------------------------------------------------------
#[test]
fn sweep_find_functions() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    for file in &["sample.ts", "sample.py"] {
        let result = tools::find_functions::handle(&ws, json!({"file_path": file}));
        assert_no_error(&result, "ast_find_functions");
        assert_workspace_relative_path(&result, "ast_find_functions");
        assert_truncated_is_bool_if_present(&result, "ast_find_functions");
        let funcs = result["functions"].as_array().unwrap();
        assert!(funcs.len() >= 2, "{}: expected at least 2 functions, got {}", file, funcs.len());
    }
}

// ---------------------------------------------------------------
// Tool 10: ast_find_classes
// ---------------------------------------------------------------
#[test]
fn sweep_find_classes() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    for file in &["sample.ts", "sample.py"] {
        let result = tools::find_classes::handle(&ws, json!({"file_path": file}));
        assert_no_error(&result, "ast_find_classes");
        assert_workspace_relative_path(&result, "ast_find_classes");
        assert_truncated_is_bool_if_present(&result, "ast_find_classes");
        let classes = result["classes"].as_array().unwrap();
        assert!(!classes.is_empty(), "{}: expected at least 1 class, got {}", file, classes.len());
    }
}

// ---------------------------------------------------------------
// Tool 11: ast_chunk_file
// ---------------------------------------------------------------
#[test]
fn sweep_chunk_file() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    for strategy in &["top_level", "function_class", "semantic_blocks"] {
        let result =
            tools::chunk_file::handle(&ws, json!({"file_path": "sample.ts", "strategy": strategy}));
        assert_no_error(&result, "ast_chunk_file");
        assert_workspace_relative_path(&result, "ast_chunk_file");
        assert_truncated_is_bool_if_present(&result, "ast_chunk_file");
        let chunks = result["chunks"].as_array().unwrap();
        assert!(!chunks.is_empty(), "strategy {}: expected at least 1 chunk", strategy);
        for chunk in chunks {
            assert!(chunk["kind"].is_string(), "chunk missing kind");
            assert!(chunk["text"].is_string(), "chunk missing text");
        }
    }
}

// ---------------------------------------------------------------
// Tool 12: ast_enclosing_node
// ---------------------------------------------------------------
#[test]
fn sweep_enclosing_node() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    // Find enclosing node at a position inside the Processor class
    // The class definition in sample.ts starts around line 15
    let result = tools::enclosing_node::handle(
        &ws,
        json!({"file_path": "sample.ts", "line": 16, "character": 4}),
    );
    assert_no_error(&result, "ast_enclosing_node");
    assert_workspace_relative_path(&result, "ast_enclosing_node");
    assert_truncated_is_bool_if_present(&result, "ast_enclosing_node");
    assert!(result["ancestors"].is_array(), "expected ancestors array, got {:?}", result);
    let ancestors = result["ancestors"].as_array().unwrap();
    assert!(!ancestors.is_empty(), "expected at least 1 ancestor");
}

// ---------------------------------------------------------------
// Cross-cutting: verify field presence on every tool that returns a structured response
// ---------------------------------------------------------------
#[test]
fn sweep_tool_list_count() {
    // Verify the registry reports exactly 23 tools (12 V1 + 11 V2)
    use ast_mcp::mcp::register_tools;
    let dir = tempfile::tempdir().unwrap();
    let ws = workspace(&dir);
    let tool_specs = register_tools::tools(&ws);
    assert_eq!(tool_specs.len(), 39, "expected exactly 39 tools, got {}", tool_specs.len());

    // Every tool must have a name, description, and inputSchema
    for spec in &tool_specs {
        assert!(!spec.name.is_empty());
        assert!(!spec.description.is_empty());
        assert!(spec.input_schema.is_object());
    }

    // Verify all expected tool names are present
    let names: Vec<&str> = tool_specs.iter().map(|s| s.name).collect();
    let expected = &[
        "ast_health_check",
        "ast_list_supported_languages",
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
    for exp in expected {
        assert!(names.contains(exp), "missing tool: {}", exp);
    }
}

// ---------------------------------------------------------------
// Dispatch coverage: verify every registered tool dispatches to a handler
// ---------------------------------------------------------------
#[test]
fn sweep_dispatch_all_tools_return_json() {
    let dir = tempfile::tempdir().unwrap();
    setup_fixtures(&dir);
    let ws = workspace(&dir);

    // Dispatch each tool — verify it returns valid JSON and no crash
    let cases: &[(&str, Value)] = &[
        ("ast_health_check", json!({})),
        ("ast_list_supported_languages", json!({})),
        ("ast_parse_file", json!({"file_path": "sample.ts"})),
        ("ast_file_outline", json!({"file_path": "sample.ts"})),
        ("ast_top_level_nodes", json!({"file_path": "sample.ts"})),
        ("ast_query", json!({"file_path": "sample.ts", "query": "(function_declaration) @f"})),
        ("ast_find_imports", json!({"file_path": "sample.ts"})),
        ("ast_find_exports", json!({"file_path": "sample.ts"})),
        ("ast_find_functions", json!({"file_path": "sample.ts"})),
        ("ast_find_classes", json!({"file_path": "sample.ts"})),
        ("ast_chunk_file", json!({"file_path": "sample.ts"})),
        ("ast_enclosing_node", json!({"file_path": "sample.ts", "line": 0, "character": 0})),
    ];

    for (name, args) in cases {
        let result = ast_mcp::mcp::register_tools::dispatch(name, args.clone(), &ws);
        assert!(result.is_some(), "dispatch({}) returned None — tool not registered", name);
        let val = result.unwrap();
        assert!(val.is_object(), "dispatch({}) returned non-object: {:?}", name, val);
        // Must be valid JSON when serialized
        let _json_str = serde_json::to_string(&val)
            .unwrap_or_else(|e| panic!("dispatch({}) result is not valid JSON: {}", name, e));
    }
}
