# Spec Phase 3: Parser Registry

## Phase Goal

Wire the 5 V1 Tree-sitter parsers, route by extension, and expose a stable `ParserDefinition` interface that future languages (Go, Rust, Java, C/C++) can be added to.

## Dependencies

- Requires: phase-2 (safety, errors, constants).
- Produces: `parser::registry`, `parser::parse`, `languages::*`, `shared::language`.

## Existing Code References

- Pattern to follow: None (greenfield).
- Related module: `safety::paths::resolve_file` — `parse_source` operates on already-resolved paths.
- Test pattern: `tests/safety/paths.rs` (phase 2) — extend with a parse-helper test.

## Technical Approach

A `ParserDefinition` is a static record that names a language, lists its extensions, and provides a `fn() -> Language`. The registry is a `&'static [ParserDefinition]` of 5 entries. `parse_source` is a thin wrapper around `tree_sitter::Parser`.

## File Changes

### New Files

| File | Purpose |
|---|---|
| `src/parser/mod.rs` | Public surface. |
| `src/parser/registry.rs` | `ParserDefinition`, `registry()`, `for_extension`, `for_language`. |
| `src/parser/parse.rs` | `parse_source`, `ParseStatus`. |
| `src/languages/mod.rs` | Module declarations. |
| `src/languages/typescript.rs` | Re-export `tree_sitter_typescript::language_typescript` and `language_tsx`. |
| `src/languages/javascript.rs` | Re-export `tree_sitter_javascript::language`. |
| `src/languages/python.rs` | Re-export `tree_sitter_python::language`. |
| `src/shared/language.rs` | `LanguageId` enum. |

### Modified Files

| File | Change |
|---|---|
| `Cargo.toml` | Add `tree-sitter`, `tree-sitter-typescript`, `tree-sitter-javascript`, `tree-sitter-python`. Pin exact minor versions. |

## Implementation Steps

1. Add dependencies to `Cargo.toml`:
   ```toml
   tree-sitter = "0.22"
   tree-sitter-typescript = "0.20"
   tree-sitter-javascript = "0.20"
   tree-sitter-python = "0.20"
   ```
2. `src/shared/language.rs`:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
   pub enum LanguageId { TypeScript, TypeScriptReact, JavaScript, JavaScriptReact, Python }
   impl LanguageId {
       pub fn as_str(&self) -> &'static str { /* "typescript", ... */ }
   }
   impl std::str::FromStr for LanguageId { /* "typescript" → TypeScript, ... */ }
   ```
3. `src/languages/typescript.rs`:
   ```rust
   pub fn language() -> tree_sitter::Language { tree_sitter_typescript::language_typescript() }
   pub fn language_tsx() -> tree_sitter::Language { tree_sitter_typescript::language_tsx() }
   ```
4. `src/languages/javascript.rs`:
   ```rust
   pub fn language() -> tree_sitter::Language { tree_sitter_javascript::language() }
   ```
5. `src/languages/python.rs`:
   ```rust
   pub fn language() -> tree_sitter::Language { tree_sitter_python::language() }
   ```
6. `src/parser/registry.rs`:
   ```rust
   use tree_sitter::Language;
   use crate::shared::language::LanguageId;
   use crate::languages::{typescript, javascript, python};

   pub struct ParserDefinition {
       pub language: LanguageId,
       pub extensions: &'static [&'static str],
       pub tree_sitter_language: fn() -> Language,
   }

   pub fn registry() -> &'static [ParserDefinition] {
       &[
           ParserDefinition { language: LanguageId::TypeScript,        extensions: &[".ts"],     tree_sitter_language: typescript::language },
           ParserDefinition { language: LanguageId::TypeScriptReact,   extensions: &[".tsx"],    tree_sitter_language: typescript::language_tsx },
           ParserDefinition { language: LanguageId::JavaScript,       extensions: &[".js", ".mjs", ".cjs"], tree_sitter_language: javascript::language },
           ParserDefinition { language: LanguageId::JavaScriptReact,  extensions: &[".jsx"],    tree_sitter_language: javascript::language },
           ParserDefinition { language: LanguageId::Python,           extensions: &[".py"],     tree_sitter_language: python::language },
       ]
   }

   pub fn for_extension(ext: &str) -> Option<&'static ParserDefinition> {
       registry().iter().find(|d| d.extensions.contains(&ext))
   }
   pub fn for_language(lang: LanguageId) -> Option<&'static ParserDefinition> {
       registry().iter().find(|d| d.language == lang)
   }
   ```
7. `src/parser/parse.rs`:
   ```rust
   use tree_sitter::{Parser, Tree};
   use crate::shared::language::LanguageId;
   use crate::parser::registry;

   pub struct ParseStatus {
       pub has_syntax_error: bool,
       pub root_kind: String,
       pub node_count: usize,
       pub parse_time_ms: u64,
   }

   pub fn parse_source(source: &str, lang: LanguageId) -> Result<(Tree, ParseStatus), crate::shared::errors::AstToolError> {
       let def = registry::for_language(lang).ok_or_else(|| AstToolError::ParserUnavailable(lang.as_str().to_string()))?;
       let mut parser = Parser::new();
       parser.set_language((def.tree_sitter_language)()).map_err(|e| AstToolError::ParserUnavailable(e.to_string()))?;
       let start = std::time::Instant::now();
       let tree = parser.parse(source, None).ok_or_else(|| AstToolError::ParseFailed("tree-sitter returned None".into()))?;
       let parse_time_ms = start.elapsed().as_millis() as u64;
       let root = tree.root_node();
       let has_syntax_error = root.has_error();
       let node_count = count_nodes(&root);
       let root_kind = root.kind().to_string();
       Ok((tree, ParseStatus { has_syntax_error, root_kind, node_count, parse_time_ms }))
   }

   fn count_nodes(n: &tree_sitter::Node) -> usize {
       let mut c = 1;
       let mut cursor = n.walk();
       for child in n.children(&mut cursor) { c += count_nodes(&child); }
       c
   }
   ```
8. `tests/parser/registry.rs`:
   - `for_extension(".ts")` → `Some(TypeScript)`.
   - `for_extension(".mjs")` → `Some(JavaScript)`.
   - `for_extension(".rb")` → `None`.
   - `for_language(LanguageId::Python)` → `Some(..)`.
9. `tests/parser/parse.rs`:
   - `parse_source("const x = 1;", LanguageId::TypeScript)` → `has_syntax_error: false`, `root_kind: "program"`, `node_count > 0`.
   - Same for TSX, JavaScript, JSX, Python.
   - A TS file with a syntax error (`const x = ;`) → `has_syntax_error: true`.
   - `parse_source` for a `LanguageId` that exists in the registry: 5 happy-path assertions.

## Data / API / Interface Contract

```rust
// shared::language
pub enum LanguageId { TypeScript, TypeScriptReact, JavaScript, JavaScriptReact, Python }
impl LanguageId { pub fn as_str(&self) -> &'static str; }
impl FromStr for LanguageId { type Err = AstToolError; }

// parser::registry
pub struct ParserDefinition { pub language: LanguageId, pub extensions: &'static [&'static str], pub tree_sitter_language: fn() -> tree_sitter::Language }
pub fn registry() -> &'static [ParserDefinition];
pub fn for_extension(ext: &str) -> Option<&'static ParserDefinition>;
pub fn for_language(lang: LanguageId) -> Option<&'static ParserDefinition>;

