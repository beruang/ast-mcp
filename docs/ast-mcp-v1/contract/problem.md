# Problem

## Current State

AI agents working on real codebases have access to two existing classes of code intelligence:

1. **Filesystem + grep** — fast, simple, but structurally blind.
2. **LSP MCP** — semantically rich, but heavy. Requires starting and maintaining a per-language server, and is not always available (offline, restricted sandboxes, lightweight environments).

Neither is well suited to a common class of agent questions:

- What is the syntax structure of this file?
- What top-level declarations exist?
- What node contains this position?
- What imports and exports does this file declare?
- What functions and classes does this file define?
- How should this file be chunked for downstream retrieval or prompts?
- What nodes match this structural pattern?

## Pain

- **LSP requires a server lifecycle.** A `gopls` or `rust-analyzer` process is non-trivial to start, version-pin, and keep healthy inside an agent runtime.
- **Grep returns text, not structure.** It cannot tell you "this is a class method named `getUser` at lines 14–28" without a follow-up parse.
- **Agent tools frequently loop.** Agents re-read files, re-tokenize, and re-derive structure. Each repetition burns context.
- **Semantic identity is risky without proper resolution.** A naive `grep "getUser"` cannot distinguish the definition at `src/user.ts:14` from a call at `src/order.ts:99`.

## Impact

Without a fast, safe, structural layer, agents either:

- Skip structure entirely and rely on grep, which loses precision and wastes context.
- Pay the full LSP cost for every "structural" question, even when semantics are not needed.
- Roll their own ad-hoc Tree-sitter in-process, repeating the same boilerplate across projects.

## Goal of the Fix

A dedicated **AST MCP** server that:

- Owns syntax and structure (not semantics).
- Boots quickly, requires no language server, and is usable when LSP MCP is unavailable.
- Exposes a small, stable, agent-friendly tool surface (12 tools).
- Operates on one workspace root with strict path safety and bounded output.

## Boundary

The architectural boundary is firm:

```text
LSP MCP = semantic intelligence
AST MCP = structural intelligence
```

The AST MCP must never claim semantic certainty (e.g., "this call resolves to `src/services/user.ts:getUser`"). It returns the syntax tree and lets a downstream tool — LSP MCP, an Agent Skill, or a Composite MCP — resolve identity.

## Source

`spec/version-1.md` § 1 (Purpose), § 2 (Architectural Boundary), § 38 (Final Design Principle).
