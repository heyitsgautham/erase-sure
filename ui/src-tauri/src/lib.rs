//! Argument checks shared by the Tauri commands that spawn the securewipe CLI.

/// Expand `~/` and `$HOME` in arguments that look like paths.
pub fn expand_paths_in_args(args: &[String]) -> Result<Vec<String>, String> {
    let mut expanded_args = Vec::new();
    
    for arg in args {
        // Check if this argument looks like a path that needs expansion
        if arg.starts_with("~/") || arg.contains("$HOME") || arg.contains("${HOME}") {
            // Use shellexpand to expand the path
            match shellexpand::full(arg) {
                Ok(expanded) => expanded_args.push(expanded.to_string()),
                Err(e) => {
                    eprintln!("Warning: Failed to expand path '{}': {}", arg, e);
                    // If expansion fails, use the original argument
                    expanded_args.push(arg.clone());
                }
            }
        } else {
            expanded_args.push(arg.clone());
        }
    }
    
    Ok(expanded_args)
}

/// Allow only known subcommands and reject flags that would skip the CLI's own guards.
pub fn sanitize_args(args: &[String]) -> Result<Vec<String>, String> {
    if args.is_empty() {
        return Err("No arguments provided".to_string());
    }

    let subcommand = &args[0];
    
    // Whitelist allowed subcommands
    const ALLOWED_SUBCOMMANDS: &[&str] = &["discover", "wipe", "backup", "cert"];
    if !ALLOWED_SUBCOMMANDS.contains(&subcommand.as_str()) {
        return Err(format!("Subcommand '{}' is not allowed", subcommand));
    }

    // Check for forbidden arguments
    const FORBIDDEN_ARGS: &[&str] = &[
        "apply", "execute", "i-know", "yes", "force", "confirm",
        "--apply", "--execute", "--i-know-what-im-doing", 
        "--yes", "--force-execute", "--confirm"
    ];

    for arg in args.iter() {
        let arg_lower = arg.to_lowercase();
        for forbidden in FORBIDDEN_ARGS {
            if arg_lower.contains(forbidden) {
                return Err(format!("Forbidden argument detected: {}", arg));
            }
        }
    }

    // Additional validation for specific subcommands
    match subcommand.as_str() {
        "wipe" => {
            // For wipe, check if this is destructive mode or planning mode
            if args.contains(&"--danger-allow-wipe".to_string()) {
                // Destructive wipe mode - allow without --format requirement
                // This will be handled by the execute_destructive_wipe command
            } else if !args.contains(&"--format".to_string()) {
                // Planning mode - require --format for safety
                return Err("Wipe command must include --format for planning mode".to_string());
            }
        }
        "backup" => {
            // For backup, allow critical disk operations with explicit flag
            // Remove the --critical-ok flag from sanitized args as it's UI-only
            return Ok(args.iter()
                .filter(|&arg| arg != "--critical-ok")
                .cloned()
                .collect());
        }
        "cert" => {
            // For cert, allow sign, verify, show, and export-pdf operations
            if args.len() > 1 {
                let operation = &args[1];
                if !["sign", "verify", "--show", "--export-pdf"].contains(&operation.as_str()) {
                    return Err(format!("Cert operation '{}' is not allowed", operation));
                }
            }
        }
        _ => {}
    }

    Ok(args.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_args_allowed_commands() {
        // Test allowed commands
        assert!(sanitize_args(&["discover".to_string()]).is_ok());
        assert!(sanitize_args(&["wipe".to_string(), "--format".to_string(), "json".to_string()]).is_ok());
        assert!(sanitize_args(&["backup".to_string(), "--device".to_string(), "/dev/sdb".to_string()]).is_ok());
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
        assert!(sanitize_args(&["wipe".to_string(), "--device".to_string(), "/dev/sdb".to_string()]).is_err());
        assert!(sanitize_args(&["wipe".to_string(), "--device".to_string(), "/dev/sdb".to_string(), "--format".to_string(), "json".to_string()]).is_ok());
    }

    #[test]
    fn test_sanitize_args_backup_critical_ok() {
        // Test that backup operations filter out --critical-ok flag
        let result = sanitize_args(&["backup".to_string(), "--device".to_string(), "/dev/sdb".to_string(), "--critical-ok".to_string()]);
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
}
