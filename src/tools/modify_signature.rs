//! `ast_modify_function_signature_preview` tool handler.
use crate::config::workspace::Workspace;
use serde_json::Value;
pub fn handle(workspace: &Workspace, arguments: Value) -> Value {
    crate::rewrite_tools::modify_signature::handle(workspace, arguments)
}
