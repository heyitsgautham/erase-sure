// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

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
    session_id: String,
}

type ProcessMap = Arc<Mutex<HashMap<String, tokio::process::Child>>>;

#[derive(Default)]
struct AppState {
    processes: ProcessMap,
}

/// Sanitize arguments to ensure only safe commands are allowed
fn sanitize_args(args: &[String]) -> Result<Vec<String>, String> {
    if args.is_empty() {
        return Err("No command provided".to_string());
    }

    let subcommand = &args[0];
    let allowed_subcommands = ["discover", "wipe", "backup", "cert"];
    
    if !allowed_subcommands.contains(&subcommand.as_str()) {
        return Err(format!("Subcommand '{}' is not allowed", subcommand));
    }

    // For wipe command, only allow planning mode
    if subcommand == "wipe" {
        let forbidden_flags = [
            "apply", "execute", "i-know", "danger", "yes", "force", "confirm",
            "--apply", "--execute", "--i-know-what-im-doing", "--danger", 
            "--yes", "--force-execute", "--confirm"
        ];
        
        for arg in args {
            let arg_lower = arg.to_lowercase();
            for forbidden in &forbidden_flags {
                if arg_lower.contains(forbidden) {
                    return Err(format!("Forbidden argument '{}' not allowed for wipe command", arg));
                }
            }
        }
    }

    // For cert command, only allow sign and verify
    if subcommand == "cert" && args.len() > 1 {
        let cert_operation = &args[1];
        if !["sign", "verify"].contains(&cert_operation.as_str()) {
            return Err(format!("Only 'sign' and 'verify' operations are allowed for cert command"));
        }
    }

    Ok(args.to_vec())
}

/// Get the securewipe executable path
fn get_executable_path() -> Result<std::path::PathBuf, String> {
    use std::env;
    
    // In development, look for the binary in the adjacent core directory
    let current_exe = env::current_exe().map_err(|e| format!("Failed to get current exe: {}", e))?;
    let app_dir = current_exe.parent().ok_or("Failed to get parent directory")?;
    
    // Try multiple possible locations for the securewipe binary
    let possible_paths = [
        // For development - from ui/src-tauri/target/debug, go to ../../core/target/release/securewipe
        app_dir.join("../../../core/target/release/securewipe"),
        // For development - from ui/src-tauri, go to ../core/target/release/securewipe  
        app_dir.join("../../core/target/release/securewipe"),
        // Check debug build too
        app_dir.join("../../../core/target/debug/securewipe"),
        app_dir.join("../../core/target/debug/securewipe"),
        // For bundled app - look in the same directory
        app_dir.join("securewipe"),
        // System PATH as fallback
    ];
    
    for path in &possible_paths {
        let canonical_path = path.canonicalize();
        if let Ok(canon_path) = canonical_path {
            if canon_path.exists() {
                return Ok(canon_path);
            }
        }
    }
    
    // Fallback to system PATH
    if cfg!(windows) {
        Ok(std::path::PathBuf::from("securewipe.exe"))
    } else {
        Ok(std::path::PathBuf::from("securewipe"))
    }
}

#[tauri::command]
async fn run_securewipe(
    args: Vec<String>,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Validate arguments
    let sanitized_args = sanitize_args(&args)?;
    
    // Generate session ID
    let session_id = Uuid::new_v4().to_string();
    
    // Prepare command
    let executable_path = get_executable_path()?;
    println!("Using securewipe binary at: {:?}", executable_path);
    let mut cmd = Command::new(&executable_path);
    cmd.args(&sanitized_args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);

    // Spawn process
    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to spawn securewipe process at {:?}: {}", executable_path, e)
    })?;

    // Get handles for stdout and stderr before storing the child
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

    // Store child process for potential cancellation
    {
        let mut processes = state.processes.lock().unwrap();
        processes.insert(session_id.clone(), child);
    }

    let app_handle_stdout = app_handle.clone();
    let app_handle_stderr = app_handle.clone();
    let app_handle_exit = app_handle.clone();
    let session_id_exit = session_id.clone();
    let state_clone = Arc::clone(&state.processes);

    // Spawn tasks to handle stdout and stderr
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        
        while let Ok(Some(line)) = lines.next_line().await {
            // Truncate extremely long lines
            let truncated_line = if line.len() > 65536 {
                format!("{}... [TRUNCATED]", &line[..65536])
            } else {
                line
            };

            let event = LogEvent {
                line: truncated_line,
                ts: chrono::Utc::now().to_rfc3339(),
                stream: "stdout".to_string(),
            };

            let _ = app_handle_stdout.emit_all("securewipe://stdout", &event);
        }
    });

    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        
        while let Ok(Some(line)) = lines.next_line().await {
            // Truncate extremely long lines
            let truncated_line = if line.len() > 65536 {
                format!("{}... [TRUNCATED]", &line[..65536])
            } else {
                line
            };

            let event = LogEvent {
                line: truncated_line,
                ts: chrono::Utc::now().to_rfc3339(),
                stream: "stderr".to_string(),
            };

            let _ = app_handle_stderr.emit_all("securewipe://stderr", &event);
        }
    });

    // Handle process exit
    tokio::spawn(async move {
        // Wait a bit then check for the process in the map
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let mut child_opt = {
            let mut processes = state_clone.lock().unwrap();
            processes.remove(&session_id_exit)
        };
        
        if let Some(mut child) = child_opt.take() {
            let exit_status = child.wait().await;
            let exit_code = match exit_status {
                Ok(status) => status.code(),
                Err(_) => Some(-1),
            };

            let exit_event = ExitEvent {
                code: exit_code,
                ts: chrono::Utc::now().to_rfc3339(),
                session_id: session_id_exit,
            };

            let _ = app_handle_exit.emit_all("securewipe://exit", &exit_event);
        }
    });

    Ok(session_id)
}

#[tauri::command]
async fn cancel_securewipe(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut child_opt = {
        let mut processes = state.processes.lock().unwrap();
        processes.remove(&session_id)
    };
    
    if let Some(mut child) = child_opt.take() {
        let _ = child.kill().await;
        Ok(())
    } else {
        Err(format!("Process with session ID '{}' not found", session_id))
    }
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![run_securewipe, cancel_securewipe])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
