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
        let project_root = current_dir.parent().and_then(|p| p.parent()).unwrap_or(&current_dir);
        
        // Try release build first, then debug build
        let release_path = project_root.join("core/target/release/securewipe");
        let debug_path = project_root.join("core/target/debug/securewipe");
        
        if release_path.exists() {
            release_path.to_string_lossy().to_string()
        } else if debug_path.exists() {
            debug_path.to_string_lossy().to_string()
        } else {
            // Fallback to PATH lookup
            "securewipe".to_string()
        }
    };

    // Check if this is a destructive wipe operation
    let is_destructive = sanitized_args.contains(&"--danger-allow-wipe".to_string());

    // For destructive operations, assume the app is run with appropriate privileges
    // WARNING: This removes security checks - only use if running as root or with proper permissions
    let mut cmd = tokio::process::Command::new(&executable);
    cmd.args(&sanitized_args);

    // Log the operation type
    if is_destructive {
        println!("WARNING: Executing destructive wipe operation without privilege escalation");
        println!("Ensure the application has appropriate permissions (run as root if needed)");
    }

    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .env("SECUREWIPE_DANGER", "1"); // Set environment variable for destructive operations

    // Set working directory to project root so relative paths work
    // For sudo, we need to make sure the working directory is set correctly
    let current_dir = std::env::current_dir().unwrap_or_default();
    let project_root = current_dir.parent().and_then(|p| p.parent()).unwrap_or(&current_dir);
    cmd.current_dir(&project_root);

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

#[tauri::command]
async fn get_home_dir() -> Result<String, String> {
    dirs::home_dir()
        .map(|path| path.to_string_lossy().to_string())
        .ok_or_else(|| "Could not determine home directory".to_string())
}

