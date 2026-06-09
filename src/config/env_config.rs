use crate::shared::types_v5::{
    PartialRuntimeCaches, PartialRuntimeDebug, PartialRuntimeLimits, PartialRuntimeScans,
    PartialRuntimeTimeouts, RuntimeConfig,
};

/// Apply environment variable overrides to a RuntimeConfig.
/// Reads variables with the `AST_` prefix (spec section 10).
pub fn apply_env_overrides(config: &mut RuntimeConfig) {
    if let Some(v) = parse_usize("AST_MAX_FILE_BYTES") {
        config.limits.max_file_bytes = v;
    }
    if let Some(v) = parse_usize("AST_MAX_WORKSPACE_FILES") {
        config.limits.max_workspace_files = v;
    }
    if let Some(v) = parse_usize("AST_MAX_WORKSPACE_RESULTS") {
        config.limits.max_workspace_results = v;
    }
    if let Some(v) = parse_usize("AST_MAX_CONTEXT_CHARACTERS") {
        config.limits.max_context_characters = v;
    }
    if let Some(v) = parse_usize("AST_MAX_PARALLELISM") {
        config.scans.max_parallelism = v;
    }
    if let Some(v) = parse_bool("AST_RESPECT_GITIGNORE") {
        config.scans.respect_gitignore = v;
    }
    if let Some(v) = parse_bool("AST_INCLUDE_HIDDEN") {
        config.scans.include_hidden = v;
    }
    if let Some(v) = parse_u64("AST_PARSE_TREE_TTL_MS") {
        config.caches.parse_tree_ttl_ms = v;
    }
    if let Some(v) = parse_usize("AST_REQUEST_LOG_MAX_ENTRIES") {
        config.caches.request_log_max_entries = v;
    }
    if let Some(v) = parse_usize("AST_MAX_CACHED_FILES") {
        config.caches.max_cached_files = v;
    }
    if let Some(v) = parse_bool("AST_VERBOSE_LOGGING") {
        config.debug.verbose_logging = v;
    }
    if let Some(v) = parse_bool("AST_INCLUDE_NODE_TEXT_IN_LOGS") {
        config.debug.include_node_text_in_logs = v;
    }
}

/// Build a RuntimeConfig from defaults + environment overrides.
pub fn from_env(workspace_path: String) -> RuntimeConfig {
    let mut config = RuntimeConfig { workspace_path, ..Default::default() };
    apply_env_overrides(&mut config);
    config
}

fn parse_usize(key: &str) -> Option<usize> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}

fn parse_u64(key: &str) -> Option<u64> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}

fn parse_bool(key: &str) -> Option<bool> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}

