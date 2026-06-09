use crate::analysis::complexity;
use crate::config::workspace::Workspace;
use serde_json::Value;

pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    let input = complexity::handle_arguments(arguments);

    if let Some(ref file_path) = input.file_path {
        let full = workspace.root().join(file_path);
        if !full.starts_with(workspace.root()) {
            return serde_json::json!({
                "error": { "code": "path_outside_workspace", "message": format!("path outside workspace: {}", file_path) }
            });
        }

        match std::fs::read_to_string(&full) {
            Ok(source) => match complexity::analyze_file(file_path, &source, &input) {
                Ok((summary, hotspots)) => {
                    let result = crate::shared::types_v5::ComplexitySummaryResult {
                        total_files_scanned: 1,
                        total_nodes_analyzed: 1,
                        files: vec![summary],
                        hotspots,
                        truncated: false,
                    };
                    serde_json::to_value(result).unwrap_or_default()
                }
                Err(e) => serde_json::json!({
                    "error": { "code": "complexity_analysis_failed", "message": e }
                }),
            },
            Err(e) => serde_json::json!({
                "error": { "code": "file_not_found", "message": e.to_string() }
            }),
        }
    } else {
        serde_json::to_value(complexity::empty_result()).unwrap_or_default()
    }
}
