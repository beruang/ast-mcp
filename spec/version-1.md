# AST MCP Server — Version 1 Specification

## 1. Purpose

Build a Rust-based MCP server that exposes safe, agent-friendly AST and structural code-intelligence tools.

Version 1 focuses on the **Core Structural MVP**:

- parse files
- inspect file structure
- extract top-level nodes
- locate enclosing syntax nodes
- extract imports and exports
- extract functions and classes
- chunk files by syntax structure
- run bounded Tree-sitter queries

The AST MCP server is responsible for **syntax and structure**, not semantic meaning.

It should complement, but not depend on, the LSP MCP server.

```text
LSP MCP = semantic intelligence
AST MCP = structural intelligence
```

---

## 2. Architectural Boundary

This MCP server is the **AST MCP**.

It exposes only `ast_*` tools.

It must not call:

```text
lsp_* tools
LSP MCP
language servers
TypeScript language service
Pyright
gopls
rust-analyzer
semantic reference engines
```

The AST MCP must remain usable even when LSP MCP is unavailable.

### AST MCP owns

```text
syntax parsing
file structure extraction
AST node inspection
imports/exports extraction
function/class extraction
syntax-aware file chunking
Tree-sitter query execution
structural outlines
```

### AST MCP does not own

```text
type information
symbol references
semantic rename
compiler diagnostics
hover information
implementation lookup
call hierarchy
type hierarchy
language-server lifecycle
```

Those belong to LSP MCP.

### Composite workflows

Higher-level workflows such as:

```text
code_context_pack
code_inspect_symbol
code_prepare_safe_rename
code_analyze_change_impact
code_prepare_diagnostic_fix
```

must live in one of:

```text
Agent Skills
Code Composite MCP
client-side orchestration layer
```

They must not be embedded inside AST MCP if they require LSP data.

---

## 3. Target Architecture

```text
Agent / MCP Client
  ↓
AST MCP Server, Rust
  ↓
Workspace Safety Layer
  ↓
Parser Registry
  ↓
Tree-sitter Parsers
  ├── tree-sitter-typescript
  ├── tree-sitter-javascript
  └── tree-sitter-python
  ↓
Workspace source files
```

The AST MCP server reads workspace files directly and parses them locally.

It does not start LSP servers.

It does not perform semantic analysis.

---

## 4. Version 1 Goals

V1 must provide the following tools:

```text
ast_health_check
ast_list_supported_languages
ast_parse_file
ast_file_outline
ast_top_level_nodes
ast_enclosing_node
ast_find_imports
ast_find_exports
ast_find_functions
ast_find_classes
ast_chunk_file
ast_query
```

V1 must support these languages:

```text
TypeScript
TSX
JavaScript
JSX
Python
```

V1 should be designed so Go, Rust, Java, and C/C++ can be added later without changing the public contracts.

---

## 5. Version 1 Non-Goals

V1 does not include:

```text
LSP integration
semantic references
type resolution
compiler diagnostics
workspace-wide indexing
workspace-wide AST query
framework-aware route extraction
React component extraction
test extraction
schema/model extraction
AST rewrite previews
file mutation
patch application
persistent database
remote workspace support
multi-root workspace support
```

---

## 6. Technology Stack

Recommended stack:

```text
Language: Rust
Runtime model: native binary
Protocol: MCP over stdio
Parsing: Tree-sitter
Serialization: serde / serde_json
Schema generation: schemars, optional
Error handling: thiserror / anyhow
File walking: ignore / walkdir, mostly reserved for later versions
Glob matching: globset, mostly reserved for later versions
Diffs: not required in V1
Async runtime: tokio if MCP transport needs async
```

Recommended Rust crates:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
anyhow = "1"
tokio = { version = "1", features = ["full"] }
tree-sitter = "0.22"
tree-sitter-typescript = "0.20"
tree-sitter-javascript = "0.20"
tree-sitter-python = "0.20"
ignore = "0.4"
globset = "0.4"
walkdir = "2"
uuid = { version = "1", features = ["v4"] }
```

If using a Rust MCP SDK, add the SDK crate selected by the implementation team.

If no Rust MCP SDK is chosen yet, the server may implement MCP stdio JSON-RPC directly behind a small transport abstraction.

---

## 7. Workspace Model

The AST MCP server runs against exactly one workspace root in V1.

Workspace root is provided by:

```bash
WORKSPACE_PATH=/absolute/path/to/repo
```

If omitted, the server may use the current working directory.

Every tool input path must resolve inside `WORKSPACE_PATH`.

### Allowed

```text
src/user.ts
/repo/src/user.ts
```

### Rejected

```text
../outside.ts
/etc/passwd
/another-project/file.ts
```

### Required behavior

```text
Input path → normalize → resolve against workspace → verify containment → operate
```

Responses must return file paths as **workspace-relative paths**.

---

## 8. Safety Requirements

### 8.1 Path Safety

Every tool must validate file paths.

Rules:

```text
1. Reject paths outside workspace.
2. Reject directories where a file is required.
3. Reject missing files unless the tool explicitly supports virtual text.
4. Reject unsupported file extensions.
5. Normalize response paths to workspace-relative paths.
```

### 8.2 No File Mutation

AST MCP V1 must never write files.

It may read files and return parsed structure.

It must not:

```text
write source files
apply patches
rename files
create files
delete files
format files
execute commands
```

### 8.3 Bounded Output

AST output can become large. Every tool that returns lists or text must support limits.

Default limits:

```text
maxNodes: 500
maxResults: 200
maxTextBytes: 20000
maxChunkLines: 120
maxChunkBytes: 30000
maxQueryMatches: 200
```

If truncated, return:

```json
{
  "truncated": true,
  "returned": 200
}
```

### 8.4 No Semantic Claims

AST tools must not claim semantic certainty.

For example:

```text
Allowed:
  "This file contains a call expression with callee text getUser."

