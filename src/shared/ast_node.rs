use serde::Serialize;

use crate::shared::position::Range;

#[derive(Debug, Clone, Serialize)]
pub struct AstNodeSummary {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<AstNodeSummary>>,
}
