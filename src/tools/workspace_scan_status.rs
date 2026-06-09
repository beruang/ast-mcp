use crate::scan::ScanRegistry;
use crate::shared::types_v5::{WorkspaceScanStatusInput, WorkspaceScanStatusResult};
use serde_json::Value;

pub fn handle(registry: &ScanRegistry, arguments: Value) -> Value {
    let input: WorkspaceScanStatusInput =
        serde_json::from_value(arguments).unwrap_or(WorkspaceScanStatusInput { scan_id: None });

    let scans = if let Some(ref id) = input.scan_id {
        registry.get(id).map(|h| vec![h.to_info()]).unwrap_or_default()
    } else {
        registry.list_all()
    };

    serde_json::to_value(WorkspaceScanStatusResult { scans }).unwrap_or_else(
        |e| serde_json::json!({ "error": { "code": "internal_error", "message": e.to_string() } }),
    )
}