Not allowed:
  "This call references src/services/user.ts:getUser."
```

Semantic identity belongs to LSP MCP.

---

## 9. Position and Range Model

Tree-sitter internally uses byte offsets and row/column points.

The external AST MCP API must use the same shared position contract as LSP MCP to allow future orchestration.

### Position

```ts
type Position = {
  line: number;      // zero-based
  character: number; // zero-based UTF-16 offset
};
```

### Range

```ts
type Range = {
  start: Position;
  end: Position;
};
```

### Important implementation note

Tree-sitter columns are byte-based for UTF-8 source text. The public API uses UTF-16 character offsets.

The Rust implementation must provide conversion helpers:

```text
UTF-16 position → byte offset
byte offset → UTF-16 position
Tree-sitter point → public Position
public Range → byte range
```

This matters for files containing emoji or non-ASCII characters.

V1 may document limitations if full UTF-16 conversion is not implemented immediately, but the public contract should still use UTF-16-compatible positions.

---

## 10. Shared Types

### WorkspaceRelativePath

```ts
type WorkspaceRelativePath = string;
```

### AstNodeSummary

```ts
type AstNodeSummary = {
  id?: string;
  kind: string;
  name?: string;
  range: Range;
  startByte?: number;
  endByte?: number;
  text?: string;
  children?: AstNodeSummary[];
};
```

Rules:

```text
id is optional in V1.
kind is the Tree-sitter node kind or normalized high-level kind.
name is best-effort structural name.
text is omitted by default unless requested.
children are included only when requested or part of outline output.
```

### AstToolError

```ts
type AstToolError = {
  error: {
    code: string;
    message: string;
    details?: unknown;
  };
};
```

Common error codes:

```text
workspace_not_found
path_outside_workspace
file_not_found
file_too_large
unsupported_language
parser_unavailable
parse_failed
syntax_error
invalid_position
invalid_range
query_invalid
query_execution_failed
result_limit_exceeded
internal_error
```

### LanguageId

```ts
type LanguageId =
  | "typescript"
  | "typescriptreact"
  | "javascript"
  | "javascriptreact"
  | "python";
```

### ParseStatus

```ts
type ParseStatus = {
  parsed: boolean;
  hasSyntaxError: boolean;
  rootKind: string;
  nodeCount: number;
  parseTimeMs: number;
};
```

---

## 11. Parser Registry

V1 parser registry:

```rust
ParserDefinition {
    language: "typescript",
    extensions: [".ts"],
    tree_sitter_language: tree_sitter_typescript::language_typescript(),
}

ParserDefinition {
    language: "typescriptreact",
    extensions: [".tsx"],
    tree_sitter_language: tree_sitter_typescript::language_tsx(),
}

ParserDefinition {
    language: "javascript",
    extensions: [".js", ".mjs", ".cjs"],
    tree_sitter_language: tree_sitter_javascript::language(),
}

ParserDefinition {
    language: "javascriptreact",
    extensions: [".jsx"],
    tree_sitter_language: tree_sitter_javascript::language(),
}

