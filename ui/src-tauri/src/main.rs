// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Tauri commands for the SecureWipe desktop app.
//!
//! Each command adapts a function from the `app` library to Tauri. The rules
//! themselves live, and are tested, in `src/{args,cli,files,wipe}.rs`.

use app::files::{self, DirectoryListing};
use app::wipe::{self, WipeConfirmation};
use app::{cli, expand_paths_in_args, sanitize_args};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tauri::Window;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
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

type ProcessMap = Arc<Mutex<HashMap<String, u32>>>;

/// The CLI gets 20 minutes before the app kills it.
const CLI_TIMEOUT: Duration = Duration::from_secs(1200);

fn project_root() -> PathBuf {
    cli::project_root(&std::env::current_dir().unwrap_or_default())
}

fn home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())
}

/// A CLI command with checked arguments, run from the repository root so relative paths work.
fn cli_command(args: &[String]) -> Result<tokio::process::Command, String> {
    let args = sanitize_args(&expand_paths_in_args(args)?)?;
    let root = project_root();
    let mut cmd = tokio::process::Command::new(cli::resolve_executable(&root));
    cmd.args(&args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    match cli::danger_env(&args) {
        Some(value) => cmd.env(cli::DANGER_ENV, value),
        None => cmd.env_remove(cli::DANGER_ENV),
    };
    Ok(cmd)
}

/// Emit each line from `reader` as an event, until the stream closes.
fn forward_lines<R>(
    window: Window,
    reader: R,
    event: &'static str,
    stream: &'static str,
) -> tokio::task::JoinHandle<()>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let event_body = LogEvent {
                line: cli::truncate_line(line),
                ts: chrono::Utc::now().to_rfc3339(),
                stream: stream.to_string(),
            };
            let _ = window.emit(event, &event_body);
        }
    })
}

#[tauri::command]
async fn run_securewipe(
    window: Window,
    args: Vec<String>,
    session_id: Option<String>,
    app_state: tauri::State<'_, ProcessMap>,
) -> Result<(), String> {
    let session_id =
        session_id.unwrap_or_else(|| format!("session_{}", chrono::Utc::now().timestamp_millis()));
    let mut child = cli_command(&args)?
        .spawn()
        .map_err(|e| format!("Failed to spawn securewipe process: {}", e))?;

    app_state
        .lock()
        .map_err(|_| "Process table is unavailable".to_string())?
        .insert(session_id.clone(), child.id().unwrap_or(0));

    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;
    let processes = app_state.inner().clone();

    tokio::spawn(async move {
        let stdout_task = forward_lines(window.clone(), stdout, "securewipe://stdout", "stdout");
        let stderr_task = forward_lines(window.clone(), stderr, "securewipe://stderr", "stderr");
        let _ = tokio::join!(stdout_task, stderr_task);

        let code = match timeout(CLI_TIMEOUT, child.wait()).await {
            Ok(Ok(status)) => status.code(),
            Ok(Err(_)) => Some(-1),
            Err(_) => {
                let _ = child.kill().await;
                Some(-2)
            }
        };

        if let Ok(mut table) = processes.lock() {
            table.remove(&session_id);
        }
        let _ = window.emit(
            "securewipe://exit",
            &ExitEvent {
                code,
                ts: chrono::Utc::now().to_rfc3339(),
            },
        );
    });

    Ok(())
}

#[tauri::command]
fn cancel_securewipe(
    session_id: String,
    app_state: tauri::State<'_, ProcessMap>,
) -> Result<(), String> {
    let pid = app_state
        .lock()
        .map_err(|_| "Process table is unavailable".to_string())?
        .remove(&session_id)
        .ok_or_else(|| "Session not found".to_string())?;

    #[cfg(unix)]
    let _ = std::process::Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .output();

    #[cfg(windows)]
    let _ = std::process::Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .output();

    Ok(())
}

#[tauri::command]
async fn browse_folders(path: Option<String>) -> Result<DirectoryListing, String> {
    let dir = match path {
        Some(p) => PathBuf::from(p),
        None => home_dir()?,
    };
    files::list_directory(&dir)
}

#[tauri::command]
async fn calculate_selection_size(paths: Vec<String>) -> Result<u64, String> {
    files::selection_size(&paths)
}

#[tauri::command]
async fn get_home_dir() -> Result<String, String> {
    Ok(home_dir()?.to_string_lossy().to_string())
}

#[tauri::command]
async fn list_cert_files(directory: String) -> Result<Vec<String>, String> {
    files::list_cert_files(Path::new(&directory))
}

