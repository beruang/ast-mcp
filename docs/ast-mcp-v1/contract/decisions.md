# Decisions

Decisions made while drafting the contract. Each decision records a choice, the reason, the alternatives considered, and the impact.

## D1. V1 scope is exactly the 12 tools named in `spec/version-1.md` § 4

- **Reason:** The spec is unambiguous and the tool surface must match exactly for downstream orchestration.
- **Alternatives:** Adding or merging tools. Rejected — would diverge from spec.
- **Impact:** Public tool names, input/output schemas, and tool count are locked.

## D2. Rust + Tree-sitter as the parser layer

- **Reason:** Recommended by the spec; matches agent-server norms; Tree-sitter is the de facto standard for syntax trees across all 5 V1 languages.
- **Alternatives:** Hand-rolled regex parsers, ANTL R, or out-of-process parsers (WASM, native binaries). Rejected — too slow, too brittle, or too heavy.
- **Impact:** Lock the `tree-sitter` and language crate versions in `Cargo.toml`. The `parser/registry.rs` module is the single point of extension for future languages.

## D3. Single workspace root per process

- **Reason:** Spec § 7. Multi-root is a V5 concern.
- **Alternatives:** Multi-root from day one. Rejected — adds a complexity tax on every tool and a containment check on every path.
- **Impact:** `WORKSPACE_PATH` env var or CWD. All path resolution goes through `safety::paths`.

## D4. Read-only server, no caches in V1

- **Reason:** Spec § 8.2 + § 5. Caching is a V5 concern.
- **Alternatives:** In-memory parse cache. Rejected — V1 must not pre-build state that V5 will redesign.
- **Impact:** Each request that needs a parse re-parses. Tests can add a per-test cache to keep integration tests fast, but the production binary has no cache.

## D5. Public positions are UTF-16

- **Reason:** Spec § 9. Aligns with LSP contract for future orchestration.
- **Alternatives:** Byte offsets, or UTF-8 code-point offsets. Rejected — diverges from the LSP MCP that V3+ workflows will need to interoperate with.
- **Impact:** Conversion helpers in `parser/positions.rs`. Tests for non-BMP characters.

## D6. Bounded output is mandatory, not optional

- **Reason:** Spec § 8.3. AST output can be huge; unbounded responses can OOM the client.
- **Alternatives:** Streaming pagination. Rejected — V1 keeps a simple request/response shape; pagination is a V2 concern.
- **Impact:** Every list- and text-returning tool has a `maxXxx` parameter and a `truncated` field. Tests verify truncation.

## D7. Errors are structured JSON, never prose

- **Reason:** Spec § 12 + § 28. Tools are consumed by agents, not humans; free-text errors are unparseable.
- **Alternatives:** MCP error codes only. Rejected — V1 uses an in-payload `error` object with `code`, `message`, and `details`.
- **Impact:** A single error-shape helper. All tools funnel through it.

## D8. Internal architecture follows `spec/version-1.md` § 29

- **Reason:** The spec lists `mcp/`, `config/`, `safety/`, `parser/`, `languages/`, `extractors/`, `tools/`, `shared/`, `utils/`. This layout cleanly separates transport, parsing, and tool handlers.
- **Alternatives:** Flat module layout, or feature-crate layout. Rejected — would obscure the boundary between transport, parser, and extractor responsibilities.
- **Impact:** File layout is locked. The `extractors/` module is the primary place where language-specific code lives; `tools/` is one thin file per MCP tool.

## D9. The 12 development milestones become the phase plan

- **Reason:** Spec § 33 enumerates 12 milestones, each a small, testable, committable unit. The phasing guide recommends phase-per-milestone for projects that already come with a clear, sequential plan.
- **Alternatives:** Fewer, larger phases (e.g., "infrastructure" + "tools"). Rejected — would conflate independent units of work and block parallelization.
- **Impact:** 12 phases, mapped 1:1 to milestones, with phases 1–4 (skeleton → safety → parsers → basic parsing) being high-risk and gates to the rest.

## D10. V2–V5 are deferred, not pre-built

- **Reason:** Spec § 36. The V1 design must keep extension points open, but the implementation must not pre-build V2 capabilities.
- **Alternatives:** Build a "plugin" layer in V1. Rejected — V5 will add this properly.
- **Impact:** No premature abstraction. `parser/registry.rs` is extensible by addition (new `ParserDefinition`), not by configuration.

## D11. Architectural lint tests enforce "no write" and "no LSP"

- **Reason:** Constraints `C1` (no file mutation) and `C2` (no LSP) are the most important invariants in the contract. Manual review is error-prone.
- **Alternatives:** Code review only. Rejected — the contract is firm enough that an automated check is cheap insurance.
- **Impact:** Two tests in `tests/architecture/`:
  - `tests/architecture/no_write.rs` — fails if any source file references `fs::write`, `tokio::fs::write`, `OpenOptions::new().write`, or `rename`.
  - `tests/architecture/no_lsp.rs` — fails if `Cargo.toml` lists any `lsp` crate, or if any source file imports one.

## D12. The 4 chunking strategies ship in one phase

- **Reason:** They share the same chunk-iteration core; splitting them across phases adds coordination cost without benefit.
- **Alternatives:** One phase per strategy. Rejected — too granular; one strategy failing would block the others.
- **Impact:** Milestone 10 ships all four strategies together, with per-strategy unit tests and a shared integration test that exercises each.