ParserDefinition {
    language: "python",
    extensions: [".py"],
    tree_sitter_language: tree_sitter_python::language(),
}
```

The registry must support:

```text
extension → parser definition
language id → parser definition
supported language listing
parser availability checks
```

---

## 12. Tool Response Format

All MCP tools should return one machine-readable JSON payload.

Recommended MCP content shape:

```json
{
  "content": [
    {
      "type": "text",
      "text": "{ ...pretty JSON... }"
    }
  ]
}
```

Tool payloads should not return prose.

Errors should also be returned as structured JSON:

```json
{
  "error": {
    "code": "unsupported_language",
    "message": "No AST parser configured for extension .rb"
  }
}
```

---

# 13. Tool: ast_health_check

## Purpose

Report workspace validity, parser availability, and supported language status.

## Input Schema

```ts
type AstHealthCheckInput = {
  workspacePath?: string;
};
```

If omitted, use server workspace.

## Output Schema

```ts
type AstHealthCheckResult = {
  workspacePath: string;
  ok: boolean;
  parsers: Array<{
    language: LanguageId;
    extensions: string[];
    available: boolean;
    parser: string;
    error?: string;
  }>;
  limits: {
    maxFileBytes: number;
    maxNodes: number;
    maxResults: number;
  };
};
```

## Behavior

1. Resolve workspace path.
2. Verify workspace exists and is a directory.
3. Check parser registry.
4. Return configured limits.
5. Do not parse files.

## Example Output

```json
{
  "workspacePath": "/repo",
  "ok": true,
  "parsers": [
    {
      "language": "typescript",
      "extensions": [".ts"],
      "available": true,
      "parser": "tree-sitter-typescript"
    },
    {
      "language": "python",
      "extensions": [".py"],
      "available": true,
      "parser": "tree-sitter-python"
    }
  ],
  "limits": {
    "maxFileBytes": 1048576,
    "maxNodes": 500,
    "maxResults": 200
  }
}
```

---

# 14. Tool: ast_list_supported_languages

## Purpose

Return parser registry and extension routing.

## Input Schema

```ts
type AstListSupportedLanguagesInput = {};
```

## Output Schema

```ts
type AstListSupportedLanguagesResult = {
  languages: Array<{
    language: LanguageId;
    extensions: string[];
    parser: string;
    available: boolean;
  }>;
};
```

## Behavior

1. Return all parser definitions.
2. Do not parse files.
3. Do not scan workspace.

---

# 15. Tool: ast_parse_file

## Purpose

Parse a file and return parse status.

This is the lowest-level validation tool for the AST MCP.

## Input Schema

```ts
type AstParseFileInput = {
  filePath: string;
  includeTree?: boolean;
  maxDepth?: number;
  includeNodeText?: boolean;
};
```

Defaults:

```ts
{
  includeTree: false,
  maxDepth: 3,
  includeNodeText: false
}
```

## Output Schema

```ts
type AstParseFileResult = {
  filePath: string;
  language: LanguageId;
  parsed: boolean;
  hasSyntaxError: boolean;
  rootKind: string;
  nodeCount: number;
  parseTimeMs: number;
  tree?: AstNodeSummary;
};
```

## Behavior

1. Validate file path.
2. Detect parser by extension.
3. Read file.
4. Enforce max file size.
5. Parse with Tree-sitter.
6. Count nodes.
7. Detect syntax errors.
8. Return root status.
9. Include bounded tree only if `includeTree` is true.

## Notes

`includeTree` should be false by default because full parse trees can be large.

If `includeTree` is true, respect:

```text
maxDepth
maxNodes
maxTextBytes
```

---

# 16. Tool: ast_file_outline

## Purpose

Return a structural outline of a file.

This tool is the AST counterpart to LSP `documentSymbol`, but it may include syntax structures that LSP does not expose.

## Input Schema

```ts
type AstFileOutlineInput = {
  filePath: string;
  maxDepth?: number;
  includeRanges?: boolean;
  includeImports?: boolean;
  includeExports?: boolean;
};
```

Defaults:

```ts
{
  maxDepth: 4,
  includeRanges: true,
  includeImports: true,
  includeExports: true
}
```

## Output Schema

```ts
type AstFileOutlineResult = {
  filePath: string;
  language: LanguageId;
  outlineText: string;
  nodes: AstOutlineNode[];
  truncated: boolean;
};
```

```ts
type AstOutlineNode = {
  kind: string;
  name?: string;
  range?: Range;
  children?: AstOutlineNode[];
};
```

## Behavior

1. Parse file.
2. Extract structurally important nodes.
3. Render compact outline text.
4. Return structured nodes.
5. Respect `maxDepth`.
6. Respect result limits.

## Structural nodes to include in V1

For TypeScript/JavaScript:

```text
import statements
export statements
function declarations
class declarations
methods
arrow functions assigned to const, best effort
interface declarations
type aliases
enums
```

For Python:

```text
imports
from imports
class definitions
function definitions
async function definitions
```

## Example Output

```json
{
  "filePath": "src/user.ts",
  "language": "typescript",
  "outlineText": "import { User } from './types'\nclass UserService\n  method getUser\n  method createUser\nfunction parseUser",
  "nodes": [
    {
      "kind": "class_declaration",
      "name": "UserService",
      "range": {
        "start": { "line": 10, "character": 0 },
        "end": { "line": 88, "character": 1 }
      },
      "children": [
        {
          "kind": "method_definition",
          "name": "getUser",
          "range": {
            "start": { "line": 14, "character": 2 },
            "end": { "line": 28, "character": 3 }
          }
        }
      ]
    }
  ],
  "truncated": false
}
```

---

# 17. Tool: ast_top_level_nodes

## Purpose

Return top-level syntax nodes in source order.

Useful for file summaries, patch planning, and structural navigation.

## Input Schema

```ts
type AstTopLevelNodesInput = {
  filePath: string;
  includeText?: boolean;
  maxTextBytes?: number;
};
```

Defaults:

```ts
{
  includeText: false,
  maxTextBytes: 20000
}
```

## Output Schema

```ts
type AstTopLevelNodesResult = {
  filePath: string;
  language: LanguageId;
  nodes: Array<{
    kind: string;
    name?: string;
    range: Range;
    text?: string;
  }>;
  returned: number;
  truncated: boolean;
};
```

## Behavior

1. Parse file.
2. Iterate direct children of root node.
3. Normalize node kind and optional name.
4. Include text only if requested.
5. Respect result and text limits.

---

# 18. Tool: ast_enclosing_node

## Purpose

Return the smallest syntax node at a position, optionally constrained by node kinds.

This is useful for locating the exact function, class, call expression, import, or declaration around a position.

## Input Schema

```ts
type AstEnclosingNodeInput = {
  filePath: string;
  position: Position;
  kinds?: string[];
  includeAncestors?: boolean;
  includeText?: boolean;
  maxTextBytes?: number;
};
```

Defaults:

```ts
{
  includeAncestors: true,
  includeText: true,
  maxTextBytes: 20000
}
```

## Output Schema

```ts
type AstEnclosingNodeResult = {
  filePath: string;
  language: LanguageId;
  position: Position;
  node?: {
    kind: string;
    name?: string;
    range: Range;
    text?: string;
  };
  ancestors: Array<{
    kind: string;
    name?: string;
    range: Range;
  }>;
};
```

## Behavior

1. Validate position.
2. Convert public position to byte offset.
3. Find smallest Tree-sitter node containing the position.
4. If `kinds` is provided, walk ancestors until matching kind is found.
5. Return ancestors from nearest parent outward or outermost to innermost. The implementation must document chosen order.
6. Include text only if requested and under limit.

## Recommended ancestor order

Return ancestors outermost to innermost:

```text
module → class → method → statement → expression
```

---

# 19. Tool: ast_find_imports

## Purpose

Extract import statements from a file.

## Input Schema

```ts
type AstFindImportsInput = {
  filePath: string;
};
```

## Output Schema

```ts
type AstFindImportsResult = {
  filePath: string;
  language: LanguageId;
  imports: AstImport[];
};
```

```ts
type AstImport = {
  source?: string;
  kind:
    | "import"
    | "from_import"
    | "require"
    | "dynamic_import"
    | "unknown";
  defaultImport?: string;
  namespaceImport?: string;
  namedImports: string[];
  aliases: Array<{
    imported: string;
    local: string;
  }>;
  isTypeOnly?: boolean;
  range: Range;
  text: string;
};
```

## TypeScript/JavaScript support

Handle:

```ts
import x from "mod";
import { a, b as c } from "mod";
import * as ns from "mod";
import type { T } from "mod";
const x = require("mod");
await import("mod");
```

## Python support

Handle:

```python
import os
import numpy as np
from pathlib import Path
from package.module import A, B as C
```

## Behavior

1. Parse file.
2. Match import-related syntax nodes.
3. Extract source string where available.
4. Extract bindings best-effort.
5. Preserve original text.
6. Return imports in source order.

---

# 20. Tool: ast_find_exports

## Purpose

Extract exports from a file.

For Python, where exports are less explicit, V1 should return best-effort public top-level definitions and `__all__` if present.

## Input Schema

```ts
type AstFindExportsInput = {
  filePath: string;
  includeBestEffortPythonExports?: boolean;
};
```

Defaults:

```ts
{
  includeBestEffortPythonExports: true
}
```

## Output Schema

```ts
type AstFindExportsResult = {
  filePath: string;
  language: LanguageId;
  exports: AstExport[];
};
```

```ts
type AstExport = {
  kind:
    | "function"
    | "class"
    | "const"
    | "let"
    | "var"
    | "type"
    | "interface"
    | "enum"
    | "re_export"
    | "default"
    | "python_public_definition"
    | "python_all"
    | "unknown";
  name?: string;
  source?: string;
  isDefault?: boolean;
  isTypeOnly?: boolean;
  range: Range;
  text: string;
};
```

## TypeScript/JavaScript support

Handle:

```ts
export function f() {}
export class C {}
export const x = 1;
export type T = string;
export interface I {}
export default function f() {}
export { a, b as c };
export * from "mod";
```

## Python support

Handle:

```python
__all__ = ["User", "get_user"]