// parser::parse
pub struct ParseStatus { pub has_syntax_error: bool, pub root_kind: String, pub node_count: usize, pub parse_time_ms: u64 }
pub fn parse_source(source: &str, lang: LanguageId) -> Result<(tree_sitter::Tree, ParseStatus), AstToolError>;
```

## Error Handling

- `AstToolError::ParserUnavailable(String)` — language id is in the enum but the registry has no entry (defensive; should not happen in V1).
- `AstToolError::ParseFailed(String)` — `Parser::parse` returned `None`.
- The caller (the tool layer) maps `LanguageId` from extension via `registry::for_extension`; if the extension is unsupported, it returns `AstToolError::UnsupportedLanguage(ext)` before reaching `parse_source`.

## Observability

- Logs: `tracing::debug!` with parse time, node count, language. **stderr** only.
- No metrics. No traces.

## Testing Requirements

### Unit Tests

- `tests/parser/registry.rs` — extension routing for all 5 languages plus a `None` case.
- `tests/parser/parse.rs` — happy path for all 5 languages, syntax-error case.

### Integration Tests

None new in this phase. The tool-level integration tests land in phase 4.

## Validation Commands

```bash
cargo build
cargo test --lib parser
cargo test --lib languages
```

## Acceptance Criteria

- [ ] `registry()` returns 5 entries, one per V1 language.
- [ ] Extension routing matches the spec table.
- [ ] `for_extension` returns `None` for an unknown extension.
- [ ] `parse_source` returns a `Tree` and a `ParseStatus` with all four fields populated.
- [ ] A TS file with `const x = ;` is parsed with `has_syntax_error: true`.
- [ ] `parse_source` for an unsupported `LanguageId` returns `ParserUnavailable` (defensive).
- [ ] `cargo clippy --all-targets -- -D warnings` passes.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Tree-sitter version drift | medium | Pin exact minor versions in `Cargo.toml`. Document the choice in `decisions.md`. |
| Node-kind names differ between parser versions | medium | Use the published `tree-sitter-typescript` 0.20.x and verify against the integration tests in phase 4. If a kind name changes, update the extractor in phase 6 — that is the only place node kinds are mentioned by name. |
| `parser.set_language` panics on an unsupported language | low | `Parser::set_language` returns `Result` in modern tree-sitter; we propagate errors as `ParserUnavailable`. |
