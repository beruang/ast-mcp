# AST MCP Server — Version 3 Specification

## 1. Purpose

Version 3 extends the Rust-based AST MCP server with **framework-aware structural extraction**.

Version 1 established the core AST parser and structural MVP:

- health checks
- parser registry
- parse file
- file outline
- top-level nodes
- enclosing node
- imports
- exports
- functions
- classes
- file chunking
- Tree-sitter query support

Version 2 added focused context and pattern search:

- enclosing scope
- node lookup by range
- node text extraction
- context for range
- context packs
- call expression extraction
- member access extraction
- literal extraction
- template literal extraction
- workspace queries
- file metrics

Version 3 adds application-aware extraction for common real-world codebase structures:

- routes
- React components
- React hooks
- tests
- decorators / annotations / attributes
- schema and model definitions
- dependency edges

The goal of V3 is to help coding agents understand **application structure**, not only generic syntax.

V3 remains AST-only. It must not call LSP tools or depend on LSP MCP.

---

## 2. Architectural Boundary

The AST MCP server is a structural analysis service.

It may use:

```text
Tree-sitter parsers
source text
syntax nodes
queries
framework-specific structural rules
workspace file scanning
bounded glob traversal
```

It must not use:

```text
LSP MCP
lsp_* tools
language servers
semantic references
type checking
compiler diagnostics
runtime execution
shell commands
```

### AST MCP owns

```text
syntax structure
imports and exports
functions and classes
calls and literals
routes
tests
components
hooks
decorators
schema/model declarations
syntax-level dependency edges
```

### AST MCP does not own

```text
semantic symbol identity
type resolution
find references
rename across project
diagnostics
implementation lookup
call hierarchy from compiler/language server
```

Those belong to LSP MCP.

Composite workflows that need both AST and LSP belong in:

```text
Agent Skills
Code Composite MCP
client-side orchestration layer
```

---

## 3. Version 3 Goals

V3 adds these tools:

```text
ast_find_routes
ast_find_react_components
ast_find_hooks
ast_find_tests
ast_find_decorators
ast_find_schema_definitions
ast_dependency_edges
```

V3 should support framework-aware extraction in a best-effort, structural way.

The output must include confidence and evidence where detection is heuristic.

---

## 4. Version 3 Non-Goals

V3 does not include:

```text
direct file mutation
rewrite previews
import insertion
syntax rewrite validation
semantic route resolution
semantic dependency resolution
type-aware component analysis
runtime framework introspection
running tests
running build tools
executing user code
LSP calls
complete framework coverage
persistent dependency graph database
```

Rewrite previews belong to AST V4.

Production hardening and cache operations belong to AST V5.

---

## 5. Required Baseline From V1 and V2

V3 assumes all V1 and V2 tools already exist.

### V1 baseline

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

### V2 baseline

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

V3 must preserve backward compatibility with V1 and V2 schemas.

---

## 6. Recommended Language and Framework Coverage

V3 should initially focus on:

```text
TypeScript / JavaScript / TSX / JSX
Python
```

Then add:

```text
Go
Rust
```

Later:

```text
Java
C/C++
```

### Initial framework targets

```text
TypeScript / JavaScript:
  Express
  Fastify
  Hono
  NestJS decorators
  Next.js route handlers
  React components
  React hooks
  Jest / Vitest / Mocha tests
  Zod schemas

Python:
  FastAPI
  Flask
  Django URL patterns, best effort
  Pytest
  unittest
  Pydantic models
  SQLAlchemy models, best effort

Go:
  net/http
  chi, best effort
  gin, best effort
  Go test functions

Rust:
  axum, best effort
  actix-web, best effort
  #[test] functions
  struct / enum / trait schema-like declarations
```

Framework support should be modular and clearly marked as best effort.

---

## 7. Shared Types

V3 uses the shared AST MCP types from V1 and V2.

### Position

```rust
type Position = {
  line: u32,
  character: u32,
}
```

External API positions should be zero-based and LSP-compatible UTF-16 offsets where possible.

Internally, Tree-sitter uses byte offsets and points. The AST MCP must convert safely between external positions and byte offsets.

### Range

```rust
type Range = {
  start: Position,
  end: Position,
}
```

### FileRange

```rust
type FileRange = {
  file_path: String, // workspace-relative
  range: Range,
}
```

