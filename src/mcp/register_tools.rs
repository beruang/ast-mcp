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
    ]
}

/// Dispatch a tool call by name to its handler.
/// Returns `None` if the tool name is not registered.
pub fn dispatch(name: &str, arguments: Value, workspace: &Workspace) -> Option<Value> {
    match name {
        "ast_health_check" => Some(tools::health_check::handle(workspace, arguments)),
        "ast_list_supported_languages" => Some(tools::list_supported_languages::handle(arguments)),
        "ast_parse_file" => Some(tools::parse_file::handle(workspace, arguments)),
        _ => None,
    }
}
