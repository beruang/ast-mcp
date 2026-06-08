use serde::{Deserialize, Serialize};
use tree_sitter::Tree;

use crate::parser::positions::ts_point_to_position;
use crate::shared::language::LanguageId;
use crate::shared::position::Range;

use super::OutlineCandidate;

/// A node in the file outline tree returned to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineNode {
    pub kind: String,
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<OutlineNode>>,
}

/// Options controlling outline extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OutlineOptions {
    /// Maximum depth to descend (default: 4).
    pub max_depth: usize,
    /// Whether to include range information (default: true).
    pub include_ranges: bool,
    /// Whether to include import statements (default: false).
    pub include_imports: bool,
    /// Whether to include export statements (default: false).
    pub include_exports: bool,
}

impl Default for OutlineOptions {
    fn default() -> Self {
        OutlineOptions {
            max_depth: 4,
            include_ranges: true,
            include_imports: false,
            include_exports: false,
        }
    }
}

/// The result of a file-outline request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFileOutlineResult {
    pub file_path: String,
    pub language: String,
    pub outline_text: String,
    pub nodes: Vec<OutlineNode>,
    pub truncated: bool,
}

/// Extract a structured outline from a parse tree.
///
/// Delegates to the language-specific `outline_candidates` function,
/// then converts the resulting candidates into `OutlineNode` values
/// (respecting `opts.max_depth` and filter flags) and renders a
/// deterministic plain-text representation.
pub fn file_outline(
    tree: &Tree,
    source: &str,
    opts: &OutlineOptions,
    lang: LanguageId,
    file_path: &str,
) -> AstFileOutlineResult {
    let candidates = get_outline_candidates(tree.root_node(), source, lang);

    let mut nodes: Vec<OutlineNode> = Vec::new();
    let mut outline_lines: Vec<String> = Vec::new();
    let mut truncated = false;

    for c in &candidates {
        // Filter imports / exports
        if !opts.include_imports && is_import_kind(&c.kind) {
            continue;
        }
        if !opts.include_exports && is_export_kind(&c.kind) {
            continue;
        }

        if nodes.len() >= crate::safety::limits::MAX_NODES {
            truncated = true;
            break;
        }

        let node = candidate_to_outline_node(c, source, opts, 0, &mut truncated);
        append_outline_text(&node, 0, &mut outline_lines);
        nodes.push(node);
    }

    let outline_text = outline_lines.join("\n");

    AstFileOutlineResult {
        file_path: file_path.to_string(),
        language: lang.as_str().to_string(),
        outline_text,
        nodes,
        truncated,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a single `OutlineCandidate` into an `OutlineNode`, recursing into
/// children up to `opts.max_depth`.
#[allow(clippy::only_used_in_recursion)]
fn candidate_to_outline_node(
    c: &OutlineCandidate,
    source: &str,
    opts: &OutlineOptions,
    depth: usize,
    truncated: &mut bool,
) -> OutlineNode {
    let range = if opts.include_ranges { Some(c.range) } else { None };

    let children: Option<Vec<OutlineNode>> = if depth < opts.max_depth && !c.children.is_empty() {
        let mut kids: Vec<OutlineNode> = Vec::new();
        for child in &c.children {
            if kids.len() >= crate::safety::limits::MAX_NODES {
                *truncated = true;
                break;
            }
            kids.push(candidate_to_outline_node(child, source, opts, depth + 1, truncated));
        }
        if kids.is_empty() {
            None
        } else {
            Some(kids)
        }
    } else if !c.children.is_empty() {
        // Depth limit reached — signal that children exist but aren't included.
        Some(vec![])
    } else {
        None
    };

    OutlineNode { kind: c.kind.clone(), name: c.name.clone(), range, children }
}

/// Append a human-readable line (and its children, indented) to `lines`.
fn append_outline_text(node: &OutlineNode, depth: usize, lines: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    let name = node.name.as_deref().unwrap_or("<anonymous>");
    lines.push(format!("{}{}: {}", indent, node.kind, name));

    if let Some(children) = &node.children {
        for child in children {
            append_outline_text(child, depth + 1, lines);
        }
    }
}

/// Extract the "name" from a node by looking for an identifier-like child.
fn node_name(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_named() {
            match child.kind() {
                "identifier"
                | "property_identifier"
                | "shorthand_property_identifier"
                | "statement_identifier" => {
                    return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                }
                _ => {
                    // Try deeper for unwrapped declarations.
                    if let Some(name) = node_name(&child, source) {
                        return Some(name);
                    }
                }
            }
        }
    }
    None
}

/// Extract an import module path as a name.
fn import_name(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "import" {
            // The next named sibling is the import clause
            continue;
        }
        if child.kind() == "string" || child.kind() == "string_fragment" {
            // Python: string inside import_statement
            return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
        }
        if child.kind() == "import_clause" {
            // TS/JS: import clause contains the identifier or namespace
            if let Some(name) = node_name(&child, source) {
                return Some(name);
            }
        }
    }
    // Fall back to the first string child.
    let mut cursor2 = node.walk();
    for child in node.children(&mut cursor2) {
        if child.kind() == "string" || child.kind() == "string_fragment" {
            return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
        }
        if child.kind() == "dotted_name" {
            return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
        }
    }
    node_name(node, source)
}

