use crate::frameworks::{confidence_high, AstDetector, AstFileContext};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstHook, Evidence, HookKind};

const BUILTIN_HOOKS: &[&str] = &[
    "useState",
    "useEffect",
    "useMemo",
    "useCallback",
    "useRef",
    "useReducer",
    "useContext",
    "useLayoutEffect",
    "useImperativeHandle",
    "useTransition",
    "useDeferredValue",
    "useId",
    "useSyncExternalStore",
];

pub struct ReactHookDetector;

impl AstDetector<AstHook> for ReactHookDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstHook> {
        let mut hooks = Vec::new();
        collect_hooks(ctx.tree.root_node(), ctx, &mut hooks);
        hooks
    }
}

fn collect_hooks(node: tree_sitter::Node, ctx: &AstFileContext, results: &mut Vec<AstHook>) {
    match node.kind() {
        "call_expression" => {
            if let Some(hook) = extract_hook_call(&node, ctx) {
                results.push(hook);
            }
        }
        "function_declaration" | "arrow_function" => {
            // Check for custom hook definition
            if let Some(hook) = extract_hook_definition(&node, ctx) {
                results.push(hook);
            }
            // Don't return — recurse for usages inside
        }
        "variable_declarator" => {
            // const useXxx = (...) => { ... }
            if let Some(hook) = extract_var_hook_def(&node, ctx) {
                results.push(hook);
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_hooks(child, ctx, results);
        }
    }
}

fn extract_hook_call(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstHook> {
    let func = node.child_by_field_name("function")?;
    let callee = func.utf8_text(ctx.source.as_bytes()).unwrap_or("");

    if !callee.starts_with("use") {
        return None;
    }

    // Check it starts with "use" followed by uppercase
    let after_use = &callee[3..];
    if after_use.is_empty() || !after_use.chars().next().is_some_and(|c| c.is_uppercase()) {
        return None;
    }

    let kind = if BUILTIN_HOOKS.contains(&callee) {
        HookKind::BuiltinUsage
    } else {
        HookKind::CustomUsage
    };

    let enclosing = find_enclosing_function(node, ctx);
    let range = node_range(node, ctx);
    let evidence = make_evidence("call_expression", node, ctx);

    Some(AstHook {
        file_path: ctx.relative_path.to_string(),
        name: callee.to_string(),
        kind,
        enclosing_component: enclosing,
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn extract_hook_definition(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstHook> {
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(ctx.source.as_bytes()).unwrap_or("");

    if !name.starts_with("use") || name.len() <= 3 {
        return None;
    }
    let after_use = &name[3..];
    if !after_use.chars().next().is_some_and(|c| c.is_uppercase()) {
        return None;
    }

    // It's a custom hook definition (not a built-in name)
    if BUILTIN_HOOKS.contains(&name) {
        return None;
    }

    let range = node_range(node, ctx);
    let evidence = make_evidence("function_declaration", node, ctx);

    Some(AstHook {
        file_path: ctx.relative_path.to_string(),
        name: name.to_string(),
        kind: HookKind::CustomDefinition,
        enclosing_component: None,
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn extract_var_hook_def(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<AstHook> {
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(ctx.source.as_bytes()).unwrap_or("");

    if !name.starts_with("use") || name.len() <= 3 {
        return None;
    }
    let after_use = &name[3..];
    if !after_use.chars().next().is_some_and(|c| c.is_uppercase()) {
        return None;
    }
    if BUILTIN_HOOKS.contains(&name) {
        return None;
    }

    let range = node_range(node, ctx);
    let evidence = make_evidence("variable_declarator", node, ctx);

    Some(AstHook {
        file_path: ctx.relative_path.to_string(),
        name: name.to_string(),
        kind: HookKind::CustomDefinition,
        enclosing_component: None,
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn find_enclosing_function(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "function_declaration" | "arrow_function" => {
                return parent
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
                    .map(|s| s.to_string());
            }
            "variable_declarator" => {
                return parent
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(ctx.source.as_bytes()).ok())
                    .map(|s| s.to_string());
            }
            _ => {}
        }
        current = parent.parent();
    }
    None
}

fn node_range(node: &tree_sitter::Node, ctx: &AstFileContext) -> Range {
    let start = ts_point_to_position(node.start_position(), ctx.source);
    let end = ts_point_to_position(node.end_position(), ctx.source);
    Range { start, end }
}

fn make_evidence(kind: &str, node: &tree_sitter::Node, ctx: &AstFileContext) -> Evidence {
    let text = node.utf8_text(ctx.source.as_bytes()).ok().map(|t| t.to_string());
    let range = Some(node_range(node, ctx));
    crate::frameworks::make_evidence(kind, text.as_deref(), range, Some(node.kind()))
}
