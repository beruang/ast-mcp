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
