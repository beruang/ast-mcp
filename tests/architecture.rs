/// Architectural lint test: verify that the codebase never writes files.
///
/// Greps every `.rs` file under `src/` for known file-write patterns.
/// The test fails if **any** match is found.
#[test]
fn no_file_write_anywhere() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        for (line_no, line) in content.lines().enumerate() {
            let line_num = line_no + 1;
            if line.contains("fs::write") {
                violations.push(format!(
                    "{}:{}: fs::write detected",
                    entry.path().display(),
                    line_num
                ));
            }
            if line.contains("OpenOptions") && line.contains(".write(") {
                violations.push(format!(
                    "{}:{}: OpenOptions::write detected",
                    entry.path().display(),
                    line_num
                ));
            }
            if line.contains(".rename(") && (line.contains("fs::") || line.contains("std::fs")) {
                violations.push(format!(
                    "{}:{}: fs::rename detected",
                    entry.path().display(),
                    line_num
                ));
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "Architectural lint FAILED — found {count} write-pattern violation(s):\n{list}",
            count = violations.len(),
            list = violations.join("\n")
        );
    }
}

/// Architectural lint test: verify that the codebase has no LSP dependency.
///
/// 1. Checks `Cargo.toml` for any dependency whose name contains "lsp"
///    (the allowlist of tree-sitter crates is exempt).
/// 2. Greps every `.rs` file under `src/` for `use lsp_` statements.
///
/// The test fails if **any** match is found.
#[test]
fn no_lsp_dependency_anywhere() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    let allowlist: &[&str] = &[
        "tree-sitter",
        "tree-sitter-typescript",
        "tree-sitter-javascript",
        "tree-sitter-python",
        "lsp-types",
    ];

    let mut violations: Vec<String> = Vec::new();

    // Check Cargo.toml dependencies
    let cargo_toml = manifest_dir.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml).unwrap();
    let mut in_deps = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[dependencies]" {
            in_deps = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_deps = false;
            continue;
        }
        if !in_deps {
            continue;
        }
        let name = trimmed
            .split_once('=')
            .map(|(n, _)| n.trim())
            .unwrap_or(trimmed)
            .split_whitespace()
            .next()
            .unwrap_or("");

        let name_lower = name.to_lowercase();
        if name_lower.contains("lsp") && !allowlist.iter().any(|a| name_lower.contains(a)) {
            violations.push(format!(
                "Cargo.toml: dependency '{}' contains 'lsp' (not in allowlist)",
                name
            ));
        }
    }

    // Check source files
    let src_dir = manifest_dir.join("src");
    if src_dir.exists() {
        for entry in walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let content = std::fs::read_to_string(entry.path()).unwrap();
            for (line_no, line) in content.lines().enumerate() {
                if line.contains("use lsp_") {
                    violations.push(format!(
                        "{}:{}: 'use lsp_' detected",
                        entry.path().display(),
                        line_no + 1
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "Architectural lint FAILED — found {count} LSP-dependency violation(s):\n{list}",
            count = violations.len(),
            list = violations.join("\n")
        );
    }
}
