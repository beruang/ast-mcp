//! Tool registry — metadata and dispatch for all V1 AST tools.
use serde_json::{json, Value};

use crate::config::workspace::Workspace;
use crate::tools;

/// Tool specification — pure metadata; no handler function.
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

/// Return metadata for all registered V1 tools.
pub fn tools(_workspace: &Workspace) -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "ast_health_check",
            description: "Health-check. Returns workspace path, available parsers, and configured limits.",
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_list_supported_languages",
            description: "List all languages for which a Tree-sitter parser is available.",
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_parse_file",
            description: "Parse a single file. Returns root-kind, node count, parse time, and optionally a depth-limited syntax tree.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file to parse"
                    },
                    "include_tree": {
                        "type": "boolean",
                        "description": "Include the full syntax tree in the response (default: false)"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth for the syntax tree (default: 3)"
                    },
                    "include_node_text": {
                        "type": "boolean",
                        "description": "Include source text for each node (default: false)"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_file_outline",
            description: "Extract a structured outline from a source file. Returns a list of outline nodes and a deterministic plain-text representation.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file to outline"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth for outline nodes (default: 4)"
                    },
                    "include_ranges": {
                        "type": "boolean",
                        "description": "Include source ranges for each outline node (default: true)"
                    },
                    "include_imports": {
                        "type": "boolean",
                        "description": "Include import statements in the outline (default: false)"
                    },
                    "include_exports": {
                        "type": "boolean",
                        "description": "Include export statements in the outline (default: false)"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_top_level_nodes",
            description: "List all top-level named nodes in a source file with their kinds, names, and ranges.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_query",
            description: "Run a Tree-sitter query against a source file. Returns pattern matches with node kinds, names, ranges, and optional text.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file to query"
                    },
                    "query": {
                        "type": "string",
                        "description": "Tree-sitter query pattern string"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of matches to return (default: 200)"
                    },
                    "include_node_text": {
                        "type": "boolean",
                        "description": "Include source text for each capture (default: true)"
                    },
                    "max_text_bytes": {
                        "type": "integer",
                        "description": "Maximum bytes of text to include per capture (default: 20000)"
                    }
                },
                "required": ["file_path", "query"]
            }),
        },
        ToolSpec {
            name: "ast_find_imports",
            description: "Find all import statements in a source file. Returns each import's kind, module path, imported names, and source range.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_find_exports",
            description: "Find all export statements in a source file. Returns each export's kind, name, source range, and re-export source if applicable.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    },
                    "include_best_effort_python": {
                        "type": "boolean",
                        "description": "Include best-effort Python public definitions and __all__ (default: true)"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_enclosing_node",
            description: "Find the enclosing AST node at a specific line/character position. Returns the ancestor chain (outermost first) with optional kind filtering.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    },
                    "line": {
                        "type": "integer",
                        "description": "0-based line number"
                    },
                    "character": {
                        "type": "integer",
                        "description": "0-based UTF-16 character offset within the line"
                    },
                    "kinds": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional whitelist of node kinds to include in the ancestor chain"
                    },
                    "include_source_text": {
                        "type": "boolean",
                        "description": "Include source text for each ancestor node (default: false)"
                    }
                },
                "required": ["file_path", "line", "character"]
            }),
        },
    ]
}

/// Dispatch a tool call by name to its handler.
/// Returns `None` if the tool name is not registered.
pub fn dispatch(name: &str, arguments: Value, workspace: &Workspace) -> Option<Value> {
    match name {
        "ast_health_check" => Some(tools::health_check::handle(workspace, arguments)),
        "ast_list_supported_languages" => Some(tools::list_supported_languages::handle(arguments)),
        "ast_parse_file" => Some(tools::parse_file::handle(workspace, arguments)),
        "ast_file_outline" => Some(tools::file_outline::handle(workspace, arguments)),
        "ast_top_level_nodes" => Some(tools::top_level_nodes::handle(workspace, arguments)),
        "ast_query" => Some(tools::query::handle(workspace, arguments)),
        "ast_find_imports" => Some(tools::find_imports::handle(workspace, arguments)),
        "ast_find_exports" => Some(tools::find_exports::handle(workspace, arguments)),
        "ast_enclosing_node" => Some(tools::enclosing_node::handle(workspace, arguments)),
        _ => None,
    }
}
