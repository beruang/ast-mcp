use crate::frameworks::{
    confidence_high, confidence_low, confidence_medium, is_pascal_case, AstDetector, AstFileContext,
};
use crate::parser::positions::ts_point_to_position;
use crate::shared::position::Range;
use crate::shared::types_v3::{AstReactComponent, ComponentKind, Evidence};

pub struct ReactComponentDetector;

impl AstDetector<AstReactComponent> for ReactComponentDetector {
    fn detect(&self, ctx: &AstFileContext) -> Vec<AstReactComponent> {
        let mut components = Vec::new();
        collect_components(ctx.tree.root_node(), ctx, &mut components);
        components
    }
}

fn collect_components(
    node: tree_sitter::Node,
    ctx: &AstFileContext,
    results: &mut Vec<AstReactComponent>,
) {
    match node.kind() {
        "function_declaration" => {
            if let Some(comp) = extract_fn_component(&node, ctx, false) {
                results.push(comp);
            }
            return;
        }
        "export_statement" => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() == "function_declaration" {
                        if let Some(comp) = extract_fn_component(&child, ctx, true) {
                            results.push(comp);
                        }
                    } else if child.kind() == "lexical_declaration" {
                        for j in 0..child.child_count() {
                            if let Some(inner) = child.child(j) {
                                if let Some(comp) = extract_var_component(&inner, ctx, true) {
                                    results.push(comp);
                                }
                            }
                        }
                    }
                }
            }
            return;
        }
        "lexical_declaration" => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if let Some(comp) = extract_var_component(&child, ctx, false) {
                        results.push(comp);
                    }
                }
            }
            return;
        }
        "class_declaration" => {
            if let Some(comp) = extract_class_component(&node, ctx) {
                results.push(comp);
            }
            return;
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_components(child, ctx, results);
        }
    }
}

fn extract_fn_component(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
    exported: bool,
) -> Option<AstReactComponent> {
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();

    if !is_pascal_case(&name) {
        return None;
    }

    let has_jsx = node_contains_jsx(node, ctx);
    let confidence = if has_jsx { confidence_high() } else { confidence_low() };

    // Check for default export
    let default_export = exported && is_default_export(node, ctx);

    // Extract props
    let (props_name, props_type_text) = extract_props(node, ctx);
    let jsx_root = if has_jsx { find_jsx_root(node, ctx) } else { None };
    let hooks = Vec::new(); // filled in by include_hooks option in handler

    let range = node_range(node, ctx);
    let evidence = make_evidence("function_declaration", node, ctx);

    Some(AstReactComponent {
        file_path: ctx.relative_path.to_string(),
        name,
        kind: ComponentKind::FunctionComponent,
        exported,
        default_export,
        props_name,
        props_type_text,
        hooks,
        jsx_root,
        range,
        confidence,
        evidence: vec![evidence],
    })
}

fn extract_var_component(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
    exported: bool,
) -> Option<AstReactComponent> {
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();

    if !is_pascal_case(&name) {
        return None;
    }

    // Check if the value is an arrow function with JSX
    let value = node.child_by_field_name("value")?;
    let is_arrow = value.kind() == "arrow_function";
    let has_jsx = node_contains_jsx(&value, ctx);

    if !has_jsx {
        return None;
    }

    let confidence = if is_arrow { confidence_high() } else { confidence_medium() };
    let default_export = exported;
    let jsx_root = find_jsx_root(&value, ctx);
    let range = node_range(node, ctx);
    let evidence = make_evidence("variable_declarator", node, ctx);

    let kind = if is_arrow {
        // Check for memo/forwardRef wrapping
        if is_wrapped_with(&value, ctx, "memo") {
            ComponentKind::MemoComponent
        } else if is_wrapped_with(&value, ctx, "forwardRef") {
            ComponentKind::ForwardRefComponent
        } else {
            ComponentKind::ArrowFunctionComponent
        }
    } else {
        ComponentKind::Unknown
    };

    Some(AstReactComponent {
        file_path: ctx.relative_path.to_string(),
        name,
        kind,
        exported,
        default_export,
        props_name: None,
        props_type_text: None,
        hooks: Vec::new(),
        jsx_root,
        range,
        confidence,
        evidence: vec![evidence],
    })
}

