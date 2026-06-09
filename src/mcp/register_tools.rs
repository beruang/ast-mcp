//! Tool registry — metadata and dispatch for all V1–V5 AST tools.
use serde_json::{json, Value};

use crate::context;
use crate::extraction;
use crate::mcp::server_context::ServerContext;
use crate::metrics;
use crate::tools;
use crate::workspace;

/// Tool specification — pure metadata; no handler function.
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

/// Return metadata for all registered tools.
pub fn tools(_ctx: &ServerContext) -> Vec<ToolSpec> {
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
        // ── V2 tools ──
        ToolSpec {
            name: "ast_enclosing_scope",
            description: "Return the syntactic scope chain (outermost to innermost) enclosing a source position. Returns scope-like containers only: module, class, function, method, etc.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    },
                    "position": {
                        "type": "object",
                        "properties": {
                            "line": { "type": "integer", "description": "0-based line number" },
                            "character": { "type": "integer", "description": "0-based UTF-16 character offset" }
                        },
                        "required": ["line", "character"]
                    },
                    "include_block_scopes": {
                        "type": "boolean",
                        "description": "Include block scopes (default: false)"
                    }
                },
                "required": ["file_path", "position"]
            }),
        },
        ToolSpec {
            name: "ast_node_at_range",
            description: "Return the smallest AST node that exactly matches or contains a source range. Supports exact, smallest_containing, and largest_contained modes.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    },
                    "range": {
                        "type": "object",
                        "properties": {
                            "start": {
                                "type": "object",
                                "properties": {
                                    "line": { "type": "integer" },
                                    "character": { "type": "integer" }
                                },
                                "required": ["line", "character"]
                            },
                            "end": {
                                "type": "object",
                                "properties": {
                                    "line": { "type": "integer" },
                                    "character": { "type": "integer" }
                                },
                                "required": ["line", "character"]
                            }
                        },
                        "required": ["start", "end"]
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["exact", "smallest_containing", "largest_contained"],
                        "description": "Match mode (default: smallest_containing)"
                    },
                    "include_text": {
                        "type": "boolean",
                        "description": "Include source text for the node (default: true)"
                    },
                    "max_text_bytes": {
                        "type": "integer",
                        "description": "Maximum bytes of text to include (default: 12000)"
                    }
                },
                "required": ["file_path", "range"]
            }),
        },
        ToolSpec {
            name: "ast_node_text",
            description: "Return exact source text for a given range without returning the entire file. Enforces max byte limit.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    },
                    "range": {
                        "type": "object",
                        "properties": {
                            "start": {
                                "type": "object",
                                "properties": {
                                    "line": { "type": "integer" },
                                    "character": { "type": "integer" }
                                },
                                "required": ["line", "character"]
                            },
                            "end": {
                                "type": "object",
                                "properties": {
                                    "line": { "type": "integer" },
                                    "character": { "type": "integer" }
                                },
                                "required": ["line", "character"]
                            }
                        },
                        "required": ["start", "end"]
                    },
                    "max_bytes": {
                        "type": "integer",
                        "description": "Maximum bytes to return (default: 20000)"
                    }
                },
                "required": ["file_path", "range"]
            }),
        },
        ToolSpec {
            name: "ast_context_for_range",
            description: "Return minimal useful syntax context around a source range. Includes target node, parent chain, optional siblings, with byte-budget enforcement.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    },
                    "range": {
                        "type": "object",
                        "properties": {
                            "start": {
                                "type": "object",
                                "properties": {
                                    "line": { "type": "integer" },
                                    "character": { "type": "integer" }
                                },
                                "required": ["line", "character"]
                            },
                            "end": {
                                "type": "object",
                                "properties": {
                                    "line": { "type": "integer" },
                                    "character": { "type": "integer" }
                                },
                                "required": ["line", "character"]
                            }
                        },
                        "required": ["start", "end"]
                    },
                    "include_parents": {
                        "type": "boolean",
                        "description": "Include parent node chain (default: true)"
                    },
                    "include_siblings": {
                        "type": "boolean",
                        "description": "Include immediate sibling summaries (default: false)"
                    },
                    "max_parent_depth": {
                        "type": "integer",
                        "description": "Maximum parent chain depth (default: 4)"
                    },
                    "max_context_bytes": {
                        "type": "integer",
                        "description": "Maximum total bytes in response (default: 30000)"
                    }
                },
                "required": ["file_path", "range"]
            }),
        },
        ToolSpec {
            name: "ast_find_calls",
            description: "Find call expressions in a file. Filter by exact callee name or substring. Returns callee text, argument text, and optional enclosing scope.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to the file" },
                    "callee": { "type": "string", "description": "Exact callee name to match" },
                    "callee_contains": { "type": "string", "description": "Substring to match in callee text" },
                    "include_arguments": { "type": "boolean", "description": "Include argument text (default: true)" },
                    "include_enclosing_scope": { "type": "boolean", "description": "Include enclosing scope for each call (default: true)" },
                    "max_results": { "type": "integer", "description": "Maximum calls to return (default: 200)" }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_find_member_access",
            description: "Find member/property access expressions in a file. Filter by property name, object substring, or full-text substring.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to the file" },
                    "property": { "type": "string", "description": "Exact property name to match" },
                    "object_contains": { "type": "string", "description": "Substring to match in object text" },
                    "full_text_contains": { "type": "string", "description": "Substring to match in full member expression text" },
                    "include_enclosing_scope": { "type": "boolean", "description": "Include enclosing scope (default: true)" },
                    "max_results": { "type": "integer", "description": "Maximum results (default: 200)" }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_find_literals",
            description: "Find literals (string, number, boolean, null, regex) in a file. Filter by literal kind, contains, or exact value match.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to the file" },
                    "literal_kind": { "type": "string", "enum": ["string", "number", "boolean", "null", "regex", "unknown"], "description": "Filter by literal kind" },
                    "contains": { "type": "string", "description": "Substring match against value text" },
                    "exact": { "type": "string", "description": "Exact value match" },
                    "include_enclosing_scope": { "type": "boolean", "description": "Include enclosing scope (default: true)" },
                    "max_results": { "type": "integer", "description": "Maximum results (default: 200)" }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_find_template_literals",
            description: "Find template literals and tagged template literals in a file. Filter by tag name or content substring.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to the file" },
                    "tag": { "type": "string", "description": "Filter by tag name (e.g. 'sql', 'gql')" },
                    "contains": { "type": "string", "description": "Substring match against template text" },
                    "include_untagged": { "type": "boolean", "description": "Include untagged templates (default: true)" },
                    "include_enclosing_scope": { "type": "boolean", "description": "Include enclosing scope (default: true)" },
                    "max_results": { "type": "integer", "description": "Maximum results (default: 100)" }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_query_workspace",
            description: "Run a bounded Tree-sitter query across workspace files. Enforces strict limits on files scanned, results returned, and bytes per file.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Tree-sitter query pattern" },
                    "language": { "type": "string", "description": "Target language filter (optional, inferred from file extension if omitted)" },
                    "glob": { "type": "string", "description": "Glob pattern to filter files (e.g. 'src/**/*.ts')" },
                    "max_files": { "type": "integer", "description": "Maximum files to scan (default: 200)" },
                    "max_results": { "type": "integer", "description": "Maximum results to return (default: 1000)" },
                    "max_bytes_per_file": { "type": "integer", "description": "Skip files larger than this (default: 1000000)" },
                    "include_text": { "type": "boolean", "description": "Include source text in captures (default: true)" }
                },
                "required": ["query"]
            }),
        },
        ToolSpec {
            name: "ast_file_metrics",
            description: "Return structural metrics for a file: line count, node count, function count, class count, max nesting depth, and more.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to the file" },
                    "include_function_metrics": { "type": "boolean", "description": "Include per-function metrics (default: false)" }
                },
                "required": ["file_path"]
            }),
        },
        // ── V3 tools ──
        ToolSpec {
            name: "ast_find_schema_definitions",
            description: "Find schema, model, and data-shape definitions. Detects Zod schemas, TypeScript interfaces/types, Pydantic BaseModel, Python dataclasses, SQLAlchemy models, Go structs, and Rust structs/enums. Extracts field names and types.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to analyze" },
                    "glob": { "type": "string", "description": "Glob pattern for workspace scan" },
                    "schema_kinds": { "type": "array", "items": { "type": "string" }, "description": "Filter by schema kind (zod, typescript_interface, pydantic, dataclass, go_struct, rust_struct, rust_enum)" },
                    "max_files": { "type": "integer", "description": "Maximum files to scan (default: 300)" },
                    "max_results": { "type": "integer", "description": "Maximum results (default: 1000)" },
                    "include_fields": { "type": "boolean", "description": "Include field names and types (default: true)" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_find_react_components",
            description: "Find React component definitions in TSX/JSX/TS/JS files. Detects function components, arrow components, class components (extends React.Component), memo/forwardRef wrappers. Returns component kind, export status, props, hooks used, and JSX root element.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to analyze" },
                    "glob": { "type": "string", "description": "Glob pattern for workspace scan" },
                    "max_files": { "type": "integer", "description": "Maximum files to scan (default: 200)" },
                    "max_results": { "type": "integer", "description": "Maximum results (default: 500)" },
                    "include_hooks": { "type": "boolean", "description": "Include hooks used inside each component (default: true)" },
                    "include_jsx_summary": { "type": "boolean", "description": "Include root JSX element name (default: false)" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_find_hooks",
            description: "Find React hooks (built-in and custom) in TSX/JSX/TS/JS files. Detects usages of 14 built-in hooks (useState, useEffect, etc.), custom hook usages (useXxx calls), and custom hook definitions.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to analyze" },
                    "glob": { "type": "string", "description": "Glob pattern for workspace scan" },
                    "max_files": { "type": "integer", "description": "Maximum files to scan (default: 200)" },
                    "max_results": { "type": "integer", "description": "Maximum results (default: 1000)" },
                    "include_usages": { "type": "boolean", "description": "Include hook usages (default: true)" },
                    "include_definitions": { "type": "boolean", "description": "Include custom hook definitions (default: true)" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_find_routes",
            description: "Find route definitions in application code. Detects Express/Fastify/Hono (app.get, router.post), Next.js (exported GET/POST handlers), NestJS (@Controller/@Get), FastAPI (@app.get), Flask (@app.route), Django (path/re_path/url), Go net/http, and Rust axum routes.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to analyze" },
                    "glob": { "type": "string", "description": "Glob pattern for workspace scan" },
                    "frameworks": { "type": "array", "items": { "type": "string" }, "description": "Filter by framework (express, fastify, hono, nextjs, nestjs, fastapi, flask, django, go_http, axum)" },
                    "max_files": { "type": "integer", "description": "Maximum files to scan (default: 200)" },
                    "max_results": { "type": "integer", "description": "Maximum results (default: 500)" },
                    "include_handler_context": { "type": "boolean", "description": "Include handler context (default: false)" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_find_decorators",
            description: "Find decorators, annotations, and attributes in source files. Detects TypeScript decorators (@Decorator), Python decorators (@decorator), and Rust attributes (#[attribute]). Attaches target declaration where possible.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to analyze" },
                    "glob": { "type": "string", "description": "Glob pattern for workspace scan" },
                    "names": { "type": "array", "items": { "type": "string" }, "description": "Filter by decorator names" },
                    "max_files": { "type": "integer", "description": "Maximum files to scan (default: 200)" },
                    "max_results": { "type": "integer", "description": "Maximum results (default: 1000)" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_find_tests",
            description: "Find test definitions in source files. Detects Jest/Vitest/Mocha (describe/it/test), Pytest (test_* functions, Test* classes), unittest, Go tests, and Rust #[test] functions. Returns suite/test/fixture/hook kind with parent names.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to analyze" },
                    "glob": { "type": "string", "description": "Glob pattern for workspace scan" },
                    "frameworks": { "type": "array", "items": { "type": "string" }, "description": "Filter by framework (jest, vitest, mocha, pytest, unittest, go_testing, rust_test)" },
                    "max_files": { "type": "integer", "description": "Maximum files to scan (default: 300)" },
                    "max_results": { "type": "integer", "description": "Maximum results (default: 1000)" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_dependency_edges",
            description: "Extract syntax-level dependency edges (imports, exports, requires, use, mod) from files. Supports TS/JS, Python, Go, and Rust. Filter by relative or external edges.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to a single file to analyze" },
                    "glob": { "type": "string", "description": "Glob pattern for workspace scan (e.g. 'src/**/*.ts')" },
                    "max_files": { "type": "integer", "description": "Maximum files to scan (default: 500)" },
                    "max_results": { "type": "integer", "description": "Maximum edges to return (default: 5000)" },
                    "include_external": { "type": "boolean", "description": "Include external package edges (default: true)" },
                    "include_relative": { "type": "boolean", "description": "Include relative/local edges (default: true)" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_context_pack",
            description: "Return a compact, agent-ready structural context pack for a file position or range. Includes requested parts: imports, exports, enclosing scope, enclosing node, top-level outline.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Workspace-relative path to the file"
                    },
                    "position": {
                        "type": "object",
                        "properties": {
                            "line": { "type": "integer" },
                            "character": { "type": "integer" }
                        },
                        "required": ["line", "character"]
                    },
                    "range": {
                        "type": "object",
                        "properties": {
                            "start": {
                                "type": "object",
                                "properties": {
                                    "line": { "type": "integer" },
                                    "character": { "type": "integer" }
                                },
                                "required": ["line", "character"]
                            },
                            "end": {
                                "type": "object",
                                "properties": {
                                    "line": { "type": "integer" },
                                    "character": { "type": "integer" }
                                },
                                "required": ["line", "character"]
                            }
                        },
                        "required": ["start", "end"]
                    },
                    "include": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["imports", "exports", "enclosing_scope", "enclosing_node", "top_level_outline", "nearby_functions", "nearby_classes"]
                        },
                        "description": "Parts to include (default: imports, exports, enclosing_scope, enclosing_node, top_level_outline)"
                    },
                    "max_bytes": {
                        "type": "integer",
                        "description": "Maximum total bytes in response (default: 30000)"
                    }
                },
                "required": ["file_path"]
            }),
        },
        // ── V4 tools ──
        ToolSpec {
            name: "ast_rewrite_preview",
            description: "Preview structural rewrites: replace, insert before/after, or delete AST nodes. Returns a unified diff, parse-after validation, and safety violations. Never writes files.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operations": {
                        "type": "array",
                        "items": { "type": "object" },
                        "description": "List of rewrite operations (replace_range, replace_node, insert_before_node, insert_after_node, delete_node)"
                    },
                    "include_diff": { "type": "boolean", "description": "Include unified diff in response (default: true)" },
                    "parse_check": { "type": "boolean", "description": "Re-parse modified files to check for syntax errors (default: true)" },
                    "max_changed_files": { "type": "integer", "description": "Maximum files to change (default: 20)" },
                    "max_edits": { "type": "integer", "description": "Maximum edits (default: 200)" }
                },
                "required": ["operations"]
            }),
        },
        ToolSpec {
            name: "ast_validate_rewrite",
            description: "Validate rewrite operations without generating a diff. Returns safety violations, overlap detection, and limit checks.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operations": {
                        "type": "array",
                        "items": { "type": "object" },
                        "description": "List of rewrite operations to validate"
                    },
                    "max_changed_files": { "type": "integer", "description": "Maximum files to change (default: 20)" },
                    "max_edits": { "type": "integer", "description": "Maximum edits (default: 200)" }
                },
                "required": ["operations"]
            }),
        },
        ToolSpec {
            name: "ast_parse_after_rewrite",
            description: "Apply edits in memory and re-parse changed files to check for syntax errors. Useful for validating externally-generated edits before applying them.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "edits": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "file_path": { "type": "string" },
                                "range": {
                                    "type": "object",
                                    "properties": {
                                        "start": { "type": "object", "properties": { "line": { "type": "integer" }, "character": { "type": "integer" } } },
                                        "end": { "type": "object", "properties": { "line": { "type": "integer" }, "character": { "type": "integer" } } }
                                    }
                                },
                                "new_text": { "type": "string" }
                            }
                        },
                        "description": "List of text edits to apply and validate"
                    },
                    "max_changed_files": { "type": "integer", "description": "Maximum files (default: 20)" },
                    "max_edits": { "type": "integer", "description": "Maximum edits (default: 200)" }
                },
                "required": ["edits"]
            }),
        },
        ToolSpec {
            name: "ast_insert_import_preview",
            description: "Preview adding or merging an import statement. Supports TypeScript/JavaScript (ES imports) and Python (import/from). Does not guarantee the imported symbol exists — semantic validation belongs to LSP.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to the file" },
                    "import": {
                        "type": "object",
                        "properties": {
                            "source": { "type": "string", "description": "Module path to import from" },
                            "default_import": { "type": "string", "description": "Default import name" },
                            "named_imports": { "type": "array", "items": { "type": "string" }, "description": "Named imports" },
                            "namespace_import": { "type": "string", "description": "Namespace import (* as name)" },
                            "is_type_only": { "type": "boolean", "description": "Type-only import (TS/JS only)" }
                        },
                        "required": ["source"]
                    },
                    "include_diff": { "type": "boolean", "description": "Include unified diff (default: true)" },
                    "parse_check": { "type": "boolean", "description": "Re-parse after rewrite (default: true)" }
                },
                "required": ["file_path", "import"]
            }),
        },
        ToolSpec {
            name: "ast_remove_unused_import_preview",
            description: "Preview removal of syntactically unused imports. Never removes side-effect imports (import 'mod'). Syntax-level approximation — not semantic analysis.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Workspace-relative path to the file" },
                    "import_names": { "type": "array", "items": { "type": "string" }, "description": "Specific import names to consider (omit for all)" },
                    "include_diff": { "type": "boolean", "description": "Include unified diff (default: true)" },
                    "parse_check": { "type": "boolean", "description": "Re-parse after rewrite (default: true)" }
                },
                "required": ["file_path"]
            }),
        },
        ToolSpec {
            name: "ast_rename_local_preview",
            description: "Preview renaming a local variable/parameter within its scope. Conservative — rejects imported/exported/top-level symbols. For cross-file semantic rename, use LSP.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string" },
                    "position": { "type": "object", "properties": { "line": { "type": "integer" }, "character": { "type": "integer" } }, "required": ["line", "character"] },
                    "new_name": { "type": "string" },
                    "scope_range": { "type": "object", "description": "Optional scope boundary" },
                    "include_diff": { "type": "boolean" },
                    "parse_check": { "type": "boolean" }
                },
                "required": ["file_path", "position", "new_name"]
            }),
        },
        ToolSpec {
            name: "ast_wrap_node_preview",
            description: "Preview wrapping a syntax node with prefix/suffix, try/catch, or call expression wrapper. Preserves indentation.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string" },
                    "range": { "type": "object" },
                    "expected_node_kind": { "type": "string" },
                    "wrapper": { "type": "object", "description": "Wrapper: {kind: prefix_suffix|try_catch|call_expression, ...}" },
                    "include_diff": { "type": "boolean" },
                    "parse_check": { "type": "boolean" }
                },
                "required": ["file_path", "range", "wrapper"]
            }),
        },
        ToolSpec {
            name: "ast_add_decorator_preview",
            description: "Preview adding a decorator/attribute to a class, method, function, or field. Supports TS/JS/Python/Rust. Preserves indentation.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string" },
                    "target_range": { "type": "object" },
                    "decorator_text": { "type": "string" },
                    "expected_target_kind": { "type": "string" },
                    "include_diff": { "type": "boolean" },
                    "parse_check": { "type": "boolean" }
                },
                "required": ["file_path", "target_range", "decorator_text"]
            }),
        },
        ToolSpec {
            name: "ast_modify_function_signature_preview",
            description: "Preview modifying a function/method signature: add/remove/rename parameters or replace the full signature. Never updates call sites.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string" },
                    "function_range": { "type": "object" },
                    "operation": { "type": "object", "description": "{kind: replace_signature|add_parameter|remove_parameter|rename_parameter, ...}" },
                    "include_diff": { "type": "boolean" },
                    "parse_check": { "type": "boolean" }
                },
                "required": ["file_path", "function_range", "operation"]
            }),
        },
        // ── V5 cache tools ──
        ToolSpec {
            name: "ast_cache_status",
            description: "Return cache sizes, TTLs, and cache health for parse tree, query result, framework result, and request log caches.",
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_clear_caches",
            description: "Clear selected AST caches. Clearing parse trees also clears dependent query and framework result caches.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "caches": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["parse_trees", "query_results", "framework_results", "request_log", "all"]
                        },
                        "description": "Cache names to clear"
                    }
                },
                "required": ["caches"]
            }),
        },
        // ── V5 config tools ──
        ToolSpec {
            name: "ast_get_config",
            description: "Return effective runtime configuration. Optionally include the source breakdown (defaults, environment, runtime overrides).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "include_defaults": {
                        "type": "boolean",
                        "description": "Include defaults/environment/runtime overrides breakdown (default: false)"
                    }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_update_runtime_config",
            description: "Update safe runtime configuration values in memory. Rejects updates to workspace_path, parser registry, and language grammar paths.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limits": { "type": "object", "description": "Partial RuntimeLimits" },
                    "timeouts_ms": { "type": "object", "description": "Partial RuntimeTimeouts" },
                    "caches": { "type": "object", "description": "Partial RuntimeCaches" },
                    "scans": { "type": "object", "description": "Partial RuntimeScans" },
                    "debug": { "type": "object", "description": "Partial RuntimeDebug" }
                },
                "required": []
            }),
        },
        // ── V5 observability tools ──
        ToolSpec {
            name: "ast_request_log",
            description: "Return recent AST MCP request history. Filter by tool name, status (ok/error/timeout/cancelled), or file path.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tool": { "type": "string", "description": "Filter by tool name" },
                    "status": {
                        "type": "string",
                        "enum": ["ok", "error", "timeout", "cancelled"],
                        "description": "Filter by request status"
                    },
                    "file_path": { "type": "string", "description": "Filter by file path (substring match)" },
                    "limit": { "type": "integer", "description": "Max entries to return (default: 50)" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_clear_request_log",
            description: "Clear request log entries. Optionally filter by tool name to clear only matching entries.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tool": { "type": "string", "description": "If provided, only clear entries for this tool" }
                },
                "required": []
            }),
        },
        // ── V5 health tools ──
        ToolSpec {
            name: "ast_readiness",
            description: "Report whether the AST MCP server is ready to serve structural analysis requests. Checks workspace, parser registry, and cache initialization.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "require_languages": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional list of required languages"
                    }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_liveness",
            description: "Report whether the AST MCP server process is alive and internally responsive. Returns uptime and optional memory usage.",
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_parser_status",
            description: "Return parser registry and parser health. Optionally filter by language.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "language": { "type": "string", "description": "Optional language filter" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_rebuild_parser_cache",
            description: "Clear and rebuild parser-related caches. Does not recompile parsers; refreshes runtime parser/query cache objects.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "languages": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Languages to rebuild (omit for all)"
                    }
                },
                "required": []
            }),
        },
        // ── V5 workspace scan tools ──
        ToolSpec {
            name: "ast_workspace_scan_status",
            description: "Return status of currently running workspace-wide scans. Optionally filter by scan ID.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "scan_id": { "type": "string", "description": "Optional scan ID to query" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_cancel_workspace_scan",
            description: "Cancel a running workspace-wide scan by scan ID.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "scan_id": { "type": "string", "description": "Scan ID to cancel" }
                },
                "required": ["scan_id"]
            }),
        },
        // ── V5 analysis tools ──
        ToolSpec {
            name: "ast_complexity_summary",
            description: "Return structural complexity information for one file or a bounded workspace scan. Includes hotspot detection with risk heuristics.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Single file to analyze" },
                    "glob": { "type": "string", "description": "Glob pattern for workspace scan" },
                    "max_files": { "type": "integer", "description": "Max files to scan (default: 200)" },
                    "include_functions": { "type": "boolean", "description": "Include function-level analysis (default: true)" },
                    "include_classes": { "type": "boolean", "description": "Include class-level analysis (default: true)" },
                    "max_results": { "type": "integer", "description": "Max hotspots to return (default: 500)" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_detect_large_nodes",
            description: "Find large syntax nodes such as huge functions, classes, components, test suites, or modules.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Single file to analyze" },
                    "glob": { "type": "string", "description": "Glob pattern for workspace scan" },
                    "max_files": { "type": "integer", "description": "Max files to scan (default: 200)" },
                    "min_lines": { "type": "integer", "description": "Minimum line threshold (default: 80)" },
                    "node_kinds": { "type": "array", "items": { "type": "string" }, "description": "Filter by node kinds" },
                    "max_results": { "type": "integer", "description": "Max results (default: 200)" }
                },
                "required": []
            }),
        },
        ToolSpec {
            name: "ast_detect_duplicate_shapes",
            description: "Detect structurally similar code shapes using heuristic fingerprinting. Supports identifier and literal normalization.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "glob": { "type": "string", "description": "Glob pattern for workspace scan" },
                    "max_files": { "type": "integer", "description": "Max files to scan (default: 200)" },
                    "min_node_lines": { "type": "integer", "description": "Minimum node lines (default: 10)" },
                    "node_kinds": { "type": "array", "items": { "type": "string" }, "description": "Filter by node kinds" },
                    "normalize_identifiers": { "type": "boolean", "description": "Normalize identifiers (default: true)" },
                    "normalize_literals": { "type": "boolean", "description": "Normalize literals (default: true)" },
                    "max_candidates": { "type": "integer", "description": "Max candidates (default: 200)" }
                },
                "required": ["glob"]
            }),
        },
    ]
}

