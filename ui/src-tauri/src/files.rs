//! File system reads behind the file browser and the certificate list.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileSystemEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub entries: Vec<FileSystemEntry>,
    pub total_size: u64,
    pub total_items: usize,
}

fn modified_time(metadata: &fs::Metadata) -> Option<String> {
    let modified = metadata.modified().ok()?;
    let secs = modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let time = chrono::DateTime::from_timestamp(i64::try_from(secs).ok()?, 0)?;
    Some(time.format("%Y-%m-%d %H:%M:%S").to_string())
}

/// List one folder: folders first, then files, each sorted by name without regard to case.
/// Entries that cannot be read are skipped.
pub fn list_directory(dir: &Path) -> Result<DirectoryListing, String> {
    if !dir.exists() {
        return Err(format!("Path does not exist: {}", dir.display()));
    }
    let read = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut entries = Vec::new();
    let mut total_size = 0u64;
    for entry in read.flatten() {
        let path = entry.path();
        let metadata = entry.metadata().ok();
        let is_dir = path.is_dir();
        let size = if is_dir {
            None
        } else {
            metadata.as_ref().map(fs::Metadata::len)
        };
        total_size += size.unwrap_or(0);
        entries.push(FileSystemEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: path.to_string_lossy().to_string(),
            is_dir,
            size,
            modified: metadata.as_ref().and_then(modified_time),
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    let total_items = entries.len();
    Ok(DirectoryListing {
        entries,
        total_size,
        total_items,
    })
}

/// Total bytes of the regular files under `dir`.
///
/// Symbolic links are not followed, so a link back to a parent folder cannot loop
/// forever, and a linked file is not counted twice.
pub fn directory_size(dir: &Path) -> Result<u64, String> {
    let mut total = 0u64;
    let mut pending = vec![dir.to_path_buf()];
    while let Some(current) = pending.pop() {
        let read = fs::read_dir(&current)
            .map_err(|e| format!("Failed to read directory {}: {}", current.display(), e))?;
        for entry in read.flatten() {
            let Ok(metadata) = fs::symlink_metadata(entry.path()) else {
                continue;
            };
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                total += metadata.len();
            }
        }
    }
    Ok(total)
}

/// Total bytes of the given files and folders. Paths that no longer exist count as zero.
/// A link the user picked is followed; links found inside folders are not.
pub fn selection_size(paths: &[String]) -> Result<u64, String> {
    let mut total = 0u64;
    for path in paths.iter().map(Path::new) {
        let Ok(metadata) = fs::metadata(path) else {
            continue;
        };
        if metadata.is_file() {
            total += metadata.len();
        } else if metadata.is_dir() {
            total += directory_size(path)?;
        }
    }
    Ok(total)
}

/// The `.json` files in the certificate folder, newest name first.
/// A folder that does not exist yet means no certificates, not an error.
pub fn list_cert_files(dir: &Path) -> Result<Vec<String>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let read =
        fs::read_dir(dir).map_err(|e| format!("Failed to read certificate directory: {}", e))?;
    let mut files: Vec<String> = read
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| path.to_string_lossy().to_string())
        .collect();
    files.sort_unstable_by(|a, b| b.cmp(a));
    Ok(files)
}

/// Resolve a path the UI wants opened, which must exist.
pub fn resolve_open_path(path: &str) -> Result<PathBuf, String> {
    Path::new(path)
        .canonicalize()
        .map_err(|_| format!("Invalid or non-existent path: {}", path))
}
