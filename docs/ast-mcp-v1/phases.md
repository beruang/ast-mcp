# Phases: AST MCP Server V1

12 phases, mapped 1:1 to the development milestones in `spec/version-1.md` § 33. Each phase is independently understandable, testable, and committable.

## Summary

- **Phase 1:** Rust project skeleton — Cargo crate, stdio transport, one dummy tool.
- **Phase 2:** Workspace safety — `WORKSPACE_PATH` loading, path validation, file size limits, **architectural lint tests** (no-write, no-LSP).
- **Phase 3:** Parser registry — TypeScript, TSX, JavaScript, JSX, Python, extension routing.
- **Phase 4:** Basic parsing — `ast_health_check`, `ast_list_supported_languages`, `ast_parse_file`.
- **Phase 5:** Position conversion — UTF-16 ↔ byte-offset helpers with non-BMP tests.
- **Phase 6:** Outline and top-level nodes — `ast_file_outline`, `ast_top_level_nodes`.
- **Phase 7:** Enclosing node — `ast_enclosing_node` with kind filter and ancestors.
- **Phase 8:** Imports and exports — `ast_find_imports`, `ast_find_exports` for TS/JS/Python.
- **Phase 9:** Functions and classes — `ast_find_functions`, `ast_find_classes` with method/parameter extraction.
- **Phase 10:** Chunking — `ast_chunk_file` with all 4 strategies.
- **Phase 11:** Tree-sitter query — `ast_query` with timeout and capture normalization.
- **Phase 12:** V1 acceptance tests — run all unit, integration, and safety tests; record timings.

## Dependency Graph

```text
phase-1 (skeleton)
  └── phase-2 (workspace safety) ← critical gate
        └── phase-3 (parser registry)
              ├── phase-4 (basic parsing) ← first user-visible tool
              │     └── phase-5 (position conversion)
              │           ├── phase-6 (outline + top-level)
              │           │     └── phase-7 (enclosing node)
              │           │           └── phase-8 (imports/exports)
              │           │                 └── phase-9 (functions/classes)
              │           │                       └── phase-10 (chunking)
              │           │                             └── phase-11 (query)
              │           │                                   └── phase-12 (acceptance)
```

A linear chain is appropriate here: each phase builds on the previous and the spec already enumerates a sequential milestone plan.

## Phase Index

| ID | Title | Depends On | Risk | Spec | Status |
|---|---|---|---:|---|---|
| phase-1 | Rust project skeleton | — | medium | `spec/spec-phase-1.md` | Draft |
| phase-2 | Workspace safety | phase-1 | critical | `spec/spec-phase-2.md` | Draft |
| phase-3 | Parser registry | phase-2 | high | `spec/spec-phase-3.md` | Draft |
| phase-4 | Basic parsing tools | phase-3 | medium | `spec/spec-phase-4.md` | Draft |
| phase-5 | Position conversion | phase-4 | high | `spec/spec-phase-5.md` | Draft |
| phase-6 | Outline and top-level nodes | phase-5 | medium | `spec/spec-phase-6.md` | Draft |
| phase-7 | Enclosing node | phase-6 | low | `spec/spec-phase-7.md` | Draft |
| phase-8 | Imports and exports | phase-7 | medium | `spec/spec-phase-8.md` | Draft |
| phase-9 | Functions and classes | phase-8 | medium | `spec/spec-phase-9.md` | Draft |
| phase-10 | Chunking | phase-9 | medium | `spec/spec-phase-10.md` | Draft |
| phase-11 | Tree-sitter query | phase-10 | medium | `spec/spec-phase-11.md` | Draft |
| phase-12 | V1 acceptance tests | phase-11 | low | `spec/spec-phase-12.md` | Draft |

## Parallelization Notes

Phases 1–4 must run sequentially because they build the foundation. From phase 5 onward the chain is still linear because each phase consumes APIs the previous one stabilized. No two phases share files in a way that requires lock or sequencing beyond the chain.

The only file that multiple phases touch is `src/shared/position.rs` and `src/shared/range.rs` (phase 5 introduces the helpers; phases 6–11 consume them). Phase 5 owns these files; later phases only read.

## Shared File Risks

| File | Risk | Resolution |
|---|---|---|
| `src/safety/paths.rs` | Phase 2 owns; later phases must not bypass it. | All path inputs flow through `safety::paths::resolve`. Lint test enforces a single import. |
| `src/parser/positions.rs` | Phase 5 owns; phases 6–11 consume. | Phase 5 freezes the public API; later phases only add `from`/`to` impls, not new signatures. |
| `src/parser/registry.rs` | Phase 3 owns; later phases add `ParserDefinition` entries only. | Phase 3 freezes the public API. |
| `src/tools/mod.rs` | All phases add one file each. | Each phase adds one file; `mod.rs` is updated once per phase in the same commit. |

## Per-Phase Detail Files

- [`phases/phase-1.md`](./phases/phase-1.md)
- [`phases/phase-2.md`](./phases/phase-2.md)
- [`phases/phase-3.md`](./phases/phase-3.md)
- [`phases/phase-4.md`](./phases/phase-4.md)
- [`phases/phase-5.md`](./phases/phase-5.md)
- [`phases/phase-6.md`](./phases/phase-6.md)
- [`phases/phase-7.md`](./phases/phase-7.md)
- [`phases/phase-8.md`](./phases/phase-8.md)
- [`phases/phase-9.md`](./phases/phase-9.md)
- [`phases/phase-10.md`](./phases/phase-10.md)
- [`phases/phase-11.md`](./phases/phase-11.md)
- [`phases/phase-12.md`](./phases/phase-12.md)