fn is_import_kind(kind: &str) -> bool {
    kind == "import_statement" || kind == "import_from_statement"
}

fn is_export_kind(kind: &str) -> bool {
    kind == "export_statement"
}

// ---------------------------------------------------------------------------
// Language dispatch
// ---------------------------------------------------------------------------

fn get_outline_candidates(
    root: tree_sitter::Node,
    source: &str,
    lang: LanguageId,
) -> Vec<OutlineCandidate> {
    match lang {
        LanguageId::TypeScript | LanguageId::TypeScriptReact => {
            crate::languages::typescript::outline_candidates(root, source)
        }
        LanguageId::JavaScript | LanguageId::JavaScriptReact => {
            crate::languages::javascript::outline_candidates(root, source)
        }
        LanguageId::Python => crate::languages::python::outline_candidates(root, source),
    }
}

// ---------------------------------------------------------------------------
// Public helpers (also used by top_level and language modules)
// ---------------------------------------------------------------------------

/// Build a single OutlineCandidate from a tree-sitter node.
/// Uses `node_name` (or `import_name` for imports) to extract the display name.
pub fn make_candidate(node: &tree_sitter::Node, source: &str) -> OutlineCandidate {
    let kind = node.kind().to_string();
    let name =
        if is_import_kind(&kind) { import_name(node, source) } else { node_name(node, source) };

    let start_pos = ts_point_to_position(node.start_position(), source);
    let end_pos = ts_point_to_position(node.end_position(), source);

    OutlineCandidate {
        kind,
        name,
        range: Range { start: start_pos, end: end_pos },
        children: Vec::new(),
    }
}

/// Recursively walk named children of `parent` looking for significant nodes.
/// Calls the `is_significant` closure to decide whether a child should be
/// included.
pub fn collect_candidates<F>(
    parent: &tree_sitter::Node,
    source: &str,
    is_significant: &F,
) -> Vec<OutlineCandidate>
where
    F: Fn(&str) -> bool,
{
    let mut result: Vec<OutlineCandidate> = Vec::new();
    let mut cursor = parent.walk();
    for child in parent.children(&mut cursor) {
        if result.len() >= crate::safety::limits::MAX_NODES {
            break;
        }
        if !child.is_named() {
            continue;
        }
        let kind = child.kind();
        // Unwrap decorated_definition for Python.
        if kind == "decorated_definition" {
            if let Some(inner) = child.child_by_field_name("definition") {
                if is_significant(inner.kind()) {
                    result.push(make_candidate(&inner, source));
                }
            }
            continue;
        }
        if is_significant(kind) {
            result.push(make_candidate(&child, source));
        }
    }
    result
}