### ToolError

```rust
type ToolError = {
  error: {
    code: String,
    message: String,
    details: Option<serde_json::Value>,
  }
}
```

---

## 8. Shared V3 Types

### Confidence

```rust
type Confidence = "low" | "medium" | "high";
```

Use confidence whenever detection is heuristic.

### Evidence

```rust
type Evidence = {
  kind: String,
  text: Option<String>,
  range: Option<Range>,
  node_kind: Option<String>,
}
```

### FrameworkDetection

```rust
type FrameworkDetection = {
  framework: String,
  confidence: Confidence,
  evidence: Vec<Evidence>,
}
```

### ExtractedItemBase

```rust
type ExtractedItemBase = {
  file_path: String,
  language: String,
  range: Range,
  confidence: Confidence,
  evidence: Vec<Evidence>,
}
```

---

## 9. Tool: ast_find_routes

### Purpose

Find route definitions in application code.

This tool detects routes structurally. It does not semantically resolve handler references unless the handler is syntactically inline or directly named.

### Input Schema

```rust
type AstFindRoutesInput = {
  file_path: Option<String>,
  glob: Option<String>,
  frameworks: Option<Vec<String>>,
  max_files: Option<u32>,
  max_results: Option<u32>,
  include_handler_context: Option<bool>,
}
```

Defaults:

```json
{
  "max_files": 200,
  "max_results": 500,
  "include_handler_context": false
}
```

Rules:

- Either `file_path` or `glob` must be provided.
- If `file_path` is provided, analyze one file.
- If `glob` is provided, run a bounded workspace scan.
- `frameworks` filters detectors.

### Output Schema

```rust
type AstFindRoutesResult = {
  routes: Vec<AstRoute>,
  returned: u32,
  truncated: bool,
  scanned_files: u32,
}
```

```rust
type AstRoute = {
  file_path: String,
  language: String,
  framework: String,
  method: Option<String>,
  path: Option<String>,
  handler_name: Option<String>,
  handler_kind: Option<String>,
  range: Range,
  path_range: Option<Range>,
  handler_range: Option<Range>,
  confidence: Confidence,
  evidence: Vec<Evidence>,
}
```

### Supported V3 patterns

#### Express / Fastify / Hono

Examples:

```ts
app.get('/users/:id', getUserHandler);
router.post('/users', async (req, res) => {});
server.route({ method: 'GET', url: '/users/:id', handler });
```

Detect:

```text
app.get
app.post
app.put
app.patch
app.delete
router.get
router.post
fastify.get
fastify.post
server.route
app.route
```

#### Next.js route handlers

Examples:

```ts
export async function GET(request: Request) {}
export async function POST(request: Request) {}
```

Detect exported functions named:

```text
GET
POST
PUT
PATCH
DELETE
HEAD
OPTIONS
```

Path may be inferred from file path if the project uses `app/**/route.ts` or `pages/api/**`.

This inference should be marked `confidence: medium` unless routing convention is clear.

#### NestJS

Examples:

```ts
@Controller('/users')
class UserController {
  @Get('/:id')
  getUser() {}
}
```

Detect:

```text
@Controller
@Get
@Post
@Put
@Patch
@Delete
```

Combine controller prefix and method decorator path when syntactically available.

#### FastAPI

Examples:

```py
@app.get('/users/{id}')
async def get_user(id: str): ...

router.post('/users')
def create_user(): ...
```

Detect decorators:

```text
@app.get
@app.post
@router.get
@router.post
```

#### Flask

Examples:

```py
@app.route('/users/<id>', methods=['GET'])
def get_user(id): ...
```

Detect:

```text
@app.route
@blueprint.route
```

Extract `methods` list if available.

#### Django URL patterns, best effort

Examples:

```py
path('users/<str:id>/', views.get_user)
re_path(r'^users/(?P<id>[^/]+)/$', views.get_user)
```

Detect calls:

```text
path
re_path
url
```

### Behavior

1. Validate workspace path or glob.
2. Select parser by file extension.
3. Parse files.
4. Run framework detector modules.
5. Normalize route path/method/handler.
6. Attach confidence and evidence.
7. Apply result limits.
8. Return workspace-relative file paths.

### Safety

- Do not execute code.
- Do not import framework modules.
- Do not read files outside workspace.
- Bound workspace scans by `max_files` and `max_results`.