def public_function(): ...
class PublicClass: ...
```

Python export detection is best-effort only.

---

# 21. Tool: ast_find_functions

## Purpose

Extract functions and methods from a file.

## Input Schema

```ts
type AstFindFunctionsInput = {
  filePath: string;
  includeMethods?: boolean;
  includeAnonymous?: boolean;
  includeText?: boolean;
  maxTextBytes?: number;
};
```

Defaults:

```ts
{
  includeMethods: true,
  includeAnonymous: false,
  includeText: false,
  maxTextBytes: 20000
}
```

## Output Schema

```ts
type AstFindFunctionsResult = {
  filePath: string;
  language: LanguageId;
  functions: AstFunction[];
  returned: number;
  truncated: boolean;
};
```

```ts
type AstFunction = {
  name?: string;
  kind:
    | "function"
    | "method"
    | "constructor"
    | "arrow_function"
    | "function_expression"
    | "async_function"
    | "lambda"
    | "unknown";
  async?: boolean;
  exported?: boolean;
  parameters: Array<{
    name?: string;
    typeText?: string;
    optional?: boolean;
    defaultValueText?: string;
  }>;
  returnTypeText?: string;
  range: Range;
  bodyRange?: Range;
  text?: string;
  parentName?: string;
};
```

## TypeScript/JavaScript support

Handle:

```ts
function f(a: string): number {}
async function f() {}
const f = () => {};
const f = function() {};
class C { method(x: string) {} }
```

## Python support

Handle:

```python
def f(x: str) -> int: ...
async def f(): ...
class C:
    def method(self): ...
