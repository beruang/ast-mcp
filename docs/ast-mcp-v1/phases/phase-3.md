# Phase 3: Parser Registry

> Spec milestone: `spec/version-1.md` § 33 Milestone 3 + § 11 (Parser Registry)

## Goal

Wire the five V1 Tree-sitter parsers, route by extension, and expose a stable `ParserDefinition` interface that future languages (Go, Rust, Java, C/C++) can be added to.

## Dependencies

- Requires: phase-2 (safety) — extensions are validated against this registry.
- Produces: `parser::registry`, `parser::parse`, `languages::*` modules.

## Risk

- **high** — Tree-sitter version drift can change node-kind names. Pinning is essential.

## Value

Every other tool depends on a working parser. The registry is the only place where Tree-sitter crates are mentioned.

## Implementation Notes

- `Cargo.toml` adds (pin exact minor versions in the first commit; record the choice in `decisions.md`):
  - `tree-sitter = "0.22"`
  - `tree-sitter-typescript = "0.20"`
  - `tree-sitter-javascript = "0.20"`
  - `tree-sitter-python = "0.20"`
- `src/parser/registry.rs`:
  - `pub struct ParserDefinition { pub language: LanguageId, pub extensions: &'static [&'static str], pub tree_sitter_language: fn() -> Language }`.
  - `pub fn registry() -> &'static [ParserDefinition]` returns 5 entries per spec § 11.
  - `pub fn for_extension(ext: &str) -> Option<&'static ParserDefinition>`.
  - `pub fn for_language(lang: LanguageId) -> Option<&'static ParserDefinition>`.
- `src/parser/parse.rs`:
  - `pub fn parse_source(source: &str, lang: LanguageId) -> Result<(Tree, ParseStatus), AstToolError>`.
  - Uses `tree_sitter::Parser`, sets the language, parses with `parse(source, None)`.
  - Counts nodes via a simple recursive walk.
  - Detects syntax errors by scanning for any node with `is_error()` or `is_missing()`.
  - Measures parse time with `std::time::Instant`.
- `src/languages/mod.rs` exposes one module per language:
  - `pub mod typescript; pub mod javascript; pub mod python;` — each re-exports the tree-sitter language and any future language-specific helpers.
- `src/shared/language.rs`:
  - `pub enum LanguageId { TypeScript, TypeScriptReact, JavaScript, JavaScriptReact, Python }` with `as_str()` and `FromStr`.

## Validation

```bash
cargo build
cargo test --lib parser
```

The unit tests must confirm: each language loads; each known extension routes to the right language; an unknown extension returns `None`.

## Acceptance

- [ ] All 5 V1 languages load successfully (`ast_health_check` would now report them as `available: true` — wired in phase 4).
- [ ] Extension routing: `.ts → typescript`, `.tsx → typescriptreact`, `.js → javascript`, `.mjs → javascript`, `.cjs → javascript`, `.jsx → javascriptreact`, `.py → python`.
- [ ] Unknown extensions return `None` from `for_extension`.
- [ ] `parse_source` returns a tree, a node count, and a `hasSyntaxError` flag.
- [ ] No language-specific code lives outside `src/languages/`. The `parser/` and `tools/` modules are language-agnostic.
- [ ] A unit test parses a minimal "hello, world" file in each of the 5 languages and asserts `hasSyntaxError: false`.
