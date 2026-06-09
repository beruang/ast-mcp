/// Maximum file size in bytes (1 MiB).
pub const MAX_FILE_BYTES: u64 = 1_048_576;

/// Maximum AST nodes to return in any single response.
pub const MAX_NODES: usize = 500;

/// Maximum results for list-returning tools.
pub const MAX_RESULTS: usize = 200;

/// Maximum source text bytes to include in any response.
pub const MAX_TEXT_BYTES: usize = 20_000;

/// Maximum lines per chunk.
pub const MAX_CHUNK_LINES: usize = 120;

/// Maximum bytes per chunk.
pub const MAX_CHUNK_BYTES: usize = 30_000;

/// Maximum query matches.
pub const MAX_QUERY_MATCHES: usize = 200;

/// Parse timeout in milliseconds.
pub const PARSE_TIMEOUT_MS: u64 = 5_000;

/// Query execution timeout in milliseconds.
pub const QUERY_TIMEOUT_MS: u64 = 5_000;

// ── V2 defaults ──

/// Maximum bytes for context pack responses.
pub const MAX_CONTEXT_BYTES: usize = 30_000;

/// Maximum files scanned in a workspace query.
pub const MAX_WORKSPACE_QUERY_FILES: usize = 200;

/// Maximum total results across all files in a workspace query.
pub const MAX_WORKSPACE_QUERY_RESULTS: usize = 1_000;

/// Maximum bytes per file in workspace query (skip larger files).
pub const MAX_BYTES_PER_WORKSPACE_FILE: usize = 1_000_000;

/// Workspace query timeout in milliseconds.
pub const WORKSPACE_QUERY_TIMEOUT_MS: u64 = 20_000;

/// Maximum parallel threads for workspace queries.
pub const MAX_WORKSPACE_PARALLELISM: usize = 8;