```

## Behavior

1. Parse file.
2. Find function-like nodes.
3. Extract best-effort names.
4. Extract parameters.
5. Extract return type annotation where structurally available.
6. Mark methods with parent class name when available.
7. Include text only if requested and under limit.

---

# 22. Tool: ast_find_classes

## Purpose

Extract class definitions from a file.

## Input Schema

```ts
type AstFindClassesInput = {
  filePath: string;
  includeMethods?: boolean;
  includeText?: boolean;
  maxTextBytes?: number;
};
```

Defaults:

```ts
{
  includeMethods: true,
  includeText: false,
  maxTextBytes: 20000
}
```

## Output Schema

```ts
type AstFindClassesResult = {
  filePath: string;
  language: LanguageId;
  classes: AstClass[];
  returned: number;
  truncated: boolean;
};
```

```ts
type AstClass = {
  name: string;
  exported?: boolean;
  extendsText?: string;
  implementsText: string[];
  decoratorsText: string[];
  methods: Array<{
    name?: string;
    kind: "method" | "constructor" | "getter" | "setter" | "unknown";
    range: Range;
  }>;
  range: Range;
  bodyRange?: Range;
  text?: string;
};
```

## TypeScript/JavaScript support

Handle:

```ts
export class UserService extends Base implements IService {}
class C { constructor() {} get x() {} set x(v) {} }
```

## Python support

Handle:

```python
class UserService(Base):
    def get_user(self): ...
```

## Behavior

1. Parse file.
2. Find class declarations/definitions.
3. Extract name.
4. Extract superclass/extends/base classes best-effort.
5. Extract implements list for TypeScript.
6. Extract methods if requested.
7. Include text only if requested and under limit.

---

# 23. Tool: ast_chunk_file

## Purpose

Chunk a file by syntax structure.

This is one of the most important AST V1 tools because it provides better context chunks for agents and retrieval.

## Input Schema

```ts
type AstChunkFileInput = {
  filePath: string;
  strategy?:
    | "top_level"
    | "function_class"
    | "semantic_blocks"
    | "max_lines_with_ast_boundaries";
  maxChunkLines?: number;
  maxChunkBytes?: number;
  includeImports?: boolean;
  includeText?: boolean;
};
```

Defaults:

```ts
{
  strategy: "semantic_blocks",
  maxChunkLines: 120,
  maxChunkBytes: 30000,
  includeImports: true,
  includeText: true
}
```

## Output Schema

```ts
type AstChunkFileResult = {
  filePath: string;
  language: LanguageId;
  strategy: string;
  chunks: AstChunk[];
  returned: number;
  truncated: boolean;
};
```

```ts
type AstChunk = {
  id: string;
  kind: string;
  name?: string;
  range: Range;
  startLine: number;
  endLine: number;
  byteLength: number;
  text?: string;
};
```

## Strategy: top_level

One chunk per top-level syntax node.

## Strategy: function_class

Chunks functions, classes, methods, and important type declarations.

## Strategy: semantic_blocks

Recommended default.

Groups:

```text
import block
export/type block
class declarations
function declarations
large methods as individual chunks
remaining top-level statements
```

## Strategy: max_lines_with_ast_boundaries

Creates chunks near `maxChunkLines`, but tries to split on AST node boundaries.

## Behavior

1. Parse file.
2. Identify candidate structural nodes.
3. Group nodes according to strategy.
4. Include import block if requested.
5. Split large nodes if they exceed hard limits.
6. Generate stable chunk IDs.
7. Include source text if requested.
8. Respect line and byte limits.

## Chunk ID recommendation

```text
{filePath}:{kind}:{name}:{startLine}-{endLine}
```

Example:

```text
src/user.ts:class:UserService:10-88
```

---

# 24. Tool: ast_query

## Purpose

Run a bounded Tree-sitter query against one file.

This is an advanced structural search tool for agent workflows and internal development.

## Input Schema

```ts
type AstQueryInput = {
  filePath: string;
  query: string;
  maxResults?: number;
  includeNodeText?: boolean;
  maxTextBytes?: number;
};
```

Defaults:

```ts
{
  maxResults: 200,
  includeNodeText: true,
  maxTextBytes: 20000
}
```

## Output Schema

```ts
type AstQueryResult = {
  filePath: string;
  language: LanguageId;
  matches: AstQueryMatch[];
  returned: number;
  truncated: boolean;
};
```

```ts
type AstQueryMatch = {
  patternIndex?: number;
  captures: Array<{
    name: string;
    kind: string;
    range: Range;
    text?: string;
  }>;
};
```

## Behavior

1. Validate file path.
2. Parse file.
3. Compile Tree-sitter query for the file language.
4. Execute query.
5. Normalize captures.
6. Include text only if requested and under limit.
7. Enforce `maxResults`.

## Error Handling

Invalid query returns:

```json
{
  "error": {
    "code": "query_invalid",
    "message": "Tree-sitter query failed to compile.",
    "details": {
      "language": "typescript"
    }
  }
}
```

## Example Query

```scheme
(function_declaration
  name: (identifier) @function.name)