#[tauri::command]
async fn read_file_content(file_path: String) -> Result<String, String> {
    fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file {}: {}", file_path, e))
}

#[tauri::command]
async fn file_exists(file_path: String) -> Result<bool, String> {
    Ok(Path::new(&file_path).exists())
}

#[tauri::command]
async fn open_path(path: String) -> Result<(), String> {
    let target = files::resolve_open_path(&path, dirs::home_dir().as_deref())?;

    #[cfg(target_os = "linux")]
    let opener = std::process::Command::new("xdg-open").arg(&target).spawn();

    #[cfg(target_os = "macos")]
    let opener = std::process::Command::new("open").arg(&target).spawn();

    #[cfg(target_os = "windows")]
    let opener = std::process::Command::new("cmd")
        .args(["/C", "start", "", &target.to_string_lossy()])
        .spawn();

    opener
        .map(|_| ())
        .map_err(|e| format!("Failed to open file: {}", e))
}

#[tauri::command]
async fn generate_pdf_for_cert(cert_json_path: String) -> Result<String, String> {
    let cert_content = fs::read_to_string(&cert_json_path)
        .map_err(|e| format!("Failed to read certificate file: {}", e))?;
    let cert_data: serde_json::Value = serde_json::from_str(&cert_content)
        .map_err(|e| format!("Failed to parse certificate JSON: {}", e))?;
    let cert_id = cert_data
        .get("cert_id")
        .and_then(|v| v.as_str())
        .ok_or("Certificate ID not found in JSON")?;

    let home = home_dir()?;
    let backups_dir = home.join("SecureWipe").join("backups");
    fs::create_dir_all(&backups_dir)
        .map_err(|e| format!("Failed to create backups directory: {}", e))?;

    let args = ["cert", "--export-pdf", cert_id].map(String::from);
    let output = cli_command(&args)?
        .output()
        .await
        .map_err(|e| format!("Failed to execute securewipe: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "CLI command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // The CLI hands PDF rendering to a Python script that can finish shortly after it exits.
    tokio::time::sleep(Duration::from_millis(3000)).await;

    let exported = home
        .join("SecureWipe")
        .join("certificates")
        .join(format!("{}.pdf", cert_id));
    if !exported.exists() {
        return Err(format!(
            "PDF was not generated by CLI. Expected at: {}",
            exported.display()
        ));
    }
    let copy = backups_dir.join(format!("{}.pdf", cert_id));
    fs::copy(&exported, &copy)
        .map_err(|e| format!("Failed to copy PDF to backups directory: {}", e))?;
    Ok(copy.to_string_lossy().to_string())
}

#[tauri::command]
async fn execute_destructive_wipe(
    window: Window,
    confirmation: WipeConfirmation,
    backup_cert_id: Option<String>,
    app_state: tauri::State<'_, ProcessMap>,
) -> Result<(), String> {
    wipe::check_confirmation(&confirmation)?;
    let args = wipe::build_wipe_args(&confirmation, backup_cert_id);
    let session_id = format!("wipe_{}", chrono::Utc::now().timestamp_millis());

    let _ = window.emit(
        "wipe://start",
        &serde_json::json!({
            "session_id": session_id,
            "device": confirmation.device,
            "policy": confirmation.policy,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    );

    run_securewipe(window, args, Some(session_id), app_state).await
}

#[tauri::command]
async fn validate_wipe_device(device: String) -> Result<serde_json::Value, String> {
    let output = std::process::Command::new("lsblk")
        .args(["-J", "-o", "NAME,MODEL,SERIAL,SIZE,TYPE,MOUNTPOINT"])
        .arg(&device)
        .output()
        .map_err(|e| format!("Failed to get device info: {}", e))?;
    if !output.status.success() {
        return Err(format!("Device {} not found or inaccessible", device));
    }
    let mut device_info: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse device info: {}", e))?;

    let mounts = std::process::Command::new("mount")
        .output()
        .map_err(|e| format!("Failed to check mounts: {}", e))?;
    let is_critical = wipe::is_critical_mount(&String::from_utf8_lossy(&mounts.stdout), &device);

    if let Some(first) = device_info["blockdevices"]
        .as_array_mut()
        .and_then(|devices| devices.first_mut())
    {
        first["is_critical"] = serde_json::Value::Bool(is_critical);
        first["path"] = serde_json::Value::String(device);
    }
    Ok(device_info)
}

fn main() {
    // Load .env so the backend sees settings without a shell prefix.
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
