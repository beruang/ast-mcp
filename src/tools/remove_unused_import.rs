//! `ast_remove_unused_import_preview` tool handler.
use crate::config::workspace::Workspace;
use serde_json::Value;
pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    crate::rewrite_tools::remove_unused_import::handle(workspace, arguments)
}
