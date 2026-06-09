//! `ast_add_decorator_preview` tool handler.
use crate::config::workspace::Workspace;
use serde_json::Value;
pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    crate::rewrite_tools::add_decorator::handle(workspace, arguments)
}
