pub mod chunks;
pub mod classes;
pub mod enclosing_node;
pub mod exports;
pub mod functions;
pub mod imports;
pub mod outline;
pub mod queries;
pub mod top_level;

use serde::{Deserialize, Serialize};

use crate::shared::position::Range;

/// A candidate node for outline extraction, produced by language-specific
/// functions that walk the root's named children looking for
/// structurally-significant nodes (functions, classes, imports, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineCandidate {
    pub kind: String,
    pub name: Option<String>,
    pub range: Range,
    pub children: Vec<OutlineCandidate>,
}
