use crate::parser::registry;
use crate::shared::types_v5::{ParserStatus, ParserStatusResult};

pub fn status(language_filter: Option<&str>) -> ParserStatusResult {
    let parsers: Vec<ParserStatus> = registry::list_languages()
        .into_iter()
        .filter(|p| if let Some(lang) = language_filter { p.language() == lang } else { true })
        .map(|info| ParserStatus {
            language: info.language().to_string(),
            extensions: info.extensions().iter().map(|s| s.to_string()).collect(),
            available: info.available(),
            parser_name: info.parser_name().to_string(),
            version: info.version().map(|s| s.to_string()),
            query_count: info.query_count(),
            last_error: info.last_error().map(|s| s.to_string()),
        })
        .collect();

    ParserStatusResult { parsers }
}