/// Apply partial runtime overrides to a config. Returns list of rejected updates.
pub fn apply_partial(
    config: &mut RuntimeConfig,
    limits: Option<PartialRuntimeLimits>,
    timeouts_ms: Option<PartialRuntimeTimeouts>,
    caches: Option<PartialRuntimeCaches>,
    scans: Option<PartialRuntimeScans>,
    debug: Option<PartialRuntimeDebug>,
) -> Vec<super::validate_config::RejectedUpdate> {
    let mut rejected = Vec::new();

    if let Some(l) = limits {
        if let Some(v) = l.max_file_bytes {
            super::validate_config::validate_limit_usize(
                &mut config.limits.max_file_bytes,
                v,
                "limits.max_file_bytes",
                &mut rejected,
            );
        }
        if let Some(v) = l.max_parse_tree_nodes {
            super::validate_config::validate_limit_usize(
                &mut config.limits.max_parse_tree_nodes,
                v,
                "limits.max_parse_tree_nodes",
                &mut rejected,
            );
        }
        if let Some(v) = l.max_query_results {
            super::validate_config::validate_limit_usize(
                &mut config.limits.max_query_results,
                v,
                "limits.max_query_results",
                &mut rejected,
            );
        }
        if let Some(v) = l.max_workspace_files {
            super::validate_config::validate_limit_usize(
                &mut config.limits.max_workspace_files,
                v,
                "limits.max_workspace_files",
                &mut rejected,
            );
        }
        if let Some(v) = l.max_workspace_results {
            super::validate_config::validate_limit_usize(
                &mut config.limits.max_workspace_results,
                v,
                "limits.max_workspace_results",
                &mut rejected,
            );
        }
        if let Some(v) = l.max_context_characters {
            super::validate_config::validate_limit_usize(
                &mut config.limits.max_context_characters,
                v,
                "limits.max_context_characters",
                &mut rejected,
            );
        }
        if let Some(v) = l.max_chunk_lines {
            super::validate_config::validate_limit_usize(
                &mut config.limits.max_chunk_lines,
                v,
                "limits.max_chunk_lines",
                &mut rejected,
            );
        }
        if let Some(v) = l.max_changed_files {
            super::validate_config::validate_limit_usize(
                &mut config.limits.max_changed_files,
                v,
                "limits.max_changed_files",
                &mut rejected,
            );
        }
        if let Some(v) = l.max_edits {
            super::validate_config::validate_limit_usize(
                &mut config.limits.max_edits,
                v,
                "limits.max_edits",
                &mut rejected,
            );
        }
        if let Some(v) = l.max_duplicate_candidates {
            super::validate_config::validate_limit_usize(
                &mut config.limits.max_duplicate_candidates,
                v,
                "limits.max_duplicate_candidates",
                &mut rejected,
            );
        }
    }

    if let Some(t) = timeouts_ms {
        if let Some(v) = t.parse_file {
            super::validate_config::validate_timeout(
                &mut config.timeouts_ms.parse_file,
                v,
                "timeouts_ms.parse_file",
                &mut rejected,
            );
        }
        if let Some(v) = t.query_file {
            super::validate_config::validate_timeout(
                &mut config.timeouts_ms.query_file,
                v,
                "timeouts_ms.query_file",
                &mut rejected,
            );
        }
        if let Some(v) = t.query_workspace {
            super::validate_config::validate_timeout(
                &mut config.timeouts_ms.query_workspace,
                v,
                "timeouts_ms.query_workspace",
                &mut rejected,
            );
        }
        if let Some(v) = t.chunk_file {
            super::validate_config::validate_timeout(
                &mut config.timeouts_ms.chunk_file,
                v,
                "timeouts_ms.chunk_file",
                &mut rejected,
            );
        }
        if let Some(v) = t.framework_extraction {
            super::validate_config::validate_timeout(
                &mut config.timeouts_ms.framework_extraction,
                v,
                "timeouts_ms.framework_extraction",
                &mut rejected,
            );
        }
        if let Some(v) = t.rewrite_preview {
            super::validate_config::validate_timeout(
                &mut config.timeouts_ms.rewrite_preview,
                v,
                "timeouts_ms.rewrite_preview",
                &mut rejected,
            );
        }
        if let Some(v) = t.complexity_summary {
            super::validate_config::validate_timeout(
                &mut config.timeouts_ms.complexity_summary,
                v,
                "timeouts_ms.complexity_summary",
                &mut rejected,
            );
        }
        if let Some(v) = t.duplicate_detection {
            super::validate_config::validate_timeout(
                &mut config.timeouts_ms.duplicate_detection,
                v,
                "timeouts_ms.duplicate_detection",
                &mut rejected,
            );
        }
    }

    if let Some(c) = caches {
        if let Some(v) = c.parse_tree_ttl_ms {
            super::validate_config::validate_timeout(
                &mut config.caches.parse_tree_ttl_ms,
                v,
                "caches.parse_tree_ttl_ms",
                &mut rejected,
            );
        }
        if let Some(v) = c.query_result_ttl_ms {
            super::validate_config::validate_timeout(
                &mut config.caches.query_result_ttl_ms,
                v,
                "caches.query_result_ttl_ms",
                &mut rejected,
            );
        }
        if let Some(v) = c.framework_result_ttl_ms {
            super::validate_config::validate_timeout(
                &mut config.caches.framework_result_ttl_ms,
                v,
                "caches.framework_result_ttl_ms",
                &mut rejected,
            );
        }
        if let Some(v) = c.request_log_max_entries {
            super::validate_config::validate_request_log_entries(
                &mut config.caches.request_log_max_entries,
                v,
                "caches.request_log_max_entries",
                &mut rejected,
            );
        }
        if let Some(v) = c.max_cached_files {
            super::validate_config::validate_limit_usize(
                &mut config.caches.max_cached_files,
                v,
                "caches.max_cached_files",
                &mut rejected,
            );
        }
    }

    if let Some(s) = scans {
        if let Some(v) = s.max_parallelism {
            super::validate_config::validate_parallelism(
                &mut config.scans.max_parallelism,
                v,
                "scans.max_parallelism",
                &mut rejected,
            );
        }
        if let Some(v) = s.respect_gitignore {
            config.scans.respect_gitignore = v;
        }
        if let Some(v) = s.include_hidden {
            config.scans.include_hidden = v;
        }
        if let Some(v) = s.default_exclude_globs {
            config.scans.default_exclude_globs = v;
        }
    }

    if let Some(d) = debug {
        if let Some(v) = d.verbose_logging {
            config.debug.verbose_logging = v;
        }
        if let Some(v) = d.include_node_text_in_logs {
            config.debug.include_node_text_in_logs = v;
        }
        if let Some(v) = d.include_raw_tree_debug {
            config.debug.include_raw_tree_debug = v;
        }
    }

    rejected
}
