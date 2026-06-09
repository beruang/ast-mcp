//! SafetyViolation factory functions for V4 rewrite tools.

use crate::shared::types_v4::SafetyViolation;

pub fn outside_workspace(path: &str) -> SafetyViolation {
    SafetyViolation {
        violation_type: "outside_workspace".into(),
        message: format!("path is outside workspace: {}", path),
        file_path: Some(path.to_string()),
        details: None,
    }
}

pub fn file_not_found(path: &str) -> SafetyViolation {
    SafetyViolation {
        violation_type: "file_not_found".into(),
        message: format!("file not found: {}", path),
        file_path: Some(path.to_string()),
        details: None,
    }
}

pub fn unsupported_language(path: &str, language: &str) -> SafetyViolation {
    SafetyViolation {
        violation_type: "unsupported_language".into(),
        message: format!("unsupported language '{}' for file: {}", language, path),
        file_path: Some(path.to_string()),
        details: Some(serde_json::json!({ "language": language })),
    }
}

pub fn unsupported_operation(operation: &str, reason: &str) -> SafetyViolation {
    SafetyViolation {
        violation_type: "unsupported_operation".into(),
        message: format!("unsupported operation '{}': {}", operation, reason),
        file_path: None,
        details: Some(serde_json::json!({ "operation": operation, "reason": reason })),
    }
}

pub fn invalid_range(path: &str, message: &str) -> SafetyViolation {
    SafetyViolation {
        violation_type: "invalid_range".into(),
        message: format!("invalid range in {}: {}", path, message),
        file_path: Some(path.to_string()),
        details: None,
    }
}

pub fn range_not_node_aligned(path: &str, message: &str) -> SafetyViolation {
    SafetyViolation {
        violation_type: "range_not_node_aligned".into(),
        message: format!("range not aligned to node in {}: {}", path, message),
        file_path: Some(path.to_string()),
        details: None,
    }
}

pub fn node_kind_mismatch(path: &str, expected: &str, actual: &str) -> SafetyViolation {
    SafetyViolation {
        violation_type: "node_kind_mismatch".into(),
        message: format!(
            "node kind mismatch in {}: expected '{}', got '{}'",
            path, expected, actual
        ),
        file_path: Some(path.to_string()),
        details: Some(serde_json::json!({ "expected": expected, "actual": actual })),
    }
}

pub fn too_many_files(count: u32, limit: u32) -> SafetyViolation {
    SafetyViolation {
        violation_type: "too_many_files".into(),
        message: format!("{} files would be changed, limit is {}", count, limit),
        file_path: None,
        details: Some(serde_json::json!({ "count": count, "limit": limit })),
    }
}

pub fn too_many_edits(count: u32, limit: u32) -> SafetyViolation {
    SafetyViolation {
        violation_type: "too_many_edits".into(),
        message: format!("{} edits requested, limit is {}", count, limit),
        file_path: None,
        details: Some(serde_json::json!({ "count": count, "limit": limit })),
    }
}

pub fn new_text_too_large(path: &str, size: u64, limit: u64) -> SafetyViolation {
    SafetyViolation {
        violation_type: "new_text_too_large".into(),
        message: format!("new text for {} is {} bytes, limit is {}", path, size, limit),
        file_path: Some(path.to_string()),
        details: Some(serde_json::json!({ "size": size, "limit": limit })),
    }
}

pub fn diff_too_large(size: u64, limit: u64) -> SafetyViolation {
    SafetyViolation {
        violation_type: "diff_too_large".into(),
        message: format!("generated diff is {} bytes, limit is {}", size, limit),
        file_path: None,
        details: Some(serde_json::json!({ "size": size, "limit": limit })),
    }
}

pub fn overlapping_edits(path: &str) -> SafetyViolation {
    SafetyViolation {
        violation_type: "overlapping_edits".into(),
        message: format!("overlapping edits detected in {}", path),
        file_path: Some(path.to_string()),
        details: None,
    }
}

pub fn syntax_error_after_rewrite(path: &str, message: &str) -> SafetyViolation {
    SafetyViolation {
        violation_type: "syntax_error_after_rewrite".into(),
        message: format!("syntax error after rewrite in {}: {}", path, message),
        file_path: Some(path.to_string()),
        details: None,
    }
}

pub fn ambiguous_target(message: &str) -> SafetyViolation {
    SafetyViolation {
        violation_type: "ambiguous_rewrite_target".into(),
        message: format!("ambiguous rewrite target: {}", message),
        file_path: None,
        details: None,
    }
}

pub fn internal_error(message: &str) -> SafetyViolation {
    SafetyViolation {
        violation_type: "internal_error".into(),
        message: format!("internal error: {}", message),
        file_path: None,
        details: None,
    }
}