```

---

## 25. Language-Specific Extraction Rules

V1 should use language-specific extractor modules.

Recommended structure:

```text
src/
  languages/
    mod.rs
    typescript.rs
    javascript.rs
    python.rs
```

### TypeScript / TSX

V1 should recognize:

```text
import_statement
export_statement
function_declaration
class_declaration
method_definition
interface_declaration
type_alias_declaration
enum_declaration
lexical_declaration with arrow function
variable_declaration with function expression
```

### JavaScript / JSX

V1 should recognize:

```text
import_statement
export_statement
function_declaration
class_declaration
method_definition
lexical_declaration with arrow function
variable_declaration with function expression
require call expression
```

### Python

V1 should recognize:

```text
import_statement
import_from_statement
function_definition
class_definition
decorated_definition, best effort
assignment to __all__
```

---

## 26. Name Extraction Rules

Name extraction is best-effort and structural.

Examples:

```ts
function getUser() {}
// name: getUser

const getUser = () => {};
// name: getUser

class UserService {}
// name: UserService

export default function () {}
// name: undefined, kind indicates default export
```

Python examples:

```python
def get_user(): pass
# name: get_user

class UserService: pass
# name: UserService
```

If no stable name exists, omit `name` rather than inventing one.

---

## 27. Runtime Limits

Recommended V1 defaults:

```text
maxFileBytes: 1 MiB
maxNodes: 500
maxResults: 200
maxTextBytes: 20000
maxChunkLines: 120
maxChunkBytes: 30000
parseTimeoutMs: 5000
queryTimeoutMs: 5000
```

V1 may implement these as constants.

Runtime configuration can be added in AST V5.

---

## 28. Error Handling

All tools should return structured errors.

### Unsupported language

```json
{
  "error": {
    "code": "unsupported_language",
    "message": "No AST parser configured for extension .rb"
  }
}
```

### Path outside workspace

```json
{
  "error": {
    "code": "path_outside_workspace",
    "message": "Path escapes workspace: ../outside.ts"
  }
}
```

### File too large

```json
{
  "error": {
    "code": "file_too_large",
    "message": "File exceeds maxFileBytes limit."
  }
}
```

### Invalid position

```json
{
  "error": {
    "code": "invalid_position",
    "message": "Position is outside file bounds."
  }
}
```

### Invalid query

```json
{
  "error": {
    "code": "query_invalid",
    "message": "Tree-sitter query failed to compile."
  }
}
```

---

## 29. Internal Architecture

Recommended Rust project structure:

```text
ast-mcp/
  Cargo.toml
  README.md
  src/
    main.rs

    mcp/
      mod.rs
      transport.rs
      register_tools.rs
      schemas.rs
      responses.rs

    config/
      mod.rs
      defaults.rs
      workspace.rs

    safety/
      mod.rs
      paths.rs
      limits.rs

    parser/
      mod.rs
      registry.rs
      parse.rs
      tree.rs
      positions.rs
      queries.rs

    languages/
      mod.rs
      typescript.rs
      javascript.rs
      python.rs

    extractors/
      mod.rs
      outline.rs
      top_level.rs
      enclosing_node.rs
      imports.rs
      exports.rs
      functions.rs
      classes.rs
      chunks.rs

    tools/
      mod.rs
      health_check.rs
      list_supported_languages.rs
      parse_file.rs
      file_outline.rs
      top_level_nodes.rs
      enclosing_node.rs
      find_imports.rs
      find_exports.rs
      find_functions.rs
      find_classes.rs
      chunk_file.rs
      query.rs

    shared/
      mod.rs
      position.rs
      range.rs
      errors.rs
      ast_node.rs
      language.rs

    utils/
      mod.rs
      text.rs
      ids.rs
      time.rs
