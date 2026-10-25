//! File system reads, all on temporary folders.

use app::files::{
    directory_size, list_cert_files, list_directory, resolve_open_path, selection_size,
};
use std::fs;
use std::path::Path;

fn write(path: &Path, bytes: usize) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, vec![b'x'; bytes]).unwrap();
}

#[test]
fn listing_puts_folders_first_and_sorts_names_without_case() {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("b.txt"), 3);
    write(&dir.path().join("A.txt"), 4);
    fs::create_dir(dir.path().join("zeta")).unwrap();
    fs::create_dir(dir.path().join("Alpha")).unwrap();

    let listing = list_directory(dir.path()).unwrap();

    let names: Vec<_> = listing.entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["Alpha", "zeta", "A.txt", "b.txt"]);
    assert_eq!(listing.total_items, 4);
    assert_eq!(listing.total_size, 7);
    assert_eq!(listing.entries[0].size, None);
    assert_eq!(listing.entries[2].size, Some(4));
    assert!(listing.entries[2].modified.is_some());
}

#[test]
fn listing_a_missing_folder_says_so() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("gone");
    let err = list_directory(&missing).unwrap_err();
    assert!(err.starts_with("Path does not exist"), "{err}");
}

#[test]
fn sizes_add_up_files_in_nested_folders() {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("a/one.bin"), 10);
    write(&dir.path().join("a/b/two.bin"), 20);
    write(&dir.path().join("three.bin"), 5);

    assert_eq!(directory_size(dir.path()).unwrap(), 35);
    let selection = [
        dir.path().join("a").to_string_lossy().to_string(),
        dir.path().join("three.bin").to_string_lossy().to_string(),
        dir.path().join("missing").to_string_lossy().to_string(),
    ];
    assert_eq!(selection_size(&selection).unwrap(), 35);
}

#[cfg(unix)]
#[test]
fn sizes_do_not_follow_a_link_back_to_a_parent_folder() {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("data/file.bin"), 8);
    std::os::unix::fs::symlink(dir.path(), dir.path().join("data/loop")).unwrap();

    // Following the link counted the file again at every level, 16 times on macOS,
    // until the OS stopped resolving the chain.
    assert_eq!(directory_size(dir.path()).unwrap(), 8);
}

#[test]
fn certificate_files_are_json_only_and_newest_name_first() {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("2026-01.json"), 1);
    write(&dir.path().join("2026-02.json"), 1);
    write(&dir.path().join("2026-02.pdf"), 1);
    fs::create_dir(dir.path().join("folder.json")).unwrap();

    let files = list_cert_files(dir.path()).unwrap();

    let names: Vec<_> = files
        .iter()
        .map(|f| {
            Path::new(f)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect();
    assert_eq!(names, ["2026-02.json", "2026-01.json"]);
}

#[test]
fn a_missing_certificate_folder_means_no_certificates() {
    let dir = tempfile::tempdir().unwrap();
    assert!(list_cert_files(&dir.path().join("certificates"))
        .unwrap()
        .is_empty());
}

#[test]
fn open_paths_must_exist() {
    let dir = tempfile::tempdir().unwrap();
    let canonical = dir.path().canonicalize().unwrap();
    assert_eq!(
        resolve_open_path(&dir.path().to_string_lossy()).unwrap(),
        canonical
    );
    let missing = dir.path().join("nope");
    let err = resolve_open_path(&missing.to_string_lossy()).unwrap_err();
    assert!(err.starts_with("Invalid or non-existent path"), "{err}");
}