---

## 10. Tool: ast_find_react_components

### Purpose

Find React component definitions in TypeScript/JavaScript/TSX/JSX files.

### Input Schema

```rust
type AstFindReactComponentsInput = {
  file_path: Option<String>,
  glob: Option<String>,
  max_files: Option<u32>,
  max_results: Option<u32>,
  include_hooks: Option<bool>,
  include_jsx_summary: Option<bool>,
}
```

Defaults:

```json
{
  "max_files": 200,
  "max_results": 500,
  "include_hooks": true,
  "include_jsx_summary": false
}
```

### Output Schema

```rust
type AstFindReactComponentsResult = {
  components: Vec<AstReactComponent>,
  returned: u32,
  truncated: bool,
  scanned_files: u32,
}
```

```rust
type AstReactComponent = {
  file_path: String,
  name: String,
  kind: "function_component" | "arrow_function_component" | "class_component" | "memo_component" | "forward_ref_component" | "unknown",
  exported: bool,
  default_export: bool,
  props_name: Option<String>,
  props_type_text: Option<String>,
  hooks: Vec<String>,
  jsx_root: Option<String>,
  range: Range,
  confidence: Confidence,
  evidence: Vec<Evidence>,
}
```

### Supported V3 patterns

```tsx
export function UserCard(props: UserCardProps) {
  return <div />;
}

const UserCard = ({ user }: Props) => <div />;

export default function UserPage() {
  return <UserCard />;
}

const UserCard = memo(function UserCard() { return <div />; });

const UserCard = forwardRef<HTMLDivElement, Props>((props, ref) => <div ref={ref} />);
```

### Detection rules

High confidence:

```text
function or arrow function has PascalCase name and returns JSX
exported function with PascalCase name and JSX body
class extends React.Component or Component
memo/forwardRef wrapping component-like function
```

Medium confidence:

```text
PascalCase function with JSX inside body
PascalCase variable assigned to arrow function with JSX descendant
```

Low confidence:

```text
PascalCase function without direct JSX but likely component by naming/export
```

### Behavior

1. Validate path or glob.
2. Restrict to `.tsx`, `.jsx`, `.ts`, `.js` where parser supports JSX.
3. Parse files.
4. Detect component declarations.
5. Extract props parameter/type text when syntactically available.
6. If `include_hooks`, detect hook calls inside component range.
7. If `include_jsx_summary`, include root JSX element name where possible.
8. Apply limits.

### Safety

This tool must not execute React code or resolve imports semantically.

---

## 11. Tool: ast_find_hooks

### Purpose

Find React hooks and custom hook declarations/usages.

### Input Schema

```rust
type AstFindHooksInput = {
  file_path: Option<String>,
  glob: Option<String>,
  max_files: Option<u32>,
  max_results: Option<u32>,
  include_usages: Option<bool>,
  include_definitions: Option<bool>,
}
```

Defaults:

```json
{
  "max_files": 200,
  "max_results": 1000,
  "include_usages": true,
  "include_definitions": true
}
```

### Output Schema

```rust
type AstFindHooksResult = {
  hooks: Vec<AstHook>,
  returned: u32,
  truncated: bool,
  scanned_files: u32,
}
```

```rust
type AstHook = {
  file_path: String,
  name: String,
  kind: "builtin_usage" | "custom_usage" | "custom_definition",
  enclosing_component: Option<String>,
  range: Range,
  confidence: Confidence,
  evidence: Vec<Evidence>,
}
```

### Detection rules

Built-in hook usage:

```text
useState
useEffect
useMemo
useCallback
useRef
useReducer
useContext
useLayoutEffect
useImperativeHandle
useTransition
useDeferredValue
useId
useSyncExternalStore
```

Custom hook usage:

```text
Call expression callee starts with "use" followed by uppercase letter.
```

Custom hook definition:

```text
function useSomething(...)
const useSomething = (...) => ...
export function useSomething(...)
```

### Behavior

1. Parse supported JS/TS files.
2. Detect hook call expressions.
3. Detect custom hook declarations.
4. If possible, attach enclosing component/function from AST scopes.
5. Apply limits.

---

## 12. Tool: ast_find_tests

### Purpose

Find tests and test suites in source files.

### Input Schema

