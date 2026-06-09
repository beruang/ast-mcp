use crate::config::runtime_config::RuntimeConfigStore;
use crate::shared::types_v5::{UpdateRuntimeConfigInput, UpdateRuntimeConfigResult};
use serde_json::Value;

pub fn handle(store: &RuntimeConfigStore, arguments: Value) -> Value {
    let input: UpdateRuntimeConfigInput = match serde_json::from_value(arguments) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({
                "error": {
                    "code": "invalid_runtime_config",
                    "message": format!("invalid config update: {}", e)
                }
            })
        }
    };

    // Reject attempts to change immutable fields
    let mut rejected = Vec::new();

    // workspace_path, parser registry, language grammar paths are immutable.
    // These are not present in UpdateRuntimeConfigInput, so they can't be changed
    // through this API. If they were somehow included, they'd be ignored.

    let updated = store.update(
        input.limits,
        input.timeouts_ms,
        input.caches,
        input.scans,
        input.debug,
        &mut rejected,
    );

    let config = store.current();

    serde_json::to_value(UpdateRuntimeConfigResult {
        updated: updated || rejected.is_empty(),
        config,
        rejected,
    })
    .unwrap_or_else(
        |e| serde_json::json!({ "error": { "code": "internal_error", "message": e.to_string() } }),
    )
}
