//! `ast_insert_import_preview` tool handler.
use crate::config::workspace::Workspace;
use serde_json::Value;
pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    crate::rewrite_tools::insert_import::handle(workspace, arguments)
}