```rust
type AstFindTestsInput = {
  file_path: Option<String>,
  glob: Option<String>,
  frameworks: Option<Vec<String>>,
  max_files: Option<u32>,
  max_results: Option<u32>,
}
```

Defaults:

```json
{
  "max_files": 300,
  "max_results": 1000
}
```

### Output Schema

```rust
type AstFindTestsResult = {
  tests: Vec<AstTestItem>,
  returned: u32,
  truncated: bool,
  scanned_files: u32,
}
```

```rust
type AstTestItem = {
  file_path: String,
  language: String,
  framework: String,
  kind: "suite" | "test" | "fixture" | "hook" | "unknown",
  name: Option<String>,
  range: Range,
  parent_name: Option<String>,
  confidence: Confidence,
  evidence: Vec<Evidence>,
}
```

### Supported V3 patterns

#### Jest / Vitest / Mocha

Detect calls:

```text
describe
it
test
beforeEach
afterEach
beforeAll
afterAll
```

Examples:

```ts
describe('UserService', () => {
  it('returns user by id', () => {});
});
```

#### Pytest

Detect:

```text
functions named test_*
classes named Test*
@pytest.mark.* decorators
fixtures with @pytest.fixture
```

#### Python unittest

Detect:

```text
classes extending unittest.TestCase, best effort structurally
methods named test_*
```

#### Go tests

Detect:

```text
func TestXxx(t *testing.T)
func BenchmarkXxx(b *testing.B)
func FuzzXxx(f *testing.F)
```

#### Rust tests

Detect:

```text
#[test]
fn test_name() {}

#[tokio::test]
async fn test_name() {}
```

### Behavior

1. Select detector by language and framework filter.
2. Parse files.
3. Detect test/suite/fixture/hook nodes.
4. Attach parent suite where structurally available.
5. Apply limits.

---

## 13. Tool: ast_find_decorators

### Purpose

Find decorators, annotations, and attributes.

This is useful for frameworks like NestJS, Angular, FastAPI, Flask, Python decorators, Java annotations, and Rust attributes.

### Input Schema

```rust
type AstFindDecoratorsInput = {
  file_path: Option<String>,
  glob: Option<String>,
  names: Option<Vec<String>>,
  max_files: Option<u32>,
  max_results: Option<u32>,
}
```

Defaults:

```json
{
  "max_files": 200,
  "max_results": 1000
}
```

### Output Schema

```rust
type AstFindDecoratorsResult = {
  decorators: Vec<AstDecorator>,
  returned: u32,
  truncated: bool,
  scanned_files: u32,
}
```

```rust
type AstDecorator = {
  file_path: String,
  language: String,
  name: String,
  arguments_text: Vec<String>,
  target_kind: Option<String>,
  target_name: Option<String>,
  range: Range,
  target_range: Option<Range>,
  confidence: Confidence,
  evidence: Vec<Evidence>,
}
```

### Supported V3 patterns

TypeScript decorators:

```ts
@Controller('/users')
@Get('/:id')
@Injectable()
```

Python decorators:

```py
@app.get('/users')
@pytest.fixture
@dataclass
```

Rust attributes:

```rust
#[test]
#[derive(Debug)]
#[tokio::main]
```

Java annotations, if Java parser is enabled:

```java
@RestController
@GetMapping("/users")
@Test
```

### Behavior

1. Parse file(s).
2. Extract decorator/annotation/attribute nodes.
3. Normalize decorator name.
4. Extract argument text if available.
5. Attach target declaration kind/name/range when possible.
6. Filter by `names` if provided.

---

## 14. Tool: ast_find_schema_definitions

### Purpose

Find schema, model, validation, and data-shape definitions.

This is best-effort structural extraction.

### Input Schema

```rust
type AstFindSchemaDefinitionsInput = {
  file_path: Option<String>,
  glob: Option<String>,
  schema_kinds: Option<Vec<String>>,
  max_files: Option<u32>,
  max_results: Option<u32>,
  include_fields: Option<bool>,
}
```

Defaults:

```json
{
  "max_files": 300,
  "max_results": 1000,
  "include_fields": true
}
```

### Output Schema

```rust
type AstFindSchemaDefinitionsResult = {
  schemas: Vec<AstSchemaDefinition>,
  returned: u32,
  truncated: bool,
  scanned_files: u32,
}
```

