//! How the desktop app finds and runs the securewipe CLI.

use std::path::{Path, PathBuf};

/// Longest output line forwarded to the UI; longer lines are cut with a note.
pub const MAX_LINE_BYTES: usize = 65536;

/// Cut `line` to at most [`MAX_LINE_BYTES`] bytes plus a note with its full length.
pub fn truncate_line(line: String) -> String {
    if line.len() > MAX_LINE_BYTES {
        format!(
            "{}... [TRUNCATED: {} bytes]",
            &line[..MAX_LINE_BYTES],
            line.len()
        )
    } else {
        line
    }
}

/// The repository root, two levels above `ui/src-tauri` where `tauri dev` runs.
pub fn project_root(cwd: &Path) -> PathBuf {
    cwd.parent()
        .and_then(Path::parent)
        .unwrap_or(cwd)
        .to_path_buf()
}

/// The CLI to run: a local release build, then a debug build, then `securewipe` on PATH.
pub fn resolve_executable(project_root: &Path) -> PathBuf {
    if cfg!(windows) {
        return PathBuf::from("securewipe.exe");
    }
    let release = project_root.join("core/target/release/securewipe");
    let debug = project_root.join("core/target/debug/securewipe");
    if release.exists() {
        release
    } else if debug.exists() {
        debug
    } else {
        PathBuf::from("securewipe")
    }
}