#[tauri::command]
async fn list_cert_files(directory: String) -> Result<Vec<String>, String> {
    let cert_dir = Path::new(&directory);
    
    if !cert_dir.exists() {
        return Ok(Vec::new()); // Return empty list if directory doesn't exist yet
    }

    let mut cert_files = Vec::new();
    
    match fs::read_dir(cert_dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(extension) = path.extension() {
                                if extension == "json" {
                                    cert_files.push(path.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        Err(e) => return Err(format!("Failed to read certificate directory: {}", e)),
    }

    // Sort by filename (which should sort by creation time due to timestamp-based naming)
    cert_files.sort();
    cert_files.reverse(); // Most recent first

    Ok(cert_files)
}

#[tauri::command]
async fn read_file_content(file_path: String) -> Result<String, String> {
    match fs::read_to_string(&file_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read file {}: {}", file_path, e)),
    }
}

#[tauri::command]
async fn file_exists(file_path: String) -> Result<bool, String> {
    Ok(Path::new(&file_path).exists())
}

#[tauri::command]
async fn open_path(path: String) -> Result<(), String> {
    use std::process::Command;
    
    // Validate and canonicalize path to prevent traversal attacks
    let canonical_path = match Path::new(&path).canonicalize() {
        Ok(p) => p,
        Err(_) => return Err(format!("Invalid or non-existent path: {}", path))
    };
    
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&canonical_path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&canonical_path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/C", "start", "", &canonical_path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
async fn generate_pdf_for_cert(
    _window: Window,
    cert_json_path: String,
    _session_id: Option<String>,
    _app_state: tauri::State<'_, ProcessMap>,
) -> Result<String, String> {
    // Extract cert_id from the JSON file to determine PDF path
    let cert_content = fs::read_to_string(&cert_json_path)
        .map_err(|e| format!("Failed to read certificate file: {}", e))?;
    
    let cert_data: serde_json::Value = serde_json::from_str(&cert_content)
        .map_err(|e| format!("Failed to parse certificate JSON: {}", e))?;
    
    let cert_id = cert_data.get("cert_id")
        .and_then(|v| v.as_str())
        .ok_or("Certificate ID not found in JSON")?;
    
    // Get home directory for custom PDF save location
    let home_dir = dirs::home_dir()
        .ok_or("Could not determine home directory")?;
    
    let backups_dir = home_dir.join("SecureWipe").join("backups");
    
    // Create backups directory if it doesn't exist
    if !backups_dir.exists() {
        fs::create_dir_all(&backups_dir)
            .map_err(|e| format!("Failed to create backups directory: {}", e))?;
    }
    
    // Run CLI command synchronously and wait for completion
    let args = vec![
        "cert".to_string(),
        "--export-pdf".to_string(),
        cert_id.to_string()
    ];
    
    // Expand and sanitize args
    let expanded_args = expand_paths_in_args(&args)?;
    let sanitized_args = sanitize_args(&expanded_args)?;
    
    // Determine executable path 
    let executable = if cfg!(windows) {
        "securewipe.exe".to_string()
    } else {
        let current_dir = std::env::current_dir().unwrap_or_default();
        let project_root = current_dir.parent().and_then(|p| p.parent()).unwrap_or(&current_dir);
        
        let release_path = project_root.join("core/target/release/securewipe");
        let debug_path = project_root.join("core/target/debug/securewipe");
        
        if release_path.exists() {
            release_path.to_string_lossy().to_string()
        } else if debug_path.exists() {
            debug_path.to_string_lossy().to_string()
        } else {
            "securewipe".to_string()
        }
    };

    // Run the CLI command synchronously
    let mut cmd = tokio::process::Command::new(executable);
    cmd.args(&sanitized_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    // Set working directory to project root so relative paths work
    let current_dir = std::env::current_dir().unwrap_or_default();
    let project_root = current_dir.parent().and_then(|p| p.parent()).unwrap_or(&current_dir);
    cmd.current_dir(project_root);

    let output = cmd.output().await
        .map_err(|e| format!("Failed to execute securewipe: {}", e))?;
    
    // Check if command succeeded with debugging
    println!("CLI command completed with status: {:?}", output.status);
    println!("CLI stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("CLI stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("CLI command failed: {}", stderr));
    }
    
    // Check if PDF was generated
    let default_pdf_path = home_dir.join("SecureWipe").join("certificates").join(format!("{}.pdf", cert_id));
    let custom_pdf_path = backups_dir.join(format!("{}.pdf", cert_id));
    
    println!("Looking for PDF at: {}", default_pdf_path.display());
    
    // Wait longer to ensure Python script completes (increased from 500ms to 3000ms)
    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
    
    println!("After wait - PDF exists: {}", default_pdf_path.exists());
    
    if default_pdf_path.exists() {
        // Copy to custom location
        fs::copy(&default_pdf_path, &custom_pdf_path)
            .map_err(|e| format!("Failed to copy PDF to backups directory: {}", e))?;
        
        println!("PDF copied to: {}", custom_pdf_path.display());
        Ok(custom_pdf_path.to_string_lossy().to_string())
    } else {
        // Additional debugging - check if directory exists and list contents
        let cert_dir = default_pdf_path.parent().unwrap();
        if cert_dir.exists() {
            if let Ok(entries) = fs::read_dir(cert_dir) {
                println!("Certificate directory contents:");
                for entry in entries {
                    if let Ok(entry) = entry {
                        println!("  - {}", entry.path().display());
                    }
                }
            }
        } else {
            println!("Certificate directory does not exist: {}", cert_dir.display());
        }
        
        Err(format!("PDF was not generated by CLI. Expected at: {}", default_pdf_path.display()))
    }
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

#[derive(Debug, Serialize, Deserialize)]
struct WipeConfirmation {
    device: String,
    serial: String,
    policy: String,
    user_input: String,
}

#[tauri::command]
async fn execute_destructive_wipe(
    window: Window,
    confirmation: WipeConfirmation,
    backup_cert_id: Option<String>,
    app_state: tauri::State<'_, ProcessMap>,
) -> Result<(), String> {
    // Critical safety check: validate confirmation
    let expected_confirmation = format!("WIPE {}", confirmation.serial);
    if confirmation.user_input != expected_confirmation {
        return Err(format!(
            "Confirmation failed. Expected '{}', got '{}'", 
            expected_confirmation, 
            confirmation.user_input
        ));
    }

    // SECUREWIPE_DANGER is now set in run_securewipe for all operations

    // Build the wipe command arguments
    let mut args = vec![
        "wipe".to_string(),
        "--device".to_string(),
        confirmation.device.clone(),
        "--policy".to_string(),
        confirmation.policy.clone(),
        "--danger-allow-wipe".to_string(),
        "--sign".to_string(), // Always sign wipe certificates
    ];

    // Add backup cert linkage if provided
    if let Some(backup_id) = backup_cert_id {
        args.push("--backup-cert-id".to_string());
        args.push(backup_id);
    }

    // Generate session ID for tracking
    let session_id = format!("wipe_{}", chrono::Utc::now().timestamp_millis());

    // Emit start event to frontend
    let _ = window.emit("wipe://start", &serde_json::json!({
        "session_id": session_id,
        "device": confirmation.device,
        "policy": confirmation.policy,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // Execute the wipe command
    run_securewipe(window, args, Some(session_id), app_state).await
}

#[tauri::command]
async fn validate_wipe_device(device: String) -> Result<serde_json::Value, String> {
    // Get device information including serial number
    let output = std::process::Command::new("lsblk")
        .arg("-J")
        .arg("-o")
        .arg("NAME,MODEL,SERIAL,SIZE,TYPE,MOUNTPOINT")
        .arg(&device)
        .output()
        .map_err(|e| format!("Failed to get device info: {}", e))?;

    if !output.status.success() {
        return Err(format!("Device {} not found or inaccessible", device));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let device_info: serde_json::Value = serde_json::from_str(&output_str)
        .map_err(|e| format!("Failed to parse device info: {}", e))?;

    // Check if device is mounted (critical)
    let mount_output = std::process::Command::new("mount")
        .output()
        .map_err(|e| format!("Failed to check mounts: {}", e))?;

    let mount_str = String::from_utf8_lossy(&mount_output.stdout);
    let is_critical = mount_str.lines().any(|line| {
        line.contains(&device) && (
            line.contains(" / ") ||
            line.contains(" /boot ") ||
            line.contains(" /usr ") ||
            line.contains(" /etc ")
        )
    });

    // Extract device details for confirmation
    let mut device_details = device_info.clone();
    if let Some(blockdevices) = device_details["blockdevices"].as_array_mut() {
        if let Some(device_obj) = blockdevices.first_mut() {
            device_obj["is_critical"] = serde_json::Value::Bool(is_critical);
            device_obj["path"] = serde_json::Value::String(device.clone());
        }
    }

    Ok(device_details)
}

fn main() {
    // Load .env so backend sees SECUREWIPE_DANGER without shell prefix
    let _ = dotenvy::dotenv();
    let process_map: ProcessMap = Arc::new(Mutex::new(HashMap::new()));

    tauri::Builder::default()
        .manage(process_map)
        .invoke_handler(tauri::generate_handler![
            run_securewipe, 
            cancel_securewipe,
            execute_destructive_wipe,
            validate_wipe_device,
            browse_folders,
            calculate_selection_size,
            get_home_dir,
            list_cert_files,
            read_file_content,
            file_exists,
            open_path,
            generate_pdf_for_cert
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
