# Contract: AST MCP Server V1

**Status:** Draft
**Created:** 2026-06-08
**Updated:** 2026-06-08
**Confidence Score:** 98/100
**Project Slug:** `ast-mcp-v1`
**Source Input:** `spec/version-1.md` (2,404 lines, 38 sections, 12 tools, 5 languages)
**Target Repository:** `/Volumes/Workspace/rnd/workflow/mcp/ast/` (Rust crate to be created at the project root)

## Summary

Build a Rust-based MCP stdio server that exposes 12 `ast_*` tools for parsing and structural inspection of TypeScript, TSX, JavaScript, JSX, and Python source files. The server is read-only, scope-bounded to a single workspace root, and structurally grounded in Tree-sitter. It complements the LSP MCP server and explicitly disclaims any semantic responsibility.

## Confidence Summary

| Dimension | Score | Reason |
|---|---:|---|
| Problem Clarity | 20 | Spec section 1 + section 2 state the problem, the architectural boundary, and the LSP vs AST split explicitly. |
| Goal Definition | 20 | 12 tools, 5 languages, exact tech stack, file layout, and milestones are enumerated in the spec. |
| Success Criteria | 19 | 11 acceptance criteria groups in section 34 cover all 12 tools. Small gap: one item ("never call LSP services") needs an architectural lint test. |
| Scope Boundaries | 20 | Section 5 explicitly enumerates 17 V1 non-goals, reinforced by the "Final Design Principle" (section 38). |
| Consistency | 19 | No contradictions; "Final Design Principle" reinforces the boundary. Tiny tension: chunking strategies in section 23 are partially overlapping, will resolve in spec. |

## Problem Statement

AI agents working on multi-language codebases need fast, safe, structural code intelligence (what does this file contain, what node surrounds this position, what functions and classes exist, how should I chunk this file). The existing LSP MCP server provides semantic intelligence but is heavy, requires language servers, and is not always available. AST MCP fills the structural gap with a read-only, scope-bounded, language-server-free service.

Detailed: [`contract/problem.md`](./contract/problem.md)

## Goals

- Ship a Rust stdio MCP server exposing exactly 12 `ast_*` tools.
- Support TypeScript, TSX, JavaScript, JSX, and Python at the parser level.
- Enforce workspace-relative paths, bounded output, and a 1 MiB per-file cap.
- Provide syntax-aware chunking and bounded Tree-sitter queries for agent workflows.
- Remain usable when LSP MCP is unavailable; never call any `lsp_*` tool, language server, or semantic engine.

Detailed: [`contract/goals.md`](./contract/goals.md)

## Success Criteria

- `ast_health_check`, `ast_list_supported_languages`, `ast_parse_file` work for all 5 languages.
- `ast_file_outline`, `ast_top_level_nodes`, `ast_enclosing_node` return bounded, source-ordered structural data.
- `ast_find_imports`, `ast_find_exports`, `ast_find_functions`, `ast_find_classes` produce best-effort extraction for TS/JS/Python.
- `ast_chunk_file` supports all four strategies (`top_level`, `function_class`, `semantic_blocks`, `max_lines_with_ast_boundaries`).
- `ast_query` compiles and runs bounded Tree-sitter queries with structured errors on invalid input.
- All path inputs are validated; all output is bounded; no tool writes files.

Detailed: [`contract/success-criteria.md`](./contract/success-criteria.md)

## Scope Boundaries

**In scope (V1):** 12 tools, 5 languages, Tree-sitter parsing, workspace safety, bounded output, position/range conversion, language-specific extractors, unit + integration + safety tests.

**Out of scope (V1):** LSP integration, semantic references, type resolution, compiler diagnostics, workspace-wide indexing, framework-aware extraction, AST rewrites, file mutation, persistent state, remote or multi-root workspaces.

Detailed: [`contract/scope.md`](./contract/scope.md)

## Constraints

- Read-only. The server may not write, rename, create, delete, or format files.
- No LSP. No `lsp_*` tool calls. No language server processes. No TypeScript service. No Pyright. No gopls. No rust-analyzer.
- Single workspace root per server process, supplied via `WORKSPACE_PATH` env var or CWD.
- Public API uses UTF-16 character positions; Tree-sitter is byte-based. Conversion helpers are required.
- Default limits are constants in V1: 500 nodes, 200 results, 20,000 text bytes, 120 chunk lines, 30,000 chunk bytes, 200 query matches, 1 MiB files, 5 s parse and query timeouts.
- Responses are JSON. Errors are structured (`AstToolError`). No prose.

Detailed: [`contract/constraints.md`](./contract/constraints.md)

## Assumptions

Detailed: [`contract/assumptions.md`](./contract/assumptions.md)

## Decisions

Detailed: [`contract/decisions.md`](./contract/decisions.md)

## Risks

Detailed: [`contract/risks.md`](./contract/risks.md)

## Approval

- Status: **Approved**
- Approved By: user (via /brainstorm clarification)
- Approved At: 2026-06-08

## Changelog

- 2026-06-08 — Initial draft generated from `spec/version-1.md`. Confidence 98/100. Approved with the target repo set to `/Volumes/Workspace/rnd/workflow/mcp/ast/`.
