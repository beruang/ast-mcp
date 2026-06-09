use serde::{Deserialize, Serialize};

use crate::shared::position::Range;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNodeSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_range: Option<(usize, usize)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<AstNodeSummary>>,
}

/// Mode for ast_node_at_range.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeAtRangeMode {
    #[default]
    SmallestContaining,
    Exact,
    LargestContained,
}