```

---

## 30. Module Responsibilities

### `mcp/`

Responsible for MCP transport, tool registration, request parsing, and response formatting.

### `config/`

Responsible for workspace root and default limits.

### `safety/`

Responsible for path validation and limit enforcement.

### `parser/`

Responsible for Tree-sitter parser registry, parsing, node traversal, position conversion, and query execution.

### `languages/`

Language-specific node-kind mappings and extraction helpers.

### `extractors/`

High-level structural extraction logic.

### `tools/`

One file per MCP tool handler.

### `shared/`

Shared public API types used across tools.

---

## 31. Tool Registration Requirements

Each tool should define:

```text
name
description
input schema
handler
```

Example tool names:

```text
ast_health_check
ast_parse_file
ast_file_outline
```

Tool descriptions should be agent-friendly and precise.

Example:

```text
ast_file_outline:
Return a syntax-based outline of a source file using Tree-sitter. This tool is structural and does not resolve semantic references or types.
```

---

## 32. Example Tool Calls

### Health Check

```json
{
  "tool": "ast_health_check",
  "arguments": {}
}
```

### Parse File

```json
{
  "tool": "ast_parse_file",
  "arguments": {
    "filePath": "src/user.ts",
    "includeTree": false
  }
}
```

### File Outline

```json
{
  "tool": "ast_file_outline",
  "arguments": {
    "filePath": "src/user.ts",
    "maxDepth": 4,
    "includeRanges": true
  }
}
```

### Top-Level Nodes

```json
{
  "tool": "ast_top_level_nodes",
  "arguments": {
    "filePath": "src/user.ts",
    "includeText": false
  }
}
```

### Enclosing Node

```json
{
  "tool": "ast_enclosing_node",
  "arguments": {
    "filePath": "src/user.ts",
    "position": {
      "line": 20,
      "character": 12
    },
    "kinds": ["function_declaration", "method_definition", "class_declaration"],
    "includeAncestors": true,
    "includeText": true
  }
}
```

### Find Imports

```json
{
  "tool": "ast_find_imports",
  "arguments": {
    "filePath": "src/user.ts"
  }
}
```

### Find Exports

```json
{
  "tool": "ast_find_exports",
  "arguments": {
    "filePath": "src/user.ts"
  }
}
```

### Find Functions

```json
{
  "tool": "ast_find_functions",
  "arguments": {
    "filePath": "src/user.ts",
    "includeMethods": true,
    "includeText": false
  }
}
```

### Find Classes

```json
{
  "tool": "ast_find_classes",
  "arguments": {
    "filePath": "src/user.ts",
    "includeMethods": true
  }
}
```

### Chunk File

```json
{
  "tool": "ast_chunk_file",
  "arguments": {
    "filePath": "src/user.ts",
    "strategy": "semantic_blocks",
    "maxChunkLines": 120,
    "includeImports": true,
    "includeText": true
  }
}
```

### Query

```json
{
  "tool": "ast_query",
  "arguments": {
    "filePath": "src/user.ts",
    "query": "(function_declaration name: (identifier) @function.name)",
    "maxResults": 100,
    "includeNodeText": true
  }
}
```

---

## 33. Development Milestones

### Milestone 1: Rust project skeleton

Implement:

```text
Cargo project
MCP stdio transport
basic JSON responses
one dummy tool
```

### Milestone 2: Workspace safety

Implement:

```text
WORKSPACE_PATH loading
safe path resolution
workspace-relative output paths
file size limits
```

### Milestone 3: Parser registry

Implement:

```text
TypeScript parser
TSX parser
JavaScript parser
JSX routing
Python parser
extension routing
```

### Milestone 4: Basic parsing

Implement:

```text
ast_health_check
ast_list_supported_languages
ast_parse_file
node counting
syntax error detection
```

### Milestone 5: Position conversion

Implement:

```text
byte offset to public Position
public Position to byte offset
Range normalization
Unicode tests
```

### Milestone 6: Outline and top-level nodes

Implement:

```text
ast_file_outline
ast_top_level_nodes
language-specific outline extractors
```

### Milestone 7: Enclosing node

Implement:

```text
ast_enclosing_node
ancestor extraction
node kind filtering
bounded node text
```

### Milestone 8: Imports and exports

Implement:

```text
ast_find_imports
ast_find_exports
TypeScript/JavaScript extraction
Python extraction
```

### Milestone 9: Functions and classes

Implement:

```text
ast_find_functions
ast_find_classes
method extraction
parameter extraction
return type extraction, best effort
```

### Milestone 10: Chunking

Implement:

```text
ast_chunk_file
top_level strategy
function_class strategy
semantic_blocks strategy
max_lines_with_ast_boundaries strategy
```

### Milestone 11: Tree-sitter query

Implement:

```text
ast_query
query compilation
capture normalization
query errors
result limits
```

### Milestone 12: V1 acceptance tests

Run all unit and integration tests.

---

## 34. Acceptance Criteria

V1 is acceptable when the following are true.

### Health and language support

- `ast_health_check` returns workspace status and parser availability.
- `ast_list_supported_languages` returns TypeScript, TSX, JavaScript, JSX, and Python.

### Parsing

- `ast_parse_file` parses valid `.ts`, `.tsx`, `.js`, `.jsx`, and `.py` files.
- It reports syntax errors when Tree-sitter detects them.
- It enforces file size limits.

### Outline

- `ast_file_outline` returns classes/functions/imports for TypeScript/JavaScript.
- `ast_file_outline` returns classes/functions/imports for Python.
- Outline output is bounded and deterministic.

### Top-level nodes

- `ast_top_level_nodes` returns direct root children in source order.
- It optionally includes node text.

### Enclosing node

- `ast_enclosing_node` finds the smallest node at a valid position.
- It supports kind filtering.
- It returns ancestors.

### Imports

- `ast_find_imports` detects ES imports.
- `ast_find_imports` detects CommonJS `require`, best effort.
- `ast_find_imports` detects Python imports and from-imports.

### Exports

- `ast_find_exports` detects TypeScript/JavaScript exports.
- `ast_find_exports` returns Python public definitions and `__all__`, best effort.

### Functions and classes

- `ast_find_functions` detects functions and methods.
- `ast_find_classes` detects classes and methods.
- It returns parameters and return type text where structurally available.

### Chunking

- `ast_chunk_file` returns syntax-aligned chunks.
- It respects line and byte limits.
- It includes import chunks when requested.

### Query

- `ast_query` runs valid Tree-sitter queries.
- It returns captures with normalized ranges.
- Invalid queries return structured errors.

### Safety

- Paths outside the workspace are rejected.
- Unsupported extensions return `unsupported_language`.
- V1 tools never write files.
- V1 tools never call LSP services.
- Output limits are enforced.

---

## 35. Testing Requirements

### Unit tests

Required unit tests:

```text
workspace path validation
workspace-relative path conversion
extension-to-language routing
UTF-16 position to byte offset conversion
byte offset to UTF-16 position conversion
node counting
syntax error detection
name extraction
import extraction
export extraction
function extraction
class extraction
chunk ID generation
query compile error handling
result truncation
```

### Integration tests

Required integration tests:

```text
parse TypeScript file
parse TSX file
parse JavaScript file
parse JSX file
parse Python file
extract TypeScript imports/exports
extract Python imports
extract TypeScript functions/classes
extract Python functions/classes
chunk TypeScript file
chunk Python file
run TypeScript Tree-sitter query
run Python Tree-sitter query
```

### Safety tests

Required safety tests:

```text
reject ../outside.ts
reject /etc/passwd
reject unsupported extension
reject missing file
reject directory path
reject too-large file
truncate large output
never write files
no LSP dependency
```

---

## 36. Future Version Hooks

V1 should be designed so later versions can add:

### AST V2

```text
ast_enclosing_scope
ast_node_at_range
ast_node_text
ast_context_for_range
ast_context_pack
ast_find_calls
ast_find_member_access
ast_find_literals
ast_find_template_literals
ast_query_workspace
ast_file_metrics
```

### AST V3

```text
ast_find_routes
ast_find_react_components
ast_find_hooks
ast_find_tests
ast_find_decorators
ast_find_schema_definitions
ast_dependency_edges
```

### AST V4

```text
ast_rewrite_preview
ast_insert_import_preview
ast_wrap_node_preview
ast_add_decorator_preview
ast_modify_function_signature_preview
ast_validate_rewrite
ast_parse_after_rewrite
```

### AST V5

```text
ast_complexity_summary
ast_detect_large_nodes
ast_cache_status
ast_clear_caches
ast_request_log
ast_get_config
ast_update_runtime_config
ast_readiness
ast_liveness
```

Do not implement these in V1, but avoid designs that make them difficult.

---

## 37. Final V1 Tool Surface

```ts
type AstMcpV1 = {
  ast_health_check(input: AstHealthCheckInput): AstHealthCheckResult;
  ast_list_supported_languages(input: AstListSupportedLanguagesInput): AstListSupportedLanguagesResult;
  ast_parse_file(input: AstParseFileInput): AstParseFileResult;

  ast_file_outline(input: AstFileOutlineInput): AstFileOutlineResult;
  ast_top_level_nodes(input: AstTopLevelNodesInput): AstTopLevelNodesResult;
  ast_enclosing_node(input: AstEnclosingNodeInput): AstEnclosingNodeResult;

  ast_find_imports(input: AstFindImportsInput): AstFindImportsResult;
  ast_find_exports(input: AstFindExportsInput): AstFindExportsResult;
  ast_find_functions(input: AstFindFunctionsInput): AstFindFunctionsResult;
  ast_find_classes(input: AstFindClassesInput): AstFindClassesResult;

  ast_chunk_file(input: AstChunkFileInput): AstChunkFileResult;
  ast_query(input: AstQueryInput): AstQueryResult;
};
```

---

## 38. Final Design Principle

AST MCP V1 should be a fast, safe, structural code-intelligence server.

It should answer:

```text
What is the syntax structure of this file?
What top-level declarations exist?
What node contains this position?
What imports and exports exist?
What functions and classes exist?
How should this file be chunked?
What nodes match this Tree-sitter query?
```

It should not answer:

```text
What type is this symbol?
Where is this symbol referenced semantically?
Can this symbol be renamed safely across the project?
What diagnostics does the compiler report?
```

The clean boundary is:

```text
Need structure? Use AST MCP.
Need meaning? Use LSP MCP.
Need both? Use Agent Skills or Code Composite MCP.
```