```rust
type AstSchemaDefinition = {
  file_path: String,
  language: String,
  kind: String,
  name: Option<String>,
  framework: Option<String>,
  fields: Vec<AstSchemaField>,
  range: Range,
  confidence: Confidence,
  evidence: Vec<Evidence>,
}
```

```rust
type AstSchemaField = {
  name: String,
  type_text: Option<String>,
  required: Option<bool>,
  range: Option<Range>,
}
```

### Supported V3 patterns

#### TypeScript / JavaScript

Zod:

```ts
const UserSchema = z.object({
  id: z.string(),
  email: z.string().email(),
});
```

Yup, best effort:

```ts
const UserSchema = yup.object({ ... });
```

Mongoose, best effort:

```ts
new Schema({ name: String })
```

TypeScript interfaces/types as schema-like data shapes:

```ts
interface User { id: string; email: string }
type User = { id: string; email: string }
```

#### Python

Pydantic:

```py
class User(BaseModel):
    id: str
    email: str
```

Dataclasses:

```py
@dataclass
class User:
    id: str
```

SQLAlchemy, best effort:

```py
class User(Base):
    __tablename__ = 'users'
    id = Column(String, primary_key=True)
```

#### Go

Structs:

```go
type User struct {
  ID string `json:"id"`
}
```

#### Rust

Structs/enums:

```rust
struct User {
  id: String,
}
```

### Behavior

1. Parse file(s).
2. Run language-specific schema detectors.
3. Extract schema name and fields when syntactically available.
4. Attach framework/kind and confidence.
5. Apply limits.

### Confidence rules

High confidence:

```text
Known framework pattern such as z.object, BaseModel, dataclass, Go struct, Rust struct.
```

Medium confidence:

```text
Type/interface/object shape that looks schema-like but no framework marker.
```

Low confidence:

```text
Heuristic object shape or class with data fields only.
```

---

## 15. Tool: ast_dependency_edges

### Purpose

Extract syntax-level dependency edges from a file or bounded workspace scan.

This tool does not resolve dependencies semantically. It reports import/use/require/include edges as written.

### Input Schema

```rust
type AstDependencyEdgesInput = {
  file_path: Option<String>,
  glob: Option<String>,
  max_files: Option<u32>,
  max_results: Option<u32>,
  include_external: Option<bool>,
  include_relative: Option<bool>,
}
```

Defaults:

```json
{
  "max_files": 500,
  "max_results": 5000,
  "include_external": true,
  "include_relative": true
}
```

### Output Schema

```rust
type AstDependencyEdgesResult = {
  edges: Vec<AstDependencyEdge>,
  returned: u32,
  truncated: bool,
  scanned_files: u32,
}
```

```rust
type AstDependencyEdge = {
  from_file: String,
  to_specifier: String,
  kind: "import" | "export" | "require" | "use" | "include" | "mod" | "package" | "unknown",
  is_relative: bool,
  is_type_only: Option<bool>,
  range: Range,
  confidence: Confidence,
  evidence: Vec<Evidence>,
}
```

### Supported V3 patterns

TypeScript/JavaScript:

```text
import ... from "x"
export ... from "x"
require("x")
import("x") when string literal
```

Python:

```text
import x
from x import y
```

Go:

```text
import "x"
import alias "x"
```

Rust:

```text
use crate::x
use super::x
mod x
extern crate x
```

C/C++:

```text
#include <x>
#include "x"
```

Java:

```text
import com.example.X
```

### Behavior

1. Parse file(s).
2. Reuse V1 `ast_find_imports`, `ast_find_exports`, and language-specific extraction where possible.
3. Normalize dependency edges.
4. Do not resolve relative paths unless a future resolver is added.
5. Apply filters and limits.

---

## 16. Workspace Scan Rules

V3 introduces more workspace-wide extraction tools.

All workspace scans must be explicit and bounded.

Required constraints:

```text
max_files
max_results
workspace boundary validation
glob validation
ignore support for .git, node_modules, target, dist, build, vendor by default
```

Default ignored directories:

```text
.git
node_modules
dist
build
coverage
target
vendor
.venv
venv
__pycache__
.next
.nuxt
.turbo
```

Recommended Rust crates:

```text
ignore
globset
rayon
```

Workspace scans must return:

