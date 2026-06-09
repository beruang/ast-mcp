use crate::parser::registry;
use crate::shared::types_v5::{ParserRebuildFailure, RebuildParserCacheResult};

pub fn rebuild(languages: Option<&[String]>) -> RebuildParserCacheResult {
    let all_languages = registry::list_languages();
    let target_langs: Vec<&str> = if let Some(langs) = languages {
        langs.iter().map(|s| s.as_str()).collect()
    } else {
        all_languages.iter().map(|info| info.language()).collect()
    };

    let mut rebuilt = Vec::new();
    let mut failed = Vec::new();

    for lang in &target_langs {
        match registry::rebuild_for_language(lang) {
            Ok(()) => rebuilt.push(lang.to_string()),
            Err(e) => failed
                .push(ParserRebuildFailure { language: lang.to_string(), error: e.to_string() }),
        }
    }

    RebuildParserCacheResult { rebuilt, failed }
}