fn extract_class_component(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> Option<AstReactComponent> {
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(ctx.source.as_bytes()).unwrap_or("").to_string();

    // Check extends React.Component / Component / PureComponent
    let extends_react = extends_react_component(node, ctx);
    if !extends_react {
        return None;
    }

    let range = node_range(node, ctx);
    let evidence = make_evidence("class_declaration", node, ctx);

    Some(AstReactComponent {
        file_path: ctx.relative_path.to_string(),
        name,
        kind: ComponentKind::ClassComponent,
        exported: false,
        default_export: false,
        props_name: None,
        props_type_text: None,
        hooks: Vec::new(),
        jsx_root: None,
        range,
        confidence: confidence_high(),
        evidence: vec![evidence],
    })
}

fn node_contains_jsx(node: &tree_sitter::Node, ctx: &AstFileContext) -> bool {
    contains_kind(node, ctx, "jsx_element", "jsx_self_closing_element", "jsx_fragment")
}

fn contains_kind(
    node: &tree_sitter::Node,
    _ctx: &AstFileContext,
    k1: &str,
    k2: &str,
    k3: &str,
) -> bool {
    if node.kind() == k1 || node.kind() == k2 || node.kind() == k3 {
        return true;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if contains_kind(&child, _ctx, k1, k2, k3) {
                return true;
            }
        }
    }
    false
}

fn is_default_export(node: &tree_sitter::Node, _ctx: &AstFileContext) -> bool {
    if let Some(parent) = node.parent() {
        if parent.kind() == "export_statement" {
            for i in 0..parent.child_count() {
                if let Some(child) = parent.child(i) {
                    if child.kind() == "default" {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn extract_props(
    node: &tree_sitter::Node,
    ctx: &AstFileContext,
) -> (Option<String>, Option<String>) {
    let params = match node.child_by_field_name("parameters") {
        Some(p) => p,
        None => return (None, None),
    };
    for i in 0..params.child_count() {
        if let Some(param) = params.child(i) {
            if param.is_named() && param.kind() != "{" && param.kind() != "[" {
                if let Some(name) = param.child_by_field_name("name") {
                    let pname = name.utf8_text(ctx.source.as_bytes()).ok().map(|s| s.to_string());
                    let ptype = param
                        .child_by_field_name("type")
                        .and_then(|t| t.utf8_text(ctx.source.as_bytes()).ok())
                        .map(|s| s.to_string());
                    return (pname, ptype);
                }
            }
        }
    }
    (None, None)
}

fn find_jsx_root(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    find_first_jsx_element(node, ctx)
}

fn find_first_jsx_element(node: &tree_sitter::Node, ctx: &AstFileContext) -> Option<String> {
    if node.kind() == "jsx_element" || node.kind() == "jsx_self_closing_element" {
        // Get the opening element name
        if let Some(opening) = node.child(0) {
            if opening.kind() == "jsx_opening_element"
                || opening.kind() == "jsx_self_closing_element"
            {
                for i in 0..opening.child_count() {
                    if let Some(child) = opening.child(i) {
                        if child.kind() == "identifier" || child.kind() == "member_expression" {
                            return child
                                .utf8_text(ctx.source.as_bytes())
                                .ok()
                                .map(|s| s.to_string());
                        }
                    }
                }
            }
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if let Some(name) = find_first_jsx_element(&child, ctx) {
                return Some(name);
            }
        }
    }
    None
}

fn extends_react_component(node: &tree_sitter::Node, ctx: &AstFileContext) -> bool {
    if let Some(heritage) = node.child_by_field_name("extends") {
        if let Some(clause) = heritage.child(0) {
            let text = clause.utf8_text(ctx.source.as_bytes()).unwrap_or("");
            return text == "Component"
                || text == "PureComponent"
                || text == "React.Component"
                || text == "React.PureComponent";
        }
    }
    false
}

fn is_wrapped_with(node: &tree_sitter::Node, ctx: &AstFileContext, name: &str) -> bool {
    // Check if this node's parent is a call_expression with callee = name
    if let Some(parent) = node.parent() {
        if parent.kind() == "call_expression" {
            if let Some(func) = parent.child_by_field_name("function") {
                let callee = func.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                return callee == name;
            }
        }
    }
    false
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
