use serde::Serialize;
use tree_sitter::Tree;

use crate::shared::errors::AstToolError;

use crate::safety::limits;

/// A single source code chunk.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstChunk {
    pub start_line: u32,
    pub end_line: u32,
    pub start_byte: usize,
    pub end_byte: usize,
    pub text: String,
    pub kind: String,
}

/// Strategy for splitting a file into chunks.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkStrategy {
    TopLevel,
    FunctionClass,
    SemanticBlocks,
    MaxLinesWithAstBoundaries,
}

/// Options controlling chunking.
#[derive(Debug, Clone)]
pub struct ChunkOptions {
    pub strategy: ChunkStrategy,
    pub max_results: usize,
    pub max_lines_per_chunk: usize,
    pub max_bytes_per_chunk: usize,
}

impl Default for ChunkOptions {
    fn default() -> Self {
        ChunkOptions {
            strategy: ChunkStrategy::TopLevel,
            max_results: limits::MAX_RESULTS,
            max_lines_per_chunk: limits::MAX_CHUNK_LINES,
            max_bytes_per_chunk: limits::MAX_CHUNK_BYTES,
        }
    }
}

/// Result of chunking a file.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkResult {
    pub chunks: Vec<AstChunk>,
    pub strategy: String,
    pub total_chunks: usize,
}

/// Split a parsed file into chunks according to the given strategy.
pub fn chunk_file(
    tree: &Tree,
    source: &str,
    _file_path: &str,
    opts: &ChunkOptions,
) -> Result<ChunkResult, AstToolError> {
    let chunks = match opts.strategy {
        ChunkStrategy::TopLevel => chunk_top_level(tree, source, opts),
        ChunkStrategy::FunctionClass => chunk_function_class(tree, source, opts),
        ChunkStrategy::SemanticBlocks => chunk_semantic_blocks(tree, source, opts),
        ChunkStrategy::MaxLinesWithAstBoundaries => chunk_max_lines(tree, source, opts),
    };

    let total = chunks.len();
    let strategy_name = match opts.strategy {
        ChunkStrategy::TopLevel => "top_level",
        ChunkStrategy::FunctionClass => "function_class",
        ChunkStrategy::SemanticBlocks => "semantic_blocks",
        ChunkStrategy::MaxLinesWithAstBoundaries => "max_lines_with_ast_boundaries",
    };

    Ok(ChunkResult { chunks, strategy: strategy_name.to_string(), total_chunks: total })
}

// ---------------------------------------------------------------------------
// Strategy: TopLevel
// ---------------------------------------------------------------------------

fn chunk_top_level(tree: &Tree, source: &str, opts: &ChunkOptions) -> Vec<AstChunk> {
    let root = tree.root_node();
    let mut chunks = Vec::new();

    for i in 0..root.child_count() {
        if chunks.len() >= opts.max_results {
            break;
        }
        let child = match root.child(i) {
            Some(c) => c,
            None => continue,
        };
        if !child.is_named() {
            continue;
        }

        let text = &source[child.start_byte()..child.end_byte()];
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }

        let start_pos = child.start_position();
        let end_pos = child.end_position();

        let chunk = AstChunk {
            start_line: start_pos.row as u32,
            end_line: end_pos.row as u32,
            start_byte: child.start_byte(),
            end_byte: child.end_byte(),
            text: capped_text(trimmed, opts.max_bytes_per_chunk),
            kind: child.kind().to_string(),
        };

        chunks.push(chunk);
    }

    chunks
}

// ---------------------------------------------------------------------------
// Strategy: FunctionClass
// ---------------------------------------------------------------------------

fn chunk_function_class(tree: &Tree, source: &str, opts: &ChunkOptions) -> Vec<AstChunk> {
    let root = tree.root_node();
    let mut chunks = Vec::new();

    collect_function_class_chunks(&root, source, opts, &mut chunks);
    chunks
}

fn collect_function_class_chunks(
    node: &tree_sitter::Node,
    source: &str,
    opts: &ChunkOptions,
    chunks: &mut Vec<AstChunk>,
) {
    if chunks.len() >= opts.max_results {
        return;
    }

    for i in 0..node.child_count() {
        if chunks.len() >= opts.max_results {
            return;
        }
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if !child.is_named() {
            continue;
        }

        let is_fn_or_class = matches!(
            child.kind(),
            "function_declaration"
                | "generator_function_declaration"
                | "method_definition"
                | "arrow_function"
                | "function_signature"
                | "class_declaration"
                | "class_expression"
                | "function_definition"
                | "class_definition"
                | "decorated_definition"
                | "export_statement"
        );

        if is_fn_or_class {
            let text = &source[child.start_byte()..child.end_byte()];
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                let start_pos = child.start_position();
                let end_pos = child.end_position();

                chunks.push(AstChunk {
                    start_line: start_pos.row as u32,
                    end_line: end_pos.row as u32,
                    start_byte: child.start_byte(),
                    end_byte: child.end_byte(),
                    text: capped_text(trimmed, opts.max_bytes_per_chunk),
                    kind: child.kind().to_string(),
                });
            }
        } else {
            // Recurse into block-like structures
            if should_descend_for_chunks(child.kind()) {
                collect_function_class_chunks(&child, source, opts, chunks);
            }
        }
    }
}

fn should_descend_for_chunks(kind: &str) -> bool {
    matches!(
        kind,
        "program"
            | "module"
            | "statement_block"
            | "block"
            | "class_body"
            | "body"
            | "export_statement"
    )
}

