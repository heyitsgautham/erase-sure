// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Stdio;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

#[derive(Debug, Serialize, Deserialize)]
struct LogEvent {
    line: String,
    ts: String,
    stream: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExitEvent {
    code: Option<i32>,
    ts: String,
}

fn sanitize_args(args: &[String]) -> Result<Vec<String>, String> {
    if args.is_empty() {
        return Err("No subcommand provided".to_string());
    }

    let subcommand = &args[0];
    let allowed_subcommands = ["discover", "wipe", "backup", "cert"];
    
    if !allowed_subcommands.contains(&subcommand.as_str()) {
        return Err(format!("Subcommand '{}' is not allowed", subcommand));
    }

    // Check for forbidden flags that imply destructive execution
    let forbidden_patterns = [
        "apply", "execute", "i-know", "danger", "yes", "force", "confirm",
        "--apply", "--execute", "--i-know-what-im-doing", "--danger", 
        "--yes", "--force-execute", "--confirm"
    ];

    for arg in args.iter() {
        let arg_lower = arg.to_lowercase();
        for pattern in &forbidden_patterns {
            if arg_lower.contains(pattern) {
                return Err(format!("Forbidden argument detected: {}", arg));
            }
        }
    }

    // For wipe subcommand, ensure it's planning only
    if subcommand == "wipe" {
        // Must have --format json or similar planning flags
        let has_format = args.iter().any(|arg| arg == "--format");
        if !has_format {
            return Err("Wipe command must include --format flag for planning mode".to_string());
        }
    }

    // For cert subcommand, only allow sign and verify operations
    if subcommand == "cert" && args.len() > 1 {
        let operation = &args[1];
        if !["sign", "verify"].contains(&operation.as_str()) {
            return Err(format!("Cert operation '{}' is not allowed", operation));
        }
    }

    Ok(args.to_vec())
}

fn get_current_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

fn get_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "securewipe.exe"
    } else {
        "securewipe"
    }
}

#[tauri::command]
async fn run_securewipe(
    args: Vec<String>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let sanitized_args = sanitize_args(&args)?;
    let executable = get_executable_name();
    
    // Set maximum runtime based on operation
    let max_runtime = match sanitized_args.first().map(|s| s.as_str()) {
        Some("backup") => Duration::from_secs(20 * 60), // 20 minutes
        Some("discover") | Some("wipe") => Duration::from_secs(2 * 60), // 2 minutes
        Some("cert") => Duration::from_secs(1 * 60), // 1 minute
        _ => Duration::from_secs(5 * 60), // 5 minutes default
    };

    // Check if we're on a supported platform and if executable exists
    let platform_supported = cfg!(target_os = "linux");
    
    let which_result = std::process::Command::new("which")
        .arg(executable)
        .output();
    
    let executable_exists = match which_result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    };

    if !platform_supported {
        return handle_platform_error(&sanitized_args, &app_handle).await;
    }

    if !executable_exists {
        return handle_missing_binary_error(&sanitized_args, &app_handle).await;
    }

    let mut cmd = TokioCommand::new(executable);
    cmd.args(&sanitized_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to spawn securewipe process: {}. Make sure 'securewipe' is in your PATH.", e)
    })?;

    // Get handles for stdout and stderr
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let app_handle_clone = app_handle.clone();

    // Spawn async task to handle the process
    tokio::spawn(async move {
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let mut stdout_lines = stdout_reader.lines();
        let mut stderr_lines = stderr_reader.lines();

        // Read stdout and stderr concurrently
        let stdout_task = async {
            while let Ok(Some(line)) = stdout_lines.next_line().await {
                // Truncate very long lines to prevent memory issues
                let truncated_line = if line.len() > 64 * 1024 {
                    format!("{}... [TRUNCATED]", &line[..64 * 1024])
                } else {
                    line
                };

                let event = LogEvent {
                    line: truncated_line,
                    ts: get_current_timestamp(),
                    stream: "stdout".to_string(),
                };

                if let Err(e) = app_handle_clone.emit_all("securewipe://stdout", &event) {
                    eprintln!("Failed to emit stdout event: {}", e);
                }
            }
        };

        let stderr_task = async {
            while let Ok(Some(line)) = stderr_lines.next_line().await {
                let truncated_line = if line.len() > 64 * 1024 {
                    format!("{}... [TRUNCATED]", &line[..64 * 1024])
                } else {
                    line
                };

                let event = LogEvent {
                    line: truncated_line,
                    ts: get_current_timestamp(),
                    stream: "stderr".to_string(),
                };

                if let Err(e) = app_handle_clone.emit_all("securewipe://stderr", &event) {
                    eprintln!("Failed to emit stderr event: {}", e);
                }
            }
        };

        // Wait for either completion or timeout
        let result = timeout(max_runtime, async {
            tokio::join!(stdout_task, stderr_task);
            child.wait().await
        }).await;

        let exit_code = match result {
            Ok(Ok(status)) => status.code(),
            Ok(Err(e)) => {
                eprintln!("Process error: {}", e);
                None
            }
            Err(_) => {
                // Timeout occurred, kill the process
                if let Err(e) = child.kill().await {
                    eprintln!("Failed to kill timed-out process: {}", e);
                }
                None
            }
        };

        let exit_event = ExitEvent {
            code: exit_code,
            ts: get_current_timestamp(),
        };

        if let Err(e) = app_handle_clone.emit_all("securewipe://exit", &exit_event) {
            eprintln!("Failed to emit exit event: {}", e);
        }
    });

    Ok(())
}