```text
scanned_files
returned
truncated
```

---

## 17. Framework Detector Architecture

V3 should use modular detector modules.

Recommended structure:

```text
src/
  frameworks/
    mod.rs
    routes/
      mod.rs
      express.rs
      fastify.rs
      hono.rs
      nextjs.rs
      nestjs.rs
      fastapi.rs
      flask.rs
      django.rs
      go_http.rs
      rust_axum.rs
    react/
      components.rs
      hooks.rs
    tests/
      javascript.rs
      python.rs
      go.rs
      rust.rs
    decorators/
      typescript.rs
      python.rs
      rust.rs
      java.rs
    schemas/
      zod.rs
      yup.rs
      pydantic.rs
      dataclass.rs
      sqlalchemy.rs
      go_struct.rs
      rust_struct.rs
    dependencies/
      mod.rs
```

Each detector should implement a common trait.

```rust
pub trait AstDetector<T> {
    fn detect(&self, ctx: &AstFileContext) -> Vec<T>;
}
```

Where:

```rust
pub struct AstFileContext<'a> {
    pub workspace_path: &'a Path,
    pub file_path: &'a Path,
    pub relative_path: &'a str,
    pub language: &'a str,
    pub source: &'a str,
    pub tree: &'a tree_sitter::Tree,
}
```

---

## 18. Parser and Query Requirements

V3 should continue using Tree-sitter queries for most detection.

Detector implementation may combine:

```text
Tree-sitter queries
manual tree walking
source text extraction
simple naming heuristics
file path conventions
```

Do not use regex-only detection as the primary mechanism unless a grammar lacks support for the target structure.

Regex may be used as a fallback with `confidence: low`.

---

## 19. Confidence and Evidence Requirements

Every framework-aware result must include:

```text
confidence
evidence
```

### Confidence definitions

High:

```text
Direct syntax match for known framework pattern.
```

Medium:

```text
Likely framework pattern with one missing or inferred piece, such as path inferred from file convention.
```

Low:

```text
Heuristic match or fallback detection.
```

### Evidence examples

```json
{
  "kind": "call_expression",
  "text": "router.get('/users/:id', getUser)",
  "range": {
    "start": { "line": 10, "character": 0 },
    "end": { "line": 10, "character": 36 }
  },
  "node_kind": "call_expression"
}
```

Evidence text should be bounded.

Default max evidence text length:

```text
500 characters
```

---

## 20. Error Codes Added in V3

```text
framework_not_supported
framework_detection_failed
route_detection_failed
component_detection_failed
hook_detection_failed
test_detection_failed
decorator_detection_failed
schema_detection_failed
dependency_extraction_failed
workspace_scan_limit_exceeded
invalid_glob
no_file_or_glob_provided
ambiguous_file_and_glob
```

Existing generic error codes remain valid:

```text
workspace_not_found
path_outside_workspace
file_not_found
unsupported_language
parse_failed
syntax_error
request_timeout
internal_error
```

---

## 21. Request Timeout Defaults

V3 adds these defaults:

```text
ast_find_routes single file: 5000ms
ast_find_routes workspace: 30000ms
ast_find_react_components single file: 5000ms
ast_find_react_components workspace: 30000ms
ast_find_hooks single file: 5000ms
ast_find_hooks workspace: 30000ms
ast_find_tests single file: 5000ms
ast_find_tests workspace: 30000ms
ast_find_decorators single file: 5000ms
ast_find_decorators workspace: 30000ms
ast_find_schema_definitions single file: 5000ms
ast_find_schema_definitions workspace: 30000ms
ast_dependency_edges single file: 5000ms
ast_dependency_edges workspace: 30000ms
```

Workspace scan timeout should be configurable.

---

## 22. Result Limits

Default limits:

```text
routes: 500
components: 500
hooks: 1000
tests: 1000
decorators: 1000
schemas: 1000
dependency edges: 5000
workspace max files: 300 for framework tools
workspace max files: 500 for dependency edges
```

All limited responses must include:

```text
returned
truncated
scanned_files
```

---

## 23. Safety Rules

V3 must obey these safety rules:

```text
1. Never read files outside workspace.
2. Never execute code.
3. Never import application modules.
4. Never call LSP MCP.
5. Never call shell commands.
6. Bound all workspace scans.
7. Respect ignored directories.
8. Return confidence for heuristic results.
9. Return evidence for framework-aware results.
10. Truncate large evidence text.
11. Do not write files.
12. Do not produce rewrites in V3.
```

