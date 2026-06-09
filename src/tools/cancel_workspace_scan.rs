use crate::scan::ScanRegistry;
use crate::shared::types_v5::{CancelWorkspaceScanInput, CancelWorkspaceScanResult};
use serde_json::Value;

pub fn handle(registry: &ScanRegistry, arguments: Value) -> Value {
    let input: CancelWorkspaceScanInput = serde_json::from_value(arguments)
        .unwrap_or(CancelWorkspaceScanInput { scan_id: String::new() });

    if let Some(handle) = registry.get(&input.scan_id) {
        let previous = match handle.status.lock() {
            Ok(g) => g.clone(),
            Err(_) => {
                return serde_json::json!({
                    "error": { "code": "internal_error", "message": "scan status lock poisoned" }
                })
            }
        };

        if previous == "completed" || previous == "failed" {
            return serde_json::json!({
                "error": {
                    "code": "workspace_scan_already_completed",
                    "message": format!("scan {} already {}", input.scan_id, previous)
                }
            });
        }

        handle.cancellable.cancel();
        match handle.status.lock() {
            Ok(mut g) => *g = "cancelled".to_string(),
            Err(_) => {
                return serde_json::json!({
                    "error": { "code": "internal_error", "message": "scan status lock poisoned" }
                })
            }
        }

        serde_json::to_value(CancelWorkspaceScanResult {
            scan_id: input.scan_id.clone(),
            cancelled: true,
            previous_status: previous,
            new_status: "cancelled".to_string(),
        })
        .unwrap_or_default()
    } else {
        serde_json::json!({
            "error": {
                "code": "workspace_scan_not_found",
                "message": format!("scan {} not found", input.scan_id)
            }
        })
    }
}
