//! Argument checks for every securewipe CLI call the desktop app makes.

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
        "apply",
        "execute",
        "i-know",
        "yes",
        "force",
        "confirm",
        "--apply",
        "--execute",
        "--i-know-what-im-doing",
        "--yes",
        "--force-execute",
        "--confirm",
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
            return Ok(args
                .iter()
                .filter(|&arg| arg != "--critical-ok")
                .cloned()
                .collect());
        }
        // For cert, allow sign, verify, show, and export-pdf operations
        "cert" if args.len() > 1 => {
            let operation = &args[1];
            if !["sign", "verify", "--show", "--export-pdf"].contains(&operation.as_str()) {
                return Err(format!("Cert operation '{}' is not allowed", operation));
            }
        }
        _ => {}
    }

    Ok(args.to_vec())
}