---

## 24. Updated Tool Surface

```rust
type AstMcpV3 = {
  ast_find_routes(input: AstFindRoutesInput): AstFindRoutesResult;
  ast_find_react_components(input: AstFindReactComponentsInput): AstFindReactComponentsResult;
  ast_find_hooks(input: AstFindHooksInput): AstFindHooksResult;
  ast_find_tests(input: AstFindTestsInput): AstFindTestsResult;
  ast_find_decorators(input: AstFindDecoratorsInput): AstFindDecoratorsResult;
  ast_find_schema_definitions(input: AstFindSchemaDefinitionsInput): AstFindSchemaDefinitionsResult;
  ast_dependency_edges(input: AstDependencyEdgesInput): AstDependencyEdgesResult;
};
```

---

## 25. Example Tool Calls

### Find routes in one file

```json
{
  "tool": "ast_find_routes",
  "arguments": {
    "file_path": "src/routes.ts",
    "frameworks": ["express"],
    "include_handler_context": false
  }
}
```

### Find routes in workspace

```json
{
  "tool": "ast_find_routes",
  "arguments": {
    "glob": "src/**/*.{ts,tsx,js,jsx,py}",
    "max_files": 200,
    "max_results": 500
  }
}
```

### Find React components

```json
{
  "tool": "ast_find_react_components",
  "arguments": {
    "glob": "src/**/*.{tsx,jsx}",
    "include_hooks": true,
    "include_jsx_summary": true
  }
}
```

### Find hooks

```json
{
  "tool": "ast_find_hooks",
  "arguments": {
    "file_path": "src/components/UserCard.tsx",
    "include_usages": true,
    "include_definitions": true
  }
}
```

### Find tests

```json
{
  "tool": "ast_find_tests",
  "arguments": {
    "glob": "**/*.{test,spec}.{ts,tsx,js,jsx}",
    "frameworks": ["jest", "vitest"],
    "max_files": 300
  }
}
```

### Find decorators

```json
{
  "tool": "ast_find_decorators",
  "arguments": {
    "file_path": "src/users/user.controller.ts",
    "names": ["Controller", "Get", "Post"]
  }
}
```

### Find schemas

```json
{
  "tool": "ast_find_schema_definitions",
  "arguments": {
    "glob": "src/**/*.{ts,py,go,rs}",
    "schema_kinds": ["zod", "pydantic", "go_struct", "rust_struct"],
    "include_fields": true
  }
}
```

### Dependency edges

```json
{
  "tool": "ast_dependency_edges",
  "arguments": {
    "glob": "src/**/*.{ts,tsx,js,jsx,py,go,rs}",
    "include_external": true,
    "include_relative": true,
    "max_files": 500
  }
}
```

---

## 26. Development Milestones

### Milestone 1: Detector Infrastructure

Implement:

```text
AstDetector trait
AstFileContext
framework module layout
confidence/evidence helpers
bounded evidence text
```

### Milestone 2: Dependency Edges

Implement:

```text
ast_dependency_edges
TypeScript/JavaScript imports/exports/requires
Python imports
Go imports
Rust use/mod
```

### Milestone 3: Tests

Implement:

```text
ast_find_tests
Jest/Vitest/Mocha
Pytest
Go tests
Rust tests
```

### Milestone 4: Decorators

Implement:

```text
ast_find_decorators
TypeScript decorators
Python decorators
Rust attributes
```

### Milestone 5: Routes

Implement:

```text
ast_find_routes
Express/Fastify/Hono
NestJS decorators
Next.js route handlers
FastAPI
Flask
```

### Milestone 6: React Components and Hooks

Implement:

```text
ast_find_react_components
ast_find_hooks
function components
arrow components
memo/forwardRef
builtin hooks
custom hook declarations/usages
```

### Milestone 7: Schema Definitions

Implement:

```text
ast_find_schema_definitions
Zod
TypeScript interfaces/types
Pydantic
Dataclasses
Go structs
Rust structs
```

### Milestone 8: Workspace Scan Hardening

Implement:

```text
glob validation
ignore directories
max files
max results
timeouts
parallel scan bounds
```

