use ast_mcp::config::defaults::MAX_FILE_BYTES;
use ast_mcp::config::workspace::Workspace;
use ast_mcp::safety::paths::{ensure_under_size, file_size, resolve_file};

use std::fs;
use std::os::unix::fs as unix_fs;

fn setup_workspace() -> (tempfile::TempDir, Workspace, String) {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.ts");
    fs::write(&file_path, "const x = 1;\n").unwrap();

    std::env::set_var("WORKSPACE_PATH", dir.path().to_string_lossy().as_ref());

    let ws = Workspace::from_env().unwrap();
    (dir, ws, "test.ts".to_string())
}

fn clear_workspace_env() {
    std::env::remove_var("WORKSPACE_PATH");
}

#[test]
fn resolve_valid_file() {
    let (_dir, ws, rel) = setup_workspace();
    let resolved = resolve_file(&ws, &rel).unwrap();
    assert!(resolved.absolute.ends_with("test.ts"));
    assert_eq!(resolved.workspace_relative, "test.ts");
    clear_workspace_env();
}

#[test]
fn reject_traversal_dotdot() {
    let (_dir, ws, _rel) = setup_workspace();
    let result = resolve_file(&ws, "../outside.ts");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.code(), "path_outside_workspace");
    }
    clear_workspace_env();
}

#[test]
fn reject_absolute_path() {
    let (_dir, ws, _rel) = setup_workspace();
    let result = resolve_file(&ws, "/etc/passwd");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.code(), "path_outside_workspace");
    }
    clear_workspace_env();
}

#[test]
fn reject_directory() {
    let dir = tempfile::tempdir().unwrap();
    let sub_dir = dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();

    std::env::set_var("WORKSPACE_PATH", dir.path().to_string_lossy().as_ref());
    let ws = Workspace::from_env().unwrap();

    let result = resolve_file(&ws, "subdir");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.code(), "file_not_found");
    }
    clear_workspace_env();
}

#[test]
fn reject_symlink_escape() {
    let dir = tempfile::tempdir().unwrap();
    let symlink_path = dir.path().join("escape");
    unix_fs::symlink("/etc/hosts", &symlink_path).unwrap();

    std::env::set_var("WORKSPACE_PATH", dir.path().to_string_lossy().as_ref());
    let ws = Workspace::from_env().unwrap();

    let result = resolve_file(&ws, "escape");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.code(), "path_outside_workspace");
    }
    clear_workspace_env();
}

#[test]
fn reject_missing_file() {
    let (_dir, ws, _rel) = setup_workspace();
    let result = resolve_file(&ws, "does_not_exist.ts");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.code(), "file_not_found");
    }
    clear_workspace_env();
}

#[test]
fn reject_empty_path() {
    let (_dir, ws, _rel) = setup_workspace();
    let result = resolve_file(&ws, "");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.code(), "file_not_found");
    }
    clear_workspace_env();
}

#[test]
fn ensure_under_size_rejects_too_large() {
    let result = ensure_under_size(MAX_FILE_BYTES + 1);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.code(), "file_too_large");
    }
}

#[test]
fn ensure_under_size_accepts_at_limit() {
    let result = ensure_under_size(MAX_FILE_BYTES);
    assert!(result.is_ok());
}

#[test]
fn file_size_returns_correct_value() {
    let (_dir, ws, rel) = setup_workspace();
    let resolved = resolve_file(&ws, &rel).unwrap();
    let size = file_size(&resolved.absolute).unwrap();
    assert_eq!(size, 13);
    clear_workspace_env();
}

#[test]
fn workspace_from_env_reads_env_var() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("WORKSPACE_PATH", dir.path().to_string_lossy().as_ref());
    let ws = Workspace::from_env().unwrap();
    assert!(ws.root().ends_with(dir.path().file_name().unwrap()));
    clear_workspace_env();
}

#[test]
fn workspace_from_env_rejects_nonexistent() {
    std::env::set_var("WORKSPACE_PATH", "/nonexistent/path/xyz");
    let result = Workspace::from_env();
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.code(), "workspace_not_found");
    }
    clear_workspace_env();
}
