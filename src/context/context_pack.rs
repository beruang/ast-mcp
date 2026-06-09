//! ast_context_pack — agent-ready structural context pack for a file position/range.
use serde::Deserialize;
use serde_json::json;

use crate::config::defaults::MAX_CONTEXT_BYTES;
use crate::config::workspace::Workspace;
use crate::extractors;
use crate::extractors::outline::OutlineOptions;
use crate::parser;
use crate::safety;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;
use crate::shared::position::{Position, Range};
use crate::shared::types_v2::{ContextBlock, ContextPackPart};
use crate::text::{position_encoding, text_budget};

#[derive(Deserialize)]
#[serde(default)]
pub struct AstContextPackInput {
    pub file_path: String,
    pub position: Option<serde_json::Value>,
    pub range: Option<Range>,
    pub include: Vec<ContextPackPart>,
    pub max_bytes: usize,
}

impl Default for AstContextPackInput {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            position: None,
            range: None,
            include: vec![
                ContextPackPart::Imports,
                ContextPackPart::Exports,
                ContextPackPart::EnclosingScope,
                ContextPackPart::EnclosingNode,
                ContextPackPart::TopLevelOutline,
            ],
            max_bytes: MAX_CONTEXT_BYTES,
        }
    }
}

pub fn handle(workspace: &Workspace, args: serde_json::Value) -> serde_json::Value {
    let input: AstContextPackInput = match serde_json::from_value(args) {
        Ok(v) => v,
        Err(_) => {
            return AstToolError::InvalidPosition("invalid input".into()).payload();
        }
    };

    let resolved = match safety::paths::resolve_file(workspace, &input.file_path) {
        Ok(r) => r,
        Err(e) => return e.payload(),
    };

    let meta = match std::fs::metadata(&resolved.absolute) {
        Ok(m) => m,
        Err(e) => return AstToolError::FileNotFound(e.to_string()).payload(),
    };
    if let Err(e) = safety::paths::ensure_under_size(meta.len()) {
        return e.payload();
    }

    let source = match std::fs::read_to_string(&resolved.absolute) {
        Ok(s) => s,
        Err(e) => return AstToolError::FileNotFound(e.to_string()).payload(),
    };

    let lang = match extension_to_language(&resolved.workspace_relative) {
        Some(l) => l,
        None => {
            let ext = std::path::Path::new(&resolved.workspace_relative)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            return AstToolError::UnsupportedLanguage(ext.to_string()).payload();
        }
    };

    let (tree, _status) = match parser::parse::parse_source(&source, lang) {
        Ok(t) => t,
        Err(e) => return e.payload(),
    };

    let root = tree.root_node();

    // Resolve target node from position or range
    let target_node: Option<tree_sitter::Node> = if let Some(ref range) = input.range {
        let (sb, eb) = match position_encoding::validate_range_in_bounds(&source, *range) {
            Ok(v) => v,
            Err(e) => return e.payload(),
        };
        crate::context::node_at_range_helpers::find_smallest_containing(root, sb, eb)
    } else {
        None
    };

    let mut budget = text_budget::TextBudget::new(input.max_bytes);
    let mut blocks: Vec<ContextBlock> = Vec::new();
    let mut summaries: Vec<serde_json::Value> = Vec::new();

    let fp = &resolved.workspace_relative;

    for part in &input.include {
        if budget.exceeded {
            break;
        }
        match part {
            ContextPackPart::Imports => {
                let imports = extractors::imports::find_imports(&tree, &source, lang);
                let text = serde_json::to_string(&imports).unwrap_or_default();
                let rem = budget.remaining();
                if rem > 0 && !text.is_empty() {
                    let (txt, truncated) = text_budget::truncate_text(&text, rem);
                    budget.try_spend(txt.len());
                    blocks.push(ContextBlock {
                        label: "imports".into(),
                        kind: "imports".into(),
                        file_path: fp.clone(),
                        range: zero_range(),
                        text: txt.into(),
                        truncated,
                    });
                }
            }
            ContextPackPart::Exports => {
                let exports = extractors::exports::find_exports(&tree, &source, lang, true);
                let text = serde_json::to_string(&exports).unwrap_or_default();
                let rem = budget.remaining();
                if rem > 0 && !text.is_empty() {
                    let (txt, truncated) = text_budget::truncate_text(&text, rem);
                    budget.try_spend(txt.len());
                    blocks.push(ContextBlock {
                        label: "exports".into(),
                        kind: "exports".into(),
                        file_path: fp.clone(),
                        range: zero_range(),
                        text: txt.into(),
                        truncated,
                    });
                }
            }
            ContextPackPart::EnclosingScope => {
                if let Some(ref tn) = target_node {
                    let scopes =
                        crate::context::enclosing_scope::ancestor_scopes(*tn, &source, false);
                    summaries.push(json!({"part": "enclosing_scope", "scopes": scopes}));
                }
            }
            ContextPackPart::EnclosingNode => {
                if let Some(ref tn) = target_node {
                    let sm = crate::context::node_at_range_helpers::make_summary(
                        tn, &source, true, 8000,
                    );
                    summaries.push(json!({"part": "enclosing_node", "node": sm}));
                }
            }
            ContextPackPart::TopLevelOutline => {
                let opts = OutlineOptions {
                    max_depth: 3,
                    include_ranges: true,
                    include_imports: false,
                    include_exports: false,
                };
                let outline = extractors::outline::file_outline(&tree, &source, &opts, lang, fp);
                summaries.push(json!({"part": "top_level_outline", "outline": outline}));
            }
            _ => {} // NearbyFunctions/NearbyClasses deferred to later
        }
    }

    json!({
        "filePath": fp,
        "language": lang.as_str(),
        "blocks": blocks,
        "summaries": summaries,
        "truncated": budget.exceeded,
    })
}

fn zero_range() -> Range {
    Range { start: Position { line: 0, character: 0 }, end: Position { line: 0, character: 0 } }
}

fn extension_to_language(path: &str) -> Option<LanguageId> {
    let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str())?;
    let dotted = format!(".{}", ext);
    parser::registry::for_extension(&dotted).map(|d| d.language)
}