// ---------------------------------------------------------------------------
// Strategy: SemanticBlocks
// ---------------------------------------------------------------------------

fn chunk_semantic_blocks(tree: &Tree, source: &str, opts: &ChunkOptions) -> Vec<AstChunk> {
    let root = tree.root_node();
    let mut chunks = Vec::new();

    collect_semantic_chunks(&root, source, opts, &mut chunks);
    chunks
}

fn collect_semantic_chunks(
    node: &tree_sitter::Node,
    source: &str,
    opts: &ChunkOptions,
    chunks: &mut Vec<AstChunk>,
) {
    if chunks.len() >= opts.max_results {
        return;
    }

    for i in 0..node.child_count() {
        if chunks.len() >= opts.max_results {
            return;
        }
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };
        if !child.is_named() {
            continue;
        }

        let is_semantic = matches!(
            child.kind(),
            "function_declaration"
                | "generator_function_declaration"
                | "method_definition"
                | "arrow_function"
                | "function_signature"
                | "class_declaration"
                | "class_expression"
                | "function_definition"
                | "class_definition"
                | "decorated_definition"
                | "export_statement"
                | "import_statement"
                | "import_from_statement"
                | "if_statement"
                | "for_statement"
                | "for_in_statement"
                | "while_statement"
                | "switch_statement"
                | "try_statement"
                | "lexical_declaration"
                | "variable_declaration"
        );

        if is_semantic {
            let text = &source[child.start_byte()..child.end_byte()];
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                let start_pos = child.start_position();
                let end_pos = child.end_position();

                chunks.push(AstChunk {
                    start_line: start_pos.row as u32,
                    end_line: end_pos.row as u32,
                    start_byte: child.start_byte(),
                    end_byte: child.end_byte(),
                    text: capped_text(trimmed, opts.max_bytes_per_chunk),
                    kind: child.kind().to_string(),
                });
            }
        } else if should_descend_for_chunks(child.kind()) {
            collect_semantic_chunks(&child, source, opts, chunks);
        }
    }
}

// ---------------------------------------------------------------------------
// Strategy: MaxLinesWithAstBoundaries
// ---------------------------------------------------------------------------

fn chunk_max_lines(tree: &Tree, source: &str, opts: &ChunkOptions) -> Vec<AstChunk> {
    // First, get all top-level named nodes as boundaries
    let root = tree.root_node();
    let mut boundaries: Vec<(usize, usize, String)> = Vec::new(); // (start_byte, end_byte, kind)

    for i in 0..root.child_count() {
        let child = match root.child(i) {
            Some(c) => c,
            None => continue,
        };
        if !child.is_named() {
            continue;
        }
        boundaries.push((child.start_byte(), child.end_byte(), child.kind().to_string()));
    }

    if boundaries.is_empty() {
        // No named nodes, return whole file as one chunk
        let lines: Vec<&str> = source.lines().collect();
        let text = capped_text(source, opts.max_bytes_per_chunk);
        return vec![AstChunk {
            start_line: 0,
            end_line: lines.len().saturating_sub(1) as u32,
            start_byte: 0,
            end_byte: source.len(),
            text,
            kind: "file".to_string(),
        }];
    }

    let max_lines = opts.max_lines_per_chunk.max(1);
    let source_len = source.len();
    let mut chunks = Vec::new();
    let mut current_start = boundaries[0].0;
    let mut current_lines: usize = 0;
    let mut last_byte = current_start;

    for (start_byte, end_byte, _kind) in &boundaries {
        if chunks.len() >= opts.max_results {
            break;
        }

        let node_source = &source[*start_byte..*end_byte];
        let node_lines = node_source.lines().count();

        if current_lines + node_lines > max_lines && current_lines > 0 {
            // Flush current chunk
            let text = capped_text(&source[current_start..last_byte], opts.max_bytes_per_chunk);
            let start_line = byte_to_line(source, current_start);
            let end_line = byte_to_line(source, last_byte.saturating_sub(1));

            chunks.push(AstChunk {
                start_line: start_line as u32,
                end_line: end_line as u32,
                start_byte: current_start,
                end_byte: last_byte,
                text,
                kind: "chunk".to_string(),
            });

            current_start = *start_byte;
            current_lines = 0;
        }

        current_lines += node_lines;
        last_byte = *end_byte;
    }

    // Flush remaining
    if last_byte > current_start && chunks.len() < opts.max_results {
        let text = capped_text(
            &source[current_start..last_byte.min(source_len)],
            opts.max_bytes_per_chunk,
        );
        let start_line = byte_to_line(source, current_start);
        let end_line =
            byte_to_line(source, last_byte.saturating_sub(1).min(source_len.saturating_sub(1)));

        chunks.push(AstChunk {
            start_line: start_line as u32,
            end_line: end_line as u32,
            start_byte: current_start,
            end_byte: last_byte.min(source_len),
            text,
            kind: "chunk".to_string(),
        });
    }

    chunks
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn capped_text(text: &str, max_bytes: usize) -> String {
    if text.len() > max_bytes {
        let mut bound = max_bytes;
        while bound > 0 && !text.is_char_boundary(bound) {
            bound -= 1;
        }
        text[..bound].to_string()
    } else {
        text.to_string()
    }
}

fn byte_to_line(source: &str, byte: usize) -> usize {
    let byte = byte.min(source.len());
    source[..byte].lines().count().saturating_sub(1)
}
