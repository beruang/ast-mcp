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
    QueryInvalid(String, Option<serde_json::Value>),
    #[error("query execution failed: {0}")]
    QueryExecutionFailed(String, Option<serde_json::Value>),
    #[error("result limit exceeded")]
    ResultLimitExceeded,
    #[error("range out of bounds")]
    RangeOutOfBounds,
    #[error("node not found")]
    NodeNotFound,
    #[error("scope not found")]
    ScopeNotFound,
    #[error("feature unsupported for language: {0}")]
    AstFeatureUnsupportedForLanguage(String),
    #[error("workspace query limit exceeded")]
    QueryWorkspaceLimitExceeded,
    #[error("workspace query timeout")]
    WorkspaceQueryTimeout,
    #[error("invalid glob pattern: {0}")]
    InvalidGlob(String),
    #[error("position encoding error: {0}")]
    PositionEncodingError(String),
    #[error("context budget exceeded")]
    ContextBudgetExceeded,
    #[error("internal error: {0}")]
    InternalError(String),

    // ── V4 rewrite errors (spec section 30) ──
    #[error("invalid range: {0}")]
    RewriteInvalidRange(String),
    #[error("node kind mismatch: {0}")]
    RewriteNodeKindMismatch(String),
    #[error("range not node-aligned: {0}")]
    RewriteRangeNotNodeAligned(String),
    #[error("unsupported operation: {0}")]
    RewriteUnsupportedOperation(String),
    #[error("unsupported language: {0}")]
    RewriteUnsupportedLanguage(String),
    #[error("too many files: {0}")]
    RewriteTooManyFiles(String),
    #[error("too many edits: {0}")]
    RewriteTooManyEdits(String),
    #[error("overlapping edits: {0}")]
    RewriteOverlappingEdits(String),
    #[error("new text too large: {0}")]
    RewriteNewTextTooLarge(String),
    #[error("diff too large: {0}")]
    RewriteDiffTooLarge(String),
    #[error("syntax error after rewrite: {0}")]
    RewriteSyntaxErrorAfter(String),
    #[error("ambiguous rewrite target: {0}")]
    RewriteAmbiguousTarget(String),
    #[error("import conflict: {0}")]
    RewriteImportConflict(String),
    #[error("parameter not found: {0}")]
    RewriteParameterNotFound(String),
    #[error("duplicate parameter: {0}")]
    RewriteDuplicateParameter(String),
    #[error("scope unavailable: {0}")]
    RewriteScopeUnavailable(String),
    #[error("identifier not found: {0}")]
    RewriteIdentifierNotFound(String),
    #[error("unsafe local rename: {0}")]
    RewriteUnsafeLocalRename(String),
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
            AstToolError::QueryInvalid(..) => "query_invalid",
            AstToolError::QueryExecutionFailed(..) => "query_execution_failed",
            AstToolError::ResultLimitExceeded => "result_limit_exceeded",
            AstToolError::RangeOutOfBounds => "range_out_of_bounds",
            AstToolError::NodeNotFound => "node_not_found",
            AstToolError::ScopeNotFound => "scope_not_found",
            AstToolError::AstFeatureUnsupportedForLanguage(_) => {
                "ast_feature_unsupported_for_language"
            }
            AstToolError::QueryWorkspaceLimitExceeded => "query_workspace_limit_exceeded",
            AstToolError::WorkspaceQueryTimeout => "workspace_query_timeout",
            AstToolError::InvalidGlob(_) => "invalid_glob",
            AstToolError::PositionEncodingError(_) => "position_encoding_error",
            AstToolError::ContextBudgetExceeded => "context_budget_exceeded",
            AstToolError::InternalError(_) => "internal_error",
            // V4 rewrite error codes
            AstToolError::RewriteInvalidRange(_) => "rewrite_invalid_range",
            AstToolError::RewriteNodeKindMismatch(_) => "rewrite_node_kind_mismatch",
            AstToolError::RewriteRangeNotNodeAligned(_) => "rewrite_range_not_node_aligned",
            AstToolError::RewriteUnsupportedOperation(_) => "rewrite_unsupported_operation",
            AstToolError::RewriteUnsupportedLanguage(_) => "rewrite_unsupported_language",
            AstToolError::RewriteTooManyFiles(_) => "rewrite_too_many_files",
            AstToolError::RewriteTooManyEdits(_) => "rewrite_too_many_edits",
            AstToolError::RewriteOverlappingEdits(_) => "rewrite_overlapping_edits",
            AstToolError::RewriteNewTextTooLarge(_) => "rewrite_new_text_too_large",
            AstToolError::RewriteDiffTooLarge(_) => "rewrite_diff_too_large",
            AstToolError::RewriteSyntaxErrorAfter(_) => "rewrite_syntax_error_after",
            AstToolError::RewriteAmbiguousTarget(_) => "rewrite_ambiguous_target",
            AstToolError::RewriteImportConflict(_) => "rewrite_import_conflict",
            AstToolError::RewriteParameterNotFound(_) => "rewrite_parameter_not_found",
            AstToolError::RewriteDuplicateParameter(_) => "rewrite_duplicate_parameter",
            AstToolError::RewriteScopeUnavailable(_) => "rewrite_scope_unavailable",
            AstToolError::RewriteIdentifierNotFound(_) => "rewrite_identifier_not_found",
            AstToolError::RewriteUnsafeLocalRename(_) => "rewrite_unsafe_local_rename",
        }
    }

    pub fn payload(&self) -> serde_json::Value {
        let mut err = json!({
            "code": self.code(),
            "message": self.to_string(),
        });
        // Attach details for errors that carry structured data
        match self {
            AstToolError::FileTooLarge(size, limit) => {
                err["details"] = json!({ "size": size, "limit": limit });
            }
            AstToolError::QueryInvalid(_, details)
            | AstToolError::QueryExecutionFailed(_, details) => {
                if let Some(d) = details {
                    err["details"] = d.clone();
                }
            }
            _ => {}
        }
        json!({ "error": err })
    }
}
