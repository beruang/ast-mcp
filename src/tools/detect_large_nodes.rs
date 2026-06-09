use crate::analysis::large_nodes;
use crate::config::workspace::Workspace;
use serde_json::Value;

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input = large_nodes::handle_arguments(arguments);

    if let Some(ref file_path) = input.file_path {
        let full = workspace.root().join(file_path);
        if !full.starts_with(workspace.root()) {
            return serde_json::json!({
                "error": { "code": "path_outside_workspace", "message": format!("path outside workspace: {}", file_path) }
            });
        }

        match std::fs::read_to_string(&full) {
            Ok(source) => {
                let nodes = large_nodes::detect_in_file(file_path, &source, &input);
                let result = crate::shared::types_v5::DetectLargeNodesResult {
                    nodes,
                    scanned_files: 1,
                    returned: 1,
                    truncated: false,
                };
                serde_json::to_value(result).unwrap_or_default()
            }
            Err(e) => serde_json::json!({
                "error": { "code": "file_not_found", "message": e.to_string() }
            }),
        }
    } else {
        serde_json::to_value(large_nodes::empty_result()).unwrap_or_default()
    }
}
