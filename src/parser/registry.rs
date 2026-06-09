use tree_sitter::Language;

use crate::languages::{go, javascript, python, rust, typescript};
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
        ParserDefinition {
            language: LanguageId::Go,
            extensions: &[".go"],
            tree_sitter_language: go::language,
        },
        ParserDefinition {
            language: LanguageId::Rust,
            extensions: &[".rs"],
            tree_sitter_language: rust::language,
        },
    ]
}

pub fn for_extension(ext: &str) -> Option<&'static ParserDefinition> {
    registry().iter().find(|d| d.extensions.contains(&ext))
}

pub fn for_language(lang: LanguageId) -> Option<&'static ParserDefinition> {
    registry().iter().find(|d| d.language == lang)
}

/// Lightweight info for parser status reporting.
pub struct ParserInfo {
    pub language: String,
    pub extensions: Vec<String>,
    pub available: bool,
    pub parser_name: String,
    pub version: Option<String>,
    pub query_count: usize,
    pub last_error: Option<String>,
}

impl ParserInfo {
    pub fn language(&self) -> &str {
        &self.language
    }
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
    pub fn available(&self) -> bool {
        self.available
    }
    pub fn parser_name(&self) -> &str {
        &self.parser_name
    }
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    pub fn query_count(&self) -> usize {
        self.query_count
    }
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
}

pub fn list_languages() -> Vec<ParserInfo> {
    registry()
        .iter()
        .map(|d| ParserInfo {
            language: format!("{:?}", d.language).to_lowercase(),
            extensions: d.extensions.iter().map(|s| s.to_string()).collect(),
            available: true,
            parser_name: format!("tree-sitter-{}", format!("{:?}", d.language).to_lowercase()),
            version: Some("0.21".into()),
            query_count: 0,
            last_error: None,
        })
        .collect()
}

pub fn rebuild_for_language(lang: &str) -> Result<(), String> {
    let _ = lang;
    // Rebuild is a no-op for now — parsers are statically compiled.
    // In a dynamic-loading setup, this would reload the parser library.
    Ok(())
}