/// Dispatch a tool call by name to its handler.
/// Returns `None` if the tool name is not registered.
pub fn dispatch(name: &str, arguments: Value, ctx: &ServerContext) -> Option<Value> {
    match name {
        "ast_health_check" => Some(tools::health_check::handle(&ctx.workspace, arguments)),
        "ast_list_supported_languages" => Some(tools::list_supported_languages::handle(arguments)),
        "ast_parse_file" => Some(tools::parse_file::handle(&ctx.workspace, arguments)),
        "ast_file_outline" => Some(tools::file_outline::handle(&ctx.workspace, arguments)),
        "ast_top_level_nodes" => Some(tools::top_level_nodes::handle(&ctx.workspace, arguments)),
        "ast_query" => Some(tools::query::handle(&ctx.workspace, arguments)),
        "ast_find_imports" => Some(tools::find_imports::handle(&ctx.workspace, arguments)),
        "ast_find_exports" => Some(tools::find_exports::handle(&ctx.workspace, arguments)),
        "ast_find_functions" => Some(tools::find_functions::handle(&ctx.workspace, arguments)),
        "ast_find_classes" => Some(tools::find_classes::handle(&ctx.workspace, arguments)),
        "ast_chunk_file" => Some(tools::chunk_file::handle(&ctx.workspace, arguments)),
        "ast_enclosing_node" => Some(tools::enclosing_node::handle(&ctx.workspace, arguments)),
        // V2 tools
        "ast_enclosing_scope" => Some(context::enclosing_scope::handle(&ctx.workspace, arguments)),
        "ast_node_at_range" => Some(context::node_at_range::handle(&ctx.workspace, arguments)),
        "ast_node_text" => Some(context::node_text::handle(&ctx.workspace, arguments)),
        "ast_context_for_range" => {
            Some(context::context_for_range::handle(&ctx.workspace, arguments))
        }
        "ast_context_pack" => Some(context::context_pack::handle(&ctx.workspace, arguments)),
        "ast_find_calls" => Some(extraction::calls::handle(&ctx.workspace, arguments)),
        "ast_find_member_access" => {
            Some(extraction::member_access::handle(&ctx.workspace, arguments))
        }
        "ast_find_literals" => Some(extraction::literals::handle(&ctx.workspace, arguments)),
        "ast_find_template_literals" => {
            Some(extraction::template_literals::handle(&ctx.workspace, arguments))
        }
        "ast_query_workspace" => {
            Some(workspace::query_workspace::handle(&ctx.workspace, arguments))
        }
        "ast_file_metrics" => Some(metrics::file_metrics::handle(&ctx.workspace, arguments)),
        // V3 tools
        "ast_find_schema_definitions" => {
            Some(tools::find_schema_definitions::handle(&ctx.workspace, arguments))
        }
        "ast_find_react_components" => {
            Some(tools::find_react_components::handle(&ctx.workspace, arguments))
        }
        "ast_find_hooks" => Some(tools::find_hooks::handle(&ctx.workspace, arguments)),
        "ast_find_routes" => Some(tools::find_routes::handle(&ctx.workspace, arguments)),
        "ast_find_decorators" => Some(tools::find_decorators::handle(&ctx.workspace, arguments)),
        "ast_find_tests" => Some(tools::find_tests::handle(&ctx.workspace, arguments)),
        "ast_dependency_edges" => {
            Some(tools::find_dependency_edges::handle(&ctx.workspace, arguments))
        }
        // V4 tools
        "ast_rewrite_preview" => Some(tools::rewrite_preview::handle(&ctx.workspace, arguments)),
        "ast_validate_rewrite" => Some(tools::validate_rewrite::handle(&ctx.workspace, arguments)),
        "ast_parse_after_rewrite" => {
            Some(tools::parse_after_rewrite::handle(&ctx.workspace, arguments))
        }
        "ast_insert_import_preview" => {
            Some(tools::insert_import::handle(&ctx.workspace, arguments))
        }
        "ast_remove_unused_import_preview" => {
            Some(tools::remove_unused_import::handle(&ctx.workspace, arguments))
        }
        "ast_rename_local_preview" => Some(tools::rename_local::handle(&ctx.workspace, arguments)),
        "ast_wrap_node_preview" => Some(tools::wrap_node::handle(&ctx.workspace, arguments)),
        "ast_add_decorator_preview" => {
            Some(tools::add_decorator::handle(&ctx.workspace, arguments))
        }
        "ast_modify_function_signature_preview" => {
            Some(tools::modify_signature::handle(&ctx.workspace, arguments))
        }
        // V5 tools
        "ast_get_config" => Some(tools::get_config::handle(&ctx.runtime_config, arguments)),
        "ast_update_runtime_config" => {
            Some(tools::update_runtime_config::handle(&ctx.runtime_config, arguments))
        }
        "ast_request_log" => Some(tools::request_log::handle(&ctx.request_tracker, arguments)),
        "ast_clear_request_log" => {
            Some(tools::clear_request_log::handle(&ctx.request_tracker, arguments))
        }
        "ast_cache_status" => {
            Some(tools::cache_status::handle(&ctx.cache_manager, &ctx.request_tracker))
        }
        "ast_clear_caches" => {
            Some(tools::clear_caches::handle(&ctx.cache_manager, &ctx.request_tracker, arguments))
        }
        "ast_readiness" => Some(tools::readiness::handle(&ctx.workspace, arguments)),
        "ast_liveness" => Some(tools::liveness::handle(ctx.started_at)),
        "ast_parser_status" => Some(tools::parser_status_tool::handle(arguments)),
        "ast_rebuild_parser_cache" => Some(tools::rebuild_parser_cache_tool::handle(arguments)),
        "ast_workspace_scan_status" => {
            Some(tools::workspace_scan_status::handle(&ctx.scan_registry, arguments))
        }
        "ast_cancel_workspace_scan" => {
            Some(tools::cancel_workspace_scan::handle(&ctx.scan_registry, arguments))
        }
        "ast_complexity_summary" => {
            Some(tools::complexity_summary::handle(&ctx.workspace, arguments))
        }
        "ast_detect_large_nodes" => {
            Some(tools::detect_large_nodes::handle(&ctx.workspace, arguments))
        }
        "ast_detect_duplicate_shapes" => Some(tools::detect_duplicate_shapes::handle(arguments)),
        _ => None,
    }
}
