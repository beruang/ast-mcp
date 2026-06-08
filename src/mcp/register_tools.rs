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
            name: "ast_find_functions",
            description: "Find all function definitions in a source file. Returns each function's kind, name, parameters, return type, and source range.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    },
                    "include_anonymous": {
                        "type": "boolean",
                        "description": "Include anonymous/lambda/arrow functions (default: true)"
                    },
                    "include_parameters": {
                        "type": "boolean",
                        "description": "Include parameter details for each function (default: true)"
                    },
                    "include_return_type": {
                        "type": "boolean",
                        "description": "Include return type annotations (default: true)"
                    },
                    "include_signature": {
                        "type": "boolean",
                        "description": "Include the full function signature text (default: true)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of functions to return (default: 200)"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_find_classes",
            description: "Find all class definitions in a source file. Returns each class's name, extends/implements, methods, and source range.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    },
                    "include_methods": {
                        "type": "boolean",
                        "description": "Include methods for each class (default: true)"
                    },
                    "include_extends": {
                        "type": "boolean",
                        "description": "Include extends/superclass information (default: true)"
                    },
                    "include_implements": {
                        "type": "boolean",
                        "description": "Include implements information for TS/JS (default: true)"
                    },
                    "include_decorators": {
                        "type": "boolean",
                        "description": "Include decorator annotations for TS/JS (default: true)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of classes to return (default: 200)"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_chunk_file",
            description: "Split a source file into chunks using a chosen strategy (top_level, function_class, semantic_blocks, or max_lines).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    },
                    "strategy": {
                        "type": "string",
                        "description": "Chunking strategy: top_level, function_class, semantic_blocks, or max_lines (default: top_level)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of chunks to return (default: 200)"
                    },
                    "max_lines_per_chunk": {
                        "type": "integer",
                        "description": "Maximum lines per chunk for max_lines strategy (default: 120)"
                    },
                    "max_bytes_per_chunk": {
                        "type": "integer",
                        "description": "Maximum bytes per chunk text (default: 30000)"
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
        "ast_file_outline" => Some(tools::file_outline::handle(workspace, arguments)),
        "ast_top_level_nodes" => Some(tools::top_level_nodes::handle(workspace, arguments)),
        "ast_query" => Some(tools::query::handle(workspace, arguments)),
        "ast_find_imports" => Some(tools::find_imports::handle(workspace, arguments)),
        "ast_find_exports" => Some(tools::find_exports::handle(workspace, arguments)),
        "ast_find_functions" => Some(tools::find_functions::handle(workspace, arguments)),
        "ast_find_classes" => Some(tools::find_classes::handle(workspace, arguments)),
        "ast_chunk_file" => Some(tools::chunk_file::handle(workspace, arguments)),
        _ => None,
    }
}
