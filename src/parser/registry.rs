use tree_sitter::Language;

use crate::languages::{javascript, python, typescript};
use crate::shared::language::LanguageId;

pub struct ParserDefinition {
    pub language: LanguageId,
    pub extensions: &'static [&'static str],
    pub tree_sitter_language: fn() -> Language,
}

pub fn registry() -> &'static [ParserDefinition] {
    &[
        ParserDefinition {
            language: LanguageId::TypeScript,
            extensions: &[".ts"],
            tree_sitter_language: typescript::language,
        },
        ParserDefinition {
            language: LanguageId::TypeScriptReact,
            extensions: &[".tsx"],
            tree_sitter_language: typescript::language_tsx,
        },
        ParserDefinition {
            language: LanguageId::JavaScript,
            extensions: &[".js", ".mjs", ".cjs"],
            tree_sitter_language: javascript::language,
        },
        ParserDefinition {
            language: LanguageId::JavaScriptReact,
            extensions: &[".jsx"],
            tree_sitter_language: javascript::language,
        },
        ParserDefinition {
            language: LanguageId::Python,
            extensions: &[".py"],
            tree_sitter_language: python::language,
        },
    ]
}

pub fn for_extension(ext: &str) -> Option<&'static ParserDefinition> {
    registry().iter().find(|d| d.extensions.contains(&ext))
}

pub fn for_language(lang: LanguageId) -> Option<&'static ParserDefinition> {
    registry().iter().find(|d| d.language == lang)
}
