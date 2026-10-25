//! How the desktop app finds and runs the securewipe CLI.

use std::path::{Path, PathBuf};

/// Longest output line forwarded to the UI; longer lines are cut with a note.
pub const MAX_LINE_BYTES: usize = 65536;

/// The environment variable the CLI checks before any destructive wipe.
pub const DANGER_ENV: &str = "SECUREWIPE_DANGER";

/// Cut `line` to at most [`MAX_LINE_BYTES`] bytes plus a note with its full length.
///
/// The cut lands on a character boundary; slicing at a fixed byte index panicked on
/// multi-byte UTF-8 and killed the task that streams CLI output.
pub fn truncate_line(line: String) -> String {
    if line.len() <= MAX_LINE_BYTES {
        return line;
    }
    let mut end = MAX_LINE_BYTES;
    while !line.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}... [TRUNCATED: {} bytes]", &line[..end], line.len())
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

/// True for a destructive wipe, the only call that may carry the danger flag.
pub fn is_destructive(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--danger-allow-wipe")
}

/// The value of [`DANGER_ENV`] for this call, or `None` to remove it.
///
/// Discovery, planning, backup and certificate calls never get the flag, even when
/// the app's own environment or `.env` sets it.
pub fn danger_env(args: &[String]) -> Option<&'static str> {
    is_destructive(args).then_some("1")
}
