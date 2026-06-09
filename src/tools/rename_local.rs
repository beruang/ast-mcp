//! `ast_rename_local_preview` tool handler.
use crate::config::workspace::Workspace;
use serde_json::Value;
pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    crate::rewrite_tools::rename_local::handle(workspace, arguments)
}
