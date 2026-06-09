use std::sync::RwLock;

use crate::shared::types_v5::{
    PartialRuntimeCaches, PartialRuntimeDebug, PartialRuntimeLimits, PartialRuntimeScans,
    PartialRuntimeTimeouts, RejectedConfigUpdate, RuntimeCaches, RuntimeConfig, RuntimeDebug,
    RuntimeLimits, RuntimeScans, RuntimeTimeouts,
};

// ── Defaults ──

#[allow(clippy::derivable_impls)]
impl Default for RuntimeConfig {
    fn default() -> Self {
        RuntimeConfig {
            workspace_path: String::new(),
            limits: RuntimeLimits::default(),
            timeouts_ms: RuntimeTimeouts::default(),
            caches: RuntimeCaches::default(),
            scans: RuntimeScans::default(),
            debug: RuntimeDebug::default(),
        }
    }
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        RuntimeLimits {
            max_file_bytes: 1_048_576,
            max_parse_tree_nodes: 200_000,
            max_query_results: 1_000,
            max_workspace_files: 500,
            max_workspace_results: 5_000,
            max_context_characters: 20_000,
            max_chunk_lines: 160,
            max_changed_files: 100,
            max_edits: 1_000,
            max_duplicate_candidates: 200,
        }
    }
}

impl Default for RuntimeTimeouts {
    fn default() -> Self {
        RuntimeTimeouts {
            parse_file: 5_000,
            query_file: 5_000,
            query_workspace: 30_000,
            chunk_file: 10_000,
            framework_extraction: 30_000,
            rewrite_preview: 10_000,
            complexity_summary: 30_000,
            duplicate_detection: 60_000,
        }
    }
}

impl Default for RuntimeCaches {
    fn default() -> Self {
        RuntimeCaches {
            parse_tree_ttl_ms: 300_000,
            query_result_ttl_ms: 120_000,
            framework_result_ttl_ms: 120_000,
            request_log_max_entries: 500,
            max_cached_files: 1_000,
        }
    }
}

impl Default for RuntimeScans {
    fn default() -> Self {
        RuntimeScans {
            max_parallelism: 8,
            respect_gitignore: true,
            include_hidden: false,
            default_exclude_globs: vec![
                "**/node_modules/**".into(),
                "**/.git/**".into(),
                "**/dist/**".into(),
                "**/build/**".into(),
                "**/target/**".into(),
                "**/.venv/**".into(),
                "**/__pycache__/**".into(),
            ],
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for RuntimeDebug {
    fn default() -> Self {
        RuntimeDebug {
            verbose_logging: false,
            include_node_text_in_logs: false,
            include_raw_tree_debug: false,
        }
    }
}

// ── Hard safety ceilings ──

pub const HARD_MAX_FILE_BYTES: usize = 50_000_000;
pub const HARD_MAX_PARALLELISM: usize = 64;
pub const HARD_MAX_REQUEST_LOG_ENTRIES: usize = 10_000;

// ── RuntimeConfigStore ──

pub struct RuntimeConfigStore {
    config: RwLock<RuntimeConfig>,
    defaults: RuntimeConfig,
    env_overrides: RuntimeConfig,
}

impl RuntimeConfigStore {
    pub fn new(workspace_path: String) -> Self {
        let defaults = RuntimeConfig::default();
        let mut env_overrides = defaults.clone();
        env_overrides.workspace_path = workspace_path.clone();
        super::env_config::apply_env_overrides(&mut env_overrides);

        let mut config = env_overrides.clone();
        config.workspace_path = workspace_path;

        RuntimeConfigStore {
            config: RwLock::new(config),
            defaults: defaults.clone(),
            env_overrides,
        }
    }

    pub fn current(&self) -> RuntimeConfig {
        self.config.read().unwrap().clone()
    }

    pub fn workspace_path(&self) -> String {
        self.config.read().unwrap().workspace_path.clone()
    }

    pub fn update(
        &self,
        limits: Option<PartialRuntimeLimits>,
        timeouts_ms: Option<PartialRuntimeTimeouts>,
        caches: Option<PartialRuntimeCaches>,
        scans: Option<PartialRuntimeScans>,
        debug: Option<PartialRuntimeDebug>,
        rejected: &mut Vec<RejectedConfigUpdate>,
    ) -> bool {
        let mut config = self.config.write().unwrap();
        let rejected_raw = super::env_config::apply_partial(
            &mut config,
            limits,
            timeouts_ms,
            caches,
            scans,
            debug,
        );
        rejected.extend(
            rejected_raw
                .into_iter()
                .map(|r| RejectedConfigUpdate { path: r.path, reason: r.reason }),
        );
        true
    }

    pub fn defaults(&self) -> RuntimeConfig {
        self.defaults.clone()
    }

    pub fn env_overrides(&self) -> RuntimeConfig {
        self.env_overrides.clone()
    }

    pub fn runtime_overrides() -> RuntimeConfig {
        RuntimeConfig::default()
    }
}
