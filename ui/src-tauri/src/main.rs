// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::fs;
use tauri::Window;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{timeout, Duration};

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

#[derive(Debug, Serialize, Deserialize)]
struct FileSystemEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
    modified: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DirectoryListing {
    entries: Vec<FileSystemEntry>,
    total_size: u64,
    total_items: usize,
}

type ProcessMap = Arc<Mutex<HashMap<String, u32>>>;

#[tauri::command]
async fn run_securewipe(
    window: Window,
    args: Vec<String>,
    session_id: Option<String>,
    app_state: tauri::State<'_, ProcessMap>,
) -> Result<(), String> {
    // Expand paths in arguments first, then sanitize
    let expanded_args = expand_paths_in_args(&args)?;
    let sanitized_args = sanitize_args(&expanded_args)?;
    
    // Generate session ID if not provided
    let session_id = session_id.unwrap_or_else(|| {
        format!("session_{}", chrono::Utc::now().timestamp_millis())
    });

    // Determine executable path based on platform
    let executable = if cfg!(windows) {
        "securewipe.exe".to_string() // In production, this should be bundled
    } else {
        // For development, use the built binary from the core directory
        let current_dir = std::env::current_dir().unwrap_or_default();
        let project_root = current_dir.parent().unwrap_or(&current_dir);
        let core_path = project_root.join("core/target/debug/securewipe");
        
        if core_path.exists() {
            core_path.to_string_lossy().to_string()
        } else {
            // Fallback to PATH lookup
            "securewipe".to_string()
        }
    };

    // Spawn the process
    let mut cmd = tokio::process::Command::new(executable);
    cmd.args(&sanitized_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to spawn securewipe process: {}", e)
    })?;

    // Get child PID for cancellation
    let child_id = child.id().unwrap_or(0);
    
    // Store child PID for potential cancellation
    {
        let mut processes = app_state.lock().unwrap();
        processes.insert(session_id.clone(), child_id);
    }

    // Get handles to stdout and stderr
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

    let window_clone = window.clone();
    let session_clone = session_id.clone();
    let app_state_clone = app_state.inner().clone();

    // Spawn task to handle process lifecycle
    tokio::spawn(async move {
        // Create readers for stdout and stderr
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Create tasks for reading stdout and stderr
        let window_stdout = window_clone.clone();
        let stdout_task = tokio::spawn(async move {
            let mut lines = stdout_reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                // Truncate oversized lines
                let truncated_line = if line.len() > 65536 {
                    format!("{}... [TRUNCATED: {} bytes]", &line[..65536], line.len())
                } else {
                    line
                };

                let event = LogEvent {
                    line: truncated_line,
                    ts: chrono::Utc::now().to_rfc3339(),
                    stream: "stdout".to_string(),
                };

                let _ = window_stdout.emit("securewipe://stdout", &event);
            }
        });

        let window_stderr = window_clone.clone();
        let stderr_task = tokio::spawn(async move {
            let mut lines = stderr_reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                // Truncate oversized lines
                let truncated_line = if line.len() > 65536 {
                    format!("{}... [TRUNCATED: {} bytes]", &line[..65536], line.len())
                } else {
                    line
                };

                let event = LogEvent {
                    line: truncated_line,
                    ts: chrono::Utc::now().to_rfc3339(),
                    stream: "stderr".to_string(),
                };

                let _ = window_stderr.emit("securewipe://stderr", &event);
            }
        });

        // Wait for both reading tasks to complete
        let _ = tokio::join!(stdout_task, stderr_task);

        // Wait for the process to complete with timeout
        let timeout_duration = Duration::from_secs(1200); // 20 minutes
        let exit_status = timeout(timeout_duration, child.wait()).await;

        let exit_code = match exit_status {
            Ok(Ok(status)) => status.code(),
            Ok(Err(_)) => Some(-1), // Process error
            Err(_) => {
                // Timeout - kill the process
                let _ = child.kill().await;
                Some(-2) // Timeout code
            }
        };

        // Remove from process map
        {
            let mut processes = app_state_clone.lock().unwrap();
            processes.remove(&session_clone);
        }

        // Emit exit event
        let exit_event = ExitEvent {
            code: exit_code,
            ts: chrono::Utc::now().to_rfc3339(),
        };

        let _ = window_clone.emit("securewipe://exit", &exit_event);
    });

    Ok(())
}