### Milestone 9: Acceptance Tests

Run all V1, V2, and V3 acceptance tests.

---

## 27. Acceptance Criteria

V3 is acceptable when all V1 and V2 acceptance criteria still pass and the following are true.

### Routes

- `ast_find_routes` detects Express-style routes.
- `ast_find_routes` detects FastAPI-style decorators.
- `ast_find_routes` detects NestJS controller/method decorators where supported.
- Route outputs include method, path, framework, range, confidence, and evidence.
- Workspace scans are bounded.

### React components

- `ast_find_react_components` detects function components.
- It detects arrow function components.
- It detects default exported function components.
- It detects hooks used inside component range when requested.
- It returns confidence and evidence.

### Hooks

- `ast_find_hooks` detects built-in hook usages.
- It detects custom hook usages.
- It detects custom hook definitions.
- It attaches enclosing component/function where possible.

### Tests

- `ast_find_tests` detects Jest/Vitest test/suite calls.
- It detects Pytest test functions.
- It detects Go test functions.
- It detects Rust `#[test]` functions.

### Decorators

- `ast_find_decorators` detects TypeScript decorators.
- It detects Python decorators.
- It detects Rust attributes.
- It attaches target declaration when possible.

### Schemas

- `ast_find_schema_definitions` detects Zod schemas.
- It detects TypeScript interface/type object shapes.
- It detects Pydantic models.
- It detects Go structs.
- It detects Rust structs.
- It extracts fields where syntactically available.

### Dependency edges

- `ast_dependency_edges` extracts JS/TS imports/exports/requires.
- It extracts Python imports.
- It extracts Go imports.
- It extracts Rust use/mod declarations.
- It marks relative/external edges when possible.

### Decoupling

- AST MCP does not call LSP MCP.
- AST MCP does not depend on language servers.
- AST MCP does not perform semantic resolution.

### Safety

- No V3 tool writes files.
- No V3 tool executes code.
- No V3 tool reads outside workspace.
- Workspace scans enforce limits.
- Evidence text is bounded.

---

## 28. Testing Requirements

### Unit Tests

Required unit tests:

```text
route detector: Express
route detector: Fastify/Hono
route detector: NestJS
route detector: FastAPI
route detector: Flask
React function component detector
React arrow component detector
React memo/forwardRef detector
hook usage detector
custom hook definition detector
Jest/Vitest test detector
Pytest detector
Go test detector
Rust test attribute detector
TypeScript decorator detector
Python decorator detector
Rust attribute detector
Zod schema detector
Pydantic model detector
Go struct schema detector
Rust struct schema detector
dependency edge extraction per language
confidence assignment
evidence truncation
glob validation
workspace scan limit enforcement
```

### Integration Tests

Required integration tests:

```text
scan TypeScript app routes
scan Python FastAPI app routes
scan React component tree files
scan mixed tests in JS/Python/Go/Rust fixtures
scan schema definitions across TS/Python/Go/Rust fixtures
scan dependency edges across workspace fixture
```

### Safety Tests

Required safety tests:

```text
reject path outside workspace
reject invalid glob
reject ambiguous file_path and glob if policy requires only one
respect ignored directories
truncate workspace scan by max_files
truncate results by max_results
no file writes
no process execution
no LSP dependency
```

---

## 29. Recommended Rust Crates

Core crates:

```text
tree-sitter
tree-sitter-typescript
tree-sitter-javascript
tree-sitter-python
tree-sitter-go
tree-sitter-rust
serde
serde_json
schemars
anyhow
thiserror
tokio
```

Workspace scanning:

```text
ignore
globset
rayon
walkdir
```

Utilities:

```text
regex
indexmap
tracing
tracing-subscriber
```

Diff/rewrite crates are not required until V4.

---

## 30. Final V3 Design Principle

V3 makes AST MCP application-aware, not semantic.

```text
V3 may detect framework structures.
V3 may detect routes, tests, components, hooks, decorators, schemas, and dependency edges.
V3 may run bounded workspace scans.
V3 must not call LSP.
V3 must not execute code.
V3 must not write files.
```

The clean boundary remains:

```text
Need syntax/application structure? Use AST MCP.
Need semantic meaning/types/references? Use LSP MCP.
Need both? Use Agent Skills or Code Composite MCP.
```
