use tree_sitter::Query;

use crate::parser::registry;
use crate::shared::errors::AstToolError;
use crate::shared::language::LanguageId;

pub fn compile_query(language: LanguageId, source: &str) -> Result<Query, AstToolError> {
    let def = registry::for_language(language)
        .ok_or_else(|| AstToolError::ParserUnavailable(language.as_str().into()))?;
    let lang = (def.tree_sitter_language)();
    Query::new(lang, source).map_err(|e| {
        let details = serde_json::json!({
            "language": language.as_str(),
            "row": e.row,
            "column": e.column,
            "offset": e.offset,
            "kind": format!("{:?}", e.kind),
        });
        AstToolError::QueryInvalid(
            format!("row {} column {}: {}", e.row, e.column, e.message),
            Some(details),
        )
    })
}