#[tauri::command]
fn cancel_securewipe(
    session_id: String,
    app_state: tauri::State<'_, ProcessMap>,
) -> Result<(), String> {
    let mut processes = app_state.lock().unwrap();
    
    if let Some(pid) = processes.remove(&session_id) {
        // Use system kill command to terminate the process
        #[cfg(unix)]
        {
            use std::process::Command;
            let _ = Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .output();
        }
        
        #[cfg(windows)]
        {
            use std::process::Command;
            let _ = Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .output();
        }
        
        Ok(())
    } else {
        Err("Session not found".to_string())
    }
}

fn expand_paths_in_args(args: &[String]) -> Result<Vec<String>, String> {
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

fn sanitize_args(args: &[String]) -> Result<Vec<String>, String> {
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
        "apply", "execute", "i-know", "danger", "yes", "force", "confirm",
        "--apply", "--execute", "--i-know-what-im-doing", "--danger", 
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
            // For wipe, ensure we're only doing planning
            if !args.contains(&"--format".to_string()) {
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
            // For cert, only allow sign and verify operations
            if args.len() > 1 {
                let operation = &args[1];
                if !["sign", "verify"].contains(&operation.as_str()) {
                    return Err(format!("Cert operation '{}' is not allowed", operation));
                }
            }
        }
        _ => {}
    }

    Ok(args.to_vec())
}

#[tauri::command]
async fn browse_folders(path: Option<String>) -> Result<DirectoryListing, String> {
    let browse_path = match path {
        Some(p) => Path::new(&p).to_path_buf(),
        None => dirs::home_dir().ok_or("Could not determine home directory")?,
    };

    if !browse_path.exists() {
        return Err(format!("Path does not exist: {}", browse_path.display()));
    }

    let mut entries = Vec::new();
    let mut total_size = 0u64;
    let mut total_items = 0usize;

    match fs::read_dir(&browse_path) {
        Ok(dir_entries) => {
            for entry in dir_entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let metadata = entry.metadata().ok();
                        let is_dir = path.is_dir();
                        
                        let size = if !is_dir {
                            metadata.as_ref().map(|m| m.len())
                        } else {
                            None
                        };

                        let modified = metadata.as_ref()
                            .and_then(|m| m.modified().ok())
                            .and_then(|time| {
                                time.duration_since(std::time::UNIX_EPOCH)
                                    .ok()
                                    .map(|duration| {
                                        chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                            .unwrap_or_default()
                                    })
                            });

                        if let Some(file_size) = size {
                            total_size += file_size;
                        }
                        total_items += 1;

                        entries.push(FileSystemEntry {
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: path.to_string_lossy().to_string(),
                            is_dir,
                            size,
                            modified,
                        });
                    }
                    Err(_) => continue,
                }
            }
        }
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    }

    // Sort: directories first, then files, both alphabetically
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(DirectoryListing {
        entries,
        total_size,
        total_items,
    })
}

#[tauri::command]
async fn calculate_selection_size(paths: Vec<String>) -> Result<u64, String> {
    let mut total_size = 0u64;

    for path_str in paths {
        let path = Path::new(&path_str);
        if !path.exists() {
            continue;
        }

        if path.is_file() {
            if let Ok(metadata) = fs::metadata(path) {
                total_size += metadata.len();
            }
        } else if path.is_dir() {
            total_size += calculate_directory_size(path).await?;
        }
    }

    Ok(total_size)
}

fn calculate_directory_size(dir: &Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64, String>> + Send + '_>> {
    Box::pin(async move {
        let mut total_size = 0u64;
        
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if path.is_file() {
                                if let Ok(metadata) = fs::metadata(&path) {
                                    total_size += metadata.len();
                                }
                            } else if path.is_dir() {
                                // Recursive calculation
                                total_size += calculate_directory_size(&path).await?;
                            }
                        }
                        Err(_) => continue,
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read directory {}: {}", dir.display(), e)),
        }

        Ok(total_size)
    })
}

fn main() {
    let process_map: ProcessMap = Arc::new(Mutex::new(HashMap::new()));

    tauri::Builder::default()
        .manage(process_map)
        .invoke_handler(tauri::generate_handler![
            run_securewipe, 
            cancel_securewipe,
            browse_folders,
            calculate_selection_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
