//! Tool registry — metadata and dispatch for all V1 and V2 AST tools.
use serde_json::{json, Value};

use crate::config::workspace::Workspace;
use crate::context;
use crate::extraction;
use crate::metrics;
use crate::tools;
use crate::workspace;

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
        "ast_enclosing_node" => Some(tools::enclosing_node::handle(workspace, arguments)),
        // V2 tools
        "ast_enclosing_scope" => Some(context::enclosing_scope::handle(workspace, arguments)),
        "ast_node_at_range" => Some(context::node_at_range::handle(workspace, arguments)),
        "ast_node_text" => Some(context::node_text::handle(workspace, arguments)),
        "ast_context_for_range" => Some(context::context_for_range::handle(workspace, arguments)),
        "ast_context_pack" => Some(context::context_pack::handle(workspace, arguments)),
        "ast_find_calls" => Some(extraction::calls::handle(workspace, arguments)),
        "ast_find_member_access" => Some(extraction::member_access::handle(workspace, arguments)),
        "ast_find_literals" => Some(extraction::literals::handle(workspace, arguments)),
        "ast_find_template_literals" => {
            Some(extraction::template_literals::handle(workspace, arguments))
        }
        "ast_query_workspace" => Some(workspace::query_workspace::handle(workspace, arguments)),
        "ast_file_metrics" => Some(metrics::file_metrics::handle(workspace, arguments)),
        // V3 tools
        "ast_find_schema_definitions" => {
            Some(tools::find_schema_definitions::handle(workspace, arguments))
        }
        "ast_find_react_components" => {
            Some(tools::find_react_components::handle(workspace, arguments))
        }
        "ast_find_hooks" => Some(tools::find_hooks::handle(workspace, arguments)),
        "ast_find_routes" => Some(tools::find_routes::handle(workspace, arguments)),
        "ast_find_decorators" => Some(tools::find_decorators::handle(workspace, arguments)),
        "ast_find_tests" => Some(tools::find_tests::handle(workspace, arguments)),
        "ast_dependency_edges" => Some(tools::find_dependency_edges::handle(workspace, arguments)),
        _ => None,
    }
}
