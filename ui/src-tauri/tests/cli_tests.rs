//! Finding and running the securewipe CLI.

use app::cli::{
    danger_env, is_destructive, project_root, resolve_executable, truncate_line, MAX_LINE_BYTES,
};
use std::path::Path;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn short_lines_pass_through_unchanged() {
    assert_eq!(truncate_line("scanning".to_string()), "scanning");
    let exact = "a".repeat(MAX_LINE_BYTES);
    assert_eq!(truncate_line(exact.clone()), exact);
}

#[test]
fn long_lines_are_cut_with_their_full_length() {
    let line = "a".repeat(MAX_LINE_BYTES + 10);
    let cut = truncate_line(line);
    assert!(cut.starts_with(&"a".repeat(MAX_LINE_BYTES)));
    assert!(cut.ends_with(&format!("... [TRUNCATED: {} bytes]", MAX_LINE_BYTES + 10)));
}

#[test]
fn long_lines_are_cut_on_a_character_boundary() {
    // "é" is two bytes, so byte MAX_LINE_BYTES falls inside a character.
    let line = format!("{}é{}", "a".repeat(MAX_LINE_BYTES - 1), "b".repeat(10));
    let cut = truncate_line(line);
    assert!(cut.starts_with(&"a".repeat(MAX_LINE_BYTES - 1)));
    assert!(!cut.contains('é'));
    assert!(cut.contains("[TRUNCATED: 65547 bytes]"));
}

#[test]
fn project_root_is_two_levels_above_the_tauri_crate() {
    assert_eq!(
        project_root(Path::new("/repo/ui/src-tauri")),
        Path::new("/repo")
    );
    assert_eq!(project_root(Path::new("/")), Path::new("/"));
}

#[cfg(unix)]
#[test]
fn executable_prefers_release_then_debug_then_path() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(resolve_executable(root.path()), Path::new("securewipe"));

    let debug = root.path().join("core/target/debug");
    std::fs::create_dir_all(&debug).unwrap();
    std::fs::write(debug.join("securewipe"), b"").unwrap();
    assert_eq!(resolve_executable(root.path()), debug.join("securewipe"));

    let release = root.path().join("core/target/release");
    std::fs::create_dir_all(&release).unwrap();
    std::fs::write(release.join("securewipe"), b"").unwrap();
    assert_eq!(resolve_executable(root.path()), release.join("securewipe"));
}

#[test]
fn only_a_destructive_wipe_gets_the_danger_flag() {
    let wipe = args(&["wipe", "--device", "/dev/sdb", "--danger-allow-wipe"]);
    assert!(is_destructive(&wipe));
    assert_eq!(danger_env(&wipe), Some("1"));

    for call in [
        args(&["discover", "--format", "json"]),
        args(&["wipe", "--device", "/dev/sdb", "--format", "json"]),
        args(&["backup", "--device", "/dev/sdb", "--dest", "/b"]),
        args(&["cert", "--export-pdf", "abc"]),
    ] {
        assert!(!is_destructive(&call));
        assert_eq!(danger_env(&call), None, "{call:?} must not carry the flag");
    }
}
