use crate::shared::errors::AstToolError;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageId {
    TypeScript,
    TypeScriptReact,
    JavaScript,
    JavaScriptReact,
    Python,
    Go,
    Rust,
}

impl LanguageId {
    pub fn as_str(&self) -> &'static str {
        match self {
            LanguageId::TypeScript => "typescript",
            LanguageId::TypeScriptReact => "tsx",
            LanguageId::JavaScript => "javascript",
            LanguageId::JavaScriptReact => "jsx",
            LanguageId::Python => "python",
            LanguageId::Go => "go",
            LanguageId::Rust => "rust",
        }
    }
}

impl FromStr for LanguageId {
    type Err = AstToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "typescript" => Ok(LanguageId::TypeScript),
            "tsx" => Ok(LanguageId::TypeScriptReact),
            "javascript" => Ok(LanguageId::JavaScript),
            "jsx" => Ok(LanguageId::JavaScriptReact),
            "python" => Ok(LanguageId::Python),
            "go" => Ok(LanguageId::Go),
            "rust" => Ok(LanguageId::Rust),
            _ => Err(AstToolError::UnsupportedLanguage(s.to_string())),
        }
    }
}
