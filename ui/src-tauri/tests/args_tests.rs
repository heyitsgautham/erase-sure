//! Argument checks for every CLI call the desktop app makes.

use app::{expand_paths_in_args, sanitize_args};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_sanitize_args_allowed_commands() {
    // Test allowed commands
    assert!(sanitize_args(&["discover".to_string()]).is_ok());
    assert!(sanitize_args(&[
        "wipe".to_string(),
        "--format".to_string(),
        "json".to_string()
    ])
    .is_ok());
    assert!(sanitize_args(&[
        "backup".to_string(),
        "--device".to_string(),
        "/dev/sdb".to_string()
    ])
    .is_ok());
    assert!(sanitize_args(&["cert".to_string(), "sign".to_string()]).is_ok());
    assert!(sanitize_args(&["cert".to_string(), "verify".to_string()]).is_ok());
}

#[test]
fn test_sanitize_args_forbidden_commands() {
    // Test forbidden commands
    assert!(sanitize_args(&["format".to_string()]).is_err());
    assert!(sanitize_args(&["dd".to_string()]).is_err());
    assert!(sanitize_args(&["rm".to_string()]).is_err());
}

#[test]
fn test_sanitize_args_forbidden_flags() {
    // Test forbidden flags
    assert!(sanitize_args(&["wipe".to_string(), "--apply".to_string()]).is_err());
    assert!(sanitize_args(&["wipe".to_string(), "--execute".to_string()]).is_err());
    assert!(sanitize_args(&["wipe".to_string(), "--i-know-what-im-doing".to_string()]).is_err());
    assert!(sanitize_args(&["wipe".to_string(), "--yes".to_string()]).is_err());
    assert!(sanitize_args(&["wipe".to_string(), "--force-execute".to_string()]).is_err());
    assert!(sanitize_args(&["backup".to_string(), "--confirm".to_string()]).is_err());
}

#[test]
fn test_sanitize_args_wipe_format_required() {
    // Test that wipe command requires --format
    assert!(sanitize_args(&[
        "wipe".to_string(),
        "--device".to_string(),
        "/dev/sdb".to_string()
    ])
    .is_err());
    assert!(sanitize_args(&[
        "wipe".to_string(),
        "--device".to_string(),
        "/dev/sdb".to_string(),
        "--format".to_string(),
        "json".to_string()
    ])
    .is_ok());
}

#[test]
fn test_sanitize_args_backup_critical_ok() {
    // Test that backup operations filter out --critical-ok flag
    let result = sanitize_args(&[
        "backup".to_string(),
        "--device".to_string(),
        "/dev/sdb".to_string(),
        "--critical-ok".to_string(),
    ]);
    assert!(result.is_ok());
    let args = result.unwrap();
    assert!(!args.contains(&"--critical-ok".to_string()));
    assert!(args.contains(&"backup".to_string()));
    assert!(args.contains(&"--device".to_string()));
}

#[test]
fn test_sanitize_args_cert_operations() {
    // Test cert operation restrictions
    assert!(sanitize_args(&["cert".to_string(), "sign".to_string()]).is_ok());
    assert!(sanitize_args(&["cert".to_string(), "verify".to_string()]).is_ok());
    assert!(sanitize_args(&["cert".to_string(), "delete".to_string()]).is_err());
    assert!(sanitize_args(&["cert".to_string(), "create".to_string()]).is_err());
}

#[test]
fn test_sanitize_args_destructive_wipe_skips_format_check() {
    // execute_destructive_wipe passes --danger-allow-wipe; the CLI then enforces
    // SECUREWIPE_DANGER=1 and the typed confirmation itself.
    let args = ["wipe", "--device", "/dev/sdb", "--danger-allow-wipe"].map(String::from);
    assert!(sanitize_args(&args).is_ok());
}

#[test]
fn test_expand_paths_in_args_expands_home_only() {
    let home = dirs::home_dir().unwrap();
    let out = expand_paths_in_args(&["~/Documents".to_string(), "/dev/sdb".to_string()]).unwrap();
    assert_eq!(out[0], home.join("Documents").to_string_lossy());
    assert_eq!(out[1], "/dev/sdb");
}

#[test]
fn test_sanitize_args_empty_input() {
    assert!(sanitize_args(&[]).is_err());
}

#[test]
fn test_sanitize_args_allows_paths_that_contain_forbidden_words() {
    // A substring match used to refuse these backups.
    let backup = args(&[
        "backup",
        "--device",
        "/dev/sdb",
        "--dest",
        "/backups",
        "--paths",
        "/home/me/Pictures/yesterday,/home/me/forced-labour-notes,/home/me/applyform",
    ]);
    assert_eq!(sanitize_args(&backup).unwrap(), backup);
}

#[test]
fn test_sanitize_args_rejects_forbidden_flags_in_any_case_or_with_a_value() {
    for flag in ["--YES", "--Force", "--confirm=true", "apply", "--force"] {
        let attempt = args(&["wipe", "--device", "/dev/sdb", "--format", "json", flag]);
        assert!(sanitize_args(&attempt).is_err(), "{flag} should be refused");
    }
}
