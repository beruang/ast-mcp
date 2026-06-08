/// Architectural lint: assert no bare `unwrap()` or `expect()` in library
/// source code outside of explicitly allowed modules.
///
/// Allowed contexts:
/// - `tests/` directory (anything)
/// - `src/main.rs`
/// - `src/safety/paths.rs` (explicit validation path)
/// - `src/parser/` (Tree-sitter FFI boundary, where panics signal broken invariants)
/// - `src/config/workspace.rs` (initialization)
///
/// Everything else must use `Result` propagation or properly handle errors.
#[test]
fn no_unwrap_or_expect_in_library() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");

    let allowed_files: &[&str] = &[
        "src/main.rs",
        "src/safety/paths.rs",
        "src/config/workspace.rs",
        "src/mcp/transport.rs",
    ];
    let allowed_dirs: &[&str] = &["src/parser/"];

    let mut violations: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let rel = entry
            .path()
            .strip_prefix(manifest_dir)
            .unwrap()
            .to_string_lossy()
            .into_owned();

        let is_allowed_file = allowed_files.iter().any(|a| rel == *a);
        let is_allowed_dir = allowed_dirs.iter().any(|d| rel.starts_with(d));

        if is_allowed_file || is_allowed_dir {
            continue;
        }

        let content = std::fs::read_to_string(entry.path()).unwrap();
        for (line_no, line) in content.lines().enumerate() {
            let line_num = line_no + 1;

            // Ignore comments and string literals (best-effort)
            let code = match line.split("//").next() {
                Some(c) => c,
                None => continue,
            };

            if code.contains(".unwrap()") {
                violations.push(format!("{}:{}: .unwrap() in library code", rel, line_num));
            }
            if code.contains(".expect(") {
                violations.push(format!("{}:{}: .expect() in library code", rel, line_num));
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "Architectural lint FAILED — found {count} unwrap/expect violation(s):\n{list}\n\
             Note: allowed files: {allowed:?}; allowed dirs: {dirs:?}",
            count = violations.len(),
            list = violations.join("\n"),
            allowed = allowed_files,
            dirs = allowed_dirs,
        );
    }
}
