use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AstToolError {
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(String),
    #[error("path outside workspace: {0}")]
    PathOutsideWorkspace(String),
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("file too large: {0} bytes exceeds {1}")]
    FileTooLarge(u64, u64),
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("parser unavailable: {0}")]
    ParserUnavailable(String),
    #[error("parse failed: {0}")]
    ParseFailed(String),
    #[error("syntax error")]
    SyntaxError,
    #[error("invalid position: {0}")]
    InvalidPosition(String),
    #[error("invalid range")]
    InvalidRange,
    #[error("query invalid: {0}")]
    QueryInvalid(String),
    #[error("query execution failed: {0}")]
    QueryExecutionFailed(String),
    #[error("result limit exceeded")]
    ResultLimitExceeded,
    #[error("internal error: {0}")]
    InternalError(String),
}

impl AstToolError {
    pub fn code(&self) -> &'static str {
        match self {
            AstToolError::WorkspaceNotFound(_) => "workspace_not_found",
            AstToolError::PathOutsideWorkspace(_) => "path_outside_workspace",
            AstToolError::FileNotFound(_) => "file_not_found",
            AstToolError::FileTooLarge(_, _) => "file_too_large",
            AstToolError::UnsupportedLanguage(_) => "unsupported_language",
            AstToolError::ParserUnavailable(_) => "parser_unavailable",
            AstToolError::ParseFailed(_) => "parse_failed",
            AstToolError::SyntaxError => "syntax_error",
            AstToolError::InvalidPosition(_) => "invalid_position",
            AstToolError::InvalidRange => "invalid_range",
            AstToolError::QueryInvalid(_) => "query_invalid",
            AstToolError::QueryExecutionFailed(_) => "query_execution_failed",
            AstToolError::ResultLimitExceeded => "result_limit_exceeded",
            AstToolError::InternalError(_) => "internal_error",
        }
    }

    pub fn payload(&self) -> serde_json::Value {
        let mut err = json!({
            "code": self.code(),
            "message": self.to_string(),
        });
        // Attach details for errors that carry structured data
        if let AstToolError::FileTooLarge(size, limit) = self {
            err["details"] = json!({ "size": size, "limit": limit });
        }
        json!({ "error": err })
    }
}