async fn handle_platform_error(args: &[String], app_handle: &AppHandle) -> Result<(), String> {
    let _subcommand = args.first().map(|s| s.as_str()).unwrap_or("");
    let os_name = if cfg!(target_os = "macos") { "macOS" } else if cfg!(target_os = "windows") { "Windows" } else { "this platform" };
    
    let error_msg = format!(
        "SecureWipe is designed for Linux systems and does not support {}.\n\
        The CLI uses Linux-specific tools like lsblk, hdparm, and nvme-cli for hardware access.\n\
        \n\
        To test SecureWipe:\n\
        1. Use a Linux machine (Ubuntu, Arch, Debian, etc.)\n\
        2. Build the CLI: cd core && cargo build --release\n\
        3. Add to PATH: export PATH=\"$PWD/target/release:$PATH\"\n\
        4. Run the UI: cd ui && npm run tauri dev",
        os_name
    );

    let error_event = LogEvent {
        line: error_msg,
        ts: get_current_timestamp(),
        stream: "stderr".to_string(),
    };
    
    if let Err(e) = app_handle.emit_all("securewipe://stderr", &error_event) {
        eprintln!("Failed to emit platform error event: {}", e);
    }

    let exit_event = ExitEvent {
        code: Some(1),
        ts: get_current_timestamp(),
    };

    if let Err(e) = app_handle.emit_all("securewipe://exit", &exit_event) {
        eprintln!("Failed to emit exit event: {}", e);
    }

    Ok(())
}

async fn handle_missing_binary_error(args: &[String], app_handle: &AppHandle) -> Result<(), String> {
    let _subcommand = args.first().map(|s| s.as_str()).unwrap_or("");
    
    let error_msg = format!(
        "SecureWipe CLI binary not found in PATH.\n\
        \n\
        To fix this:\n\
        1. Build the CLI: cd core && cargo build --release\n\
        2. Add to PATH: export PATH=\"$PWD/target/release:$PATH\"\n\
        3. Verify: securewipe --help\n\
        \n\
        Or create a symlink:\n\
        sudo ln -s /path/to/erase-sure/core/target/release/securewipe /usr/local/bin/securewipe"
    );

    let error_event = LogEvent {
        line: error_msg,
        ts: get_current_timestamp(),
        stream: "stderr".to_string(),
    };
    
    if let Err(e) = app_handle.emit_all("securewipe://stderr", &error_event) {
        eprintln!("Failed to emit missing binary error event: {}", e);
    }

    let exit_event = ExitEvent {
        code: Some(127), // Command not found
        ts: get_current_timestamp(),
    };

    if let Err(e) = app_handle.emit_all("securewipe://exit", &exit_event) {
        eprintln!("Failed to emit exit event: {}", e);
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![run_securewipe])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
